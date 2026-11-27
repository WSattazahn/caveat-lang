import test from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { loadRuntimeFromDirectory } from '../lib/node.mjs';
import { real } from './helpers.mjs';
import { compilerCompatibility, moduleHold } from './fixtures/compiler-compatibility.mjs';

const full = await loadRuntimeFromDirectory(fileURLToPath(new URL('../../dist/pkg/', import.meta.url)));

for (const fixture of compilerCompatibility) {
  test(`${fixture.name} opens, describes, saves and continues in both WASM builds`, () => {
    assert.deepEqual(full.interface(fixture.source), real.interface(fixture.source));
    let expected;
    for (const runtime of [real, full]) {
      const session = runtime.open(fixture.source);
      let restored;
      try {
        const first = session.dispatch(fixture.event);
        assert.equal(first.outcome, 'accepted');
        assert.equal(first.snapshot.values[fixture.state], fixture.first);
        restored = runtime.restore(fixture.source, fixture.saved ?? session.save());
        assert.deepEqual(restored.snapshot(), session.snapshot());
        const next = session.dispatch(fixture.event);
        assert.equal(next.outcome, 'accepted');
        assert.equal(next.snapshot.values[fixture.state], fixture.next);
        assert.deepEqual(restored.dispatch(fixture.event), next);
        const result = { next, save: session.save(), view: session.view() };
        if (expected) assert.deepEqual(result, expected);
        expected = result;
        // Cached compilation must select the same interpretation too.
        const cached = runtime.restore(fixture.source, session.save());
        try { assert.deepEqual(cached.snapshot(), session.snapshot()); } finally { cached.close(); }
      } finally { session.close(); restored?.close(); }
    }
  });
}

test('bundle recovery preserves check refusal and exact save source identity', () => {
  for (const runtime of [real, full]) {
    assert.throws(() => runtime.check(moduleHold.source), error => error.kind === 'load' && /bundle/.test(error.message));
    const changed = moduleHold.source.replace('state because = 3;', 'state because = 4;');
    runtime.open(changed).close();
    assert.throws(() => runtime.restore(changed, moduleHold.saved), error => error.kind === 'restore');
  }
});
