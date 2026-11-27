import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { controlOutcome, controls, declaresGround, eventFor, parseAudit, statementHash } from './check-lean-grounds.mjs';

const demo = new URL('../examples/lean-bridge/', import.meta.url);
const manifest = JSON.parse(readFileSync(new URL('lean-grounds.json', demo), 'utf8'));
const program = readFileSync(new URL(manifest.program, demo), 'utf8');

test('the demo program names each Lean ground by its theorem', () => {
  for (const entry of manifest.grounds) assert.ok(declaresGround(program, entry.evidence, entry.theorem));
  assert.equal(declaresGround(program, 'lean_lemma', 'LeanBridgeDemo.other'), false);
  assert.equal(declaresGround('evidence lean_lemma from "LeanBridgeDemo.lt_two_pow";', 'lean_lemma', 'LeanBridgeDemo.lt_two_pow'), false);
});

test('the manifest names events the program declares', () => {
  for (const entry of manifest.grounds) {
    const pass = entry.assumes ? `^event ${entry.on_pass} assumed min 0 max \\d+;` : `^event ${entry.on_pass};`;
    assert.match(program, new RegExp(pass, 'm'));
    assert.match(program, new RegExp(`^event ${entry.on_fail} why in broken changed missing;`, 'm'));
  }
});

test('the audit output is read for the named theorem only', () => {
  const output = [
    'CAVEAT_GROUND {"name":"A.b","found":true,"type":"forall (n : Nat), P n"}',
    'CAVEAT_GROUND {"name":"A.ax","found":true,"type":"Q"}',
    "'A.b' depends on axioms: [propext, A.ax, Quot.sound]",
  ].join('\n');
  const parsed = parseAudit(output, 'A.b');
  assert.equal(parsed.type, 'forall (n : Nat), P n');
  assert.deepEqual(parsed.axioms, ['propext', 'A.ax', 'Quot.sound']);
  assert.equal(parsed.types.get('A.ax'), 'Q');
  assert.deepEqual(parseAudit('CAVEAT_GROUND {"name":"A.b","found":false}', 'A.b'), { found: false });
  assert.deepEqual(parseAudit('CAVEAT_GROUND {"name":"A.c","found":true,"type":"T"}', 'A.b'), { found: false });
  assert.deepEqual(parseAudit('CAVEAT_GROUND {"name":"A.b","found":true,"type":"T"}\n\'A.b\' does not depend on any axioms', 'A.b').axioms, []);
  assert.throws(() => parseAudit('CAVEAT_GROUND {"name":"A.b","found":true,"type":"T"}\n\'A.bc\' depends on axioms: []', 'A.b'));
});

test('a check becomes the pass event, with the assumed count where the entry lists assumptions', () => {
  const [plain, assuming] = manifest.grounds;
  assert.deepEqual(eventFor(plain, { ok: true, assumed: [] }), { event: 'lean_checked' });
  assert.deepEqual(eventFor(assuming, { ok: true, assumed: ['LeanBridgeDemo.Assumed.two_pow_add'] }), { event: 'product_checked', payload: { assumed: 1 } });
  assert.deepEqual(eventFor(assuming, { ok: true, assumed: [] }), { event: 'product_checked', payload: { assumed: 0 } });
  for (const why of ['broken', 'changed', 'missing']) {
    assert.deepEqual(eventFor(plain, { ok: false, reason: why }), { event: 'lean_failed', payload: { why } });
    assert.deepEqual(eventFor(assuming, { ok: false, reason: why }), { event: 'product_failed', payload: { why } });
  }
});

test('a control passes only on its expected reason and detail, or its expected assumed count', () => {
  const sorry = controls.find(control => control.id === 'sorry');
  assert.equal(controlOutcome(sorry, { ok: false, reason: 'broken', detail: 'unapproved axioms: sorryAx' }).passed, true);
  assert.equal(controlOutcome(sorry, { ok: false, reason: 'broken', detail: 'error: unexpected token' }).passed, false);
  assert.equal(controlOutcome(sorry, { ok: true, assumed: [] }).passed, false);
  const proven = controls.find(control => control.id === 'assumption_proven');
  assert.equal(controlOutcome(proven, { ok: true, assumed: [] }).passed, true);
  assert.equal(controlOutcome(proven, { ok: true, assumed: ['x'] }).passed, false);
  assert.equal(controlOutcome(proven, { ok: false, reason: 'broken' }).passed, false);
});

test('statement hashes are SHA-256 hex and each control expects a reason the program accepts', () => {
  assert.match(statementHash('T'), /^[0-9a-f]{64}$/);
  for (const entry of manifest.grounds) {
    for (const pinned of [entry, ...(entry.assumes ?? [])]) assert.match(pinned.statement_sha256, /^[0-9a-f]{64}$/);
  }
  for (const control of controls) {
    const source = readFileSync(new URL(manifest.grounds[control.entry].source, demo), 'utf8');
    assert.ok(typeof control.expect === 'object' || ['broken', 'changed', 'missing'].includes(control.expect));
    assert.notEqual(source.replace(control.find, control.replace), source, control.id);
  }
});
