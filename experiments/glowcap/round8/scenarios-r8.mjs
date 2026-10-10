// Round 8 acceptance scenarios for CR17-CR20 (REQUESTS.md). Same step format as
// existing-scenarios.mjs. New events are written ['doubt', { evidence }],
// ['undoubt', { evidence }] and ['retaste', { id, kind }], and dispatched as
// { type: 'doubt', evidence }, etc. `journal` and `decision.history` are
// compared in order; other lists are sets.

const DT = 0.0625; // exact in binary floating point: 16 ticks = 1 s
const seconds = (s) => ['tick', DT, s * 16];

const allowed = { reason: '', because: [], caveats: [] };

const J = {
  absorbed: (id, ev) => ({ text: `Absorbed the ${id} mushroom: it gave light.`, because: [ev], caveats: [] }),
  trust: (because, caveats) => ({ text: 'I trust glowing mushrooms now.', because, caveats }),
  distrust: (ev, caveats) => ({ text: 'I no longer trust glowing mushrooms.', because: [ev], caveats }),
};

export const SCENARIOS = [
  // ── CR17: the HUD shows countdowns ──────────────────────────────────────
  {
    id: 'S51', title: 'CR17: glow, regrowth and fade count down and reach 0 when they end', since: 'cr17',
    steps: [
      ['expect', { slime: { glowSeconds: 0 }, mushrooms: { cave: { regrowsIn: 0, fadesIn: 0 }, pit: { regrowsIn: 0, fadesIn: 0 } } }],
      ['absorb', 'cave', 'glowcap'],
      ['expect', { slime: { glowing: true, glowSeconds: 30 }, mushrooms: { cave: { present: false, regrowsIn: 45, fadesIn: 0 } } }],
      ['taste', 'pool', 'glowcap'], // glowing: in the light
      ['expect', { mushrooms: { pool: { label: 'Glowcap', fadesIn: 60, regrowsIn: 0 } } }],
      ['tick', DT, 8], // 0.5 s
      ['expect', { slime: { glowSeconds: 30 }, mushrooms: { cave: { regrowsIn: 45 }, pool: { fadesIn: 60 } } }],
      seconds(29), // 29.5 s
      ['expect', { slime: { glowing: true, glowSeconds: 1 }, mushrooms: { cave: { regrowsIn: 16 }, pool: { fadesIn: 31 } } }],
      ['tick', DT, 8], // 30 s
      ['expect', { slime: { glowing: false, glowSeconds: 0 }, mushrooms: { cave: { regrowsIn: 15 }, pool: { fadesIn: 30 } } }],
      seconds(15), // 45 s: cave regrows
      ['expect', { mushrooms: { cave: { present: true, regrowsIn: 0, fadesIn: 0 }, pool: { fadesIn: 15 } } }],
      seconds(15), // 60 s: the taste fades
      ['expect', { mushrooms: { pool: { label: 'Probably a glowcap (taste has faded)', fadesIn: 0 } } }],
    ],
  },
  {
    id: 'S52', title: 'CR17: glow and heavy side by side, pit regrowth, across a resume', since: 'cr17',
    steps: [
      ['witness', 'pit', 'duskcap'],
      ['expect', { slime: { glowSeconds: 0, heavySeconds: 0 }, mushrooms: { pit: { present: false, regrowsIn: 45 } } }],
      ['absorb', 'cave', 'glowcap'],
      seconds(10),
      ['resume'],
      ['expect', { slime: { glowSeconds: 20, heavySeconds: 0 }, mushrooms: { pit: { regrowsIn: 35 }, cave: { regrowsIn: 35 } } }],
      ['absorb', 'pool', 'duskcap'],
      ['tick', DT, 1],
      ['expect', { slime: { glowSeconds: 20, heavySeconds: 20 }, mushrooms: { pool: { regrowsIn: 45 }, pit: { regrowsIn: 35 } } }],
      seconds(35),
      ['expect', {
        slime: { glowing: false, glowSeconds: 0, heavy: false, heavySeconds: 0 },
        mushrooms: {
          pit: { present: true, regrowsIn: 0 },
          cave: { present: true, regrowsIn: 0 },
          pool: { present: false, regrowsIn: 10 },
        },
      }],
    ],
  },
  {
    id: 'S53', title: 'CR17: eating a tasted mushroom hides its fade countdown; a new life counts afresh', since: 'cr17',
    steps: [
      ['taste', 'ruin', 'duskcap'], // dark
      ['expect', { mushrooms: { ruin: { fadesIn: 60, regrowsIn: 0 } } }],
      seconds(20),
      ['expect', { mushrooms: { ruin: { fadesIn: 40 } } }],
      ['witness', 'ruin', 'duskcap'],
      ['expect', { mushrooms: { ruin: { present: false, fadesIn: 0, regrowsIn: 45 } } }],
      seconds(45),
      ['expect', { mushrooms: { ruin: { present: true, label: 'Too risky — taste first', fadesIn: 0, regrowsIn: 0 } } }],
      ['taste', 'ruin', 'glowcap'], // dark
      ['expect', { mushrooms: { ruin: { because: ['taste_ruin_2'], fadesIn: 60 } } }],
      ['tick', DT, 952], // 59.5 s
      ['expect', { mushrooms: { ruin: { label: 'Probably a glowcap (tasted in the dark)', fadesIn: 1 } } }],
      ['tick', DT, 8], // 60 s
      ['expect', { mushrooms: { ruin: { label: 'Probably a glowcap (taste has faded)', fadesIn: 0 } } }],
    ],
  },
  // ── CR18: doubt what you saw ────────────────────────────────────────────
  {
    id: 'S54', title: 'CR18: a doubted witness leaves the belief but not the decision', since: 'cr18',
    steps: [
      ['absorb', 'cave', 'glowcap'],   // 1: commit
      ['witness', 'pool', 'duskcap'],  // 2: reopen
      ['witness', 'ruin', 'duskcap'],  // 3: twice bitten
      ['expect', { mushrooms: { grove: { label: 'Too risky — taste first', canAbsorb: false } } }],
      ['reject', { type: 'doubt', evidence: 'absorb_cave' }],
      ['reject', { type: 'doubt', evidence: 'witness_grove' }],
      ['doubt', { evidence: 'witness_ruin' }],
      ['expect', {
        mushrooms: {
          grove: {
            present: true, label: 'Could be a duskcap — taste first', canAbsorb: true, canTaste: true,
            because: ['witness_pool'], caveats: ['secondhand'], why: { absorb: allowed },
          },
          ruin: { present: false, why: { absorb: { reason: 'Already eaten', because: ['witness_ruin'], caveats: ['secondhand', 'doubted'] } } },
        },
        belief: { state: 'uncertain', note: '', supportedBy: ['absorb_cave'], contradictedBy: ['witness_pool'], caveats: ['secondhand'] },
        decision: {
          state: 'reopened', basis: ['absorb_cave'], reopenedBy: ['witness_pool', 'witness_ruin'], caveats: [],
          history: [
            { change: 'committed', because: ['absorb_cave'] },
            { change: 'reopened', because: ['witness_pool'] },
            { change: 'reopened', because: ['witness_ruin'] },
          ],
        },
      }],
      ['reject', { type: 'doubt', evidence: 'witness_ruin' }],
      ['doubt', { evidence: 'witness_pool' }],
      ['expect', {
        mushrooms: { grove: { label: 'Probably a glowcap', because: ['absorb_cave'], caveats: [] } },
        belief: {
          state: 'probably_safe', text: 'Glowing mushrooms give you light.', note: 'Based on one observation.',
          supportedBy: ['absorb_cave'], contradictedBy: [], caveats: [],
        },
        decision: { state: 'reopened', reopenedBy: ['witness_pool', 'witness_ruin'] },
      }],
      ['absorb', 'grove', 'glowcap'], // 4: only the first glowcap since witness_ruin
      ['expect', {
        belief: { state: 'probably_safe', note: 'Based on 2 observations.', supportedBy: ['absorb_cave', 'absorb_grove'] },
        decision: { state: 'reopened', basis: ['absorb_cave'] },
      }],
    ],
  },
  {
    id: 'S55', title: 'CR18: doubt needs a remembered witness and an undoubted observation left', since: 'cr18',
    steps: [
      ['witness', 'cave', 'glowcap'],  // 1: commit
      ['reject', { type: 'doubt', evidence: 'witness_cave' }], // the only observation
      ['reject', { type: 'doubt', evidence: 'witness_nowhere' }],
      ['witness', 'pool', 'glowcap'],  // 2
      ['doubt', { evidence: 'witness_cave' }],
      ['reject', { type: 'doubt', evidence: 'witness_pool' }], // the last undoubted one
      ['expect', {
        mushrooms: { cave: { why: { absorb: { reason: 'Already eaten', because: ['witness_cave'], caveats: ['secondhand', 'doubted'] } } } },
        belief: { state: 'probably_safe', note: 'Based on one observation.', supportedBy: ['witness_pool'], caveats: ['secondhand'] },
        decision: { state: 'committed', basis: ['witness_cave'], caveats: ['secondhand'] },
      }],
      ['absorb', 'ruin', 'glowcap'],   // 3
      ['absorb', 'grove', 'glowcap'],  // 4
      seconds(45), // all four regrow
      ['absorb', 'cave', 'glowcap'],   // 5
      ['absorb', 'pool', 'glowcap'],   // 6
      ['absorb', 'ruin', 'glowcap'],   // 7: forgets witness_cave
      ['absorb', 'grove', 'glowcap'],  // 8: forgets witness_pool
      ['reject', { type: 'doubt', evidence: 'witness_pool' }], // forgotten
      ['reject', { type: 'doubt', evidence: 'absorb_ruin' }],  // not a witness
      ['expect', {
        belief: {
          state: 'probably_safe', note: 'Based on 6 observations.', caveats: [],
          supportedBy: ['absorb_ruin', 'absorb_grove', 'absorb_cave_2', 'absorb_pool_2', 'absorb_ruin_2', 'absorb_grove_2'],
        },
      }],
    ],
  },
  {
    id: 'S56', title: 'CR18: a doubt survives resume and never commits trust by itself', since: 'cr18',
    steps: [
      ['witness', 'pit', 'duskcap'],   // 1
      ['absorb', 'cave', 'glowcap'],   // 2: uncertain, no trust
      ['doubt', { evidence: 'witness_pit' }],
      ['resume'],
      ['expect', {
        mushrooms: { pit: { present: false, why: { absorb: { reason: 'Already eaten', because: ['witness_pit'], caveats: ['secondhand', 'doubted'] } } } },
        belief: { state: 'probably_safe', supportedBy: ['absorb_cave'], contradictedBy: [], caveats: [] },
        decision: { state: 'none', basis: [], history: [] },
      }],
      ['absorb', 'pool', 'glowcap'],   // 3: commits
      ['expect', {
        belief: { note: 'Based on 2 observations.' },
        decision: { state: 'committed', basis: ['absorb_pool'], caveats: [], history: [{ change: 'committed', because: ['absorb_pool'] }] },
      }],
      seconds(45), // pit regrows
      ['expect', { mushrooms: { pit: { present: true, label: 'Probably a glowcap', because: ['absorb_cave', 'absorb_pool'], caveats: [] } } }],
      ['witness', 'pit', 'duskcap'],   // 4
      ['expect', {
        belief: { state: 'uncertain', supportedBy: ['absorb_cave', 'absorb_pool'], contradictedBy: ['witness_pit_2'] },
        decision: { state: 'reopened', basis: ['absorb_pool'], reopenedBy: ['witness_pit_2'] },
      }],
    ],
  },
  // ── CR19: taste again ───────────────────────────────────────────────────
  {
    id: 'S57', title: 'CR19: a faded taste can be refreshed once', since: 'cr19',
    steps: [
      ['taste', 'cave', 'glowcap'],    // 1, dark: commit
      ['expect', { mushrooms: { cave: { canRetaste: false }, pool: { canRetaste: false }, pit: { canRetaste: false } } }],
      ['reject', { type: 'retaste', id: 'cave', kind: 'glowcap' }], // not faded
      seconds(60),
      ['expect', { mushrooms: { cave: { label: 'Probably a glowcap (taste has faded)', canRetaste: true } } }],
      ['reject', { type: 'retaste', id: 'cave', kind: 'duskcap' }],
      ['reject', { type: 'retaste', id: 'pool', kind: 'glowcap' }],
      ['absorb', 'ruin', 'glowcap'],   // 2: glowing
      ['retaste', { id: 'cave', kind: 'glowcap' }], // 3, in the light
      ['expect', {
        mushrooms: {
          cave: {
            present: true, label: 'Glowcap', canAbsorb: true, canTaste: false, canRetaste: false,
            because: ['retaste_cave'], caveats: [], fadesIn: 60,
            why: { absorb: allowed, taste: { reason: 'Already tasted', because: ['retaste_cave'], caveats: [] } },
          },
        },
        belief: {
          state: 'probably_safe', note: 'Based on 3 observations.',
          supportedBy: ['taste_cave', 'absorb_ruin', 'retaste_cave'], caveats: ['tasted_in_dark', 'taste_faded'],
        },
        decision: { state: 'committed', basis: ['taste_cave'], caveats: ['tasted_in_dark'], history: [{ change: 'committed', because: ['taste_cave'] }] },
        journal: [
          { text: 'Tasted the cave mushroom: sweet.', because: ['taste_cave'], caveats: ['tasted_in_dark', 'taste_faded'] },
          J.trust(['taste_cave'], ['tasted_in_dark', 'taste_faded']),
          J.absorbed('ruin', 'absorb_ruin'),
          { text: 'Tasted the cave mushroom again: sweet.', because: ['retaste_cave'], caveats: [] },
        ],
      }],
      ['reject', { type: 'retaste', id: 'cave', kind: 'glowcap' }], // once per life
      ['reject', { type: 'taste', id: 'cave', kind: 'glowcap' }],
      seconds(60),
      ['expect', {
        mushrooms: { cave: { label: 'Probably a glowcap (taste has faded)', because: ['retaste_cave'], caveats: ['taste_faded'], canRetaste: false, fadesIn: 0 } },
      }],
    ],
  },
  {
    id: 'S58', title: 'CR19: a bitter retaste of a marked mushroom reopens trust again', since: 'cr19',
    steps: [
      ['absorb', 'cave', 'glowcap'],   // 1: commit, glowing until 30 s
      ['taste', 'pool', 'duskcap'],    // 2, in the light: reopen
      ['mark', { id: 'pool' }],
      seconds(60), // the taste fades; cave regrows at 45 s
      ['expect', { mushrooms: { pool: { marked: true, label: 'Probably a duskcap (taste has faded)', canRetaste: true } } }],
      ['reject', { type: 'retaste', id: 'pit', kind: 'duskcap' }],
      ['retaste', { id: 'pool', kind: 'duskcap' }], // 3, dark
      ['expect', {
        mushrooms: {
          pool: {
            marked: true, label: 'Probably a duskcap (tasted in the dark)', canAbsorb: false, canTaste: false, canRetaste: false,
            because: ['retaste_pool'], caveats: ['tasted_in_dark'],
            why: {
              absorb: { reason: 'Known duskcap', because: ['retaste_pool'], caveats: ['tasted_in_dark'] },
              taste: { reason: 'Already tasted', because: ['retaste_pool'], caveats: ['tasted_in_dark'] },
            },
          },
          cave: {
            present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true,
            because: ['taste_pool', 'retaste_pool'], caveats: ['taste_faded', 'tasted_in_dark'],
          },
          pit: { canRetaste: false },
        },
        belief: {
          state: 'uncertain', supportedBy: ['absorb_cave'], contradictedBy: ['taste_pool', 'retaste_pool'],
          caveats: ['taste_faded', 'tasted_in_dark'],
        },
        decision: {
          state: 'reopened', basis: ['absorb_cave'], reopenedBy: ['taste_pool', 'retaste_pool'],
          history: [
            { change: 'committed', because: ['absorb_cave'] },
            { change: 'reopened', because: ['taste_pool'] },
            { change: 'reopened', because: ['retaste_pool'] },
          ],
        },
        journal: [
          J.absorbed('cave', 'absorb_cave'),
          J.trust(['absorb_cave'], []),
          { text: 'Tasted the pool mushroom: bitter.', because: ['taste_pool'], caveats: ['taste_faded'] },
          J.distrust('taste_pool', ['taste_faded']),
          { text: 'Tasted the pool mushroom again: bitter.', because: ['retaste_pool'], caveats: ['tasted_in_dark'] },
          J.distrust('retaste_pool', ['tasted_in_dark']),
        ],
      }],
      ['reject', { type: 'witness', id: 'pool', kind: 'glowcap' }],
    ],
  },
  {
    id: 'S59', title: 'CR19: a retaste counts for recovery, survives resume, and a new life needs a fresh taste', since: 'cr19',
    steps: [
      ['absorb', 'cave', 'glowcap'],   // 1: commit
      ['absorb', 'pool', 'duskcap'],   // 2: reopen
      ['taste', 'ruin', 'glowcap'],    // 3, in the light: first glowcap since absorb_pool
      seconds(30),
      ['resume'],
      seconds(30), // taste_ruin fades
      ['expect', { mushrooms: { ruin: { label: 'Probably a glowcap (taste has faded)', canRetaste: true, fadesIn: 0 } } }],
      ['retaste', { id: 'ruin', kind: 'glowcap' }], // 4, dark: second glowcap, recommit
      ['expect', {
        mushrooms: { ruin: { label: 'Probably a glowcap (tasted in the dark)', because: ['retaste_ruin'], caveats: ['tasted_in_dark'], fadesIn: 60 } },
        decision: {
          state: 'committed', basis: ['taste_ruin', 'retaste_ruin'], reopenedBy: [], caveats: ['taste_faded', 'tasted_in_dark'],
          history: [
            { change: 'committed', because: ['absorb_cave'] },
            { change: 'reopened', because: ['absorb_pool'] },
            { change: 'committed', because: ['taste_ruin', 'retaste_ruin'] },
          ],
        },
      }],
      ['resume'],
      ['absorb', 'ruin', 'glowcap'],   // 5
      seconds(45), // ruin regrows untasted
      ['expect', {
        mushrooms: {
          ruin: {
            present: true, label: 'Could be a duskcap — taste first', canAbsorb: true, canTaste: true, canRetaste: false,
            because: ['absorb_pool'], caveats: [], fadesIn: 0,
          },
        },
        belief: { state: 'uncertain', supportedBy: ['absorb_cave', 'taste_ruin', 'retaste_ruin', 'absorb_ruin'], contradictedBy: ['absorb_pool'] },
      }],
      ['reject', { type: 'retaste', id: 'ruin', kind: 'glowcap' }],
    ],
  },
  // ── CR20: take back a doubt, and write doubts down ──────────────────────
  {
    id: 'S60', title: 'CR20: doubts and undoubts are journaled with current caveats', since: 'cr20',
    steps: [
      ['absorb', 'cave', 'glowcap'],   // 1: commit
      ['witness', 'pool', 'duskcap'],  // 2: reopen
      ['doubt', { evidence: 'witness_pool' }],
      ['expect', {
        belief: { state: 'probably_safe', supportedBy: ['absorb_cave'], contradictedBy: [] },
        journal: [
          J.absorbed('cave', 'absorb_cave'),
          J.trust(['absorb_cave'], []),
          { text: 'Saw a creature eat the pool mushroom: it grew heavy.', because: ['witness_pool'], caveats: ['secondhand', 'doubted'] },
          J.distrust('witness_pool', ['secondhand', 'doubted']),
          { text: 'I doubt what I saw at the pool mushroom.', because: ['witness_pool'], caveats: ['secondhand', 'doubted'] },
        ],
      }],
      ['reject', { type: 'undoubt', evidence: 'absorb_cave' }],
      ['reject', { type: 'undoubt', evidence: 'witness_ruin' }],
      ['undoubt', { evidence: 'witness_pool' }],
      ['expect', {
        belief: { state: 'uncertain', supportedBy: ['absorb_cave'], contradictedBy: ['witness_pool'], caveats: ['secondhand'] },
        decision: { state: 'reopened', reopenedBy: ['witness_pool'] },
        journal: [
          J.absorbed('cave', 'absorb_cave'),
          J.trust(['absorb_cave'], []),
          { text: 'Saw a creature eat the pool mushroom: it grew heavy.', because: ['witness_pool'], caveats: ['secondhand'] },
          J.distrust('witness_pool', ['secondhand']),
          { text: 'I doubt what I saw at the pool mushroom.', because: ['witness_pool'], caveats: ['secondhand'] },
          { text: 'I believe what I saw at the pool mushroom again.', because: ['witness_pool'], caveats: ['secondhand'] },
        ],
      }],
      ['reject', { type: 'undoubt', evidence: 'witness_pool' }],
      ['doubt', { evidence: 'witness_pool' }],
      ['expect', {
        journal: [
          J.trust(['absorb_cave'], []),
          { text: 'Saw a creature eat the pool mushroom: it grew heavy.', because: ['witness_pool'], caveats: ['secondhand', 'doubted'] },
          J.distrust('witness_pool', ['secondhand', 'doubted']),
          { text: 'I doubt what I saw at the pool mushroom.', because: ['witness_pool'], caveats: ['secondhand', 'doubted'] },
          { text: 'I believe what I saw at the pool mushroom again.', because: ['witness_pool'], caveats: ['secondhand', 'doubted'] },
          { text: 'I doubt what I saw at the pool mushroom.', because: ['witness_pool'], caveats: ['secondhand', 'doubted'] },
        ],
      }],
    ],
  },
  {
    id: 'S61', title: 'CR20: a forgotten doubt cannot be taken back and stays in the journal', since: 'cr20',
    steps: [
      ['witness', 'cave', 'glowcap'],  // 1: commit
      ['absorb', 'pool', 'glowcap'],   // 2
      ['doubt', { evidence: 'witness_cave' }],
      ['resume'],
      ['expect', {
        mushrooms: { cave: { why: { absorb: { reason: 'Already eaten', because: ['witness_cave'], caveats: ['secondhand', 'doubted'] } } } },
        belief: { note: 'Based on one observation.', supportedBy: ['absorb_pool'], caveats: [] },
      }],
      ['absorb', 'ruin', 'glowcap'],   // 3
      ['absorb', 'grove', 'glowcap'],  // 4
      seconds(45), // all four regrow
      ['absorb', 'cave', 'glowcap'],   // 5
      ['absorb', 'pool', 'glowcap'],   // 6
      ['absorb', 'ruin', 'glowcap'],   // 7: forgets witness_cave
      ['reject', { type: 'undoubt', evidence: 'witness_cave' }],
      ['expect', {
        belief: {
          note: 'Based on 6 observations.',
          supportedBy: ['absorb_pool', 'absorb_ruin', 'absorb_grove', 'absorb_cave_2', 'absorb_pool_2', 'absorb_ruin_2'],
        },
        decision: { state: 'committed', basis: ['witness_cave'], caveats: ['secondhand'], history: [{ change: 'committed', because: ['witness_cave'] }] },
        journal: [
          { text: 'I doubt what I saw at the cave mushroom.', because: ['witness_cave'], caveats: ['secondhand', 'doubted'] },
          J.absorbed('ruin', 'absorb_ruin'),
          J.absorbed('grove', 'absorb_grove'),
          J.absorbed('cave', 'absorb_cave_2'),
          J.absorbed('pool', 'absorb_pool_2'),
          J.absorbed('ruin', 'absorb_ruin_2'),
        ],
      }],
    ],
  },
  {
    id: 'S62', title: 'CR20: undoubting can bring back twice bitten but never touches trust', since: 'cr20',
    steps: [
      ['absorb', 'cave', 'glowcap'],   // 1: commit
      ['witness', 'pool', 'duskcap'],  // 2: reopen
      ['doubt', { evidence: 'witness_pool' }],
      ['witness', 'ruin', 'duskcap'],  // 3: reopen again
      ['expect', {
        mushrooms: { grove: { label: 'Could be a duskcap — taste first', canAbsorb: true, because: ['witness_ruin'] } },
        belief: { state: 'uncertain', contradictedBy: ['witness_ruin'] },
      }],
      ['absorb', 'grove', 'glowcap'],  // 4: first glowcap since witness_ruin
      ['undoubt', { evidence: 'witness_pool' }],
      ['expect', {
        mushrooms: {
          pit: { present: true, label: 'Too risky — taste first', canAbsorb: false, because: ['witness_pool', 'witness_ruin'], caveats: ['secondhand'] },
        },
        belief: {
          state: 'uncertain', supportedBy: ['absorb_cave', 'absorb_grove'], contradictedBy: ['witness_pool', 'witness_ruin'], caveats: ['secondhand'],
        },
        decision: {
          state: 'reopened', basis: ['absorb_cave'], reopenedBy: ['witness_pool', 'witness_ruin'],
          history: [
            { change: 'committed', because: ['absorb_cave'] },
            { change: 'reopened', because: ['witness_pool'] },
            { change: 'reopened', because: ['witness_ruin'] },
          ],
        },
        journal: [
          J.distrust('witness_pool', ['secondhand']),
          { text: 'I doubt what I saw at the pool mushroom.', because: ['witness_pool'], caveats: ['secondhand'] },
          { text: 'Saw a creature eat the ruin mushroom: it grew heavy.', because: ['witness_ruin'], caveats: ['secondhand'] },
          J.distrust('witness_ruin', ['secondhand']),
          J.absorbed('grove', 'absorb_grove'),
          { text: 'I believe what I saw at the pool mushroom again.', because: ['witness_pool'], caveats: ['secondhand'] },
        ],
      }],
      ['witness', 'pit', 'glowcap'],   // 5: second glowcap since witness_ruin: recommit
      ['expect', {
        decision: { state: 'committed', basis: ['absorb_grove', 'witness_pit'], reopenedBy: [], caveats: ['secondhand'] },
      }],
    ],
  },
];
