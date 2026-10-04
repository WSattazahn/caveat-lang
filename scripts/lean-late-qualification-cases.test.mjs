import assert from 'node:assert/strict';
import test from 'node:test';
import { LATE_SCHEMA, LATE_STATES, COMMITMENTS, LATE_CAVEATS, validateLateCase, renderLateCase, lateCases } from './lean-late-qualification-cases.mjs';

function sample() {
  return {
    schema: LATE_SCHEMA,
    id: 'late-render-witness',
    seed: { a: 3, b: -2, g: 1 },
    writes: [{ body: ['a', 'b'], guard: 'g', citations: ['a'] }],
    commits: [{ using: ['a', 'b'], guard: null, retaining: ['cb'] }, { using: null, guard: 'g', retaining: [] }],
    qualifications: [{ evidence: 'ea', caveat: 'late', guard: 'g' }, { evidence: 'eu', caveat: 'other', guard: null }],
  };
}

function rejected(change) {
  const value = sample();
  change(value);
  assert.throws(() => validateLateCase(value), TypeError);
}

test('late cases are valid, unique, deterministic, freshly allocated and cover a refusal', () => {
  const first = lateCases();
  const second = lateCases();
  assert.equal(first.length, 9);
  assert.equal(new Set(first.map(({ id }) => id)).size, first.length);
  assert.deepEqual(first, second);
  assert.deepEqual(LATE_STATES, ['a', 'b', 'g', 'x']);
  assert.deepEqual(COMMITMENTS, ['plan', 'hold', 'keep']);
  assert.ok(first.some(value => value.qualifications.some(q => q.evidence === 'eu')));
  assert.ok(first.some(value => value.seed.g === 0 && value.qualifications.some(q => q.guard === 'g')));
  for (const value of first) {
    assert.equal(validateLateCase(value), value);
    assert.equal(renderLateCase(value).events.length, 3 + value.qualifications.length);
  }
  first[0].qualifications[0].caveat = 'meta';
  assert.deepEqual(lateCases(), second);
});

test('the renderer declares the qualifier chain the harness reports to the model', () => {
  const value = sample();
  const before = structuredClone(value);
  const { source, events, qualifyFrom } = renderLateCase(value);
  assert.deepEqual(value, before);
  assert.deepEqual(events, ['seed', 'prepare', 'decide', 'learn0', 'learn1']);
  assert.equal(events[qualifyFrom], 'learn0');
  const lines = source.trimEnd().split('\n');
  for (const line of ['meta qualifies late;', 'deep qualifies meta;', 'on never reveal eu;',
    'on prepare when g != 0 set x = a + b because a;',
    'on decide commit plan because enough using a + b retaining cb;',
    'on decide when g != 0 commit hold because enough;',
    'on learn0 when g != 0 qualify ea with late;',
    'on learn1 qualify eu with other;']) assert.ok(lines.includes(line), line);
  assert.deepEqual(LATE_CAVEATS, { late: ['meta', 'deep'], meta: ['deep'], other: [] });
});

test('late fixtures refuse unknown fields, foreign names and out-of-bounds lists', () => {
  rejected(value => { value.extra = 1; });
  rejected(value => { value.schema = 'caveat-guard-citation/0.2'; });
  rejected(value => { value.id = 'Upper'; });
  rejected(value => { value.seed.a = 1001; });
  rejected(value => { value.seed.g = 2; });
  rejected(value => { value.writes = [value.writes[0], value.writes[0], value.writes[0]]; });
  rejected(value => { value.writes[0].body = ['x']; });
  rejected(value => { value.commits = []; });
  rejected(value => { value.commits = Array(4).fill(value.commits[1]); });
  rejected(value => { value.commits[0].using = []; });
  rejected(value => { value.commits[0].retaining = ['invented']; });
  rejected(value => { value.qualifications = []; });
  rejected(value => { value.qualifications[0].evidence = 'ez'; });
  rejected(value => { value.qualifications[0].caveat = 'ca'; });
  rejected(value => { value.qualifications[0].guard = 'z'; });
  rejected(value => { value.qualifications[0].extra = true; });
  rejected(value => { value.commits[0].retaining = [, 'cb']; });
});
