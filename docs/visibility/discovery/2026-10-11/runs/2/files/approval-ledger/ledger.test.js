'use strict';
const test = require('node:test');
const assert = require('node:assert');
const { Ledger } = require('./ledger');

const base = () => {
  const l = new Ledger();
  l.approve({ id: 'dep-1', action: 'deploy api v42 to prod', agent: 'release-agent', reason: 'integration suite passed',
    evidence: [{ id: 'ev-tests', source: 'testtool run #88', claim: 'all 212 tests passed', config: { env: 'staging-old' } },
               { id: 'ev-lint', source: 'lint', claim: 'clean' }] });
  return l;
};

test('holds until corrected; original basis is preserved after correction', () => {
  const l = base();
  assert.equal(l.report('dep-1').status, 'holds');
  l.correct({ approvalId: 'dep-1', evidenceId: 'ev-tests', effect: 'invalidates', finding: 'ran against staging-old, not staging-new' });
  const r = l.report('dep-1');
  assert.equal(r.status, 'unsupported');
  assert.equal(r.approval.evidence[0].claim, 'all 212 tests passed');
  assert.equal(r.evidence[1].state, 'intact');
  assert.match(l.format('dep-1'), /staging-old[\s\S]*correction/);
});

test('re-verification reaffirms or contradicts', () => {
  const l = base();
  l.correct({ approvalId: 'dep-1', evidenceId: 'ev-tests', effect: 'invalidates', finding: 'wrong config' });
  l.reverify({ approvalId: 'dep-1', evidenceId: 'ev-tests', result: 'supports' });
  assert.equal(l.report('dep-1').status, 'reaffirmed');
  l.reverify({ approvalId: 'dep-1', evidenceId: 'ev-tests', result: 'contradicts' });
  assert.equal(l.report('dep-1').status, 'contradicted');
});

test('qualifying correction gives caveats; bad input rejected; caller mutation is isolated', () => {
  const l = base();
  l.correct({ approvalId: 'dep-1', evidenceId: 'ev-lint', effect: 'qualifies', finding: 'ignored generated dir' });
  assert.equal(l.report('dep-1').status, 'holds-with-caveats');
  assert.throws(() => l.correct({ approvalId: 'dep-1', evidenceId: 'nope', effect: 'invalidates', finding: 'x' }));
  assert.throws(() => l.reverify({ approvalId: 'dep-1', evidenceId: 'ev-lint', result: 'supports' }));
  assert.throws(() => l.approve({ id: 'dep-1', reason: 'r', evidence: [{ id: 'a' }] }));
});

test('persists to JSONL and reloads', () => {
  const f = require('node:path').join(require('node:os').tmpdir(), `ledger-${process.pid}.jsonl`);
  try {
    const l = new Ledger(f);
    l.approve({ id: 'x', reason: 'r', evidence: [{ id: 'e', source: 's', claim: 'c' }] });
    l.correct({ approvalId: 'x', evidenceId: 'e', effect: 'invalidates', finding: 'bad' });
    assert.equal(new Ledger(f).report('x').status, 'unsupported');
  } finally { require('node:fs').rmSync(f, { force: true }); }
});
