import test from 'node:test';
import assert from 'node:assert/strict';
import { Ledger } from './ledger.js';

const mk = () => {
  let t = 0; const l = new Ledger({ now: () => `2026-01-01T00:00:${String(t++).padStart(2, '0')}Z` });
  l.approve({ id: 'a1', action: 'deploy', reason: 'tests green', approver: 'bot',
    evidence: [{ id: 'e1', source: 'tool', claim: 'pass', config: { x: 1 } }, { id: 'e2', source: 'lint', claim: 'clean' }] });
  return l;
};

test('holds until evidence is retracted, original rationale preserved', () => {
  const l = mk();
  assert.equal(l.report('a1').holds, true);
  l.retract({ evidenceId: 'e1', reason: 'wrong config', source: 'maintainer' });
  const r = l.report('a1');
  assert.equal(r.holds, false);
  assert.equal(r.reason, 'tests green');
  assert.equal(r.evidence[0].status, 'retracted');
  assert.equal(r.evidence[1].status, 'standing');
  assert.equal(l.undermined().length, 1);
});

test('reaffirmation restores; asOf shows what was known then', () => {
  const l = mk();
  l.retract({ evidenceId: 'e1', reason: 'wrong config', source: 'm' });   // t=1
  l.reaffirm({ evidenceId: 'e1', reason: 'rerun ok', source: 'm' });      // t=2
  assert.equal(l.report('a1').holds, true);
  assert.equal(l.report('a1', { asOf: '2026-01-01T00:00:01Z' }).holds, false);
  assert.equal(l.report('a1', { asOf: '2026-01-01T00:00:00Z' }).holds, true);
});

test('validation and tamper detection', () => {
  const l = mk();
  assert.throws(() => l.retract({ evidenceId: 'nope', reason: 'x' }));
  assert.throws(() => l.approve({ id: 'b', evidence: [] }));
  assert.equal(l.verify(), true);
});
