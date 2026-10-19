import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import { applyMutant, auditAxioms, auditInventory, auditLaws, errorInTheorem, sourcesMutants, validateInventory } from './verify-lean.mjs';

const names = ['Caveat.one', 'Caveat.two'];
const good = "'Caveat.one' depends on axioms: [propext, Quot.sound]\n'Caveat.two' does not depend on any axioms\n";

test('accepts exactly the declared inventory and permitted transitive axioms', () => {
  assert.deepEqual(auditAxioms(good, names), { 'Caveat.one': ['propext', 'Quot.sound'], 'Caveat.two': [] });
  assert.deepEqual(auditInventory('CAVEAT_THEOREM Caveat.two\nCAVEAT_THEOREM Caveat.one\n', names), [...names].reverse());
});

test('incomplete, duplicate, malformed and extra audit output fails', () => {
  for (const output of ['', good.split('\n')[0], good + good, good + "'Caveat.extra' does not depend on any axioms\n",
    good.replace('does not depend on any axioms', 'audit unavailable')]) {
    assert.throws(() => auditAxioms(output, names));
  }
});

test('sorry, native-evaluation trust and custom axioms fail even behind another theorem', () => {
  for (const axiom of ['sorryAx', 'Lean.trustCompiler', 'Caveat.hiddenAssumption']) {
    assert.throws(() => auditAxioms(good.replace('propext, Quot.sound', axiom), names), /Unapproved axiom/);
  }
});

test('an unregistered or omitted environment theorem fails independently of the axiom audit', () => {
  for (const output of ['', 'CAVEAT_THEOREM Caveat.one\n',
    'CAVEAT_THEOREM Caveat.one\nCAVEAT_THEOREM Caveat.two\nCAVEAT_THEOREM Caveat.extra\n',
    'CAVEAT_THEOREM Caveat.one\nCAVEAT_THEOREM Caveat.one\n']) {
    assert.throws(() => auditInventory(output, names));
  }
});

test('empty, duplicate, wrong-namespace and malformed registries fail', () => {
  for (const value of [[], null, ['Caveat.one', 'Caveat.one'], ['Other.one'], ['Caveat.'], [42]]) {
    assert.throws(() => validateInventory(value));
  }
});


test("authored-law registry must be nonempty and refer to inventoried theorems", () => {
  assert.equal(auditLaws(["Caveat.one"], names), 1);
  assert.throws(() => auditLaws([], names));
  assert.throws(() => auditLaws(["Caveat.missing"], names));
});

test('each Caveat Sources mutant changes the model once and names a theorem it has', () => {
  const model = readFileSync(new URL('../proofs/lean/Caveat/Sources.lean', import.meta.url), 'utf8');
  for (const mutant of sourcesMutants) {
    assert.notEqual(applyMutant(model, mutant), model);
    assert.match(model, new RegExp('^theorem ' + mutant.breaks + ' ', 'm'));
  }
  assert.throws(() => applyMutant('no anchor here', sourcesMutants[0]), /not unique/);
});

test('a mutant counts only when Lean fails inside the theorem it targets', () => {
  const model = 'def a := 1\ntheorem target (x : Nat) :\n  x = x := rfl\ntheorem other : True := trivial\n';
  assert.equal(errorInTheorem('M.lean:3:2: error: boom', model, 'target'), true);
  assert.equal(errorInTheorem('M.lean:4:2: error: boom', model, 'target'), false);
  assert.equal(errorInTheorem('M.lean:1:2: error: boom', model, 'target'), false);
  assert.equal(errorInTheorem('M.lean:3:2: warning: fine', model, 'target'), false);
  assert.equal(errorInTheorem('M.lean:3:2: error: boom', model, 'missing'), false);
});
