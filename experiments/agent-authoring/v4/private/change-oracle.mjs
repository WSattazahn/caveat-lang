// Model of the ferry task as amended by the change request: the pier
// anemometer. It reuses the phase-one model and adds one wind instrument.
import { BUOY, MAST, count, current, decisionText, input, largest, makeTask } from './oracle.mjs';
import { decide, gust, malformed, metres, knots, pick, phase1Cases, seeded, sequences, warning, wave } from './corpus.mjs';
import { resume } from './oracle.mjs';

export const PIER = { event: 'pier_gust', param: 'kt', stream: 'pier_wind', caveat: 'unverified_pier', cap: 6, safe: 30, wind: true };

export const phase2 = makeTask({
  signatures: { gust: { kt: [0, 90] }, pier_gust: { kt: [0, 90] }, wave: { m: [0, 6] }, warning: {}, decide: {} },
  instruments: [MAST, PIER, BUOY],
  consequences: { sheltered_mast: 'material', unverified_pier: 'material', buoy_drift: 'low', regional_forecast: 'low' },
  hud: state => ({
    gusts: count(state, ['wind']),
    pier_gusts: count(state, ['pier_wind']),
    waves: count(state, ['swell']),
    strongest: largest(state, ['wind', 'pier_wind']),
    highest: largest(state, ['swell']),
    warned: state.warned ? 1 : 0,
    decision: decisionText(state),
    frozen: current(state)?.value ?? -1,
    revision: state.decisions.length,
  }),
});

export const pier = kt => input('pier_gust', { kt });
const seq = (id, ...events) => ({ id, events: events.flat() });
const malformedPier = [input('pier_gust', {}), input('pier_gust', { kt: 91 }), input('pier_gust', { m: 1 }),
  input('pier_gust', { kt: '5' }), input('pier_gust', { kt: 5, m: 1 })];

// The unchanged phase-one cases still hold, except where the amended decide
// rule or display changes their outcome; the model decides that, not the case.
export const phase2Cases = [
  ...phase1Cases.slice(0, 9).map(test => ({ ...test, id: test.id.replace(/^P1/, 'P2-kept') })),
  seq('P2-malformed-pier', ...malformedPier, pier(0), pier(90), resume(), ...malformedPier),
  seq('P2-pier-and-mast-gusts-count-together', gust(10), pier(12), wave(1), wave(1), decide(), resume(), pier(31)),
  seq('P2-pier-alone-counts', pier(5), pier(6), wave(1), wave(1.5), decide(), gust(2), warning(), decide()),
  seq('P2-strongest-from-either', gust(12), pier(28), wave(1), wave(0.5), decide(), pier(30), pier(30.001), resume(), decide()),
  seq('P2-pier-gust-reopens', gust(5), gust(6), wave(1), wave(1), decide(), pier(29.999), pier(45), pier(50), decide()),
  seq('P2-six-pier-gusts-then-refused', pier(1), pier(2), pier(3), pier(4), pier(5), pier(6), resume(), pier(7), pier(60),
    gust(8), wave(1), wave(1), decide(), gust(9)),
  seq('P2-grounds-cover-both-anemometers', pier(20), gust(10), wave(0.2), wave(0.3), pier(25), decide(), warning(), gust(3), decide()),
  seq('P2-hold-from-pier', pier(35), gust(10), wave(1), wave(1), decide(), pier(80), warning(), decide(), resume(), decide()),
  seq('P2-four-decisions-across-instruments', gust(1), pier(2), wave(1), wave(1), decide(), pier(31), decide(), gust(40), decide(),
    decide()),
  seq('P2-strongest-before-any-gust', wave(1), wave(2), warning(), resume(), pier(0), gust(0)),
  seq('P2-mast-cap-unchanged', ...Array.from({ length: 11 }, (_, i) => gust(i)), pier(1), wave(1), wave(1), decide()),
];

export const PHASE2_SEED = 0xfe7702;
export function phase2Draw(rng) {
  const roll = rng();
  if (roll < 0.22) return gust(knots(rng));
  if (roll < 0.42) return pier(knots(rng));
  if (roll < 0.62) return wave(metres(rng));
  if (roll < 0.69) return warning();
  if (roll < 0.92) return decide();
  return pick(rng, [...malformed, ...malformedPier]);
}

export const phase2Corpus = () => [
  ...phase2Cases.map(test => ({ ...test, kind: 'targeted' })),
  ...sequences(PHASE2_SEED, 100, phase2Draw).map(test => ({ ...test, kind: 'seeded' })),
];
