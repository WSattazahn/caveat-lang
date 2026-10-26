// Real WASM provenance limits are refusals (`limit/provenance_limit`, D022),
// not fatal outcomes: each accepted boundary is inspected before overflow, and
// after the refused event the same session goes on, unchanged.
import test from 'node:test';
import assert from 'node:assert/strict';
import { real } from './helpers.mjs';

const countSource = `evidence e from sensor; caveat extra consequence low;
  state value = 0; event seed; event overflow; event advance; state count = 0;
  ${Array.from({ length: 1023 }, (_, i) => `caveat c${i} consequence low; c${i} qualifies e;`).join('\n')}
  on seed reveal e; on seed set value = qualified(1, e);
  on overflow qualify e with extra;
  on advance set count = count + 1;`;
const left = 'a'.repeat(32768);
const right = 'b'.repeat(32768);
const bytesSource = `evidence ${left} from left_sensor; evidence ${right} from right_sensor;
  evidence x from extra_sensor;
  state a = 0; state b = 0; state total = 0; event seed; event overflow; event advance; state count = 0;
  on seed reveal ${left}; on seed reveal ${right}; on seed reveal x;
  on seed set a = qualified(1, ${left}); on seed set b = qualified(2, ${right});
  on seed set total = a + b;
  on overflow set total = total + qualified(0, x);
  on advance set count = count + 1;`;

const limits = [
  {
    name: '1024 identifiers', source: countSource,
    check(snapshot) {
      const provenance = snapshot.qualified_values.value.provenance;
      assert.deepEqual(provenance.evidence, ['e']);
      assert.equal(provenance.caveats.length, 1023);
      assert.equal(new Set(provenance.caveats).size, 1023);
    },
  },
  {
    name: '65536 name bytes', source: bytesSource,
    check(snapshot) {
      const provenance = snapshot.qualified_values.total.provenance;
      assert.deepEqual(provenance.evidence, [left, right]);
      assert.equal(provenance.evidence.reduce((sum, name) => sum + Buffer.byteLength(name, 'utf8'), 0), 65536);
      assert.deepEqual(provenance.caveats, []);
    },
  },
];

const capture = session => ({ snapshot: session.snapshotText(), view: session.viewText(), save: session.save() });

for (const method of ['dispatch', 'dispatchView']) {
  for (const limit of limits) {
    test(`${method}: exactly ${limit.name} accepts; one excess is refused as limit/provenance_limit and the session goes on`, () => {
      const session = real.open(limit.source);
      try {
        const boundary = session[method]('seed');
        assert.equal(boundary.outcome, 'accepted');
        limit.check(session.snapshot());
        const before = capture(session);
        const refused = session[method]('overflow');
        assert.equal(refused.outcome, 'rejected');
        assert.equal(refused.origin, 'limit');
        assert.equal(refused.code, 'provenance_limit');
        assert.match(refused.message, new RegExp(`value provenance exceeds limit ${limit.name}$`));
        assert.equal(session.state, 'open');
        assert.deepEqual(capture(session), before, 'the refused event changed nothing');
        assert.equal(session[method]('overflow').code, 'provenance_limit', 'refused again, the same way');
        assert.equal(session[method]('advance').outcome, 'accepted', 'the session goes on');
        const restored = real.restore(limit.source, session.save());
        try {
          assert.deepEqual(capture(restored), capture(session));
        } finally { restored.close(); }
      } finally {
        session.close();
      }
    });
  }
}
