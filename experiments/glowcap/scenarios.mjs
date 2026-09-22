// Frozen acceptance scenarios for PROTOCOL.md. Both implementations run exactly
// these. A scenario applies from `since` through `until` (inclusive); a change
// request that deliberately changes an earlier expectation ends the old version
// and starts a replacement, so every phase has one complete, consistent suite.
//
// Steps:
//   ['absorb', id, kind]   ['taste', id, kind]   ['tick', dt, times = 1]
//   ['expect', partialView]  only the listed fields are checked; lists are sets
//   ['reject', event]        dispatch must throw and leave the view unchanged

export const PHASES = ['base', 'cr1', 'cr2', 'cr3', 'cr4', 'cr5', 'cr6', 'cr7', 'cr8'];

const DT = 0.0625; // exact in binary floating point: 16 ticks = 1 s
const seconds = (s) => ['tick', DT, s * 16];

const unknown = { present: true, label: 'Glowing mushroom', canAbsorb: true, canTaste: true, because: [] };
const consumed = { present: false, label: '', canAbsorb: false, canTaste: false, because: [] };

export const SCENARIOS = [
  {
    id: 'S01', title: 'nothing known yet',
    steps: [
      ['expect', {
        slime: { glowing: false, heavy: false },
        mushrooms: { cave: unknown, pool: unknown, ruin: unknown },
        belief: { state: 'none', text: '', note: '', supportedBy: [], contradictedBy: [] },
        decision: { state: 'none', basis: [], reopenedBy: [] },
      }],
    ],
  },
  {
    id: 'S02', title: 'one glowcap: generalise, commit to trust',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['expect', {
        slime: { glowing: true, heavy: false },
        mushrooms: {
          cave: consumed,
          pool: { present: true, label: 'Probably a glowcap', canAbsorb: true, canTaste: true, because: ['absorb_cave'] },
          ruin: { label: 'Probably a glowcap', because: ['absorb_cave'] },
        },
        belief: {
          state: 'probably_safe', text: 'Glowing mushrooms give you light.',
          note: 'Based on one observation.', supportedBy: ['absorb_cave'], contradictedBy: [],
        },
        decision: { state: 'committed', basis: ['absorb_cave'], reopenedBy: [] },
      }],
    ],
  },
  {
    id: 'S03', title: 'a look-alike contradicts the belief and reopens trust', until: 'cr3',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['absorb', 'pool', 'duskcap'],
      ['expect', {
        slime: { glowing: true, heavy: true },
        mushrooms: {
          pool: { present: false },
          ruin: { label: 'Could be a duskcap — taste first', canAbsorb: true, canTaste: true, because: ['absorb_cave', 'absorb_pool'] },
        },
        belief: {
          state: 'uncertain', text: 'Not every glowing mushroom is safe. Taste before absorbing.',
          note: '', supportedBy: ['absorb_cave'], contradictedBy: ['absorb_pool'],
        },
        decision: { state: 'reopened', basis: ['absorb_cave'], reopenedBy: ['absorb_pool'] },
      }],
    ],
  },
  {
    id: 'S03b', title: 'CR4: the uncertain label cites the counterexample only', since: 'cr4',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['absorb', 'pool', 'duskcap'],
      ['expect', {
        mushrooms: { ruin: { label: 'Could be a duskcap — taste first', because: ['absorb_pool'], caveats: [] } },
        belief: { state: 'uncertain', supportedBy: ['absorb_cave'], contradictedBy: ['absorb_pool'] },
        decision: { state: 'reopened', basis: ['absorb_cave'], reopenedBy: ['absorb_pool'] },
      }],
    ],
  },
  {
    id: 'S04', title: 'a taste identifies one mushroom and teaches nothing general', until: 'cr2',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['absorb', 'pool', 'duskcap'],
      ['taste', 'ruin', 'glowcap'],
      ['expect', {
        mushrooms: { ruin: { present: true, label: 'Glowcap', canAbsorb: true, canTaste: false, because: ['taste_ruin'] } },
        belief: { state: 'uncertain', supportedBy: ['absorb_cave'], contradictedBy: ['absorb_pool'] },
        decision: { state: 'reopened', basis: ['absorb_cave'], reopenedBy: ['absorb_pool'] },
      }],
    ],
  },
  {
    id: 'S04b', title: 'CR3: a taste in the light also supports the belief', since: 'cr3',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['absorb', 'pool', 'duskcap'],
      ['taste', 'ruin', 'glowcap'],
      ['expect', {
        mushrooms: { ruin: { present: true, label: 'Glowcap', canAbsorb: true, canTaste: false, because: ['taste_ruin'], caveats: [] } },
        belief: { state: 'uncertain', supportedBy: ['absorb_cave', 'taste_ruin'], contradictedBy: ['absorb_pool'], caveats: [] },
        decision: { state: 'reopened', basis: ['absorb_cave'], reopenedBy: ['absorb_pool'], caveats: [] },
      }],
    ],
  },
  {
    id: 'S05', title: 'glow lasts 30 seconds',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['tick', DT, 472], // 29.5 s
      ['expect', { slime: { glowing: true } }],
      seconds(1), // 30.5 s
      ['expect', { slime: { glowing: false } }],
    ],
  },
  {
    id: 'S06', title: 'heaviness lasts 20 seconds',
    steps: [
      ['absorb', 'pool', 'duskcap'],
      ['tick', DT, 312], // 19.5 s
      ['expect', { slime: { heavy: true } }],
      seconds(1), // 20.5 s
      ['expect', { slime: { heavy: false } }],
    ],
  },
  {
    id: 'S07', title: 'first absorption is a duskcap: generalise the other way, no trust',
    steps: [
      ['absorb', 'pool', 'duskcap'],
      ['expect', {
        slime: { glowing: false, heavy: true },
        mushrooms: {
          cave: { label: 'Probably a duskcap', canAbsorb: true, canTaste: true, because: ['absorb_pool'] },
          ruin: { label: 'Probably a duskcap', because: ['absorb_pool'] },
        },
        belief: {
          state: 'probably_unsafe', text: 'Glowing mushrooms make you heavy.',
          note: 'Based on one observation.', supportedBy: [], contradictedBy: ['absorb_pool'],
        },
        decision: { state: 'none', basis: [], reopenedBy: [] },
      }],
    ],
  },
  {
    id: 'S08', title: 'support after a contradiction never commits', until: 'cr3',
    steps: [
      ['absorb', 'pool', 'duskcap'],
      ['absorb', 'cave', 'glowcap'],
      ['expect', {
        mushrooms: { ruin: { label: 'Could be a duskcap — taste first', because: ['absorb_pool', 'absorb_cave'] } },
        belief: { state: 'uncertain', supportedBy: ['absorb_cave'], contradictedBy: ['absorb_pool'] },
        decision: { state: 'none', basis: [], reopenedBy: [] },
      }],
    ],
  },
  {
    id: 'S08b', title: 'CR4: support after a contradiction, counterexample cited', since: 'cr4',
    steps: [
      ['absorb', 'pool', 'duskcap'],
      ['absorb', 'cave', 'glowcap'],
      ['expect', {
        mushrooms: { ruin: { label: 'Could be a duskcap — taste first', because: ['absorb_pool'] } },
        belief: { state: 'uncertain', supportedBy: ['absorb_cave'], contradictedBy: ['absorb_pool'] },
        decision: { state: 'none', basis: [], reopenedBy: [] },
      }],
    ],
  },
  {
    id: 'S09', title: 'more support widens the belief but not the frozen basis',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['absorb', 'ruin', 'glowcap'],
      ['expect', {
        mushrooms: { pool: { label: 'Probably a glowcap', because: ['absorb_cave', 'absorb_ruin'] } },
        belief: { state: 'probably_safe', note: 'Based on 2 observations.', supportedBy: ['absorb_cave', 'absorb_ruin'] },
        decision: { state: 'committed', basis: ['absorb_cave'], reopenedBy: [] },
      }],
    ],
  },
  {
    id: 'S10', title: 'a bitter taste forbids absorbing that mushroom',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['taste', 'pool', 'duskcap'],
      ['expect', { mushrooms: { pool: { present: true, label: 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: ['taste_pool'] } } }],
      ['reject', { type: 'absorb', id: 'pool', kind: 'duskcap' }],
      ['reject', { type: 'absorb', id: 'pool', kind: 'glowcap' }],
      ['reject', { type: 'taste', id: 'pool', kind: 'duskcap' }],
    ],
  },
  {
    id: 'S11', title: 'a sweet taste, then absorbing that mushroom', until: 'cr2',
    steps: [
      ['taste', 'ruin', 'glowcap'],
      ['expect', {
        mushrooms: { ruin: { label: 'Glowcap', canAbsorb: true, canTaste: false, because: ['taste_ruin'] } },
        belief: { state: 'none' },
      }],
      ['absorb', 'ruin', 'glowcap'],
      ['expect', {
        slime: { glowing: true },
        mushrooms: { ruin: { present: false } },
        belief: { state: 'probably_safe', supportedBy: ['absorb_ruin'] },
        decision: { state: 'committed', basis: ['absorb_ruin'] },
      }],
    ],
  },
  {
    id: 'S11b', title: 'CR3: a sweet taste in the dark commits trust and qualifies what follows', since: 'cr3',
    steps: [
      ['taste', 'ruin', 'glowcap'],
      ['expect', {
        mushrooms: {
          ruin: { label: 'Probably a glowcap (tasted in the dark)', canAbsorb: true, canTaste: false, because: ['taste_ruin'], caveats: ['tasted_in_dark'] },
          cave: { label: 'Probably a glowcap', because: ['taste_ruin'], caveats: ['tasted_in_dark'] },
        },
        belief: { state: 'probably_safe', supportedBy: ['taste_ruin'], caveats: ['tasted_in_dark'] },
        decision: { state: 'committed', basis: ['taste_ruin'], caveats: ['tasted_in_dark'] },
      }],
      ['absorb', 'ruin', 'glowcap'],
      ['expect', {
        slime: { glowing: true },
        mushrooms: { ruin: { present: false } },
        belief: { state: 'probably_safe', supportedBy: ['taste_ruin', 'absorb_ruin'], caveats: ['tasted_in_dark'] },
        decision: { state: 'committed', basis: ['taste_ruin'], caveats: ['tasted_in_dark'] },
      }],
    ],
  },
  {
    id: 'S12', title: 'rejected events leave no trace',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['reject', { type: 'absorb', id: 'cave', kind: 'glowcap' }],
      ['reject', { type: 'taste', id: 'cave', kind: 'glowcap' }],
      ['reject', { type: 'tick', dt: 0.2 }],
      ['reject', { type: 'tick', dt: -0.1 }],
      ['reject', { type: 'absorb', id: 'nowhere', kind: 'glowcap' }],
      ['reject', { type: 'absorb', id: 'pool', kind: 'bluecap' }],
      ['taste', 'ruin', 'glowcap'],
      ['reject', { type: 'taste', id: 'ruin', kind: 'glowcap' }],
      ['reject', { type: 'absorb', id: 'ruin', kind: 'duskcap' }],
    ],
  },
  {
    id: 'S13', title: 'a second glowcap restarts the glow',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      seconds(20),
      ['absorb', 'ruin', 'glowcap'],
      seconds(29),
      ['expect', { slime: { glowing: true } }],
      seconds(2),
      ['expect', { slime: { glowing: false } }],
    ],
  },
  {
    id: 'S14', title: 'glowing and heavy at once',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['absorb', 'pool', 'duskcap'],
      ['expect', { slime: { glowing: true, heavy: true } }],
    ],
  },
  {
    id: 'S15', title: 'CR1: a fourth mushroom obeys every rule', since: 'cr1',
    steps: [
      ['expect', { mushrooms: { grove: unknown } }],
      ['absorb', 'grove', 'glowcap'],
      ['expect', {
        mushrooms: { grove: consumed, cave: { label: 'Probably a glowcap', because: ['absorb_grove'] } },
        decision: { state: 'committed', basis: ['absorb_grove'] },
      }],
      ['absorb', 'pool', 'duskcap'],
      ['expect', {
        mushrooms: { cave: { label: 'Could be a duskcap — taste first' } },
        decision: { state: 'reopened', reopenedBy: ['absorb_pool'] },
      }],
    ],
  },
  {
    id: 'S16', title: 'CR2: twice bitten after trusting', since: 'cr2',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['absorb', 'pool', 'duskcap'],
      ['absorb', 'ruin', 'duskcap'],
      ['expect', {
        mushrooms: { grove: { present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: ['absorb_pool', 'absorb_ruin'] } },
        belief: { state: 'uncertain', contradictedBy: ['absorb_pool', 'absorb_ruin'] },
        decision: { state: 'reopened', basis: ['absorb_cave'], reopenedBy: ['absorb_pool', 'absorb_ruin'] },
      }],
      ['reject', { type: 'absorb', id: 'grove', kind: 'glowcap' }],
      ['taste', 'grove', 'glowcap'],
      ['expect', { mushrooms: { grove: { label: 'Glowcap', canAbsorb: true, canTaste: false, because: ['taste_grove'] } } }],
      ['absorb', 'grove', 'glowcap'],
      ['expect', { mushrooms: { grove: { present: false } } }],
    ],
  },
  {
    id: 'S17', title: 'CR2: twice bitten without ever trusting', since: 'cr2',
    steps: [
      ['absorb', 'pool', 'duskcap'],
      ['absorb', 'ruin', 'duskcap'],
      ['expect', {
        mushrooms: { cave: { label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: ['absorb_pool', 'absorb_ruin'] } },
        belief: { state: 'probably_unsafe', note: 'Based on 2 observations.' },
        decision: { state: 'none' },
      }],
      ['reject', { type: 'absorb', id: 'cave', kind: 'glowcap' }],
    ],
  },
  {
    id: 'S18', title: 'CR3: a bitter taste in the dark qualifies everything built on it', since: 'cr3',
    steps: [
      ['taste', 'pool', 'duskcap'],
      ['expect', {
        mushrooms: {
          pool: { label: 'Probably a duskcap (tasted in the dark)', canAbsorb: false, canTaste: false, because: ['taste_pool'], caveats: ['tasted_in_dark'] },
          cave: { label: 'Probably a duskcap', canAbsorb: true, because: ['taste_pool'], caveats: ['tasted_in_dark'] },
        },
        belief: { state: 'probably_unsafe', note: 'Based on one observation.', contradictedBy: ['taste_pool'], caveats: ['tasted_in_dark'] },
        decision: { state: 'none', caveats: [] },
      }],
      ['absorb', 'cave', 'glowcap'],
      ['expect', {
        mushrooms: { ruin: { label: 'Could be a duskcap — taste first', caveats: ['tasted_in_dark'] } },
        belief: { state: 'uncertain', supportedBy: ['absorb_cave'], contradictedBy: ['taste_pool'], caveats: ['tasted_in_dark'] },
      }],
      ['taste', 'ruin', 'glowcap'],
      ['expect', {
        mushrooms: { ruin: { label: 'Glowcap', because: ['taste_ruin'], caveats: [] } },
        belief: { supportedBy: ['absorb_cave', 'taste_ruin'] },
        decision: { state: 'none' },
      }],
    ],
  },
  {
    id: 'S19', title: 'CR3: a bitter taste reopens trust; a frozen basis keeps its own caveats', since: 'cr3',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['taste', 'pool', 'duskcap'],
      ['expect', {
        mushrooms: { pool: { label: 'Duskcap — avoid', caveats: [] } },
        belief: { state: 'uncertain', contradictedBy: ['taste_pool'], caveats: [] },
        decision: { state: 'reopened', basis: ['absorb_cave'], reopenedBy: ['taste_pool'], caveats: [] },
      }],
      seconds(31),
      ['taste', 'ruin', 'glowcap'],
      ['expect', {
        mushrooms: { ruin: { label: 'Probably a glowcap (tasted in the dark)', caveats: ['tasted_in_dark'] } },
        belief: { supportedBy: ['absorb_cave', 'taste_ruin'], caveats: ['tasted_in_dark'] },
        decision: { state: 'reopened', basis: ['absorb_cave'], caveats: [] },
      }],
    ],
  },
  {
    id: 'S20', title: 'CR4: a counterexample tasted in the dark is cited with its caveat', since: 'cr4',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      seconds(31),
      ['taste', 'pool', 'duskcap'],
      ['expect', {
        mushrooms: { ruin: { label: 'Could be a duskcap — taste first', because: ['taste_pool'], caveats: ['tasted_in_dark'] } },
        belief: { state: 'uncertain', supportedBy: ['absorb_cave'], contradictedBy: ['taste_pool'], caveats: ['tasted_in_dark'] },
        decision: { state: 'reopened', basis: ['absorb_cave'], reopenedBy: ['taste_pool'], caveats: [] },
      }],
    ],
  },
  // ── Round 3: blind change requests (PROTOCOL.md) ─────────────────────────
  {
    id: 'S21', title: 'CR5: a taste fades after a minute', since: 'cr5',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['taste', 'pool', 'duskcap'],
      seconds(59),
      ['expect', { mushrooms: { pool: { label: 'Duskcap — avoid', caveats: [] } } }],
      seconds(1),
      ['expect', {
        mushrooms: { pool: { label: 'Probably a duskcap (taste has faded)', canAbsorb: false, canTaste: false, because: ['taste_pool'], caveats: ['taste_faded'] } },
        belief: { contradictedBy: ['taste_pool'], caveats: ['taste_faded'] },
        decision: { basis: ['absorb_cave'], caveats: [] },
      }],
      ['reject', { type: 'absorb', id: 'pool', kind: 'duskcap' }],
    ],
  },
  {
    id: 'S22', title: 'CR5: a dark taste fades and keeps both caveats; the decision keeps its own', since: 'cr5',
    steps: [
      ['taste', 'ruin', 'glowcap'],
      seconds(60),
      ['expect', {
        mushrooms: {
          ruin: { label: 'Probably a glowcap (taste has faded)', canAbsorb: true, canTaste: false, because: ['taste_ruin'], caveats: ['tasted_in_dark', 'taste_faded'] },
          cave: { label: 'Probably a glowcap', because: ['taste_ruin'], caveats: ['tasted_in_dark', 'taste_faded'] },
        },
        belief: { caveats: ['tasted_in_dark', 'taste_faded'] },
        decision: { state: 'committed', basis: ['taste_ruin'], caveats: ['tasted_in_dark'] },
      }],
    ],
  },
  {
    id: 'S23', title: 'CR6: trust keeps its history', since: 'cr6',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['absorb', 'pool', 'duskcap'],
      ['absorb', 'ruin', 'duskcap'],
      ['expect', {
        decision: {
          history: [
            { change: 'committed', because: ['absorb_cave'] },
            { change: 'reopened', because: ['absorb_pool'] },
            { change: 'reopened', because: ['absorb_ruin'] },
          ],
        },
      }],
    ],
  },
  {
    id: 'S24', title: 'CR6: no trust, no history', since: 'cr6',
    steps: [
      ['expect', { decision: { history: [] } }],
      ['absorb', 'pool', 'duskcap'],
      ['expect', { decision: { history: [] } }],
    ],
  },
  {
    id: 'S25', title: 'CR7: trust recovers after two glowcaps, then reopens again', since: 'cr7',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['absorb', 'pool', 'duskcap'],
      ['taste', 'ruin', 'glowcap'],
      ['expect', { decision: { state: 'reopened' } }],
      ['absorb', 'ruin', 'glowcap'],
      ['expect', {
        belief: { state: 'uncertain' },
        decision: {
          state: 'committed', basis: ['taste_ruin', 'absorb_ruin'], reopenedBy: [], caveats: [],
          history: [
            { change: 'committed', because: ['absorb_cave'] },
            { change: 'reopened', because: ['absorb_pool'] },
            { change: 'committed', because: ['taste_ruin', 'absorb_ruin'] },
          ],
        },
      }],
      ['absorb', 'grove', 'duskcap'],
      ['expect', {
        decision: {
          state: 'reopened', basis: ['taste_ruin', 'absorb_ruin'], reopenedBy: ['absorb_grove'],
          history: [
            { change: 'committed', because: ['absorb_cave'] },
            { change: 'reopened', because: ['absorb_pool'] },
            { change: 'committed', because: ['taste_ruin', 'absorb_ruin'] },
            { change: 'reopened', because: ['absorb_grove'] },
          ],
        },
      }],
    ],
  },
  {
    id: 'S26', title: 'CR7: a contradiction in between resets the count', since: 'cr7',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['taste', 'pool', 'duskcap'],
      ['taste', 'ruin', 'glowcap'],
      ['taste', 'grove', 'duskcap'],
      ['absorb', 'ruin', 'glowcap'],
      ['expect', { decision: { state: 'reopened', basis: ['absorb_cave'], reopenedBy: ['taste_pool', 'taste_grove'] } }],
    ],
  },
  {
    id: 'S27', title: 'CR8: heaviness stacks up to 30 seconds', since: 'cr8',
    steps: [
      ['expect', { slime: { heavy: false, heavySeconds: 0 } }],
      ['absorb', 'pool', 'duskcap'],
      ['expect', { slime: { heavy: true, heavySeconds: 20 } }],
      seconds(5),
      ['expect', { slime: { heavySeconds: 15 } }],
      ['absorb', 'ruin', 'duskcap'],
      ['expect', { slime: { heavySeconds: 30 } }],
      ['tick', DT, 8], // 0.5 s
      ['expect', { slime: { heavySeconds: 30 } }],
      seconds(29),
      ['expect', { slime: { heavy: true, heavySeconds: 1 } }],
      ['tick', DT, 8],
      ['expect', { slime: { heavy: false, heavySeconds: 0 } }],
    ],
  },
];

export const EXPLANATION_KEYS = new Set(['because', 'supportedBy', 'contradictedBy', 'basis', 'reopenedBy', 'caveats', 'history']);

export function applies(scenario, phase) {
  const at = PHASES.indexOf(phase);
  const since = PHASES.indexOf(scenario.since ?? 'base');
  const until = PHASES.indexOf(scenario.until ?? PHASES.at(-1));
  return at >= since && at <= until;
}
