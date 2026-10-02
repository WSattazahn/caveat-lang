// Real WASM provenance limits are fatal outcomes, not reusable refusals.
// Each accepted boundary is inspected before overflow; after overflow only the
// kit's lifecycle guards are exercised, never the discarded runtime session.
import test from 'node:test';
import assert from 'node:assert/strict';
import { CaveatError } from '../lib/session.mjs';
import { real } from './helpers.mjs';

const countSource = `evidence e from sensor; caveat extra consequence low;
  state value = 0; event seed; event overflow;
  ${Array.from({ length: 1023 }, (_, i) => `caveat c${i} consequence low; c${i} qualifies e;`).join('\n')}
  on seed reveal e; on seed set value = qualified(1, e);
  on overflow qualify e with extra;`;
const left = 'a'.repeat(32768);
const right = 'b'.repeat(32768);
const bytesSource = `evidence ${left} from left_sensor; evidence ${right} from right_sensor;
  evidence x from extra_sensor;
  state a = 0; state b = 0; state total = 0; event seed; event overflow;
  on seed reveal ${left}; on seed reveal ${right}; on seed reveal x;
  on seed set a = qualified(1, ${left}); on seed set b = qualified(2, ${right});
  on seed set total = a + b;
  on overflow set total = total + qualified(0, x);`;
const siblingSource = 'state count = 0; event advance; on advance set count = count + 1;';

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

for (const method of ['dispatch', 'dispatchView']) {
  for (const limit of limits) {
    test(`${method}: exactly ${limit.name} accepts; one excess is fatal and ends only that wrapper`, () => {
      const session = real.open(limit.source);
      const sibling = real.open(siblingSource);
      try {
        const boundary = session[method]('seed');
        assert.equal(boundary.outcome, 'accepted');
        limit.check(session.snapshot());
        assert.throws(() => session[method]('overflow'), error => {
          assert.ok(error instanceof CaveatError);
          assert.equal(error.kind, 'fatal');
          assert.equal(error.report.schema, 'caveat-dispatch/0.1');
          assert.equal(error.report.outcome, 'fatal');
          assert.equal(error.report.code, 'unclassified');
          assert.ok(error.report.message.includes(`value provenance exceeds limit ${limit.name}`));
          return true;
        });
        assert.equal(session.state, 'fatal');
        assert.equal(real.trapped, false, 'structured fatal does not poison the WASM instance');
        for (const call of ['dispatch', 'dispatchView', 'snapshot', 'snapshotText', 'view', 'viewText', 'save']) {
          assert.throws(() => session[call]('seed'), error => error instanceof CaveatError && error.kind === 'fatal', call);
        }
        assert.equal(sibling[method]('advance').outcome, 'accepted', 'unrelated sibling remains usable');
        assert.doesNotThrow(() => session.close());
        assert.equal(session.state, 'closed');
        assert.doesNotThrow(() => session.close(), 'close is idempotent after fatal');
        assert.throws(() => session[method]('seed'), error => error instanceof CaveatError && error.kind === 'closed');
        assert.equal(sibling[method]('advance').outcome, 'accepted', 'closing the discarded wrapper leaves its sibling usable');
      } finally {
        session.close();
        sibling.close();
      }
    });
  }
}
