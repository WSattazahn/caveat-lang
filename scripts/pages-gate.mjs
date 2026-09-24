// Pages only publishes the artifact from a successful runtime run for current main.
// Keep selection separate from GitHub's transport so refusal paths are testable.
import { appendFile, readFile, stat } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const workflowPath = '.github/workflows/runtime.yml';
const mainRef = 'refs/heads/main';
const shaPattern = /^[0-9a-f]{40}$/;

function requireValue(condition, message) {
  if (!condition) throw new Error(message);
}

function sameRepository(actual, expected) {
  return typeof actual === 'string' && actual.toLowerCase() === expected.toLowerCase();
}

function positiveId(value, label) {
  const id = Number(value);
  requireValue(Number.isSafeInteger(id) && id > 0, `Invalid ${label}.`);
  return id;
}

async function pages(api, path, key) {
  const values = [];
  for (let page = 1; ; page++) {
    const response = await api(`${path}${path.includes('?') ? '&' : '?'}per_page=100&page=${page}`);
    requireValue(Array.isArray(response[key]), `GitHub did not return ${key}.`);
    values.push(...response[key]);
    if (response[key].length < 100) return values;
  }
}

function validateRun(run, { repository, workflowId, sourceSha, runId }) {
  requireValue(run.id === runId, 'GitHub returned a different runtime run.');
  requireValue(run.workflow_id === workflowId && run.path?.split('@')[0] === workflowPath,
    'The selected run is not the runtime workflow.');
  requireValue(sameRepository(run.repository?.full_name, repository)
    && sameRepository(run.head_repository?.full_name, repository),
  'The runtime run must belong to this repository.');
  requireValue(run.event === 'push' && run.head_branch === 'main',
    'Only runtime runs from pushes to main can deploy.');
  requireValue(run.status === 'completed' && run.conclusion === 'success',
    'The complete runtime workflow must have succeeded.');
  requireValue(run.head_sha === sourceSha, 'The runtime run is stale: main has moved.');
}

async function currentMain(api, repository) {
  const reference = await api(`/repos/${repository}/git/ref/heads/main`);
  requireValue(shaPattern.test(reference.object?.sha), 'GitHub returned an invalid main revision.');
  return reference.object.sha;
}

async function latestRuntimeRun(api, repository, sourceSha) {
  // Do not filter for success: a newer pending/failed run must not be hidden
  // by an older successful run for the same commit.
  const runs = await pages(api,
    `/repos/${repository}/actions/workflows/runtime.yml/runs?event=push&branch=main&head_sha=${sourceSha}`,
    'workflow_runs');
  runs.sort((a, b) => b.id - a.id);
  requireValue(runs.length > 0, 'Current main has no runtime push run. Run runtime CI first.');
  return positiveId(runs[0].id, 'runtime run ID');
}

export async function selectCandidate({ api, repository, eventName, event, ref }) {
  requireValue(eventName === 'workflow_run' || eventName === 'workflow_dispatch',
    'Pages requires a completed runtime run or a manual redeploy.');
  if (eventName === 'workflow_dispatch') {
    requireValue(ref === mainRef, 'Manual Pages redeploys must run from main.');
  }
  const sourceSha = await currentMain(api, repository);
  const workflow = await api(`/repos/${repository}/actions/workflows/runtime.yml`);
  const latestId = await latestRuntimeRun(api, repository, sourceSha);
  let runId;
  if (eventName === 'workflow_run') {
    requireValue(event.action === 'completed', 'The runtime workflow has not completed.');
    runId = positiveId(event.workflow_run?.id, 'runtime run ID');
  } else {
    runId = latestId;
  }
  requireValue(runId === latestId, 'The runtime run was superseded by a newer run for this revision.');
  const run = await api(`/repos/${repository}/actions/runs/${runId}`);
  validateRun(run, { repository, workflowId: workflow.id, sourceSha, runId });

  const artifacts = await pages(api, `/repos/${repository}/actions/runs/${runId}/artifacts`, 'artifacts');
  const matches = artifacts.filter(artifact => artifact.name === 'browser-dist');
  requireValue(matches.length === 1, 'The runtime run must have exactly one browser-dist artifact.');
  const artifact = matches[0];
  requireValue(artifact.expired === false, 'The tested browser-dist artifact has expired; rerun runtime CI.');
  requireValue(artifact.workflow_run?.id === runId && artifact.workflow_run?.head_sha === sourceSha,
    'The browser-dist artifact does not match the selected runtime run and revision.');

  // workflow_run has no push.before. Preserve the scope already decided by
  // runtime CI, including multi-commit pushes, instead of guessing with HEAD^.
  const jobs = await pages(api, `/repos/${repository}/actions/runs/${runId}/jobs?filter=latest`, 'jobs');
  const door = jobs.filter(job => job.name === 'Legacy Door 3D QA');
  requireValue(door.length === 1 && ['success', 'skipped'].includes(door[0].conclusion),
    'The runtime run has no conclusive Legacy Door regression scope.');
  return {
    source_sha: sourceSha,
    runtime_run_id: runId,
    runtime_run_attempt: positiveId(run.run_attempt, 'runtime run attempt'),
    artifact_id: positiveId(artifact.id, 'browser-dist artifact ID'),
    legacy_door: door[0].conclusion === 'success',
  };
}

export async function recheckCandidate({ api, repository, sourceSha, runId, runAttempt }) {
  requireValue(shaPattern.test(sourceSha), 'Invalid selected source revision.');
  requireValue(await currentMain(api, repository) === sourceSha,
    'Pages deployment stopped because main moved after artifact selection.');
  const workflow = await api(`/repos/${repository}/actions/workflows/runtime.yml`);
  const selectedId = positiveId(runId, 'runtime run ID');
  requireValue(await latestRuntimeRun(api, repository, sourceSha) === selectedId,
    'The runtime run was superseded while deployment was queued.');
  const run = await api(`/repos/${repository}/actions/runs/${selectedId}`);
  validateRun(run, { repository, workflowId: workflow.id, sourceSha, runId: selectedId });
  requireValue(run.run_attempt === positiveId(runAttempt, 'selected runtime run attempt'),
    'The runtime run was rerun after artifact selection; use its new successful completion.');
}

export function validateBuildInfo(info, sourceSha) {
  requireValue(shaPattern.test(sourceSha) && info?.revision === sourceSha,
    'The tested artifact build-info revision differs from the selected source.');
  requireValue(info.clean === true && info.compiled === true,
    'Pages requires a clean, compiled runtime build.');
  requireValue(info.host === 'x86_64-unknown-linux-gnu', 'Pages requires the tested Linux runtime build.');
}

function githubApi(token) {
  requireValue(token, 'GITHUB_TOKEN is required.');
  const base = process.env.GITHUB_API_URL || 'https://api.github.com';
  return async path => {
    const response = await fetch(`${base}${path}`, { headers: {
      authorization: `Bearer ${token}`,
      accept: 'application/vnd.github+json',
      'X-GitHub-Api-Version': '2022-11-28',
    } });
    requireValue(response.ok, `GitHub request failed (${response.status}) for ${path}.`);
    return response.json();
  };
}

async function main() {
  const command = process.argv[2];
  if (command === 'verify-build') {
    const info = JSON.parse(await readFile('dist/build-info.json', 'utf8'));
    validateBuildInfo(info, process.env.PAGE_SOURCE_SHA);
    for (const file of ['LICENSE', 'THIRD_PARTY_NOTICES.md']) {
      requireValue(await readFile(`dist/${file}`, 'utf8') === await readFile(file, 'utf8'),
        `The tested artifact ${file} differs from its source revision.`);
    }
    for (const file of ['pkg/caveat_runtime.js', 'pkg/caveat_runtime_bg.wasm',
      'pkg-reactive/caveat_runtime.js', 'pkg-reactive/caveat_runtime_bg.wasm',
      'slime_glow_ability.cav', 'slime-glow-policy.js']) {
      requireValue((await stat(`dist/${file}`)).isFile(), `The tested artifact is missing ${file}.`);
    }
    console.log(`Tested Linux artifact verified at ${info.revision}.`);
    return;
  }
  const repository = process.env.GITHUB_REPOSITORY;
  requireValue(/^[\w.-]+\/[\w.-]+$/.test(repository || ''), 'Invalid GITHUB_REPOSITORY.');
  const api = githubApi(process.env.GITHUB_TOKEN);
  if (command === 'select') {
    const candidate = await selectCandidate({ api, repository,
      eventName: process.env.GITHUB_EVENT_NAME,
      event: JSON.parse(await readFile(process.env.GITHUB_EVENT_PATH, 'utf8')),
      ref: process.env.GITHUB_REF });
    requireValue(process.env.GITHUB_OUTPUT, 'GITHUB_OUTPUT is required.');
    await appendFile(process.env.GITHUB_OUTPUT,
      Object.entries(candidate).map(([key, value]) => `${key}=${value}\n`).join(''));
    console.log(`Selected runtime run ${candidate.runtime_run_id}, artifact ${candidate.artifact_id}, source ${candidate.source_sha}.`);
  } else if (command === 'recheck') {
    await recheckCandidate({ api, repository, sourceSha: process.env.PAGE_SOURCE_SHA,
      runId: process.env.PAGE_RUNTIME_RUN_ID, runAttempt: process.env.PAGE_RUNTIME_RUN_ATTEMPT });
    console.log('Current main and the complete runtime result still match the selected artifact.');
  } else {
    throw new Error('Usage: node scripts/pages-gate.mjs select|verify-build|recheck');
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
