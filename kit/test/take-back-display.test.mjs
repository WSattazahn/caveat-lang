// D029: explain, review and dependents show a take-back as the runtime
// records it (spec/caveat-take-back-0.1.md). The relation a take-back marks
// stays in the view's `relations`, so a display that reads relations alone
// shows the caveat as if the evidence still carried it. From the AI change
// review demo's friction item 2.
import assert from 'node:assert/strict';
import test from 'node:test';
import { dependents, explain, formatDependents, formatExplanation } from '../lib/explain.mjs';
import { formatReview, review } from '../lib/review.mjs';
import { real } from './helpers.mjs';

const changeReview = `
claim sound;
caveat unreviewed consequence material;
evidence account from "the AI's account of a change";
evidence review from "a human review";
decisions ok limit 4;
decisions shipped limit 1 irreversible;
event propose; event accept; event review_it; event release;
on propose reveal account supports sound;
on propose qualify account with unreviewed;
on accept commit ok because enough using qualified(1, account);
on review_it reveal review supports sound;
on review_it unqualify account with unreviewed;
on review_it reopen ok because review;
on release commit shipped because enough using latest(ok);
`;
const events = ['propose', 'accept', 'review_it', 'accept', 'release'].map(event => ({ event, payload: {} }));

function run() {
  const session = real.open(changeReview);
  const sent = events.map(({ event, payload }) => {
    const { snapshot: _, ...outcome } = session.dispatch(event, payload);
    return { event, payload, outcome };
  });
  const snapshot = session.snapshot();
  session.close();
  return { snapshot, sent };
}

test('explain lists only the caveats evidence carries now, and the take-back beside it', () => {
  const { snapshot, sent } = run();
  const report = explain(snapshot, sent);
  const account = report.evidence.find(item => item.id === 'account');
  assert.deepEqual(account.caveats, []);
  assert.deepEqual(account.taken_back, [{ caveat: 'unreviewed', sequence: 3, event: 'review_it' }]);
  assert.equal(report.evidence.find(item => item.id === 'review').taken_back, undefined);
  const [ok] = report.decisions;
  const [first, second] = ok.revisions;
  // Frozen grounds are left as they were (Take-back 0.1, "What was already recorded is left alone").
  assert.deepEqual(first.grounds.caveats, ['unreviewed']);
  assert.deepEqual(second.grounds.caveats, []);
  assert.deepEqual(second.retained, ['unreviewed']);
  const text = formatExplanation(report, 'change review');
  assert.match(text, /^  account supports sound\n    unreviewed taken back at #3 review_it$/m);
  assert.doesNotMatch(text, /account supports sound  caveats/);
  // The commit made while account carried the caveat still shows it carried.
  assert.match(text, /ok@1 = 1  superseded\n      based on account \(caveats: unreviewed\)/);
  assert.match(text, /ok@2 = 1  in force\n      based on account\n(.*\n)*?      also carrying, through its lineage: unreviewed/);
});

test('review says the reviewed revision carries the caveat through its lineage, not its grounds', () => {
  const session = real.open(changeReview);
  try {
    const report = review(() => real.open(changeReview), events, 'ok@2');
    assert.deepEqual(report.grounds, [{ evidence: 'account', caveats: [] }]);
    assert.deepEqual(report.retained, ['unreviewed']);
    const text = formatReview(report, 'change review');
    assert.match(text, /^  based on account\n  also carrying, through its lineage: unreviewed$/m);
    const first = formatReview(review(() => real.open(changeReview), events, 'ok@1'), 'change review');
    assert.match(first, /^  based on account \(caveats: unreviewed\)$/m);
  } finally { session.close(); }
});

test('dependents lists the take-back and tells a kept caveat from a retained one', () => {
  const { snapshot } = run();
  const report = dependents(snapshot, 'unreviewed');
  assert.deepEqual(report.taken_back, [{ evidence: 'account', caveat: 'unreviewed', sequence: 3, event: 'review_it' }]);
  // Nothing carries the caveat through a live relation now.
  assert.deepEqual(report.claims, []);
  assert.deepEqual(report.decisions.map(item => [item.id, item.basis]), [['ok@1', 'retained'], ['ok@2', 'kept']]);
  const text = formatDependents(report, 'change review', events.length);
  assert.match(text, /^Taken back\n  unreviewed from account at #3 review_it$/m);
  assert.match(text, /^  ok@2 = 1  in force  carrying through its lineage: unreviewed$/m);
  const evidence = dependents(snapshot, 'account');
  assert.deepEqual(evidence.taken_back, report.taken_back);
  assert.equal(dependents(snapshot, 'review').taken_back, undefined);
});
