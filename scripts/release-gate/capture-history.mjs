// Records what GitHub says today about every release publish-npm.yml has
// published, for release-gate.test.mjs to replay through the Caveat gate and
// the legacy checks. Each release's revision and the Runtime run it shipped
// from come from its docs/releases/v*-npm-publication.json record. Only the
// fields the gate reads are kept.
//
//   GITHUB_TOKEN=... node scripts/release-gate/capture-history.mjs [TAG REVISION RUNTIME_RUN]... > scripts/fixtures/release-gate-history.json
//
// A release whose publication record is not on main yet can be named on the
// command line: its tag, the tagged revision and the Runtime run it shipped from.
import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const repository = 'WSattazahn/caveat-lang';
const base = process.env.GITHUB_API_URL || 'https://api.github.com';

async function api(url) {
  const headers = { accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28' };
  if (process.env.GITHUB_TOKEN) headers.authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  const response = await fetch(`${base}${url}`, { headers });
  if (!response.ok) throw new Error(`GitHub request failed (${response.status}) for ${url}.`);
  return response.json();
}

const pick = (value, keys) => Object.fromEntries(keys.filter(key => key in value).map(key => [key, value[key]]));
const run = value => ({ ...pick(value, ['id', 'workflow_id', 'path', 'event', 'head_branch', 'head_sha', 'status', 'conclusion', 'run_attempt']),
  repository: { full_name: value.repository?.full_name }, head_repository: { full_name: value.head_repository?.full_name } });
const artifact = value => ({ ...pick(value, ['id', 'name', 'expired']), workflow_run: pick(value.workflow_run ?? {}, ['id', 'head_sha']) });

async function all(url, key) {
  const values = [];
  for (let page = 1; ; page++) {
    const response = await api(`${url}${url.includes('?') ? '&' : '?'}per_page=100&page=${page}`);
    values.push(...response[key]);
    if (response[key].length < 100) return values;
  }
}

// The first value under KEY anywhere in a record.
function found(value, key) {
  if (!value || typeof value !== 'object') return undefined;
  if (typeof value[key] === 'string') return value[key];
  for (const child of Object.values(value)) {
    const inner = found(child, key);
    if (inner !== undefined) return inner;
  }
  return undefined;
}

const releases = [];
async function capture(tag, revision, runtimeRun) {
  const workflow = await api(`/repos/${repository}/actions/workflows/runtime.yml`);
  const runs = await all(`/repos/${repository}/actions/workflows/runtime.yml/runs?event=push&branch=main&head_sha=${revision}`, 'workflow_runs');
  const artifacts = {};
  for (const { id } of runs) artifacts[id] = (await all(`/repos/${repository}/actions/runs/${id}/artifacts`, 'artifacts')).map(artifact);
  releases.push({ tag, revision, shipped_runtime_run: Number.isSafeInteger(runtimeRun) ? runtimeRun : null,
    workflow: pick(workflow, ['id', 'path']), runs: runs.map(run), artifacts });
}
const directory = path.join(root, 'docs', 'releases');
for (const name of (await readdir(directory)).filter(file => /^v.*-npm-publication\.json$/.test(file)).sort()) {
  const text = await readFile(path.join(directory, name), 'utf8');
  const record = JSON.parse(text);
  if (record.publication?.workflow !== '.github/workflows/publish-npm.yml') continue;
  const runtimeRun = Number(/\/actions\/runs\/(\d+)/.exec(found(record, 'runtimeRun') ?? '')?.[1]);
  await capture(record.tag, record.revision, runtimeRun);
}
const named = process.argv.slice(2);
for (let index = 0; index + 2 < named.length; index += 3) {
  const [tag, revision, run] = named.slice(index, index + 3);
  if (!releases.some(release => release.tag === tag)) await capture(tag, revision, Number(run));
}
console.log(JSON.stringify({ schema: 'caveat-release-gate-history/0.1', repository, capturedAt: new Date().toISOString(), releases }, null, 1));
