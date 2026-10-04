// Round 7 acceptance scenarios for CR13-CR16 (REQUESTS.md). Same step format as
// existing-scenarios.mjs. New events are written ['mark', { id }] and
// ['unmark', { id }] and dispatched as { type: 'mark', id }.
// `journal` (CR16) and `decision.history` are compared in order.

const DT = 0.0625; // exact in binary floating point: 16 ticks = 1 s
const seconds = (s) => ['tick', DT, s * 16];

const allowed = { reason: '', because: [], caveats: [] };
const outOfReach = { reason: 'Out of reach', because: [], caveats: [] };
const markedWhy = { reason: 'Marked to avoid', because: [], caveats: [] };

export const SCENARIOS = [
  // ── CR13: the slime forgets ─────────────────────────────────────────────
  {
    id: 'S39', title: 'CR13: forgetting old bites lifts "too risky" and lets trust commit', since: 'cr13',
    steps: [
      ['absorb', 'pool', 'duskcap'],   // 1
      ['absorb', 'ruin', 'duskcap'],   // 2
      ['reject', { type: 'absorb', id: 'cave', kind: 'glowcap' }],
      ['taste', 'cave', 'glowcap'],    // 3, in the dark
      ['taste', 'grove', 'glowcap'],   // 4, in the dark
      ['absorb', 'cave', 'glowcap'],   // 5
      ['absorb', 'grove', 'glowcap'],  // 6
      ['expect', {
        belief: {
          state: 'uncertain', supportedBy: ['taste_cave', 'taste_grove', 'absorb_cave', 'absorb_grove'],
          contradictedBy: ['absorb_pool', 'absorb_ruin'],
        },
        decision: { state: 'none' },
      }],
      seconds(45), // all four regrow
      ['expect', { mushrooms: { cave: { present: true, label: 'Too risky — taste first', canAbsorb: false } } }],
      ['taste', 'pool', 'glowcap'],    // 7: forgets absorb_pool
      ['expect', {
        mushrooms: {
          cave: {
            present: true, label: 'Could be a duskcap — taste first', canAbsorb: true, canTaste: true,
            because: ['absorb_ruin'], caveats: [], why: { absorb: allowed },
          },
        },
        belief: {
          state: 'uncertain', note: '',
          supportedBy: ['taste_cave', 'taste_grove', 'absorb_cave', 'absorb_grove', 'taste_pool_2'],
          contradictedBy: ['absorb_ruin'], caveats: ['tasted_in_dark'],
        },
        decision: { state: 'none', basis: [] },
      }],
      ['taste', 'ruin', 'glowcap'],    // 8: forgets absorb_ruin
      ['expect', {
        mushrooms: {
          cave: {
            label: 'Probably a glowcap', canAbsorb: true,
            because: ['taste_cave', 'taste_grove', 'absorb_cave', 'absorb_grove', 'taste_pool_2', 'taste_ruin_2'],
            caveats: ['tasted_in_dark'],
          },
          ruin: { label: 'Probably a glowcap (tasted in the dark)', because: ['taste_ruin_2'] },
        },
        belief: {
          state: 'probably_safe', text: 'Glowing mushrooms give you light.', note: 'Based on 6 observations.',
          supportedBy: ['taste_cave', 'taste_grove', 'absorb_cave', 'absorb_grove', 'taste_pool_2', 'taste_ruin_2'],
          contradictedBy: [], caveats: ['tasted_in_dark'],
        },
        decision: {
          state: 'committed', basis: ['taste_ruin_2'], reopenedBy: [], caveats: ['tasted_in_dark'],
          history: [{ change: 'committed', because: ['taste_ruin_2'] }],
        },
      }],
    ],
  },
  {
    id: 'S40', title: 'CR13: a forgotten taste still names its mushroom; reopenedBy and history keep six', since: 'cr13',
    steps: [
      ['absorb', 'cave', 'glowcap'],   // 1: commit
      ['taste', 'grove', 'duskcap'],   // 2, in the light
      ['taste', 'pool', 'duskcap'],    // 3
      ['taste', 'ruin', 'duskcap'],    // 4
      ['witness', 'pool', 'duskcap'],  // 5
      ['witness', 'ruin', 'duskcap'],  // 6
      ['expect', {
        decision: {
          state: 'reopened', reopenedBy: ['taste_grove', 'taste_pool', 'taste_ruin', 'witness_pool', 'witness_ruin'],
          history: [
            { change: 'committed', because: ['absorb_cave'] },
            { change: 'reopened', because: ['taste_grove'] },
            { change: 'reopened', because: ['taste_pool'] },
            { change: 'reopened', because: ['taste_ruin'] },
            { change: 'reopened', because: ['witness_pool'] },
            { change: 'reopened', because: ['witness_ruin'] },
          ],
        },
      }],
      seconds(45), // cave, pool, ruin regrow
      ['witness', 'cave', 'duskcap'],  // 7: forgets absorb_cave
      ['witness', 'pool', 'duskcap'],  // 8: forgets taste_grove
      ['expect', {
        mushrooms: {
          grove: { present: true, label: 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: ['taste_grove'], caveats: [] },
          ruin: {
            present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true,
            because: ['taste_pool', 'taste_ruin', 'witness_pool', 'witness_ruin', 'witness_cave_2', 'witness_pool_2'],
            caveats: ['secondhand'],
          },
        },
        belief: {
          state: 'probably_unsafe', text: 'Glowing mushrooms make you heavy.', note: 'Based on 6 observations.',
          supportedBy: [],
          contradictedBy: ['taste_pool', 'taste_ruin', 'witness_pool', 'witness_ruin', 'witness_cave_2', 'witness_pool_2'],
          caveats: ['secondhand'],
        },
        decision: {
          state: 'reopened', basis: ['absorb_cave'], caveats: [],
          reopenedBy: ['taste_pool', 'taste_ruin', 'witness_pool', 'witness_ruin', 'witness_cave_2', 'witness_pool_2'],
          history: [
            { change: 'reopened', because: ['taste_pool'] },
            { change: 'reopened', because: ['taste_ruin'] },
            { change: 'reopened', because: ['witness_pool'] },
            { change: 'reopened', because: ['witness_ruin'] },
            { change: 'reopened', because: ['witness_cave_2'] },
            { change: 'reopened', because: ['witness_pool_2'] },
          ],
        },
      }],
      ['reject', { type: 'absorb', id: 'grove', kind: 'duskcap' }],
      ['reject', { type: 'witness', id: 'grove', kind: 'glowcap' }],
    ],
  },
  {
    id: 'S41', title: 'CR13: the window survives resume; recovery counts and forgetting interleave', since: 'cr13',
    steps: [
      ['taste', 'cave', 'glowcap'],    // 1, dark: commit
      ['taste', 'pool', 'glowcap'],    // 2, dark
      ['taste', 'ruin', 'glowcap'],    // 3, dark
      ['taste', 'grove', 'duskcap'],   // 4, dark: reopen
      ['absorb', 'cave', 'glowcap'],   // 5
      ['resume'],
      ['absorb', 'pool', 'glowcap'],   // 6: second glowcap since the contradiction, recommit
      ['absorb', 'ruin', 'glowcap'],   // 7: forgets taste_cave
      ['expect', {
        belief: {
          state: 'uncertain',
          supportedBy: ['taste_pool', 'taste_ruin', 'absorb_cave', 'absorb_pool', 'absorb_ruin'],
          contradictedBy: ['taste_grove'], caveats: ['tasted_in_dark'],
        },
        decision: { state: 'committed', basis: ['absorb_cave', 'absorb_pool'], reopenedBy: [], caveats: [] },
      }],
      ['resume'],
      ['witness', 'grove', 'duskcap'], // 8: forgets taste_pool, reopens
      ['expect', {
        belief: {
          state: 'uncertain',
          supportedBy: ['taste_ruin', 'absorb_cave', 'absorb_pool', 'absorb_ruin'],
          contradictedBy: ['taste_grove', 'witness_grove'], caveats: ['tasted_in_dark', 'secondhand'],
        },
        decision: {
          state: 'reopened', basis: ['absorb_cave', 'absorb_pool'], reopenedBy: ['witness_grove'], caveats: [],
          history: [
            { change: 'committed', because: ['taste_cave'] },
            { change: 'reopened', because: ['taste_grove'] },
            { change: 'committed', because: ['absorb_cave', 'absorb_pool'] },
            { change: 'reopened', because: ['witness_grove'] },
          ],
        },
      }],
    ],
  },
  // ── CR14: the player can mark a mushroom ────────────────────────────────
  {
    id: 'S42', title: 'CR14: a marked mushroom cannot be absorbed but can be tasted', since: 'cr14',
    steps: [
      ['expect', { mushrooms: { cave: { marked: false }, pool: { marked: false } } }],
      ['mark', { id: 'pool' }],
      ['expect', {
        mushrooms: {
          pool: {
            present: true, marked: true, label: 'Glowing mushroom', canAbsorb: false, canTaste: true, because: [],
            why: { absorb: markedWhy, taste: allowed },
          },
          cave: { marked: false, canAbsorb: true },
        },
        belief: { state: 'none' },
      }],
      ['reject', { type: 'absorb', id: 'pool', kind: 'glowcap' }],
      ['reject', { type: 'mark', id: 'pool' }],
      ['reject', { type: 'mark', id: 'nowhere' }],
      ['reject', { type: 'unmark', id: 'cave' }],
      ['taste', 'pool', 'glowcap'],
      ['expect', {
        mushrooms: {
          pool: {
            marked: true, label: 'Probably a glowcap (tasted in the dark)', canAbsorb: false, canTaste: false,
            because: ['taste_pool'], caveats: ['tasted_in_dark'],
            why: { absorb: markedWhy, taste: { reason: 'Already tasted', because: ['taste_pool'], caveats: ['tasted_in_dark'] } },
          },
        },
        belief: { state: 'probably_safe', supportedBy: ['taste_pool'] },
        decision: { state: 'committed', basis: ['taste_pool'] },
      }],
      ['unmark', { id: 'pool' }],
      ['expect', { mushrooms: { pool: { marked: false, canAbsorb: true, why: { absorb: allowed } } } }],
      ['reject', { type: 'unmark', id: 'pool' }],
      ['absorb', 'pool', 'glowcap'],
      ['expect', {
        slime: { glowing: true },
        mushrooms: { pool: { present: false, marked: false } },
        belief: { supportedBy: ['taste_pool', 'absorb_pool'] },
      }],
    ],
  },
  {
    id: 'S43', title: 'CR14: why-not priority, witnesses ignore marks, eating clears them', since: 'cr14',
    steps: [
      ['taste', 'cave', 'duskcap'],    // dark
      ['mark', { id: 'cave' }],
      ['expect', {
        mushrooms: {
          cave: {
            marked: true, label: 'Probably a duskcap (tasted in the dark)', canAbsorb: false,
            why: { absorb: { reason: 'Known duskcap', because: ['taste_cave'], caveats: ['tasted_in_dark'] } },
          },
        },
      }],
      ['witness', 'ruin', 'duskcap'],
      ['mark', { id: 'grove' }],
      ['expect', {
        mushrooms: {
          grove: {
            marked: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true,
            because: ['taste_cave', 'witness_ruin'], why: { absorb: markedWhy },
          },
          pool: {
            marked: false,
            why: { absorb: { reason: 'Too risky untasted', because: ['taste_cave', 'witness_ruin'], caveats: ['tasted_in_dark', 'secondhand'] } },
          },
        },
      }],
      ['witness', 'grove', 'duskcap'],
      ['expect', {
        mushrooms: { grove: { present: false, marked: false, why: { absorb: { reason: 'Already eaten', because: ['witness_grove'], caveats: ['secondhand'] } } } },
        belief: { contradictedBy: ['taste_cave', 'witness_ruin', 'witness_grove'] },
      }],
      ['reject', { type: 'unmark', id: 'grove' }],
      ['reject', { type: 'mark', id: 'grove' }],
      seconds(45),
      ['expect', {
        mushrooms: {
          grove: {
            present: true, marked: false, label: 'Too risky — taste first', canAbsorb: false,
            because: ['taste_cave', 'witness_ruin', 'witness_grove'], caveats: ['tasted_in_dark', 'secondhand'],
            why: { absorb: { reason: 'Too risky untasted', because: ['taste_cave', 'witness_ruin', 'witness_grove'], caveats: ['tasted_in_dark', 'secondhand'] } },
          },
          cave: { present: true, marked: true },
        },
        belief: { state: 'probably_unsafe', contradictedBy: ['taste_cave', 'witness_ruin', 'witness_grove'] },
        decision: { state: 'none', history: [] },
      }],
    ],
  },
  {
    id: 'S44', title: 'CR14: marks survive resume', since: 'cr14',
    steps: [
      ['mark', { id: 'ruin' }],
      ['mark', { id: 'grove' }],
      ['resume'],
      ['expect', {
        mushrooms: {
          ruin: { marked: true, canAbsorb: false }, grove: { marked: true, canAbsorb: false },
          cave: { marked: false, canAbsorb: true },
        },
      }],
      ['unmark', { id: 'grove' }],
      ['resume'],
      ['expect', { mushrooms: { grove: { marked: false, canAbsorb: true }, ruin: { marked: true } } }],
      ['reject', { type: 'absorb', id: 'ruin', kind: 'glowcap' }],
      ['absorb', 'grove', 'glowcap'],
      ['expect', { decision: { state: 'committed', basis: ['absorb_grove'] } }],
      ['witness', 'ruin', 'glowcap'],
      ['expect', {
        mushrooms: { ruin: { present: false, marked: false } },
        belief: { supportedBy: ['absorb_grove', 'witness_ruin'] },
      }],
    ],
  },
  // ── CR15: a mushroom out of reach ───────────────────────────────────────
  {
    id: 'S45', title: 'CR15: the pit can only be witnessed, and regrows', since: 'cr15',
    steps: [
      ['expect', {
        mushrooms: {
          pit: {
            present: true, marked: false, label: 'Glowing mushroom', canAbsorb: false, canTaste: false,
            because: [], caveats: [], why: { absorb: outOfReach, taste: outOfReach },
          },
        },
      }],
      ['reject', { type: 'absorb', id: 'pit', kind: 'glowcap' }],
      ['reject', { type: 'taste', id: 'pit', kind: 'glowcap' }],
      ['reject', { type: 'mark', id: 'pit' }],
      ['reject', { type: 'unmark', id: 'pit' }],
      ['witness', 'pit', 'glowcap'],
      ['expect', {
        slime: { glowing: false },
        mushrooms: {
          pit: {
            present: false, label: '', canAbsorb: false, canTaste: false, because: [],
            why: {
              absorb: { reason: 'Already eaten', because: ['witness_pit'], caveats: ['secondhand'] },
              taste: { reason: 'Already eaten', because: ['witness_pit'], caveats: ['secondhand'] },
            },
          },
        },
        belief: { state: 'probably_safe', supportedBy: ['witness_pit'] },
        decision: { state: 'committed', basis: ['witness_pit'], caveats: ['secondhand'] },
      }],
      seconds(45),
      ['expect', {
        mushrooms: {
          pit: {
            present: true, label: 'Probably a glowcap', canAbsorb: false, canTaste: false,
            because: ['witness_pit'], caveats: ['secondhand'], why: { absorb: outOfReach, taste: outOfReach },
          },
        },
      }],
      ['witness', 'pit', 'duskcap'],
      ['expect', {
        mushrooms: { cave: { label: 'Could be a duskcap — taste first', because: ['witness_pit_2'], caveats: ['secondhand'] } },
        belief: { state: 'uncertain', supportedBy: ['witness_pit'], contradictedBy: ['witness_pit_2'] },
        decision: { state: 'reopened', basis: ['witness_pit'], reopenedBy: ['witness_pit_2'] },
      }],
    ],
  },
  {
    id: 'S46', title: 'CR15: out of reach comes before too risky; a pit witness helps trust recover', since: 'cr15',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['absorb', 'pool', 'duskcap'],
      ['absorb', 'ruin', 'duskcap'],
      ['expect', {
        mushrooms: {
          pit: {
            present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: false,
            because: ['absorb_pool', 'absorb_ruin'], why: { absorb: outOfReach, taste: outOfReach },
          },
          grove: {
            label: 'Too risky — taste first', canAbsorb: false, canTaste: true,
            why: { absorb: { reason: 'Too risky untasted', because: ['absorb_pool', 'absorb_ruin'], caveats: [] }, taste: allowed },
          },
        },
      }],
      ['witness', 'pit', 'glowcap'],
      ['expect', {
        belief: { supportedBy: ['absorb_cave', 'witness_pit'], contradictedBy: ['absorb_pool', 'absorb_ruin'] },
        decision: { state: 'reopened', reopenedBy: ['absorb_pool', 'absorb_ruin'] },
      }],
      ['taste', 'grove', 'glowcap'], // glowing, so no dark caveat
      ['expect', {
        mushrooms: { grove: { label: 'Glowcap', caveats: [] } },
        decision: {
          state: 'committed', basis: ['witness_pit', 'taste_grove'], reopenedBy: [], caveats: ['secondhand'],
          history: [
            { change: 'committed', because: ['absorb_cave'] },
            { change: 'reopened', because: ['absorb_pool'] },
            { change: 'reopened', because: ['absorb_ruin'] },
            { change: 'committed', because: ['witness_pit', 'taste_grove'] },
          ],
        },
      }],
    ],
  },
  {
    id: 'S47', title: 'CR15: the pit regrows across a resume and stays out of reach', since: 'cr15',
    steps: [
      ['witness', 'pit', 'duskcap'],
      seconds(30),
      ['resume'],
      seconds(14),
      ['expect', { mushrooms: { pit: { present: false } } }],
      seconds(1),
      ['expect', {
        mushrooms: {
          pit: { present: true, label: 'Probably a duskcap', canAbsorb: false, canTaste: false, because: ['witness_pit'], caveats: ['secondhand'] },
          cave: { label: 'Probably a duskcap', canAbsorb: true, canTaste: true, because: ['witness_pit'] },
        },
      }],
      ['reject', { type: 'taste', id: 'pit', kind: 'duskcap' }],
      ['reject', { type: 'absorb', id: 'pit', kind: 'duskcap' }],
    ],
  },
  // ── CR16: the journal ───────────────────────────────────────────────────
  {
    id: 'S48', title: 'CR16: the journal records observations and trust, and later caveats', since: 'cr16',
    steps: [
      ['expect', { journal: [] }],
      ['absorb', 'cave', 'glowcap'],
      ['expect', {
        journal: [
          { text: 'Absorbed the cave mushroom: it gave light.', because: ['absorb_cave'], caveats: [] },
          { text: 'I trust glowing mushrooms now.', because: ['absorb_cave'], caveats: [] },
        ],
      }],
      seconds(31),
      ['taste', 'pool', 'duskcap'], // dark
      ['mark', { id: 'ruin' }],
      ['reject', { type: 'absorb', id: 'ruin', kind: 'glowcap' }],
      ['reject', { type: 'taste', id: 'pool', kind: 'duskcap' }],
      ['unmark', { id: 'ruin' }],
      ['expect', {
        journal: [
          { text: 'Absorbed the cave mushroom: it gave light.', because: ['absorb_cave'], caveats: [] },
          { text: 'I trust glowing mushrooms now.', because: ['absorb_cave'], caveats: [] },
          { text: 'Tasted the pool mushroom: bitter.', because: ['taste_pool'], caveats: ['tasted_in_dark'] },
          { text: 'I no longer trust glowing mushrooms.', because: ['taste_pool'], caveats: ['tasted_in_dark'] },
        ],
      }],
      seconds(60), // the taste fades; cave regrew at 45 s
      ['expect', {
        mushrooms: { cave: { present: true } },
        journal: [
          { text: 'Absorbed the cave mushroom: it gave light.', because: ['absorb_cave'], caveats: [] },
          { text: 'I trust glowing mushrooms now.', because: ['absorb_cave'], caveats: [] },
          { text: 'Tasted the pool mushroom: bitter.', because: ['taste_pool'], caveats: ['tasted_in_dark', 'taste_faded'] },
          { text: 'I no longer trust glowing mushrooms.', because: ['taste_pool'], caveats: ['tasted_in_dark', 'taste_faded'] },
        ],
        decision: { state: 'reopened', caveats: [] },
      }],
    ],
  },
  {
    id: 'S49', title: 'CR16: the journal keeps six entries and names mushrooms without lives', since: 'cr16',
    steps: [
      ['absorb', 'cave', 'glowcap'],
      ['absorb', 'pool', 'glowcap'],
      ['taste', 'ruin', 'glowcap'],     // glowing
      ['witness', 'ruin', 'glowcap'],
      ['witness', 'grove', 'duskcap'],
      ['expect', {
        journal: [
          { text: 'I trust glowing mushrooms now.', because: ['absorb_cave'], caveats: [] },
          { text: 'Absorbed the pool mushroom: it gave light.', because: ['absorb_pool'], caveats: [] },
          { text: 'Tasted the ruin mushroom: sweet.', because: ['taste_ruin'], caveats: [] },
          { text: 'Saw a creature eat the ruin mushroom: it glowed.', because: ['witness_ruin'], caveats: ['secondhand'] },
          { text: 'Saw a creature eat the grove mushroom: it grew heavy.', because: ['witness_grove'], caveats: ['secondhand'] },
          { text: 'I no longer trust glowing mushrooms.', because: ['witness_grove'], caveats: ['secondhand'] },
        ],
      }],
      ['resume'],
      seconds(45), // everything regrows
      ['absorb', 'cave', 'glowcap'],
      ['absorb', 'pool', 'glowcap'],    // second glowcap since witness_grove: recommit
      ['expect', {
        journal: [
          { text: 'Saw a creature eat the ruin mushroom: it glowed.', because: ['witness_ruin'], caveats: ['secondhand'] },
          { text: 'Saw a creature eat the grove mushroom: it grew heavy.', because: ['witness_grove'], caveats: ['secondhand'] },
          { text: 'I no longer trust glowing mushrooms.', because: ['witness_grove'], caveats: ['secondhand'] },
          { text: 'Absorbed the cave mushroom: it gave light.', because: ['absorb_cave_2'], caveats: [] },
          { text: 'Absorbed the pool mushroom: it gave light.', because: ['absorb_pool_2'], caveats: [] },
          { text: 'I trust glowing mushrooms now.', because: ['absorb_cave_2', 'absorb_pool_2'], caveats: [] },
        ],
        decision: { state: 'committed', basis: ['absorb_cave_2', 'absorb_pool_2'] },
      }],
    ],
  },
  {
    id: 'S50', title: 'CR16: pit witnesses are journaled; marks, regrowth and rejections are not', since: 'cr16',
    steps: [
      ['witness', 'pit', 'duskcap'],
      ['mark', { id: 'cave' }],
      ['unmark', { id: 'cave' }],
      ['reject', { type: 'absorb', id: 'pit', kind: 'duskcap' }],
      ['expect', {
        journal: [
          { text: 'Saw a creature eat the pit mushroom: it grew heavy.', because: ['witness_pit'], caveats: ['secondhand'] },
        ],
        decision: { state: 'none' },
      }],
      seconds(45), // pit regrows
      ['taste', 'cave', 'glowcap'], // dark
      ['witness', 'pit', 'glowcap'],
      ['expect', {
        journal: [
          { text: 'Saw a creature eat the pit mushroom: it grew heavy.', because: ['witness_pit'], caveats: ['secondhand'] },
          { text: 'Tasted the cave mushroom: sweet.', because: ['taste_cave'], caveats: ['tasted_in_dark'] },
          { text: 'Saw a creature eat the pit mushroom: it glowed.', because: ['witness_pit_2'], caveats: ['secondhand'] },
        ],
        belief: { state: 'uncertain', supportedBy: ['taste_cave', 'witness_pit_2'], contradictedBy: ['witness_pit'] },
        decision: { state: 'none', history: [] },
      }],
      seconds(60), // the taste fades
      ['expect', {
        journal: [
          { text: 'Saw a creature eat the pit mushroom: it grew heavy.', because: ['witness_pit'], caveats: ['secondhand'] },
          { text: 'Tasted the cave mushroom: sweet.', because: ['taste_cave'], caveats: ['tasted_in_dark', 'taste_faded'] },
          { text: 'Saw a creature eat the pit mushroom: it glowed.', because: ['witness_pit_2'], caveats: ['secondhand'] },
        ],
      }],
    ],
  },
];
