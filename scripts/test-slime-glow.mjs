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
assert.equal(initial.bindings.item.available, true);
assert.deepEqual(policy.toggleGlow().bindings.ability, initial.bindings.ability);
assert.deepEqual(policy.caveEntered().bindings.ability, initial.bindings.ability);
const acquired = policy.absorbMushroom();
assert.equal(acquired.bindings.ability.learned, true);
assert.equal(acquired.bindings.ability.active, true);
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
  assert.deepEqual(snapshot.qualified_values.active.provenance, acquired.qualified_values.learned.provenance);
  largestBytes = Math.max(largestBytes, JSON.stringify(snapshot).length);
}
assert(largestBytes < acquiredBytes + 128, 'No growing toggle archive; only sequence digits and labels change');
const disabled = snapshot;
assert.equal(disabled.bindings.ability.active, false);
assert.deepEqual(policy.absorbMushroom().bindings.ability, disabled.bindings.ability, 'Duplicate absorption must not enable');
assert.deepEqual(policy.snapshot().commitment_bases, acquired.commitment_bases);

assert.deepEqual(policy.reset(), initial, 'A new round must start without old evidence or numeric state');
const early = policy.absorbMushroom();
assert.equal(early.bindings.ability.learned, true, 'Contact can teach glow before the story threshold');
assert.deepEqual(early.commitment_bases, acquired.commitment_bases);
assert.deepEqual(policy.caveEntered().bindings.ability, early.bindings.ability);
policy.free();
policy.free();
assert.throws(() => policy.toggleGlow(), /freed/);
assert.throws(() => policy.reset(), /freed/);

const changed = source.replace('set active = learned;', 'set active = learned * 0;');
assert.notEqual(changed, source);
const changedPolicy = new SlimeGlowPolicy(changed);
const revised = changedPolicy.absorbMushroom();
assert.equal(revised.bindings.ability.active, false, 'Source alone controls automatic activation');
assert.equal(revised.bindings.ability.learned, true);
assert.deepEqual(revised.commitment_bases, acquired.commitment_bases);
assert.deepEqual(revised.qualified_values.active.provenance, acquired.qualified_values.active.provenance);
assert.equal(changedPolicy.toggleGlow().bindings.ability.active, true);
changedPolicy.free();

const failing = new WebReactiveSession(source.replace('set active = learned;', 'set active = 1 / 0;'));
const before = failing.snapshot();
assert.throws(() => failing.dispatch('absorb_mushroom', '{}'), /division by zero/);
assert.equal(failing.snapshot(), before, 'Late failure must not consume or grant');
for (const [event, payload] of [
  ['absorb_mushroom', '{"id":1}'],
  ['toggle_glow', '{"active":true}'],
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
  checks: ['locked input', 'early discovery', 'one qualified learning receipt', 'repeatable bounded toggles', 'duplicate absorption', 'reset/free', 'source variation', 'atomic failure', 'malformed input'],
};
await mkdir(new URL('test-results/', root), { recursive: true });
await writeFile(new URL('test-results/slime-glow-wasm.json', root), `${JSON.stringify(report, null, 2)}\n`);
console.log(`Slime Glow WebAssembly checks pass: ${report.toggles} toggles; graph remains ${report.symbols} symbols/${report.relations} relations.`);
console.log(`Report: ${fileURLToPath(new URL('test-results/slime-glow-wasm.json', root))}`);
