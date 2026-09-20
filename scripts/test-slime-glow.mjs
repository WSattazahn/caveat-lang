import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import init, { WebReactiveSession } from '../dist/pkg/caveat_runtime.js';
import { SlimeGlowPolicy } from '../dist/slime-glow-policy.js';

const root = new URL('../', import.meta.url);
const read = file => readFile(new URL(file, root));
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const sourceBytes = await read('examples/slime_glow_ability.cav');
const source = sourceBytes.toString('utf8');
const wasm = await read('dist/pkg/caveat_runtime_bg.wasm');
assert.deepEqual(await read('dist/slime_glow_ability.cav'), sourceBytes, 'Rebuild the exported source');
assert.deepEqual(await read('dist/slime-glow-policy.js'), await read('web/slime-glow-policy.js'), 'Rebuild the wrapper');
await init({ module_or_path: wasm });

const policy = new SlimeGlowPolicy(source);
const initial = policy.snapshot();
assert.equal(initial.bindings.ability.learned, false);
assert.equal(initial.bindings.ability.toggleAvailable, false);
assert.equal(initial.bindings.item.available, true);
assert.deepEqual(policy.toggleGlow().bindings.ability, initial.bindings.ability);
assert.deepEqual(policy.caveEntered().bindings.ability, initial.bindings.ability);
const rendererReady = policy.glowRendererReady();
assert.equal(rendererReady.bindings.ability.learned, false, 'Capability alone does not award discovery');
assert.equal(rendererReady.bindings.ability.toggleAvailable, false);
assert.equal(rendererReady.bindings.ability.name, 'Glow');
const acquired = policy.absorbMushroom();
assert.equal(acquired.bindings.ability.learned, true);
assert.equal(acquired.bindings.ability.active, true);
assert.equal(acquired.bindings.ability.toggleAvailable, true);
assert.equal(acquired.bindings.item.consumed, true);
assert.equal(acquired.bindings.item.available, false);
assert.deepEqual(acquired.commitment_bases.keep_glow, {
  value: 1,
  provenance: { evidence: ['first_mushroom'], caveats: ['single_absorption'] },
});

const acquiredBytes = JSON.stringify(acquired).length;
let snapshot;
let largestBytes = 0;
for (let index = 0; index < 10_001; index += 1) {
  snapshot = policy.toggleGlow();
  assert.equal(snapshot.bindings.ability.active, index % 2 === 1);
  assert.deepEqual(snapshot.symbols, acquired.symbols);
  assert.deepEqual(snapshot.relations, acquired.relations);
  assert.deepEqual(snapshot.commitment_bases, acquired.commitment_bases);
  assert.deepEqual(snapshot.reading_streams, {});
  assert.deepEqual(snapshot.decision_series, {});
  assert.deepEqual(snapshot.qualified_values.active.provenance, acquired.qualified_values.active.provenance);
  largestBytes = Math.max(largestBytes, JSON.stringify(snapshot).length);
}
assert(largestBytes < acquiredBytes + 128, 'No growing toggle archive; only sequence digits and labels change');
const disabled = snapshot;
assert.equal(disabled.bindings.ability.active, false);
assert.deepEqual(policy.absorbMushroom().bindings.ability, disabled.bindings.ability, 'Duplicate absorption must not enable');
assert.deepEqual(policy.snapshot().commitment_bases, acquired.commitment_bases);
assert.deepEqual(policy.glowRendererReady().bindings.ability, disabled.bindings.ability, 'Repeated capability must not re-enable');

assert.deepEqual(policy.reset(), initial, 'A new round must start without old evidence or numeric state');
const clearing = policy.clearingStarted();
assert.deepEqual(clearing.bindings.ability, initial.bindings.ability);
assert.deepEqual(clearing.commitment_bases, {});
assert.equal(clearing.bindings.objective.text, 'Squeeze through the gap. Approach the mushroom and press E to absorb it.');
assert.deepEqual(policy.clearingStarted().relations, clearing.relations, 'Repeated scene context does not grow history');
assert.deepEqual(policy.caveEntered().bindings.objective, clearing.bindings.objective, 'Clearing guidance survives a cave event');
const clearingLearned = policy.absorbMushroom();
assert.deepEqual(clearingLearned.commitment_bases, acquired.commitment_bases, 'Scene context cannot contaminate the learning receipt');
assert.equal(policy.clearingStarted().bindings.objective.text, 'Mushroom absorbed. Clearing complete.');
assert.equal(clearingLearned.bindings.ability.name, 'Mushroom discovery');
assert.equal(clearingLearned.bindings.ability.active, false);
assert.equal(clearingLearned.bindings.ability.toggleAvailable, false);
assert.equal(clearingLearned.bindings.journal.text, 'You absorbed one mushroom. Its effects are not yet established.');
for (let index = 0; index < 1_001; index += 1) {
  const next = policy.toggleGlow();
  assert.deepEqual(next.bindings, clearingLearned.bindings, 'Unready input cannot toggle or imply an effect');
  assert.deepEqual(next.symbols, clearingLearned.symbols);
  assert.deepEqual(next.relations, clearingLearned.relations);
  assert.deepEqual(next.commitment_bases, clearingLearned.commitment_bases);
  assert(JSON.stringify(next).length < JSON.stringify(clearingLearned).length + 128);
}
const clearingReady = policy.glowRendererReady();
assert.equal(clearingReady.bindings.ability.toggleAvailable, true);
assert.equal(clearingReady.bindings.ability.active, false, 'Late capability does not silently activate');
assert.equal(clearingReady.bindings.objective.text, 'Glow learned.');
assert.deepEqual(clearingReady.commitment_bases, acquired.commitment_bases);
for (let index = 0; index < 1_001; index += 1) {
  const next = policy.toggleGlow();
  assert.equal(next.bindings.ability.active, index % 2 === 0);
  assert.deepEqual(next.symbols, clearingReady.symbols);
  assert.deepEqual(next.relations, clearingReady.relations);
  assert.deepEqual(next.commitment_bases, clearingReady.commitment_bases);
  assert(JSON.stringify(next).length < JSON.stringify(clearingReady).length + 128);
}
assert.deepEqual(policy.reset(), initial, 'Reset removes clearing context as well as learning');
const early = policy.absorbMushroom();
assert.equal(early.bindings.ability.learned, true, 'Contact can teach glow before the story threshold');
assert.equal(early.bindings.ability.active, false, 'Reset removes the host capability report');
assert.equal(early.bindings.ability.toggleAvailable, false);
assert.equal(early.bindings.objective.text, 'Mushroom absorbed.');
assert.deepEqual(early.commitment_bases, acquired.commitment_bases);
assert.deepEqual(policy.caveEntered().bindings.ability, early.bindings.ability);
policy.free();
policy.free();
assert.throws(() => policy.toggleGlow(), /freed/);
assert.throws(() => policy.reset(), /freed/);

const changed = source.replace('set active = learned * renderer_ready;', 'set active = learned * renderer_ready * 0;');
assert.notEqual(changed, source);
const changedPolicy = new SlimeGlowPolicy(changed);
changedPolicy.glowRendererReady();
const revised = changedPolicy.absorbMushroom();
assert.equal(revised.bindings.ability.active, false, 'Source alone controls automatic activation');
assert.equal(revised.bindings.ability.learned, true);
assert.deepEqual(revised.commitment_bases, acquired.commitment_bases);
assert.deepEqual(revised.qualified_values.active.provenance, acquired.qualified_values.active.provenance);
assert.equal(changedPolicy.toggleGlow().bindings.ability.active, true);
changedPolicy.free();

const failing = new WebReactiveSession(source.replace('set active = learned * renderer_ready;', 'set active = 1 / 0;'));
const before = failing.snapshot();
assert.throws(() => failing.dispatch('absorb_mushroom', '{}'), /division by zero/);
assert.equal(failing.snapshot(), before, 'Late failure must not consume or grant');
for (const [event, payload] of [
  ['absorb_mushroom', '{"id":1}'],
  ['toggle_glow', '{"active":true}'],
  ['glow_renderer_ready', '{"ready":true}'],
  ['reset', '{}'],
  ['absorb_mushroom', 'null'],
]) {
  assert.throws(() => failing.dispatch(event, payload));
  assert.equal(failing.snapshot(), before);
}
failing.free();

const report = {
  source: 'examples/slime_glow_ability.cav',
  source_sha256: sha256(sourceBytes),
  source_id: initial.source_id,
  wasm_sha256: sha256(wasm),
  glue_sha256: sha256(await read('dist/pkg/caveat_runtime.js')),
  wrapper_sha256: sha256(await read('dist/slime-glow-policy.js')),
  toggles: 10_001,
  symbols: acquired.symbols.length,
  relations: acquired.relations.length,
  learned_snapshot_bytes: acquiredBytes,
  largest_toggle_snapshot_bytes: largestBytes,
  clearing_context_toggles: 1_001,
  clearing_context_relations: clearingLearned.relations.length,
  clearing_ready_toggles: 1_001,
  clearing_ready_relations: clearingReady.relations.length,
  checks: ['locked input', 'authored clearing context', 'factual unready discovery', 'verified renderer capability', 'early discovery', 'one qualified learning receipt', 'repeatable bounded toggles', 'duplicate absorption', 'reset/free', 'source variation', 'atomic failure', 'malformed input'],
};
await mkdir(new URL('test-results/', root), { recursive: true });
await writeFile(new URL('test-results/slime-glow-wasm.json', root), `${JSON.stringify(report, null, 2)}\n`);
console.log(`Slime Glow WebAssembly checks pass: ${report.toggles} toggles; graph remains ${report.symbols} symbols/${report.relations} relations.`);
console.log(`Report: ${fileURLToPath(new URL('test-results/slime-glow-wasm.json', root))}`);
