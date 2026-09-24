// Registered cases for the pond task: targeted scenarios written from the task
// text, then seeded 40-step sequences. `resume` saves and restores mid-case.
import { input, resume } from './oracle.mjs';

export const measure = cm => input('measure', { cm });
export const crack = () => input('crack');
export const decide = () => input('decide');
const seq = (id, ...events) => ({ id, events: events.flat() });
const three = (a = 20, b = 15, c = 12) => [measure(a), measure(b), measure(c)];

// Payloads that every task must refuse before any rule runs.
export const malformed = [
  input('measure', {}), input('measure', { cm: '12' }), input('measure', { cm: 60.0001 }), input('measure', { cm: -1 }),
  input('measure', { cm: 12, depth: 1 }), input('measure', { depth: 12 }), input('crack', { now: 1 }),
  input('decide', { cm: 12 }), input('thaw'), input('measure', { cm: null }), input('decide', []),
];

export const phase1Cases = [
  seq('P1-empty-session', decide(), crack(), resume(), decide()),
  seq('P1-malformed-inputs-change-nothing', ...malformed, measure(0), measure(60), resume(), ...malformed),
  seq('P1-three-measurements-before-deciding', measure(20), decide(), measure(18), decide(), measure(16), decide(), decide()),
  seq('P1-thinnest-of-all-and-exact-boundaries', measure(10), measure(9.999), measure(10.001), decide(), measure(30), decide()),
  seq('P1-open-rink-reopened-by-thin-reading', three(), decide(), measure(9.5), measure(3), resume(), decide(), crack()),
  seq('P1-crack-reopens-open-rink', three(40, 35.5, 22.25), decide(), resume(), crack(), crack(), decide(), measure(11)),
  seq('P1-crack-before-any-decision', crack(), three(), decide(), crack(), measure(10), measure(9)),
  seq('P1-closed-rink-stays-closed', measure(12), measure(8), measure(20), decide(), measure(1), crack(), decide(), resume(), decide()),
  seq('P1-three-decisions', three(25, 30, 35), decide(), crack(), decide(), measure(0), decide(), decide(), measure(50)),
  seq('P1-eight-measurements-then-refused', measure(20), measure(21), measure(22), measure(23), measure(24), measure(25),
    measure(26), measure(27), measure(28), resume(), measure(5), decide(), crack()),
  seq('P1-reopened-stays-reopened', three(), decide(), measure(2), measure(1), crack(), resume(), measure(4), decide()),
  seq('P1-zero-and-fractional-values', measure(0), measure(0.5), measure(59.75), decide(), resume(), crack(), decide()),
];

// A deterministic generator (xorshift32), so every run replays the same cases.
export function seeded(seed) {
  let state = seed >>> 0;
  return () => {
    state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
    return (state >>> 0) / 4294967296;
  };
}
const pick = (rng, values) => values[Math.floor(rng() * values.length)];

export function thickness(rng) {
  return pick(rng, [
    () => Math.round(rng() * 60 * 4) / 4,
    () => 10 + Math.round(rng() * 30),
    () => Math.round(rng() * 99) / 10,
    () => pick(rng, [10, 9.999, 10.001, 0, 60]),
  ])();
}

// Weighted draws of event makers, one per step, with a resume now and then.
export function sequences(seed, count, draw) {
  const rng = seeded(seed);
  return Array.from({ length: count }, (_, index) => ({
    id: `seed-${seed.toString(16)}-${index + 1}`,
    events: Array.from({ length: 40 }, () => (rng() < 0.08 ? resume() : draw(rng))),
  }));
}

export const PHASE1_SEED = 0x90d001;
export function phase1Draw(rng) {
  const roll = rng();
  if (roll < 0.5) return measure(thickness(rng));
  if (roll < 0.62) return crack();
  if (roll < 0.9) return decide();
  return pick(rng, malformed);
}

export const phase1Corpus = () => [
  ...phase1Cases.map(test => ({ ...test, kind: 'targeted' })),
  ...sequences(PHASE1_SEED, 100, phase1Draw).map(test => ({ ...test, kind: 'seeded' })),
];
