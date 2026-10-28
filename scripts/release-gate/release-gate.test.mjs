// The Caveat release gate must give the legacy checks' answer, refusal
// message included, on every release publish-npm.yml has published (as
// GitHub reports them today), on each of those with one thing changed, and on
// randomized run histories. It also keeps a record that explains the release.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { selectRuntimeCandidate, selectRuntimeCandidateLegacy } from '../release-publish.mjs';
import { gateRuntimeCandidate, loadKit, recordFiles } from './gate.mjs';

const kit = await loadKit();
const history = JSON.parse(await readFile(new URL('../fixtures/release-gate-history.json', import.meta.url), 'utf8'));
const repository = history.repository;

// A GitHub API answering from one release's captured state.
function github({ workflow, runs, artifacts }) {
  return async url => {
    if (url.startsWith(`/repos/${repository}/actions/workflows/runtime.yml?`) || url === `/repos/${repository}/actions/workflows/runtime.yml`) return workflow;
    let match = /\/actions\/workflows\/runtime\.yml\/runs\?.*&page=(\d+)$/.exec(url);
    if (match) return { workflow_runs: match[1] === '1' ? runs : [] };
    match = /\/actions\/runs\/(\d+)\/artifacts\?.*&page=(\d+)$/.exec(url);
    if (match) return { artifacts: match[2] === '1' ? artifacts[match[1]] ?? [] : [] };
    throw new Error(`unexpected ${url}`);
  };
}

const settle = promise => promise.then(value => `allowed: run ${value.runtime_run_id}, artifact ${value.artifact_id}`, error => `refused: ${error.message}`);
async function both(state, revision) {
  const api = github(state);
  const caveat = await settle(gateRuntimeCandidate({ api, repository, revision, kit }));
  const legacy = await settle(selectRuntimeCandidateLegacy({ api, repository, revision }));
  return { caveat, legacy };
}

test('every published release gets the same answer from the Caveat gate and the legacy checks', async () => {
  assert.ok(history.releases.length >= 16);
  for (const release of history.releases) {
    const { caveat, legacy } = await both(release, release.revision);
    assert.equal(caveat, legacy, release.tag);
    if (release.runs.length) {
      // Released from main: the gate allows the run the release shipped from.
      assert.match(caveat, new RegExp(`^allowed: run ${release.shipped_runtime_run}, `), release.tag);
    } else {
      // 1.0.1 to 1.0.3 shipped from release/1.0.x under an older gate; today's
      // gate, like the legacy one, accepts only a run from a push to main.
      assert.equal(caveat, `refused: No Runtime run from a push to main tested ${release.revision}.`, release.tag);
    }
  }
});

const clone = value => structuredClone(value);
// One thing changed in a release's real state; each must be answered alike.
const changes = {
  'newer failed run': state => { const run = { ...clone(state.runs[0]), id: state.runs[0].id + 1, conclusion: 'failure' }; state.runs.push(run); state.artifacts[run.id] = []; },
  'newer run still going': state => { const run = { ...clone(state.runs[0]), id: state.runs[0].id + 1, status: 'in_progress', conclusion: null }; state.runs.push(run); },
  'older failed run': state => { state.runs.unshift({ ...clone(state.runs[0]), id: state.runs[0].id - 1, conclusion: 'failure' }); },
  'failed': state => { state.runs[0].conclusion = 'failure'; },
  'cancelled': state => { state.runs[0].conclusion = 'cancelled'; },
  'other workflow': state => { state.runs[0].workflow_id += 1; },
  'other path': state => { state.runs[0].path = '.github/workflows/pages.yml'; },
  'pinned path': state => { state.runs[0].path = '.github/workflows/runtime.yml@refs/heads/main'; },
  'fork head': state => { state.runs[0].head_repository.full_name = 'fork/caveat-lang'; },
  'case of the name': state => { state.runs[0].repository.full_name = repository.toUpperCase(); },
  'pull request': state => { state.runs[0].event = 'pull_request'; },
  'other branch': state => { state.runs[0].head_branch = 'release/1.0.x'; },
  'other revision': state => { state.runs[0].head_sha = 'c'.repeat(40); },
  'no artifact': state => { state.artifacts[state.runs[0].id] = state.artifacts[state.runs[0].id].filter(a => a.name !== 'kit-package-candidate'); },
  'two artifacts': state => { const list = state.artifacts[state.runs[0].id]; list.push({ ...clone(list.find(a => a.name === 'kit-package-candidate')), id: 1 }); },
  'expired': state => { for (const a of state.artifacts[state.runs[0].id]) a.expired = true; },
  'expired unknown': state => { for (const a of state.artifacts[state.runs[0].id]) delete a.expired; },
  'foreign artifact': state => { for (const a of state.artifacts[state.runs[0].id]) a.workflow_run.id += 1; },
  'artifact of other revision': state => { for (const a of state.artifacts[state.runs[0].id]) a.workflow_run.head_sha = 'd'.repeat(40); },
};

test('each published release with one thing changed gets the same answer from both', async () => {
  let refused = 0;
  for (const release of history.releases.filter(release => release.runs.length)) {
    for (const [name, change] of Object.entries(changes)) {
      const state = clone(release);
      change(state);
      const { caveat, legacy } = await both(state, release.revision);
      assert.equal(caveat, legacy, `${release.tag}: ${name}`);
      if (caveat.startsWith('refused')) refused += 1;
    }
  }
  assert.ok(refused > 100, `the changes exercise refusals (${refused})`);
});

test('randomized run histories get the same answer from both', async () => {
  let seed = 20261025;
  const random = () => (seed = (seed * 1103515245 + 12345) % 2147483648) / 2147483648;
  const pickOne = values => values[Math.floor(random() * values.length)];
  const revision = 'a'.repeat(40);
  const answers = new Set();
  for (let trial = 0; trial < 400; trial++) {
    const runs = [];
    const artifacts = {};
    const ids = new Set();
    for (let index = Math.floor(random() * 4); index > 0; index--) {
      let id; do { id = 1 + Math.floor(random() * 50); } while (ids.has(id));
      ids.add(id);
      runs.push({ id, workflow_id: random() < 0.9 ? 7 : 8, path: pickOne(['.github/workflows/runtime.yml', '.github/workflows/runtime.yml@main', '.github/workflows/x.yml']),
        repository: { full_name: random() < 0.9 ? repository : 'fork/x' }, head_repository: { full_name: random() < 0.9 ? repository : 'fork/x' },
        event: random() < 0.9 ? 'push' : 'workflow_dispatch', head_branch: random() < 0.9 ? 'main' : 'other', head_sha: random() < 0.9 ? revision : 'b'.repeat(40),
        status: random() < 0.85 ? 'completed' : 'in_progress', conclusion: pickOne(['success', 'success', 'success', 'failure', 'cancelled', null]) });
      artifacts[id] = Array.from({ length: pickOne([0, 1, 1, 1, 2]) }, (_, n) => ({ id: 100 * id + n, name: pickOne(['kit-package-candidate', 'kit-package-candidate', 'browser-dist']),
        expired: pickOne([false, false, false, true, undefined]), workflow_run: { id: random() < 0.9 ? id : id + 1, head_sha: random() < 0.95 ? revision : 'c'.repeat(40) } }));
    }
    runs.sort(() => random() - 0.5);
    const { caveat, legacy } = await both({ workflow: { id: 7 }, runs, artifacts }, revision);
    assert.equal(caveat, legacy, JSON.stringify({ runs, artifacts }));
    answers.add(caveat.replace(/\d+/g, 'N'));
  }
  assert.ok(answers.size >= 8, `the histories reach every kind of answer (${[...answers].join(' | ')})`);
});

test('1.3.0: a rerun of the failed jobs (attempt 2) is the run, with attempt 1\'s candidate artifact', async () => {
  const release = history.releases.find(entry => entry.tag === 'v1.3.0');
  assert.equal(release.runs[0].run_attempt, 2);
  const { caveat, legacy } = await both(release, release.revision);
  assert.equal(caveat, 'allowed: run 40131781029, artifact 12575775253');
  assert.equal(legacy, caveat);
});

test('the gate keeps a record that explains why the release was allowed', async () => {
  const release = history.releases.find(entry => entry.tag === 'v1.3.0');
  const result = await gateRuntimeCandidate({ api: github(release), repository, revision: release.revision, kit });
  const files = recordFiles(result.record, kit.explain);
  assert.match(files['explain.txt'], /publish@1 = 40131781029 {2}in force\n {6}based on candidates@1, runs@1/);
  assert.match(files['explain.txt'], /release\.allowed = "allowed"/);
  assert.deepEqual(files['events.jsonl'].trim().split('\n').map(line => JSON.parse(line).event), ['run', 'choose_run', 'artifact', 'decide']);
});

test('a refusal is recorded too, and a disagreement with the legacy checks refuses', async () => {
  const release = clone(history.releases.find(entry => entry.tag === 'v1.3.0'));
  release.runs[0].conclusion = 'failure';
  let record = null;
  await assert.rejects(selectRuntimeCandidate({ api: github(release), repository, revision: release.revision, kit, onRecord: value => { record = value; } }),
    /The newest Runtime run for 753c8e7\w+ \(40131781029\) has not succeeded\./);
  assert.match(recordFiles(record, kit.explain)['explain.txt'], /Refused choose_run: The newest Runtime run has not succeeded\./);
  // A gate program that allowed what the legacy checks refuse is caught.
  const source = (await readFile(new URL('./release-gate.cav', import.meta.url), 'utf8'))
    .replace(/\non choose_run when run_completed == 0 or run_success == 0 reject "[^"]*";/, '');
  const lenient = { runtime: { open: () => kit.runtime.open(source) } };
  await assert.rejects(selectRuntimeCandidate({ api: github(release), repository, revision: release.revision, kit: lenient }),
    /The Caveat release gate and the legacy checks disagree; nothing is published\./);
});
