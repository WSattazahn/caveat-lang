// Checks for .github/workflows/publish-npm.yml, which publishes only the
// tarball that a successful Runtime run on main tested for the tagged
// revision. Selection, verification and refusal stay here so they are testable
// without GitHub or npm; the workflow supplies transport and the publish.
//
//   node scripts/release-publish.mjs inputs     validate dispatch inputs and tag
//   node scripts/release-publish.mjs select     find the Runtime run and artifact
//   node scripts/release-publish.mjs verify DIR check the downloaded candidate
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { appendFile, readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { scanCapabilities, tarballFiles } from './kit-capabilities.mjs';

export const PACKAGE = 'caveat-lang';
export const REGISTRY = 'https://registry.npmjs.org';
export const DIST_TAGS = ['next', 'latest'];
const RUNTIME_WORKFLOW = '.github/workflows/runtime.yml';
const CANDIDATE_ARTIFACT = 'kit-package-candidate';
const SHA = /^[0-9a-f]{40}$/;
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const same = (a, b) => typeof a === 'string' && a.toLowerCase() === b.toLowerCase();

// The workflow runs from the tag itself, so GITHUB_SHA is the tagged revision
// and the npm provenance names that commit, not a later main.
export function parseReleaseInputs({ tag, sha256: digest, distTag, ref, revision }) {
  const match = /^v(\d+\.\d+\.\d+(?:-rc\.\d+)?)$/.exec(tag ?? '');
  assert.ok(match, `Release tag must look like v0.1.0-rc.11, not ${JSON.stringify(tag)}.`);
  assert.equal(ref, `refs/tags/${tag}`, `Run this workflow from the tag ${tag} itself (Use workflow from: ${tag}), not ${ref}.`);
  assert.match(digest ?? '', /^[0-9a-f]{64}$/, 'The expected tarball SHA256 must be 64 lowercase hexadecimal characters.');
  assert.ok(DIST_TAGS.includes(distTag), `The npm dist-tag must be one of ${DIST_TAGS.join(', ')}, not ${JSON.stringify(distTag)}.`);
  assert.match(revision ?? '', SHA, 'The workflow revision must be a complete Git SHA.');
  return { tag, version: match[1], sha256: digest, distTag, revision };
}

async function pages(api, url, key) {
  const values = [];
  for (let page = 1; ; page++) {
    const response = await api(`${url}${url.includes('?') ? '&' : '?'}per_page=100&page=${page}`);
    assert.ok(Array.isArray(response[key]), `GitHub did not return ${key}.`);
    values.push(...response[key]);
    if (response[key].length < 100) return values;
  }
}

// The newest Runtime run pushed to main for the revision must have succeeded:
// an older success cannot hide a newer failure.
export async function selectRuntimeCandidate({ api, repository, revision }) {
  assert.match(revision, SHA, 'Invalid release revision.');
  const workflow = await api(`/repos/${repository}/actions/workflows/runtime.yml`);
  const runs = (await pages(api, `/repos/${repository}/actions/workflows/runtime.yml/runs?event=push&branch=main&head_sha=${revision}`, 'workflow_runs'))
    .sort((a, b) => b.id - a.id);
  assert.ok(runs.length > 0, `No Runtime run from a push to main tested ${revision}.`);
  const run = runs[0];
  assert.ok(run.workflow_id === workflow.id && run.path?.split('@')[0] === RUNTIME_WORKFLOW, 'The selected run is not the Runtime workflow.');
  assert.ok(same(run.repository?.full_name, repository) && same(run.head_repository?.full_name, repository), 'The Runtime run must belong to this repository.');
  assert.ok(run.event === 'push' && run.head_branch === 'main' && run.head_sha === revision, 'The Runtime run must be a push to main at the tagged revision.');
  assert.ok(run.status === 'completed' && run.conclusion === 'success', `The newest Runtime run for ${revision} (${run.id}) has not succeeded.`);
  const artifacts = (await pages(api, `/repos/${repository}/actions/runs/${run.id}/artifacts`, 'artifacts'))
    .filter(artifact => artifact.name === CANDIDATE_ARTIFACT);
  assert.equal(artifacts.length, 1, `Runtime run ${run.id} must have exactly one ${CANDIDATE_ARTIFACT} artifact.`);
  const [artifact] = artifacts;
  assert.equal(artifact.expired, false, `The tested ${CANDIDATE_ARTIFACT} of run ${run.id} has expired; it cannot be published.`);
  assert.ok(artifact.workflow_run?.id === run.id && artifact.workflow_run?.head_sha === revision, 'The artifact does not belong to the selected run and revision.');
  return { runtime_run_id: run.id, artifact_id: artifact.id };
}

async function findReports(directory) {
  const found = [];
  for (const entry of await readdir(directory, { withFileTypes: true, recursive: true })) {
    if (entry.isFile() && entry.name === 'report.json') found.push(path.join(entry.parentPath ?? entry.path, entry.name));
  }
  return found;
}

// The downloaded artifact must hold one tested tarball whose bytes, version,
// build and capabilities are the ones being released.
export async function verifyCandidate({ directory, version, revision, sha256: expected }) {
  const reports = await findReports(directory);
  assert.equal(reports.length, 1, `Expected one package report in the candidate, found ${reports.length}.`);
  const run = path.dirname(reports[0]);
  const sums = (await readFile(path.join(run, 'SHA256SUMS'), 'utf8')).trim().split(/\r?\n/);
  for (const line of sums) {
    const [digest, file] = line.split(/\s+\*?/);
    assert.equal(sha256(await readFile(path.join(run, file))), digest, `${file} differs from the run's SHA256SUMS.`);
  }
  const report = JSON.parse(await readFile(reports[0], 'utf8'));
  assert.equal(report.name, PACKAGE, 'The tested package is not caveat-lang.');
  assert.equal(report.version, version, `The tested package is ${report.version}, not ${version}.`);
  assert.equal(report.private, false, 'The tested package is private.');
  assert.match(report.tarball ?? '', /^[A-Za-z0-9._-]+\.tgz$/, 'The report names no tarball.');
  assert.ok(sums.some(line => line.endsWith(` ${report.tarball}`) || line.endsWith(`*${report.tarball}`)), 'SHA256SUMS does not cover the tarball.');
  const tarball = path.join(run, report.tarball);
  const bytes = await readFile(tarball);
  assert.equal(sha256(bytes), expected, `The tested tarball SHA256 is ${sha256(bytes)}, not the expected ${expected}.`);
  assert.equal(report.sha256, expected, 'The package report names a different tarball.');
  assert.equal(report.size, bytes.length, 'The tarball size differs from the package report.');
  const build = JSON.parse(await readFile(path.join(run, 'build-info.json'), 'utf8'));
  assert.equal(build.revision, revision, `The tarball was built from ${build.revision}, not the tagged ${revision}.`);
  assert.ok(build.clean === true && build.compiled === true, 'Publication requires a clean, compiled runtime build.');
  assert.equal(build.host, 'x86_64-unknown-linux-gnu', 'Publication requires the clean Linux build.');
  assert.equal(report.runtime?.revision, revision, 'The package report names a different runtime revision.');
  const capabilities = scanCapabilities(tarballFiles(tarball));
  assert.ok(capabilities.passed, `Declared capabilities differ: ${capabilities.failures.join('; ')}`);
  return { tarball, report: reports[0], bytes: bytes.length, sha256: expected, sha512: createHash('sha512').update(bytes).digest('hex') };
}

// npm never reuses a version; refuse before asking for release approval.
export async function requireUnpublished({ fetch: get = fetch, version }) {
  const response = await get(`${REGISTRY}/${PACKAGE}/${version}`);
  assert.equal(response.status, 404, `${PACKAGE}@${version} is already on npm (HTTP ${response.status}); nothing to publish.`);
}

function githubApi(token) {
  assert.ok(token, 'GITHUB_TOKEN is required.');
  const base = process.env.GITHUB_API_URL || 'https://api.github.com';
  return async url => {
    const response = await fetch(`${base}${url}`, { headers: { authorization: `Bearer ${token}`,
      accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28' } });
    assert.ok(response.ok, `GitHub request failed (${response.status}) for ${url}.`);
    return response.json();
  };
}

async function output(values) {
  assert.ok(process.env.GITHUB_OUTPUT, 'GITHUB_OUTPUT is required.');
  await appendFile(process.env.GITHUB_OUTPUT, Object.entries(values).map(([key, value]) => `${key}=${value}\n`).join(''));
}

async function main() {
  const [command, directory] = process.argv.slice(2);
  const inputs = parseReleaseInputs({ tag: process.env.RELEASE_TAG, sha256: process.env.RELEASE_SHA256,
    distTag: process.env.RELEASE_DIST_TAG, ref: process.env.GITHUB_REF, revision: process.env.GITHUB_SHA });
  if (command === 'inputs') {
    await output({ version: inputs.version });
    console.log(`Release ${inputs.tag}: ${PACKAGE}@${inputs.version} at ${inputs.revision}, dist-tag ${inputs.distTag}.`);
  } else if (command === 'select') {
    const repository = process.env.GITHUB_REPOSITORY;
    const selected = await selectRuntimeCandidate({ api: githubApi(process.env.GITHUB_TOKEN), repository, revision: inputs.revision });
    await output(selected);
    console.log(`Runtime run ${selected.runtime_run_id} tested ${inputs.revision}; artifact ${selected.artifact_id}.`);
  } else if (command === 'verify' && directory) {
    const verified = await verifyCandidate({ directory, ...inputs });
    await requireUnpublished({ version: inputs.version });
    await writeFile(path.join(directory, 'candidate.json'), `${JSON.stringify({ ...inputs, ...verified,
      runtimeRunId: process.env.RUNTIME_RUN_ID ?? null, verifiedAt: new Date().toISOString() }, null, 2)}\n`);
    await output({ tarball: path.relative(directory, verified.tarball).split(path.sep).join('/') });
    console.log(`Verified ${path.basename(verified.tarball)} (${verified.bytes} bytes, sha256 ${verified.sha256}); ${PACKAGE}@${inputs.version} is not yet on npm.`);
  } else {
    throw new Error('Usage: node scripts/release-publish.mjs inputs|select|verify DIRECTORY');
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
