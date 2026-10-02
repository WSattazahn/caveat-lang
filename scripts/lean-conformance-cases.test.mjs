import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import test from 'node:test';
import { SCHEMA, STATES, GENERATOR_SEED, MAX_CASE_BYTES, validateCase, renderCase, cases } from './lean-conformance-cases.mjs';

function sample() {
  return {
    schema: SCHEMA,
    id: 'render-witness',
    seed: { a: -3, b: 2, g: 1 },
    steps: [{ actions: [
      { target: 'x', body: ['a', 'a', 'b'], guard: 'g', citations: ['a', 'b'] },
      { target: 'y', body: [], guard: null, citations: [] },
      { target: 'x', body: ['b'], guard: null, citations: null },
    ] }],
  };
}

function rejected(change) {
  const value = sample();
  change(value);
  assert.throws(() => validateCase(value), TypeError);
}

test('fixed and generated cases are valid, unique, deterministic and freshly allocated', () => {
  const first = cases();
  const second = cases();
  assert.equal(first.length, 53);
  assert.equal(first.filter(({ id }) => id.startsWith('generated-')).length, 32);
  assert.equal(new Set(first.map(({ id }) => id)).size, first.length);
  assert.deepEqual(first, second);
  assert.equal(GENERATOR_SEED, 0x05eedca7);
  assert.deepEqual(STATES, ['a', 'b', 'g', 'x', 'y']);
  for (const value of first) {
    assert.equal(validateCase(value), value);
    assert.ok(Buffer.byteLength(JSON.stringify(value)) <= MAX_CASE_BYTES);
    assert.equal(renderCase(value).events.length, value.steps.length + 1);
  }
  first[0].seed.a = 900;
  first[0].steps[0].actions[0].body.push('g');
  assert.deepEqual(cases(), second);
});

test('renderer preserves order, arithmetic multiplicity and all citation forms exactly', () => {
  const value = sample();
  const before = structuredClone(value);
  assert.deepEqual(renderCase(value), {
    events: ['seed', 'step0'],
    source: [
      'evidence ea from "conformance a";',
      'evidence eb from "conformance b";',
      'evidence eg from "conformance guard";',
      'caveat ca consequence material;',
      'caveat cb consequence material;',
      'caveat cg consequence material;',
      'ca qualifies ea;',
      'cb qualifies eb;',
      'cg qualifies eg;',
      'state a = 0 min -1000000000 max 1000000000;',
      'state b = 0 min -1000000000 max 1000000000;',
      'state g = 0 min -1000000000 max 1000000000;',
      'state x = 0 min -1000000000 max 1000000000;',
      'state y = 0 min -1000000000 max 1000000000;',
      'event seed;',
      'event step0;',
      'on seed reveal eg;',
      'on seed reveal ea;',
      'on seed reveal eb;',
      'on seed set a = qualified(-3, ea);',
      'on seed set b = qualified(2, eb);',
      'on seed set g = qualified(1, eg);',
      'on step0 when g != 0 set x = a + a + b because a, b;',
      'on step0 set y = 0 because nothing;',
      'on step0 set x = b;',
      '',
    ].join('\n'),
  });
  assert.deepEqual(value, before);
});

test('stable mutation witnesses retain their exact source triggers and continuation steps', () => {
  const byId = new Map(cases().map((value) => [value.id, value]));
  assert.equal(byId.get('eager-cancellation').seed.a, 2);
  assert.equal(byId.get('eager-cancellation').seed.b, -2);
  assert.match(renderCase(byId.get('eager-cancellation')).source, /on step0 set x = a \+ b;/);
  assert.match(renderCase(byId.get('successful-guard')).source, /on step0 when g != 0 set x = a;/);
  assert.match(renderCase(byId.get('skipped-guard')).source, /on step1 when g != 0 set x = a because b;/);
  assert.match(renderCase(byId.get('invalid-citation')).source, /on step0 set x = a because b;/);
  assert.equal(byId.get('invalid-citation').steps.length, 2);
  assert.equal(byId.get('late-rejection').steps[0].actions.length, 2);
  assert.equal(byId.get('late-rejection').steps.length, 2);
  assert.match(renderCase(byId.get('self-citation-prewrite')).source,
    /on step0 set x = b;\non step0 set x = a because x;/);
});

test('unknown, missing and non-data fields fail at every object level', () => {
  const objects = [(value) => value, (value) => value.seed,
    (value) => value.steps[0], (value) => value.steps[0].actions[0]];
  for (const select of objects) {
    rejected((value) => { select(value).unexpected = true; });
    rejected((value) => { delete select(value)[Object.keys(select(value))[0]]; });
    rejected((value) => { select(value)[Symbol('hidden')] = 1; });
    rejected((value) => {
      const object = select(value);
      Object.defineProperty(object, Object.keys(object)[0], { get: () => 1 });
    });
  }
  for (const field of ['guard', 'citations']) {
    rejected((value) => { delete value.steps[0].actions[0][field]; });
    rejected((value) => { value.steps[0].actions[0][field] = undefined; });
  }
});

test('schema and identifiers are closed, bounded and safe for receipts', () => {
  for (const schema of ['', null, {}, `${SCHEMA}-future`]) rejected((value) => { value.schema = schema; });
  for (const id of ['', null, 2, 'Upper', '../escape', 'a\ncode', 'x'.repeat(65), 'x'.repeat(MAX_CASE_BYTES)]) {
    rejected((value) => { value.id = id; });
  }
  const value = sample();
  value.id = `a${'0'.repeat(63)}`;
  validateCase(value);
});

test('seeds require finite safe exact integers, including the declared endpoints', () => {
  for (const field of ['a', 'b', 'g']) {
    for (const bad of [NaN, Infinity, -Infinity, -0, 0.5, null, undefined, '1', 2n, 2 ** 53]) {
      rejected((value) => { value.seed[field] = bad; });
    }
  }
  for (const field of ['a', 'b']) {
    for (const bad of [-1001, 1001]) rejected((value) => { value.seed[field] = bad; });
    for (const valid of [-1000, 0, 1000]) {
      const value = sample(); value.seed[field] = valid; validateCase(value);
    }
  }
  for (const bad of [-1, 2]) rejected((value) => { value.seed.g = bad; });
  for (const valid of [0, 1]) { const value = sample(); value.seed.g = valid; validateCase(value); }
});

test('references cannot inject source and sparse lists are invalid', () => {
  for (const bad of ['', 'ea', 'A', 'a; on seed reject "injected"', null, {}, 0, undefined]) {
    rejected((value) => { value.steps[0].actions[0].target = bad; });
    if (bad !== null) rejected((value) => { value.steps[0].actions[0].guard = bad; });
    for (const field of ['body', 'citations']) {
      rejected((value) => { value.steps[0].actions[0][field] = [bad]; });
    }
  }
  rejected((value) => { value.steps[0].actions[0].body = Array(1); });
  rejected((value) => { value.steps[0].actions[0].citations = Array(1); });
  const invalid = sample(); invalid.steps[0].actions[0].target = 'x; reject "injected"';
  assert.throws(() => renderCase(invalid), TypeError);
});

test('step, action and reference bounds are enforced independently', () => {
  for (const bad of [null, {}, [], Array(9).fill(sample().steps[0]), Array(1)]) {
    rejected((value) => { value.steps = bad; });
  }
  for (const bad of [null, {}, [], Array(5).fill(sample().steps[0].actions[0]), Array(1)]) {
    rejected((value) => { value.steps[0].actions = bad; });
  }
  rejected((value) => { value.steps = Array.from({ length: 3 }, () => structuredClone(value.steps[0])); });
  for (const field of ['body', 'citations']) {
    for (const bad of [{}, 'a', Array(5).fill('a')]) {
      rejected((value) => { value.steps[0].actions[0][field] = bad; });
    }
  }
  rejected((value) => { value.steps[0].actions[0].body = null; });
  const edge = sample();
  edge.seed.a = 1000;
  const one = { actions: [{ target: 'a', body: ['a', 'a', 'a', 'a'], guard: null, citations: null }] };
  edge.steps = Array.from({ length: 8 }, () => structuredClone(one));
  validateCase(edge);
  assert.equal(renderCase(edge).events.at(-1), 'step7');
});

test('malformed containers fail instead of coercing values', () => {
  for (const bad of [null, undefined, 1, 'case', [], new Date()]) {
    assert.throws(() => validateCase(bad), TypeError);
  }
  rejected((value) => { value.seed = Object.create({ a: 1, b: 1, g: 1 }); });
  rejected((value) => { value.steps[0] = null; });
  rejected((value) => { value.steps[0].actions[0] = null; });
});

test('generated corpus content has a reviewable stable digest', () => {
  const generated = cases().filter(({ id }) => id.startsWith('generated-'));
  const digest = createHash('sha256').update(JSON.stringify(generated)).digest('hex');
  assert.equal(digest, '4c2376191129214acd361938ea9715295e5dbb8e95ff50f31b7505bc1a755c0b');
});
