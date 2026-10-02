// Public WASM session path for the diagnostics covered by native Rust tests.
import assert from 'node:assert/strict';
import test from 'node:test';
import { loadRuntimeFromDirectory } from '../lib/node.mjs';
const runtime = await loadRuntimeFromDirectory();
const declarations = `claim intent_clear; evidence memory from "memory";
readings freshness from memory limit 4; state confidence = 0;
event observe value min 0 max 100; event retry;
on observe reveal memory supports intent_clear;
on observe sample freshness = value supports intent_clear;
on observe set confidence = latest(freshness);`;

test('WASM distinguishes known wrong-kind citation from unknown symbol', () => {
  assert.throws(() => runtime.open(`${declarations} bind hud.text = "ok" because intent_clear;`),
    error => error.kind === 'load' && /intent_clear is a declared claim, not a value citation/.test(error.message));
  assert.throws(() => runtime.open(`${declarations} bind hud.text = "ok" because missing;`),
    error => error.kind === 'load' && /unknown citation identifier missing/.test(error.message));
});
test('WASM define diagnostic explains expression and pure-function choices', () => {
  assert.throws(() => runtime.open(`${declarations} define freshness_last() = latest(freshness);`), error =>
    error.kind === 'load' && /does not take parameters/.test(error.message)
    && /use define freshness_last =/.test(error.message) && /cannot capture state, history or evidence/.test(error.message));
});
test('WASM valid value citations retain exact grounds', () => {
  const session = runtime.open(`${declarations} bind hud.value = confidence when has_sample(freshness) because confidence;`);
  try {
    assert.equal(session.dispatch('observe', { value: 68 }).outcome, 'accepted');
    assert.deepEqual(session.snapshot().binding_explanations.hud.value, { evidence: ['freshness@1'], caveats: [] });
    assert.equal(session.snapshot().bindings.hud.value, 68);
  } finally { session.close(); }
});
test('WASM ungrounded citation still rejects atomically and permits retry', () => {
  const session = runtime.open(`${declarations} on observe set confidence = 1 because latest(freshness);`);
  try {
    const before = session.save();
    const outcome = session.dispatch('observe', { value: 68 });
    assert.equal(outcome.outcome, 'rejected');
    assert.equal(outcome.origin, 'evaluation');
    assert.equal(outcome.code, 'ungrounded_citation');
    assert.equal(session.save(), before);
    assert.equal(session.dispatch('retry').outcome, 'accepted');
  } finally { session.close(); }
});
