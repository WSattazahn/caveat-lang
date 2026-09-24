import test from 'node:test';
import assert from 'node:assert/strict';
import { selectCandidate, recheckCandidate, validateBuildInfo } from './pages-gate.mjs';

const repository = 'example/caveat';
const sha = 'a'.repeat(40);
const otherSha = 'b'.repeat(40);

function fixture() {
  const state = {
    main: sha,
    run: { id: 23, run_attempt: 1, workflow_id: 7, path: '.github/workflows/runtime.yml',
      repository: { full_name: repository }, head_repository: { full_name: repository },
      event: 'push', head_branch: 'main', head_sha: sha, status: 'completed', conclusion: 'success' },
    artifacts: [{ id: 45, name: 'browser-dist', expired: false,
      workflow_run: { id: 23, head_sha: sha } }],
    jobs: [{ name: 'Legacy Door 3D QA', conclusion: 'success' }],
    runs: [{ id: 23 }],
  };
  const requests = [];
  const api = async path => {
    requests.push(path);
    const url = new URL(path, 'https://api.github.com');
    const base = `/repos/${repository}`;
    if (url.pathname === `${base}/git/ref/heads/main`) return { object: { sha: state.main } };
    if (url.pathname === `${base}/actions/workflows/runtime.yml`) return { id: 7 };
    if (url.pathname === `${base}/actions/runs/23`) return structuredClone(state.run);
    if (url.pathname === `${base}/actions/runs/24`) return { ...state.run, id: 24, status: 'in_progress', conclusion: null };
    let values, key;
    if (url.pathname === `${base}/actions/workflows/runtime.yml/runs`) {
      assert.equal(url.searchParams.get('head_sha'), state.main);
      assert.equal(url.searchParams.get('branch'), 'main');
      assert.equal(url.searchParams.get('event'), 'push');
      assert.equal(url.searchParams.has('status'), false, 'must not hide failed or pending runs');
      values = state.runs; key = 'workflow_runs';
    } else if (url.pathname === `${base}/actions/runs/23/artifacts`) {
      values = state.artifacts; key = 'artifacts';
    } else if (url.pathname === `${base}/actions/runs/23/jobs`) {
      assert.equal(url.searchParams.get('filter'), 'latest');
      values = state.jobs; key = 'jobs';
    } else throw new Error(`Unexpected API request: ${path}`);
    const page = Number(url.searchParams.get('page'));
    return { [key]: structuredClone(values.slice((page - 1) * 100, page * 100)) };
  };
  const args = { api, repository, eventName: 'workflow_run',
    event: { action: 'completed', workflow_run: { id: 23 } }, ref: 'refs/heads/main' };
  return { state, requests, args };
}

test('selects the exact successful main run and immutable artifact, preserving regression scope', async () => {
  const { args, state } = fixture();
  assert.deepEqual(await selectCandidate(args), {
    source_sha: sha, runtime_run_id: 23, runtime_run_attempt: 1, artifact_id: 45, legacy_door: true,
  });
  state.jobs[0].conclusion = 'skipped';
  assert.equal((await selectCandidate(args)).legacy_door, false);
});

test('fresh API state overrides a stale successful event', async () => {
  const { args, state } = fixture();
  args.event.workflow_run.conclusion = 'success';
  state.run.status = 'in_progress';
  state.run.conclusion = null;
  await assert.rejects(selectCandidate(args), /complete runtime workflow must have succeeded/);
});

test('refuses failed, cancelled and unfinished runtime workflows', async () => {
  for (const [status, conclusion] of [['completed', 'failure'], ['completed', 'cancelled'],
    ['completed', 'timed_out'], ['in_progress', null], ['queued', null]]) {
    const { args, state } = fixture();
    Object.assign(state.run, { status, conclusion });
    await assert.rejects(selectCandidate(args), /complete runtime workflow must have succeeded/);
  }
});

test('refuses pull requests, foreign repositories, other branches and other workflows', async () => {
  for (const change of [
    { event: 'pull_request' }, { head_branch: 'feature' }, { workflow_id: 99 },
    { path: '.github/workflows/impostor.yml' },
    { repository: { full_name: 'other/repo' } },
    { head_repository: { full_name: 'other/caveat' } },
  ]) {
    const { args, state } = fixture();
    Object.assign(state.run, change);
    await assert.rejects(selectCandidate(args), /Only runtime runs|runtime workflow|this repository/);
  }
});

test('refuses stale main, unrelated events and premature completion events', async () => {
  const { args, state } = fixture();
  state.main = otherSha;
  await assert.rejects(selectCandidate(args), /main has moved/);
  state.main = sha;
  await assert.rejects(selectCandidate({ ...args, eventName: 'push' }), /requires a completed runtime/);
  await assert.rejects(selectCandidate({ ...args, event: { ...args.event, action: 'requested' } }), /has not completed/);
});

test('manual redeploy resolves current main and never falls back past a newer pending run', async () => {
  const { args, state } = fixture();
  args.eventName = 'workflow_dispatch';
  assert.equal((await selectCandidate(args)).source_sha, sha);
  state.runs.push({ id: 24 });
  await assert.rejects(selectCandidate(args), /complete runtime workflow must have succeeded/);
  state.runs = [];
  await assert.rejects(selectCandidate(args), /no runtime push run/);
  await assert.rejects(selectCandidate({ ...args, ref: 'refs/heads/feature' }), /must run from main/);
});

test('delayed completion events cannot deploy after a newer runtime run for the same SHA', async () => {
  const { args, state } = fixture();
  state.runs.push({ id: 24 });
  await assert.rejects(selectCandidate(args), /superseded by a newer run/);
  await assert.rejects(recheckCandidate({ api: args.api, repository, sourceSha: sha, runId: 23, runAttempt: 1 }),
    /superseded while deployment was queued/);
});

test('missing, expired, ambiguous and mismatched artifacts all refuse deployment', async () => {
  for (const mutate of [
    state => { state.artifacts = []; },
    state => { state.artifacts.push(structuredClone(state.artifacts[0])); },
    state => { state.artifacts[0].expired = true; },
    state => { state.artifacts[0].workflow_run.head_sha = otherSha; },
    state => { state.artifacts[0].workflow_run.id = 99; },
  ]) {
    const { args, state } = fixture(); mutate(state);
    await assert.rejects(selectCandidate(args), /exactly one|expired|does not match/);
  }
});

test('selection paginates artifact and regression-job metadata', async () => {
  const { args, state, requests } = fixture();
  state.artifacts.unshift(...Array.from({ length: 100 }, (_, i) => ({ name: `unrelated-${i}` })));
  state.jobs.unshift(...Array.from({ length: 100 }, (_, i) => ({ name: `other job ${i}` })));
  assert.equal((await selectCandidate(args)).artifact_id, 45);
  assert.ok(requests.some(path => path.includes('/artifacts?per_page=100&page=2')));
  assert.ok(requests.some(path => path.includes('/jobs?filter=latest&per_page=100&page=2')));
});

test('missing, ambiguous or failed legacy scope cannot silently skip its live QA', async () => {
  for (const jobs of [[], [{ name: 'Legacy Door 3D QA', conclusion: 'failure' }],
    [{ name: 'Legacy Door 3D QA', conclusion: 'success' }, { name: 'Legacy Door 3D QA', conclusion: 'skipped' }]]) {
    const { args, state } = fixture(); state.jobs = jobs;
    await assert.rejects(selectCandidate(args), /no conclusive Legacy Door regression scope/);
  }
});

test('final check catches main moving and runtime being rerun while deployment was queued', async () => {
  const { args, state } = fixture();
  const selected = { api: args.api, repository, sourceSha: sha, runId: 23, runAttempt: 1 };
  await recheckCandidate(selected);
  state.main = otherSha;
  await assert.rejects(recheckCandidate(selected), /main moved after artifact selection/);
  state.main = sha;
  state.run.status = 'in_progress'; state.run.conclusion = null;
  await assert.rejects(recheckCandidate(selected), /complete runtime workflow must have succeeded/);
  state.run.status = 'completed'; state.run.conclusion = 'success'; state.run.run_attempt = 2;
  await assert.rejects(recheckCandidate(selected), /rerun after artifact selection/);
});

test('only clean compiled Linux metadata for the exact source is accepted', () => {
  const info = { revision: sha, clean: true, compiled: true, host: 'x86_64-unknown-linux-gnu' };
  assert.doesNotThrow(() => validateBuildInfo(info, sha));
  for (const change of [{ revision: otherSha }, { clean: false }, { compiled: false },
    { host: 'x86_64-pc-windows-msvc' }]) {
    assert.throws(() => validateBuildInfo({ ...info, ...change }, sha), /build-info revision|clean, compiled|Linux runtime/);
  }
  assert.throws(() => validateBuildInfo(null, sha), /build-info revision/);
});
