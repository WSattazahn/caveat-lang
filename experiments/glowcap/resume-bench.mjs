// Ten minutes of play, then JSON save/restore + first view. Run after the
// differential fuzz completes so it does not compete with these measurements.
import assert from 'node:assert/strict';

for (const [name, entry] of [
  ['ts', './ts/glowcap.ts'],
  ['caveat4', './caveat4/adapter.mjs'],
  ['caveat5', './caveat5/adapter.mjs'],
]) {
  const { createPolicy, ready } = await import(new URL(entry, import.meta.url));
  await ready;
  const samplesMs = [];
  let saveBytes = 0;
  for (let run = 0; run < 3; run += 1) {
    const policy = createPolicy();
    let resumed;
    try {
      policy.dispatch({ type: 'absorb', id: 'cave', kind: 'glowcap' });
      for (let tick = 0; tick < 9600; tick += 1) policy.dispatch({ type: 'tick', dt: 0.0625 });
      const text = JSON.stringify(policy.save());
      saveBytes = Buffer.byteLength(text);
      const start = performance.now();
      resumed = createPolicy(JSON.parse(text));
      const view = resumed.view();
      samplesMs.push(performance.now() - start);
      assert.deepEqual(view, policy.view(), `${name}: restored view`);
      // Check continued play outside the timed region.
      policy.dispatch({ type: 'tick', dt: 0.05 });
      resumed.dispatch({ type: 'tick', dt: 0.05 });
      assert.deepEqual(resumed.view(), policy.view(), `${name}: continued play`);
    } finally {
      policy.free?.();
      resumed?.free?.();
    }
  }
  const medianMs = [...samplesMs].sort((a, b) => a - b)[1];
  console.log(JSON.stringify({ impl: name, saveBytes, medianMs, samplesMs }));
}
