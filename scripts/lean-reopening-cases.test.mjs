import assert from 'node:assert/strict';
import test from 'node:test';
import { REOPEN_SCHEMA, REOPEN_STATES, REOPEN_COMMITMENTS, EVIDENCE_CAVEATS, validateReopenCase, renderReopenCase, reopenCases } from './lean-reopening-cases.mjs';

function sample() {
  return {
    schema: REOPEN_SCHEMA,
    id: 'reopen-render-witness',
    seed: { a: 3, b: -2, g: 1 },
    commits: [{ using: ['a', 'b'], retaining: ['late'] }, { using: null, retaining: [] }],
    reopenings: [{ commitment: 'plan', evidence: 'eb', guard: 'g' }, { commitment: 'keep', evidence: 'eu', guard: null }],
  };
}

function rejected(change) {
  const value = sample();
  change(value);
  assert.throws(() => validateReopenCase(value), TypeError);
}

test('reopening cases are valid, unique, deterministic, freshly allocated and cover both refusals', () => {
  const first = reopenCases();
  const second = reopenCases();
  assert.equal(first.length, 9);
  assert.equal(new Set(first.map(({ id }) => id)).size, first.length);
  assert.deepEqual(first, second);
  assert.deepEqual(REOPEN_STATES, ['a', 'b', 'g']);
  assert.deepEqual(REOPEN_COMMITMENTS, ['plan', 'hold', 'keep']);
  assert.ok(first.some(value => value.reopenings.some(r => r.evidence === 'eu')));
  assert.ok(first.some(value => value.reopenings.some(r => !REOPEN_COMMITMENTS.slice(0, value.commits.length).includes(r.commitment))));
  assert.ok(first.some(value => value.seed.g === 0 && value.reopenings.some(r => r.guard === 'g')));
  for (const value of first) {
    assert.equal(validateReopenCase(value), value);
    assert.equal(renderReopenCase(value).events.length, 2 + value.reopenings.length);
  }
  first[0].reopenings[0].evidence = 'eg';
  assert.deepEqual(reopenCases(), second);
});

test('the renderer declares the caveat chain the harness reports to the model', () => {
  const value = sample();
  const before = structuredClone(value);
  const { source, events, reopenFrom } = renderReopenCase(value);
  assert.deepEqual(value, before);
  assert.deepEqual(events, ['seed', 'decide', 'doubt0', 'doubt1']);
  assert.equal(events[reopenFrom], 'doubt0');
  const lines = source.trimEnd().split('\n');
  for (const line of ['cb qualifies eb;', 'late qualifies eb;', 'meta qualifies late;', 'on never reveal eu;',
    'on decide commit plan because enough using a + b retaining late;',
    'on decide commit hold because enough;',
    'on never commit keep because enough;',
    'on doubt0 when g != 0 reopen plan because eb;',
    'on doubt1 reopen keep because eu;']) assert.ok(lines.includes(line), line);
  assert.deepEqual(EVIDENCE_CAVEATS, { ea: ['ca'], eb: ['cb', 'late', 'meta'], eg: ['cg'], eu: [] });
});

test('reopening fixtures refuse unknown fields, foreign names and out-of-bounds lists', () => {
  rejected(value => { value.extra = 1; });
  rejected(value => { value.schema = 'caveat-late-qualification/0.1'; });
  rejected(value => { value.id = 'Upper'; });
  rejected(value => { value.seed.a = 1001; });
  rejected(value => { value.seed.g = 2; });
  rejected(value => { value.commits = []; });
  rejected(value => { value.commits = Array(4).fill(value.commits[1]); });
  rejected(value => { value.commits[0].using = []; });
  rejected(value => { value.commits[0].using = ['x']; });
  rejected(value => { value.commits[0].retaining = ['invented']; });
  rejected(value => { value.commits[0].guard = null; });
  rejected(value => { value.reopenings = []; });
  rejected(value => { value.reopenings = Array(5).fill(value.reopenings[0]); });
  rejected(value => { value.reopenings[0].commitment = 'other'; });
  rejected(value => { value.reopenings[0].evidence = 'ez'; });
  rejected(value => { value.reopenings[0].guard = 'a'; });
  rejected(value => { value.reopenings[0].extra = true; });
  rejected(value => { value.commits[0].retaining = [, 'late']; });
});
