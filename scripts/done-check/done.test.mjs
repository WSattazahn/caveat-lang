// The done check's host: the facts it gathers from GitHub and the verdict
// done.cav gives on them.
import assert from 'node:assert/strict';
import test from 'node:test';
import { checkOutcome, decide, gatherFacts, loadKit, report, SWEEP_CHECK, touchesRuntime } from './done.mjs';
import { CARRY_JOB, CARRY_NOTICE } from '../ci-reuse/reuse.mjs';

const kit = await loadKit();
const repository = 'WSattazahn/caveat-lang';
const head = 'e'.repeat(40);

function github({ behind = 0, runs = [], files = ['README.md'], pulls = [], annotations = {} } = {}) {
  return async url => {
    const path = url.replace(/[?&]per_page=100&page=(\d+)$/, '');
    const first = (/[?&]page=(\d+)$/.exec(url)?.[1] ?? '1') === '1';
    if (path === `/repos/${repository}/pulls/9`) return { number: 9, title: 't', head: { sha: head }, base: { ref: 'main' } };
    if (path === `/repos/${repository}/compare/main...${head}`) return { behind_by: behind };
    if (path === `/repos/${repository}/commits/${head}/check-runs?filter=latest`) return { check_runs: first ? runs : [] };
    if (path === `/repos/${repository}/pulls/9/files`) return first ? files.map(filename => ({ filename })) : [];
    const notes = /^\/repos\/[^/]+\/[^/]+\/check-runs\/(\d+)\/annotations$/.exec(path);
    if (notes && annotations[notes[1]]) return first ? annotations[notes[1]] : [];
    if (url === `/repos/${repository}/commits?sha=main&path=ledger/events.jsonl&per_page=1`) return [{ sha: 'f'.repeat(40), commit: { committer: { date: '2026-10-24T00:00:00Z' } } }];
    if (path === `/repos/${repository}/commits/${'f'.repeat(40)}/pulls`) return [{ number: 320, merged_at: '2026-10-25T00:39:00Z', base: { ref: 'main' } }];
    if (path === `/repos/${repository}/pulls?state=closed&base=main&sort=updated&direction=desc`) return first ? pulls : [];
    throw new Error(`unexpected ${url}`);
  };
}
const run = (name, status = 'completed', conclusion = 'success') => ({ name, status, conclusion });
const merged = (number, at) => ({ number, merged_at: at, updated_at: at });

test('check outcomes and runtime paths', () => {
  assert.equal(checkOutcome(run('a')), 'passed');
  assert.equal(checkOutcome(run('a', 'completed', 'skipped')), 'passed');
  assert.equal(checkOutcome(run('a', 'completed', 'cancelled')), 'failed');
  assert.equal(checkOutcome(run('a', 'completed', 'timed_out')), 'failed');
  assert.equal(checkOutcome(run('a', 'queued', null)), 'running');
  assert.ok(touchesRuntime('runtime/src/reactive.rs') && touchesRuntime('spec/caveat-holds-0.1.md'));
  assert.ok(!touchesRuntime('kit/lib/session.mjs') && !touchesRuntime('docs/runtime/x.md'));
});

test('a green PR with current main is ready, and the ledger debt is named without blocking', async () => {
  const pulls = [merged(320, '2026-10-25T00:39:00Z'), merged(321, '2026-10-25T01:19:00Z'), { number: 324, merged_at: null, updated_at: '2026-10-25T02:00:00Z' },
    merged(322, '2026-10-25T02:10:00Z'), merged(300, '2026-10-20T00:00:00Z')];
  const facts = await gatherFacts({ api: github({ runs: [run('core semantics'), run(SWEEP_CHECK)], pulls }), repository, number: 9 });
  assert.deepEqual(facts.ledger.owed, [321, 322]);
  const verdict = await decide({ facts, kit });
  assert.equal(verdict.ready, true);
  assert.equal(verdict.reasons.verdict, 'ready; the ledger owes events for PRs merged since it last changed');
  assert.match(report(facts, verdict), /ledger: 2 merged PRs owe ledger events \(#321, #322, merged since #320 changed it\)/);
  assert.match(verdict.explanation, /done@1 = null {2}in force/);
});

test('behind main, a failed check, or a runtime change without a green sweep is not ready', async () => {
  const cases = [
    [{ behind: 2, runs: [run('core semantics')] }, /main: the head does not contain current main: merge main in \(2 commits behind main\)/],
    [{ runs: [run('core semantics'), run('native runtime', 'completed', 'failure')] }, /checks: 1 failed, 0 still running, of 2: native runtime \(failed\)/],
    [{ runs: [run('core semantics'), run(SWEEP_CHECK, 'in_progress', null)], files: ['runtime/src/lib.rs'] }, /sweep: needed, and not green on this head/],
  ];
  for (const [state, expected] of cases) {
    const facts = await gatherFacts({ api: github(state), repository, number: 9 });
    const verdict = await decide({ facts, kit });
    assert.equal(verdict.ready, false);
    assert.match(report(facts, verdict), expected);
  }
});

test('a head that carried round 1 over is ready on the same checks, and the report says what was carried', async () => {
  const carry = { ...run(CARRY_JOB), html_url: 'https://github.com/o/r/actions/runs/2/job/9' };
  const noted = (id, name, notes) => ({ ...run(name), id, output: { annotations_count: notes } });
  const runs = [run('core semantics'), noted(11, 'native runtime', 1), noted(12, SWEEP_CHECK, 2), carry,
    // Re-run in full after the carry: its latest check run has no notice.
    noted(13, 'native runtime, reactive-only and diagnostic features', 0),
    // A notice, but not the carry step's.
    noted(14, 'Lean runtime comparison', 1),
    // The carry step's notice on a check that cannot carry.
    noted(15, 'Evidence games browser QA', 1)];
  const carried = { annotation_level: 'notice', title: CARRY_NOTICE, message: 'round 1 ran this job' };
  const annotations = { 11: [carried], 12: [{ annotation_level: 'warning', title: 'retried', message: 'x' }, carried],
    14: [{ annotation_level: 'notice', title: 'Lean report', message: 'x' }], 15: [carried] };
  const facts = await gatherFacts({ api: github({ runs, files: ['runtime/src/lib.rs'], annotations }), repository, number: 9 });
  assert.deepEqual(facts.carried, { from: carry.html_url, names: ['native runtime', SWEEP_CHECK] });
  const verdict = await decide({ facts, kit });
  assert.equal(verdict.ready, true);
  const text = report(facts, verdict);
  assert.match(text, /checks: all 7 check runs passed; 2 of them carried round 1's result over to this head without running \(round-1 carry: https:\/\/github\.com\/o\/r\/actions\/runs\/2\/job\/9\)/);
  assert.match(text, /sweep: needed, and green, carried over from round 1, whose tree differs only by the ledger lines this head adds/);
  // A skipped round-1 carry check means everything ran on this head.
  const full = await gatherFacts({ api: github({ runs: [run('native runtime'), run(SWEEP_CHECK), run(CARRY_JOB, 'completed', 'skipped')], files: ['runtime/src/lib.rs'] }), repository, number: 9 });
  assert.equal(full.carried, null);
  assert.doesNotMatch(report(full, await decide({ facts: full, kit })), /carried/);
});
