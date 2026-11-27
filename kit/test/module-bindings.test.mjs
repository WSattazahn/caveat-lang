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

  test(`${name}: save helpers take the reading a release's replay save can need (D094)`, async () => {
    // 1.1.0, which reads a module's hold as its own, writes this format of
    // this bundle too, so the save cannot say which reading wrote it.
    const historical = await fixture('collision-v1.4.1-replay-1.json');
    const needsChoice = error => error.kind === 'restore' && /supply moduleBindings/.test(error.message);
    assert.throws(() => savedEvents(runtime, collision, historical), needsChoice);
    assert.throws(() => carrySave(runtime, collision, historical, legacyOnly), needsChoice);
    for (const moduleBindings of ['scoped', 'legacy-hold-capture']) {
      assert.deepEqual(savedEvents(runtime, collision, historical, { moduleBindings }), [{ event: 'm__go', payload: {} }]);
      const carried = carrySave(runtime, collision, historical, collision, { moduleBindings });
      try {
        assert.equal(carried.accepted.length, 1);
        assert.deepEqual(carried.conflicts, []);
        // The new session is the new source's, read as a fresh session reads it.
        assert.equal(carried.session.snapshot().values.m__v, 10);
      } finally { carried.session.close(); }
      // The helpers pass the reading they are given: a save that names this
      // one is read with it, and refused with the other.
      const session = runtime.restore(collision, historical, { moduleBindings });
      let marked;
      try { marked = session.save({ compact: true }); } finally { session.close(); }
      const other = moduleBindings === 'scoped' ? 'legacy-hold-capture' : 'scoped';
      const conflicts = error => error.kind === 'restore' && /conflicts with saved module_bindings/.test(error.message);
      assert.deepEqual(savedEvents(runtime, collision, marked, { moduleBindings }), [{ event: 'm__go', payload: {} }]);
      assert.throws(() => savedEvents(runtime, collision, marked, { moduleBindings: other }), conflicts);
      assert.throws(() => carrySave(runtime, collision, marked, collision, { moduleBindings: other }), conflicts);
    }
    assert.equal(runtime.openSessions, 0);
  });
}

test('the raw WASM options bridge refuses malformed or unknown choices before restoring', () => {
  for (const options of ['not JSON', 'null', '[]', '["scoped"]', '{"moduleBindings":null}', '{"moduleBindings":"guess"}', '{"unknown":true}', '{"moduleBindings":"scoped","moduleBindings":"legacy-hold-capture"}']) {
    assert.throws(() => Real.restore_options('', 'not a save', options), error => /invalid restore options:/.test(String(error)));
  }
});
