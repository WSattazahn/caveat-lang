import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { loadRuntimeFromDirectory } from '../lib/node.mjs';
import { savedEvents, carrySave } from '../lib/saves.mjs';
import { real, Real } from './helpers.mjs';

const fixture = name => readFile(new URL(`../../runtime/tests/fixtures/module-bindings/${name}`, import.meta.url), 'utf8');
const collision = await fixture('collision.bundle');
const initial = await fixture('collision-v1.4.1-initial.json');
const legacyOnly = await fixture('legacy-only.bundle');
const full = await loadRuntimeFromDirectory(fileURLToPath(new URL('../../dist/pkg/', import.meta.url)));

for (const [name, runtime] of [['reactive', real], ['full', full]]) {
  test(`${name}: explicit historical choices persist without changing fresh module scope`, () => {
    assert.throws(() => runtime.restore(collision, initial), error => error.kind === 'restore');
    // Alternate the same source's interpretations to exercise their cache keys.
    for (const [moduleBindings, expected] of [['scoped', 10], ['legacy-hold-capture', 14], ['scoped', 10]]) {
      const session = runtime.restore(collision, initial, { moduleBindings });
      let resumed;
      try {
        const saved = session.save();
        assert.equal(JSON.parse(saved).module_bindings, moduleBindings);
        resumed = runtime.restore(collision, saved);
        assert.deepEqual(resumed.snapshot(), session.snapshot());
        const first = session.dispatch('m__go');
        assert.equal(first.outcome, 'accepted');
        assert.equal(first.snapshot.values.m__v, expected);
        assert.deepEqual(resumed.dispatch('m__go'), first);
        const other = moduleBindings === 'scoped' ? 'legacy-hold-capture' : 'scoped';
        assert.throws(() => runtime.restore(collision, saved, { moduleBindings: other }), error => error.kind === 'restore');
      } finally { resumed?.close(); session.close(); }
    }
    const fresh = runtime.open(collision);
    try { assert.equal(fresh.dispatch('m__go').snapshot.values.m__v, 10); }
    finally { fresh.close(); }
    assert.equal(runtime.openSessions, 0);
  });

  test(`${name}: save helpers read a marked legacy program that fresh loading refuses`, async () => {
    assert.throws(() => runtime.open(legacyOnly), error => error.kind === 'load');
    assert.throws(() => runtime.interface(legacyOnly), error => error.kind === 'load');
    for (const [format, options] of [
      ['replay-1', { compact: true }],
      ['replay-2', { compact: true, positional: true }],
    ]) {
      const historical = await fixture(`legacy-only-v1.4.1-${format}.json`);
      const restored = runtime.restore(legacyOnly, historical, { moduleBindings: 'legacy-hold-capture' });
      let saved;
      try {
        assert.equal(restored.snapshot().values.m__v, 13);
        saved = restored.save(options);
      } finally { restored.close(); }
      const copy = String(saved);
      assert.match(saved, /caveat-replay-save/);
      assert.equal(JSON.parse(saved).module_bindings, 'legacy-hold-capture');
      assert.deepEqual(savedEvents(runtime, legacyOnly, saved), [{ event: 'm__go', payload: {} }]);
      const carried = carrySave(runtime, legacyOnly, saved, collision);
      try {
        assert.equal(carried.accepted.length, 1);
        assert.deepEqual(carried.conflicts, []);
        // Carrying explicitly replays onto the new source's corrected scope.
        assert.equal(carried.session.snapshot().values.m__v, 10);
      } finally { carried.session.close(); }
      assert.equal(saved, copy);
      assert.equal(runtime.openSessions, 0);
    }
  });
}

test('the raw WASM options bridge refuses malformed or unknown choices before restoring', () => {
  for (const options of ['not JSON', 'null', '[]', '["scoped"]', '{"moduleBindings":null}', '{"moduleBindings":"guess"}', '{"unknown":true}', '{"moduleBindings":"scoped","moduleBindings":"legacy-hold-capture"}']) {
    assert.throws(() => Real.restore_options('', 'not a save', options), error => /invalid restore options:/.test(String(error)));
  }
});
