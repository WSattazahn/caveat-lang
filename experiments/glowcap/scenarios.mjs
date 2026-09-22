// Frozen acceptance scenarios for PROTOCOL.md. Both implementations run exactly
// these. A scenario applies from `since` through `until` (inclusive); a change
// request that deliberately changes an earlier expectation ends the old version
// and starts a replacement, so every phase has one complete, consistent suite.
//
// Steps:
//   ['absorb', id, kind]   ['taste', id, kind]   ['tick', dt, times = 1]
//   ['expect', partialView]  only the listed fields are checked; lists are sets
//   ['reject', event]        dispatch must throw and leave the view unchanged
//   ['witness', id, kind]    (CR9)
//   ['resume']               (CR12) save, round-trip through JSON, resume, compare
//                            the complete views, continue on the resumed policy

export const PHASES = ['base', 'cr1', 'cr2', 'cr3', 'cr4', 'cr5', 'cr6', 'cr7', 'cr8', 'cr9', 'cr10', 'cr11', 'cr12'];

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
  // ── Round 6: blind change requests (PROTOCOL.md) ─────────────────────────
  {
    id: 'S28', title: 'CR9: a witnessed glowcap teaches, secondhand', since: 'cr9',
    steps: [
      ['witness', 'cave', 'glowcap'],
      ['expect', {
        slime: { glowing: false, heavy: false },
        mushrooms: {
          cave: consumed,
          pool: { label: 'Probably a glowcap', because: ['witness_cave'], caveats: ['secondhand'] },
        },
        belief: { state: 'probably_safe', note: 'Based on one observation.', supportedBy: ['witness_cave'], caveats: ['secondhand'] },
        decision: { state: 'committed', basis: ['witness_cave'], caveats: ['secondhand'], history: [{ change: 'committed', because: ['witness_cave'] }] },
      }],
      ['reject', { type: 'witness', id: 'cave', kind: 'glowcap' }],
      ['reject', { type: 'absorb', id: 'cave', kind: 'glowcap' }],
      ['reject', { type: 'witness', id: 'nowhere', kind: 'glowcap' }],
      ['reject', { type: 'witness', id: 'pool', kind: 'bluecap' }],
    ],
  },
  {
    id: 'S29', title: 'CR9: a witnessed duskcap reopens trust; risk does not stop another creature', since: 'cr9',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['witness', 'pool', 'duskcap'],
      ['expect', {
        slime: { glowing: true, heavy: false },
        mushrooms: { ruin: { label: 'Could be a duskcap — taste first', because: ['witness_pool'], caveats: ['secondhand'] } },
        belief: { state: 'uncertain', contradictedBy: ['witness_pool'], caveats: ['secondhand'] },
        decision: { state: 'reopened', reopenedBy: ['witness_pool'], caveats: [] },
      }],
      ['witness', 'ruin', 'duskcap'],
      ['expect', { mushrooms: { grove: { label: 'Too risky — taste first', canAbsorb: false, because: ['witness_pool', 'witness_ruin'], caveats: ['secondhand'] } } }],
      ['reject', { type: 'absorb', id: 'grove', kind: 'glowcap' }],
      ['witness', 'grove', 'glowcap'],
      ['expect', {
        mushrooms: { grove: { present: false } },
        belief: { supportedBy: ['absorb_cave', 'witness_grove'] },
        decision: { state: 'reopened', reopenedBy: ['witness_pool', 'witness_ruin'] },
      }],
    ],
  },
  {
    id: 'S30', title: 'CR9: a witness must agree with a taste', since: 'cr9',
    steps: [
      ['taste', 'cave', 'glowcap'],
      ['reject', { type: 'witness', id: 'cave', kind: 'duskcap' }],
      ['witness', 'cave', 'glowcap'],
      ['expect', {
        mushrooms: { cave: { present: false } },
        belief: { supportedBy: ['taste_cave', 'witness_cave'], caveats: ['tasted_in_dark', 'secondhand'] },
        decision: { state: 'committed', basis: ['taste_cave'], caveats: ['tasted_in_dark'] },
      }],
    ],
  },
  {
    id: 'S31', title: 'CR10: an eaten mushroom regrows as a stranger', since: 'cr10',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      seconds(44),
      ['expect', { mushrooms: { cave: { present: false } } }],
      seconds(1),
      ['expect', { mushrooms: { cave: { present: true, label: 'Probably a glowcap', canAbsorb: true, canTaste: true, because: ['absorb_cave'] } } }],
      ['absorb', 'cave', 'duskcap'],
      ['expect', {
        slime: { heavy: true },
        mushrooms: {
          cave: { present: false },
          ruin: { label: 'Could be a duskcap — taste first', because: ['absorb_cave_2'] },
        },
        belief: { state: 'uncertain', supportedBy: ['absorb_cave'], contradictedBy: ['absorb_cave_2'] },
        decision: { state: 'reopened', basis: ['absorb_cave'], reopenedBy: ['absorb_cave_2'] },
      }],
      seconds(45),
      ['taste', 'cave', 'glowcap'],
      ['expect', {
        mushrooms: { cave: { present: true, label: 'Probably a glowcap (tasted in the dark)', canAbsorb: true, canTaste: false, because: ['taste_cave_3'], caveats: ['tasted_in_dark'] } },
        belief: { supportedBy: ['absorb_cave', 'taste_cave_3'], contradictedBy: ['absorb_cave_2'] },
      }],
    ],
  },
  {
    id: 'S32', title: 'CR10: a regrown mushroom forgets its taste; the old taste still fades', since: 'cr10',
    steps: [
      ['taste', 'pool', 'duskcap'],
      ['witness', 'pool', 'duskcap'],
      seconds(45),
      ['expect', { mushrooms: { pool: { present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: ['taste_pool', 'witness_pool'], caveats: ['tasted_in_dark', 'secondhand'] } } }],
      ['taste', 'pool', 'glowcap'],
      ['expect', { mushrooms: { pool: { label: 'Probably a glowcap (tasted in the dark)', canAbsorb: true, canTaste: false, because: ['taste_pool_2'], caveats: ['tasted_in_dark'] } } }],
      seconds(15),
      ['expect', {
        mushrooms: {
          pool: { label: 'Probably a glowcap (tasted in the dark)', because: ['taste_pool_2'], caveats: ['tasted_in_dark'] },
          cave: { label: 'Too risky — taste first', caveats: ['tasted_in_dark', 'secondhand', 'taste_faded'] },
        },
        belief: { state: 'uncertain', supportedBy: ['taste_pool_2'], contradictedBy: ['taste_pool', 'witness_pool'], caveats: ['tasted_in_dark', 'secondhand', 'taste_faded'] },
      }],
      seconds(45),
      ['expect', {
        mushrooms: { pool: { label: 'Probably a glowcap (taste has faded)', caveats: ['tasted_in_dark', 'taste_faded'] } },
        decision: { state: 'none', history: [] },
      }],
    ],
  },
  {
    id: 'S33', title: 'CR10: regrowth has no limit', since: 'cr10',
    steps: [
      ['absorb', 'ruin', 'glowcap'],
      seconds(45),
      ['absorb', 'ruin', 'glowcap'],
      seconds(45),
      ['absorb', 'ruin', 'glowcap'],
      ['expect', {
        belief: { state: 'probably_safe', note: 'Based on 3 observations.', supportedBy: ['absorb_ruin', 'absorb_ruin_2', 'absorb_ruin_3'] },
        decision: { state: 'committed', basis: ['absorb_ruin'] },
      }],
      seconds(45),
      ['absorb', 'ruin', 'duskcap'],
      seconds(45),
      ['witness', 'ruin', 'glowcap'],
      seconds(45),
      ['absorb', 'ruin', 'glowcap'],
      ['expect', {
        belief: { state: 'uncertain', supportedBy: ['absorb_ruin', 'absorb_ruin_2', 'absorb_ruin_3', 'witness_ruin_5', 'absorb_ruin_6'], contradictedBy: ['absorb_ruin_4'] },
        decision: {
          state: 'committed', basis: ['witness_ruin_5', 'absorb_ruin_6'], reopenedBy: [], caveats: ['secondhand'],
          history: [
            { change: 'committed', because: ['absorb_ruin'] },
            { change: 'reopened', because: ['absorb_ruin_4'] },
            { change: 'committed', because: ['witness_ruin_5', 'absorb_ruin_6'] },
          ],
        },
      }],
    ],
  },
  {
    id: 'S34', title: 'CR11: every greyed-out action says why', since: 'cr11',
    steps: [
      ['expect', { mushrooms: { cave: { why: { absorb: { reason: '', because: [], caveats: [] }, taste: { reason: '', because: [], caveats: [] } } } } }],
      ['absorb', 'cave', 'glowcap'],
      ['taste', 'pool', 'duskcap'],
      ['expect', {
        mushrooms: {
          cave: { why: { absorb: { reason: 'Already eaten', because: ['absorb_cave'], caveats: [] }, taste: { reason: 'Already eaten', because: ['absorb_cave'], caveats: [] } } },
          pool: { why: { absorb: { reason: 'Known duskcap', because: ['taste_pool'], caveats: [] }, taste: { reason: 'Already tasted', because: ['taste_pool'], caveats: [] } } },
          ruin: { why: { absorb: { reason: '', because: [], caveats: [] }, taste: { reason: '', because: [], caveats: [] } } },
        },
      }],
      seconds(31),
      ['taste', 'ruin', 'glowcap'],
      ['expect', { mushrooms: { ruin: { why: { absorb: { reason: '', because: [] }, taste: { reason: 'Already tasted', because: ['taste_ruin'], caveats: ['tasted_in_dark'] } } } } }],
      ['witness', 'grove', 'duskcap'],
      ['expect', {
        mushrooms: {
          grove: { why: { absorb: { reason: 'Already eaten', because: ['witness_grove'], caveats: ['secondhand'] } } },
          ruin: { canAbsorb: true, why: { absorb: { reason: '', because: [] } } },
        },
      }],
      seconds(14),
      ['expect', {
        mushrooms: {
          cave: {
            present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true,
            why: { absorb: { reason: 'Too risky untasted', because: ['taste_pool', 'witness_grove'], caveats: ['secondhand'] }, taste: { reason: '', because: [], caveats: [] } },
          },
        },
      }],
      seconds(15),
      ['expect', {
        mushrooms: {
          cave: { why: { absorb: { reason: 'Too risky untasted', because: ['taste_pool', 'witness_grove'], caveats: ['secondhand', 'taste_faded'] } } },
          pool: { why: { absorb: { reason: 'Known duskcap', because: ['taste_pool'], caveats: ['taste_faded'] } } },
        },
      }],
    ],
  },
  {
    id: 'S35', title: 'CR11: a consumed mushroom cites what consumed its current life', since: 'cr11',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      seconds(45),
      ['witness', 'cave', 'glowcap'],
      ['expect', {
        mushrooms: { cave: { why: { absorb: { reason: 'Already eaten', because: ['witness_cave_2'], caveats: ['secondhand'] }, taste: { reason: 'Already eaten', because: ['witness_cave_2'], caveats: ['secondhand'] } } } },
        belief: { note: 'Based on 2 observations.', supportedBy: ['absorb_cave', 'witness_cave_2'] },
      }],
    ],
  },
  {
    id: 'S36', title: 'CR12: resume mid-play with everything in flight', since: 'cr12',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      seconds(10),
      ['taste', 'pool', 'duskcap'],
      ['witness', 'grove', 'glowcap'],
      ['resume'],
      ['expect', { slime: { glowing: true, heavy: false }, decision: { state: 'reopened', basis: ['absorb_cave'], reopenedBy: ['taste_pool'] } }],
      seconds(21),
      ['expect', { slime: { glowing: false } }],
      seconds(14),
      ['expect', { mushrooms: { cave: { present: true, label: 'Could be a duskcap — taste first', because: ['taste_pool'] } } }],
      ['resume'],
      seconds(10),
      ['expect', { mushrooms: { grove: { present: true, label: 'Could be a duskcap — taste first' } } }],
      seconds(15),
      ['expect', { mushrooms: { pool: { label: 'Probably a duskcap (taste has faded)', caveats: ['taste_faded'] } } }],
      ['reject', { type: 'absorb', id: 'pool', kind: 'duskcap' }],
      ['absorb', 'grove', 'glowcap'],
      ['expect', {
        decision: {
          state: 'committed', basis: ['witness_grove', 'absorb_grove_2'], reopenedBy: [], caveats: ['secondhand'],
          history: [
            { change: 'committed', because: ['absorb_cave'] },
            { change: 'reopened', because: ['taste_pool'] },
            { change: 'committed', because: ['witness_grove', 'absorb_grove_2'] },
          ],
        },
      }],
    ],
  },
  {
    id: 'S37', title: 'CR12: a save stays small after ten minutes of play', since: 'cr12',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      seconds(600),
      ['resume'],
      ['expect', { slime: { glowing: false }, mushrooms: { cave: { present: true, label: 'Probably a glowcap', because: ['absorb_cave'] } } }],
      ['absorb', 'cave', 'glowcap'],
      ['expect', { belief: { note: 'Based on 2 observations.', supportedBy: ['absorb_cave', 'absorb_cave_2'] } }],
    ],
  },
  {
    id: 'S38', title: 'CR12: resuming a game that has not started', since: 'cr12',
    steps: [
      ['resume'],
      ['expect', {
        slime: { glowing: false, heavy: false },
        mushrooms: { cave: unknown, pool: unknown, ruin: unknown, grove: unknown },
        belief: { state: 'none', supportedBy: [], contradictedBy: [] },
        decision: { state: 'none', basis: [], reopenedBy: [], history: [] },
      }],
      ['absorb', 'cave', 'glowcap'],
      ['expect', { decision: { state: 'committed', basis: ['absorb_cave'] } }],
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
