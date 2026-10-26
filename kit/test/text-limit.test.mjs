// A text past 65,536 bytes is a refusal (`limit/text_limit`, D025), not a
// fatal outcome: the boundary is accepted and inspected, and after the refused
// event the same session goes on, unchanged.
import test from 'node:test';
import assert from 'node:assert/strict';
import { real } from './helpers.mjs';

// The literal and the number's text: 16 bytes of it at the boundary, 17 past
// it, while the expression's source stays within its own 65,536.
const source = `state big = 0 min 0 max 2000000; state count = 0;
  event seed; event overflow; event advance;
  on seed set big = 1234567.12345678;
  on overflow set big = 1234567.123456789;
  on advance set count = count + 1;
  bind wide.text = "${'A'.repeat(65520)}" + text(big);`;

const capture = session => ({ snapshot: session.snapshotText(), view: session.viewText(), save: session.save() });

for (const method of ['dispatch', 'dispatchView']) {
  test(`${method}: exactly 65536 text bytes accepts; one excess is refused as limit/text_limit and the session goes on`, () => {
    const session = real.open(source);
    try {
      assert.equal(session[method]('seed').outcome, 'accepted');
      assert.equal(Buffer.byteLength(session.snapshot().bindings.wide.text, 'utf8'), 65536);
      const before = capture(session);
      const refused = session[method]('overflow');
      assert.equal(refused.outcome, 'rejected');
      assert.equal(refused.origin, 'limit');
      assert.equal(refused.code, 'text_limit');
      assert.match(refused.message, /expression text exceeds limit 65536 bytes$/);
      assert.deepEqual(capture(session), before, 'the refused event changed nothing');
      assert.equal(session[method]('overflow').code, 'text_limit', 'refused again, the same way');
      assert.equal(session[method]('advance').outcome, 'accepted');
      assert.equal(session.snapshot().qualified_values.count.value, 1);
    } finally { session.close(); }
  });
}
