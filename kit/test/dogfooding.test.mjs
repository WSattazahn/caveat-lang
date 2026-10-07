import assert from 'node:assert/strict';
import test from 'node:test';
import { loadRuntimeFromDirectory, defaultRuntimeDirectory } from '../lib/node.mjs';
import { explain, dependents, formatExplanation, formatDependents } from '../lib/explain.mjs';

const runtime = await loadRuntimeFromDirectory(process.env.CAVEAT_DOGFOOD_RUNTIME ?? defaultRuntimeDirectory());
const source = `claim safe; claim bad;
evidence check from "check"; evidence correction from "correction";
evidence unrelated from "unrelated";
caveat doubt consequence material;
event decide; event correct;
on decide reveal check supports safe;
on decide commit proceed because enough using qualified(1, check) retaining doubt;
on correct reveal correction opposes safe;
on correct withdraw check because correction;
on correct when committed(proceed) and not reopened(proceed) reopen proceed because correction;
`;

function withSession(events, body, program = source) {
  const session = runtime.open(program);
  try {
    for (const event of events) assert.equal(session.dispatch(event).outcome, 'accepted', event);
    body(session.snapshot());
  } finally { session.close(); }
}

test('D001: a plain commitment is explained and remains discoverable after reopening', () => {
  for (const [events, status, value] of [
    [['decide'], 'in force', 1],
    [['decide', 'correct'], 'reopened', 1],
  ]) withSession(events, snapshot => {
    const before = structuredClone(snapshot);
    const report = explain(snapshot);
    const decision = report.decisions.find(item => item.name === 'proceed');
    assert.equal(decision.kind, 'plain');
    assert.equal(decision.revisions[0].status, status);
    assert.equal(decision.revisions[0].value, value);
    assert.match(formatExplanation(report), /proceed: plain commitment/);
    const reason = 'check';
    assert.equal(dependents(snapshot, reason).decisions[0].id, 'proceed');
    assert.deepEqual(snapshot, before, 'reporting must not mutate decisions');
  });
});

test('D002: dependency queries distinguish withdrawing a subject from using it as a reason', () => {
  withSession(['decide', 'correct'], snapshot => {
    const expected = { evidence: 'check', because: 'correction', sequence: 2, event: 'correct' };
    const report = dependents(snapshot, 'correction');
    assert.deepEqual(report.withdrawals, []);
    assert.deepEqual(report.reasonForWithdrawals, [expected]);
    assert.deepEqual(dependents(snapshot, 'check').withdrawals, [expected]);
    assert.deepEqual(dependents(snapshot, 'unrelated').reasonForWithdrawals, []);
    assert.match(formatDependents(report), /Withdrawals resting on this reason\n  check withdrawn/);
  });
});

test('D003: retaining an uncarried caveat never invents evidence carrying it', () => {
  withSession(['decide'], snapshot => {
    assert.equal(snapshot.relations.some(edge => edge.relation === 'qualifies' && edge.from === 'doubt'), false);
    const text = formatExplanation(explain(snapshot));
    assert.match(text, /based on check\n      retaining: doubt/);
    assert.doesNotMatch(text, /based on check \(caveats: doubt\)/);
    const report = dependents(snapshot, 'doubt');
    assert.equal(report.decisions[0].basis, 'retained');
    assert.deepEqual(report.reasonForWithdrawals, []);
    assert.deepEqual(report.claims, []);
    assert.match(formatDependents(report), /retaining: doubt/);
    assert.doesNotMatch(formatDependents(report), /evidence with/);
  });
});

test('D005: report actual supporting and opposing relations without interpreting them as truth', () => {
  withSession(['decide', 'correct'], snapshot => {
    assert.deepEqual(dependents(snapshot, 'check').claims, [{ evidence: 'check', relation: 'supports', claim: 'safe' }]);
    assert.deepEqual(dependents(snapshot, 'correction').claims, [{ evidence: 'correction', relation: 'opposes', claim: 'safe' }]);
  });
});

test('old saved reports without the additive fields still format', () => {
  const old = { subject: 'check', kind: 'evidence', sequence: 0, decisions: [], changes: [], values: [], displayed: [] };
  assert.match(formatDependents(old), /Decisions\n  nothing/);
});
