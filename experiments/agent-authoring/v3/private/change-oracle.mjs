// SEALED until every phase-one program is frozen: the change request's model
// and cases. Its SHA-256 is registered before any author starts.
import { AUGER, current, decisionText, input, makeTask, resume, thinnest } from './oracle.mjs';
import { crack, decide, malformed, measure, phase1Cases, phase1Draw, sequences, thickness } from './corpus.mjs';

export const SONAR = { event: 'scan', stream: 'sonar_thickness', caveat: 'uncalibrated_sonar', cap: 8 };

export const phase2 = makeTask({
  signatures: { measure: { cm: [0, 60] }, scan: { cm: [0, 60] }, crack: {}, decide: {} },
  instruments: [AUGER, SONAR],
  hud: state => ({
    readings: state.readings.filter(reading => reading.stream === 'thickness').length,
    scans: state.readings.filter(reading => reading.stream === 'sonar_thickness').length,
    thinnest: thinnest(state, ['thickness', 'sonar_thickness']),
    cracked: state.cracked ? 1 : 0,
    decision: decisionText(state),
    frozen: current(state)?.value ?? -1,
    revision: state.decisions.length,
  }),
});

const scan = cm => input('scan', { cm });
const seq = (id, ...events) => ({ id, events: events.flat() });
const badScans = [input('scan', {}), input('scan', { cm: 60.5 }), input('scan', { cm: '9' }), input('scan', { cm: 9, cm2: 1 })];

export const phase2Cases = [
  // The original behavior must survive the change unchanged.
  ...phase1Cases.map(test => ({ ...test, id: test.id.replace('P1-', 'P2-unchanged-') })),
  seq('P2-malformed-scans', ...badScans, scan(0), scan(60), resume(), ...badScans),
  seq('P2-three-readings-from-either-instrument', scan(20), decide(), measure(20), decide(), scan(30), decide()),
  seq('P2-sonar-only-decision', scan(14), scan(12.5), scan(20), decide(), resume(), scan(9), decide()),
  seq('P2-mixed-grounds-and-order', scan(30), measure(14), scan(11), measure(40), decide(), resume(), scan(9.5), decide()),
  seq('P2-thin-scan-reopens-open-rink', measure(20), measure(20), scan(20), decide(), scan(4), scan(3), crack(), decide()),
  seq('P2-thinnest-across-instruments', measure(25), scan(24.5), measure(26), decide(), measure(24.25), scan(24.75), crack(), decide()),
  seq('P2-eight-each-counted-separately', scan(20), scan(21), scan(22), scan(23), scan(24), scan(25), scan(26), scan(27),
    scan(28), measure(29), resume(), measure(30), scan(5), decide()),
  seq('P2-crack-and-scans', crack(), scan(15), scan(16), decide(), scan(17), measure(9), crack(), decide(), resume(), scan(1)),
];

export const PHASE2_SEED = 0x90d002;
export function phase2Draw(rng) {
  const roll = rng();
  if (roll < 0.3) return measure(thickness(rng));
  if (roll < 0.55) return scan(thickness(rng));
  if (roll < 0.65) return crack();
  if (roll < 0.9) return decide();
  return rng() < 0.5 ? malformed[Math.floor(rng() * malformed.length)] : badScans[Math.floor(rng() * badScans.length)];
}
// Phase one's draw still appears, so sequences without scans are covered too.
export const phase2Corpus = () => [
  ...phase2Cases.map(test => ({ ...test, kind: 'targeted' })),
  ...sequences(PHASE2_SEED, 80, phase2Draw).map(test => ({ ...test, kind: 'seeded' })),
  ...sequences(PHASE2_SEED + 1, 20, phase1Draw).map(test => ({ ...test, kind: 'seeded' })),
];
export { crack, decide, resume };
