import assert from 'node:assert/strict';
import test from 'node:test';
import { compareViews } from './compare-views.mjs';

function view() {
  return {
    mushrooms: {
      grove: {
        because: ['taste_pool', 'taste_grove'], caveats: ['secondhand', 'taste_faded'],
        why: {
          absorb: { reason: 'Too risky untasted', because: ['absorb_pool', 'absorb_grove'], caveats: ['secondhand', 'taste_faded'] },
          taste: { reason: 'Already tasted', because: ['taste_pool', 'taste_grove'], caveats: ['secondhand', 'taste_faded'] },
        },
      },
    },
    belief: {
      supportedBy: ['taste_pool', 'taste_grove'], contradictedBy: ['absorb_pool', 'absorb_grove'],
      caveats: ['secondhand', 'taste_faded'],
    },
    decision: {
      basis: ['taste_pool', 'taste_grove'], reopenedBy: ['absorb_pool', 'absorb_grove'],
      caveats: ['secondhand', 'taste_faded'],
      history: [
        { change: 'committed', because: ['taste_pool', 'taste_grove'] },
        { change: 'reopened', because: ['absorb_pool'] },
      ],
    },
  };
}

test('set-valued evidence and caveats compare without order and inputs stay intact', () => {
  const ts = view();
  const caveat = structuredClone(ts);
  for (const record of [caveat.mushrooms.grove, ...Object.values(caveat.mushrooms.grove.why), caveat.belief]) {
    for (const value of Object.values(record)) if (Array.isArray(value)) value.reverse();
  }
  caveat.decision.caveats.reverse();
  const before = structuredClone({ ts, caveat });
  assert.equal(compareViews(ts, caveat), null);
  assert.deepEqual({ ts, caveat }, before);
});

for (const field of ['basis', 'reopenedBy']) {
  test(`decision.${field} must retain observation order`, () => {
    const ts = view();
    const caveat = structuredClone(ts);
    caveat.decision[field].reverse();
    assert.deepEqual(compareViews(ts, caveat), {
      at: `decision.${field}`, ts: ts.decision[field], caveat: caveat.decision[field],
    });
  });
}

test('decision history entries must remain in order', () => {
  const ts = view();
  const caveat = structuredClone(ts);
  caveat.decision.history.reverse();
  assert.equal(compareViews(ts, caveat).at, 'decision.history');
});

test('evidence inside a decision history entry must remain in order', () => {
  const ts = view();
  const caveat = structuredClone(ts);
  caveat.decision.history[0].because.reverse();
  assert.equal(compareViews(ts, caveat).at, 'decision.history');
});

test('normalising set order does not hide a missing caveat or changed reason', () => {
  const ts = view();
  const missing = structuredClone(ts);
  missing.mushrooms.grove.caveats.pop();
  assert.equal(compareViews(ts, missing).at, 'mushrooms.grove.caveats');
  const changed = structuredClone(ts);
  changed.mushrooms.grove.why.absorb.reason = 'Known duskcap';
  assert.equal(compareViews(ts, changed).at, 'mushrooms.grove.why.absorb.reason');
});

test('unrecognised lists retain their order instead of silently becoming sets', () => {
  assert.equal(compareViews({ extra: { because: ['a', 'b'] } }, { extra: { because: ['b', 'a'] } }).at,
    'extra.because');
});
