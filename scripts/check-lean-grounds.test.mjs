import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { controls, declaresGround, eventFor, parseAudit, statementHash } from './check-lean-grounds.mjs';

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
    assert.match(program, new RegExp(`^event ${entry.on_pass};`, 'm'));
    assert.match(program, new RegExp(`^event ${entry.on_fail} why in broken changed missing;`, 'm'));
  }
});

test('the audit output is read for the named theorem only', () => {
  const output = [
    'CAVEAT_GROUND {"found":true,"type":"forall (n : Nat), P n"}',
    "'A.b' depends on axioms: [propext, Quot.sound]",
  ].join('\n');
  assert.deepEqual(parseAudit(output, 'A.b'), { found: true, type: 'forall (n : Nat), P n', axioms: ['propext', 'Quot.sound'] });
  assert.deepEqual(parseAudit('CAVEAT_GROUND {"found":false}', 'A.b'), { found: false });
  assert.deepEqual(parseAudit('CAVEAT_GROUND {"found":true,"type":"T"}\n\'A.b\' does not depend on any axioms', 'A.b').axioms, []);
  assert.throws(() => parseAudit('CAVEAT_GROUND {"found":true,"type":"T"}\n\'A.bc\' depends on axioms: []', 'A.b'));
});

test('a failed check becomes the failure event with its reason', () => {
  const entry = manifest.grounds[0];
  assert.deepEqual(eventFor(entry, { ok: true }), { event: 'lean_checked' });
  for (const why of ['broken', 'changed', 'missing']) {
    assert.deepEqual(eventFor(entry, { ok: false, reason: why }), { event: 'lean_failed', payload: { why } });
  }
});

test('statement hashes are SHA-256 hex and each control expects a reason the program accepts', () => {
  assert.match(statementHash('T'), /^[0-9a-f]{64}$/);
  assert.match(manifest.grounds[0].statement_sha256, /^[0-9a-f]{64}$/);
  const source = readFileSync(new URL(manifest.grounds[0].source, demo), 'utf8');
  for (const control of controls) {
    assert.ok(['broken', 'changed', 'missing'].includes(control.expect));
    assert.notEqual(source.replace(control.find, control.replace), source, control.id);
  }
});
