// Validation and matching rules of Scenarios 0.1.
import test from 'node:test';
import assert from 'node:assert/strict';
import { ABSENT, expectAt, getPointer, match, parseScenarioFile, ScenarioFileError, validateScenarioFile } from '../lib/scenarios.mjs';

const file = (steps, extra = {}) => ({ schema: 'caveat-scenarios/0.1', source: 'p.cav', scenarios: [{ id: 'S1', title: 't', steps }], ...extra });
const send = { send: 'go' };
const invalid = (doc, pattern) => assert.throws(() => validateScenarioFile(doc), error => error instanceof ScenarioFileError && pattern.test(error.message), pattern);

test('valid files', () => {
  validateScenarioFile(file([send, { same_as: 'before', paths: ['/bindings'] }, { size: { snapshot: { max_growth: 10, since: 'before' } } }]));
  validateScenarioFile(file([{ checkpoint: 'c' }, send, { same_as: 'c', paths: [''] }, { same_as: 'initial', paths: ['/a~1b/~0'] }]));
  validateScenarioFile(file([{ send: 'go', payload: [1, null, 'x'], rejected: { origin: 'input', code: 'payload_invalid' }, repeat: 10000, note: 'n' }]));
  validateScenarioFile(file([{ expect: { '/a': { b: { $absent: true }, c: [{ $set: ['x', 'x'] }], d: { $exact: { $weird: 1 } } } } }]));
  validateScenarioFile(file([{ send: 'go', rejected: 'exact message' }, { send: 'go', rejected: { origin: 'policy', message: 'm' } }]));
});

test('invalid files are refused with a location', () => {
  invalid({ ...file([send]), extra: 1 }, /^file: unknown field "extra"/);
  invalid(file([send], { schema: 'caveat-scenarios/0.2' }), /schema must be/);
  invalid(file([]), /S1: steps must be a non-empty array/);
  invalid({ ...file([send]), scenarios: [{ id: 'A', title: 't', steps: [send] }, { id: 'A', title: 't', steps: [send] }] }, /duplicate id A/);
  invalid(file([{ send: 'go', expect: {} }]), /exactly one of/);
  invalid(file([{ send: 'go', extra: 1 }]), /S1 step 1: unknown field "extra"/);
  invalid(file([{ send: 'go', rejected: false }]), /omit rejected/);
  invalid(file([{ send: 'go', rejected: { origin: 'host' } }]), /origin "host" is invalid/);
  invalid(file([{ send: 'go', rejected: { origin: 'input', message: 'm' } }]), /only a policy rejection/);
  invalid(file([{ send: 'go', rejected: { code: 'reject' } }]), /origin must be one of/);
  invalid(file([{ send: 'go', repeat: 0 }]), /repeat must be/);
  invalid(file([{ send: 'go', repeat: 10001 }]), /repeat must be/);
  invalid(file([{ send: 'go', payload: { v: 1e999 } }]), /not finite/);
  invalid(file([{ expect: {} }]), /at least one JSON Pointer/);
  invalid(file([{ expect: { a: 1 } }]), /not a JSON Pointer/);
  invalid(file([{ expect: { '/a~2': 1 } }]), /invalid escape/);
  invalid(file([{ expect: { '/a': { $absent: false } } }]), /\$absent takes true/);
  invalid(file([{ expect: { '/a': [{ $absent: true }] } }]), /\$absent applies to a path/);
  invalid(file([{ expect: { '/a': { $set: 'x' } } }]), /\$set takes an array/);
  invalid(file([{ expect: { '/a': { $includes: [{ $exact: 1 }] } } }]), /literal values/);
  invalid(file([{ expect: { '/a': { $set: [], b: 1 } } }]), /only member/);
  invalid(file([{ expect: { '/a': { $maybe: 1 } } }]), /unknown matcher/);
});

test('before exists only after the first send; checkpoints only after they are made', () => {
  invalid(file([{ same_as: 'before', paths: ['/a'] }, send]), /before exists only after/);
  invalid(file([{ size: { snapshot: { max_growth: 1, since: 'before' } } }]), /before exists only after/);
  invalid(file([{ same_as: 'later', paths: ['/a'] }, { checkpoint: 'later' }]), /unknown state "later"/);
  invalid(file([{ checkpoint: 'initial' }]), /reserved/);
  invalid(file([{ checkpoint: 'c' }, { checkpoint: 'c' }]), /reserved or already used/);
  invalid(file([send, { same_as: 'before', paths: [] }]), /non-empty array/);
});

test('size bounds', () => {
  invalid(file([{ size: {} }]), /size bounds/);
  invalid(file([{ size: { save: { max: 1, max_growth: 1 } } }]), /either max or max_growth/);
  invalid(file([{ size: { save: { max_growth: 1, since: 'initial' } } }]), /applies to the snapshot/);
  invalid(file([{ size: { snapshot: { max: -1 } } }]), /non-negative integer/);
  invalid(file([{ size: { snapshot: { max: 5, since: 'initial' } } }]), /since goes with max_growth/);
});

test('parse refuses non-JSON text', () => {
  assert.throws(() => parseScenarioFile('{"schema":'), /not JSON/);
});

test('pointers follow RFC 6901, and array indices are exact', () => {
  const doc = { a: [10, 20], 'b/c': { '~': 1 }, '': 'empty' };
  assert.deepEqual(getPointer(doc, '/a/1'), { found: true, value: 20 });
  assert.deepEqual(getPointer(doc, '/b~1c/~0'), { found: true, value: 1 });
  assert.deepEqual(getPointer(doc, '/'), { found: true, value: 'empty' });
  for (const missing of ['/a/2', '/a/01', '/a/-', '/a/x', '/x', '/a/0/b']) assert.equal(getPointer(doc, missing).found, false, missing);
});

test('objects match partially, arrays in order, scalars exactly', () => {
  assert.equal(match({ a: 1, b: 2 }, { a: 1 }), null);
  assert.equal(match({ a: { b: 1, c: 2 } }, { a: { b: 1 } }), null);
  assert.deepEqual(match([1, 2], [2, 1]), { path: '/0', expected: 2, actual: 1 });
  assert.equal(match([1, 2], [1]).path, '');
  assert.equal(match([{ a: 1, z: 0 }], [{ a: 1 }]), null);
  assert.deepEqual(match({ a: 1 }, { b: 1 }), { path: '/b', expected: 1, actual: ABSENT });
  assert.equal(match(1, '1').path, '');
  assert.equal(match(null, {}).path, '');
});

test('$exact rejects extra members at any depth', () => {
  assert.equal(match({ a: { b: 1 } }, { $exact: { a: { b: 1 } } }), null);
  assert.notEqual(match({ a: { b: 1, c: 2 } }, { $exact: { a: { b: 1 } } }), null);
  assert.equal(match({ $weird: 1 }, { $exact: { $weird: 1 } }), null);
});

test('$set counts repeats and ignores order', () => {
  assert.equal(match(['b', 'a'], { $set: ['a', 'b'] }), null);
  assert.notEqual(match(['a', 'b'], { $set: ['a', 'a'] }), null);
  assert.notEqual(match(['a', 'a', 'b'], { $set: ['a', 'b'] }), null);
  assert.equal(match([{ x: 1, y: 2 }], { $set: [{ y: 2, x: 1 }] }), null, 'member order inside elements does not matter');
  assert.notEqual(match([{ x: 1, y: 2 }], { $set: [{ x: 1 }] }), null, 'elements compare exactly');
  assert.notEqual(match('ab', { $set: ['a', 'b'] }), null);
});

test('$includes counts repeats', () => {
  assert.equal(match(['a', 'b', 'a'], { $includes: ['a', 'a'] }), null);
  assert.notEqual(match(['a', 'b'], { $includes: ['a', 'a'] }), null);
  assert.equal(match(['a', 'b'], { $includes: [] }), null);
});

test('$absent at a path or member', () => {
  assert.equal(expectAt({ a: 1 }, '/b', { $absent: true }), null);
  assert.notEqual(expectAt({ a: 1 }, '/a', { $absent: true }), null);
  assert.equal(match({ a: 1 }, { b: { $absent: true } }), null);
  assert.deepEqual(match({ a: 1, b: null }, { b: { $absent: true } }), { path: '/b', expected: { $absent: true }, actual: null });
  assert.deepEqual(expectAt({}, '/missing', 1), { path: '/missing', expected: 1, actual: ABSENT });
});
