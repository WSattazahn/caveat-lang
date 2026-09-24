// Registered cases for the ferry task: targeted scenarios written from the task
// text, then seeded 40-step sequences. `resume` saves and restores mid-case.
import { input, resume } from './oracle.mjs';

export const gust = kt => input('gust', { kt });
export const wave = m => input('wave', { m });
export const warning = () => input('warning');
export const decide = () => input('decide');
const seq = (id, ...events) => ({ id, events: events.flat() });
const ready = (a = 12, b = 20, c = 0.8, d = 1.1) => [gust(a), wave(c), gust(b), wave(d)];

// Payloads that every task must refuse before any rule runs.
export const malformed = [
  input('gust', {}), input('gust', { kt: '12' }), input('gust', { kt: 90.0001 }), input('gust', { kt: -1 }),
  input('gust', { kt: 12, m: 1 }), input('gust', { m: 1 }), input('wave', { m: 6.5 }), input('wave', { kt: 1 }),
  input('wave', { m: null }), input('warning', { now: 1 }), input('decide', { kt: 12 }), input('calm'), input('decide', []),
];

export const phase1Cases = [
  seq('P1-empty-session', decide(), warning(), resume(), decide()),
  seq('P1-malformed-inputs-change-nothing', ...malformed, gust(0), gust(90), wave(0), wave(6), resume(), ...malformed),
  seq('P1-two-gusts-and-two-waves-before-deciding', gust(10), wave(1), decide(), gust(11), decide(), wave(1), decide(), decide()),
  seq('P1-waves-alone-are-not-enough', wave(1), wave(1.2), wave(0.4), decide(), gust(5), decide(), gust(6), decide()),
  seq('P1-strongest-of-all-and-exact-boundaries', gust(30), gust(30.001), gust(29.999), wave(1.5), wave(1.5001), decide(), gust(2), decide()),
  seq('P1-go-reopened-by-strong-gust', ready(), decide(), gust(31), gust(45), resume(), decide(), warning()),
  seq('P1-go-reopened-by-high-wave', ready(), decide(), wave(1.4999), wave(1.6), wave(3), resume(), decide(), gust(50)),
  seq('P1-warning-reopens-go', ready(29, 30, 1.5, 0), decide(), resume(), warning(), warning(), decide(), gust(1)),
  seq('P1-warning-before-any-decision', warning(), ready(), decide(), warning(), gust(31), wave(2)),
  seq('P1-hold-stays-in-force', gust(12), gust(40), wave(1), wave(1), decide(), gust(80), wave(5), warning(), decide(), resume(), decide()),
  seq('P1-four-decisions', ready(), decide(), warning(), decide(), wave(2), decide(), resume(), wave(2.5), decide(),
    wave(3), decide(), gust(31), decide()),
  seq('P1-ten-gusts-then-refused', gust(1), gust(2), gust(3), gust(4), gust(5), gust(6), gust(7), gust(8), gust(9), gust(10),
    resume(), gust(11), gust(60), wave(1), wave(1), decide(), gust(99)),
  seq('P1-ten-waves-then-refused', ...Array.from({ length: 10 }, (_, i) => wave(i / 10)), wave(0.2), gust(3), gust(4), decide(),
    resume(), wave(4), decide()),
  seq('P1-reopened-stays-reopened', ready(), decide(), gust(50), wave(2), warning(), resume(), gust(60), decide()),
  seq('P1-zero-and-fractional-values', gust(0), gust(0.25), wave(0), wave(5.75), decide(), resume(), warning(), decide()),
];

// A deterministic generator (xorshift32), so every run replays the same cases.
export function seeded(seed) {
  let state = seed >>> 0;
  return () => {
    state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
    return (state >>> 0) / 4294967296;
  };
}
export const pick = (rng, values) => values[Math.floor(rng() * values.length)];

export function knots(rng) {
  return pick(rng, [
    () => Math.round(rng() * 90 * 4) / 4,
    () => Math.round(rng() * 30),
    () => 30 + Math.round(rng() * 60) / 2,
    () => pick(rng, [30, 30.001, 29.999, 0, 90]),
  ])();
}

export function metres(rng) {
  return pick(rng, [
    () => Math.round(rng() * 6 * 20) / 20,
    () => Math.round(rng() * 15) / 10,
    () => pick(rng, [1.5, 1.5001, 1.4999, 0, 6]),
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

export const PHASE1_SEED = 0xfe7701;
export function phase1Draw(rng) {
  const roll = rng();
  if (roll < 0.34) return gust(knots(rng));
  if (roll < 0.6) return wave(metres(rng));
  if (roll < 0.68) return warning();
  if (roll < 0.92) return decide();
  return pick(rng, malformed);
}

export const phase1Corpus = () => [
  ...phase1Cases.map(test => ({ ...test, kind: 'targeted' })),
  ...sequences(PHASE1_SEED, 100, phase1Draw).map(test => ({ ...test, kind: 'seeded' })),
];
