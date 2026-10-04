// Glowcap beat policy.

type Kind = 'glowcap' | 'duskcap';

interface Mushroom {
  consumed: boolean;
  tasted: Kind | null;
  // CR10: which life this mushroom is on (1 = original), and seconds since it was consumed.
  life: number;
  sinceConsumed: number;
  // CR11: the evidence that consumed the current life ('' while present).
  consumedBy: string;
  // CR14: marked by the player to avoid absorbing.
  marked: boolean;
}

interface State {
  glow: number;
  heavy: number;
  mushrooms: Record<string, Mushroom>;
  // CR13: the remembered observations (at most MEMORY), oldest first.
  memory: string[];
  // Remembered observations split by kind, each in acceptance order.
  supportedBy: string[];
  contradictedBy: string[];
  // Caveats attached to each piece of evidence still referenced, keyed by evidence id
  // ('secondhand' on witness evidence is implied by its id and not stored).
  evidenceCaveats: Record<string, string[]>;
  // CR5: seconds elapsed (in ticks) since each taste that has not yet faded.
  tasteAges: Record<string, number>;
  decision: {
    state: 'none' | 'committed' | 'reopened';
    basis: string[];
    reopenedBy: string[];
    // Caveats the basis carried at commit time (frozen).
    caveats: string[];
    // CR6: each change to trust, in order.
    history: { change: 'committed' | 'reopened'; because: string[] }[];
  };
  // CR16: journal entries, oldest first (at most MAX_JOURNAL). '+'/'-' is an
  // observation of a glowcap/duskcap, 'C' a commitment, 'R' a reopening.
  journal: { what: '+' | '-' | 'C' | 'R'; because: string[] }[];
  // CR7: glowcap observations since the most recent contradiction, in order.
  supportSinceContradiction: string[];
}

const LEVEL = ['cave', 'pool', 'ruin', 'grove', 'pit'];
// CR15: mushrooms the slime cannot reach: only other creatures can eat them.
const OUT_OF_REACH: readonly string[] = ['pit'];
const TASTE_FADE_SECONDS = 60;
const REGROW_SECONDS = 45;
// CR10: declared capacity of lives per mushroom (spec requires at least 64).
export const MAX_LIVES = 1024;

// Evidence id for an observation of a mushroom in a given life.
function evidenceId(prefix: string, id: string, life: number): string {
  return life === 1 ? `${prefix}_${id}` : `${prefix}_${id}_${life}`;
}

function consume(m: Mushroom, by: string): void {
  m.consumed = true;
  m.sinceConsumed = 0;
  m.consumedBy = by;
  m.marked = false;
}
const EPS = 1e-9;
// CR13: how many observations the slime remembers, and the decision's bounds.
const MEMORY = 6;
const MAX_REOPENED_BY = 6;
const MAX_HISTORY = 6;
const MAX_JOURNAL = 6;

function pushBounded<T>(xs: T[], x: T, max: number): void {
  xs.push(x);
  while (xs.length > max) xs.shift();
}
const KINDS: readonly string[] = ['glowcap', 'duskcap'];

function initial(): State {
  const mushrooms: Record<string, Mushroom> = {};
  for (const id of LEVEL) mushrooms[id] = { consumed: false, tasted: null, life: 1, sinceConsumed: 0, consumedBy: '', marked: false };
  return {
    glow: 0,
    heavy: 0,
    mushrooms,
    memory: [],
    supportedBy: [],
    contradictedBy: [],
    evidenceCaveats: {},
    tasteAges: {},
    decision: { state: 'none', basis: [], reopenedBy: [], caveats: [], history: [] },
    supportSinceContradiction: [],
    journal: [],
  };
}

function beliefState(s: State): string {
  const sup = s.supportedBy.length > 0;
  const con = s.contradictedBy.length > 0;
  if (sup && con) return 'uncertain';
  if (sup) return 'probably_safe';
  if (con) return 'probably_unsafe';
  return 'none';
}

function note(n: number): string {
  return n === 1 ? 'Based on one observation.' : `Based on ${n} observations.`;
}

interface MushroomView {
  present: boolean;
  label: string;
  marked: boolean;
  canAbsorb: boolean;
  canTaste: boolean;
  because: string[];
  caveats: string[];
  why: { absorb: WhyNot; taste: WhyNot };
}

interface WhyNot {
  reason: string;
  because: string[];
  caveats: string[];
}

function caveatsOf(s: State, evidence: readonly string[]): string[] {
  const out = new Set<string>();
  for (const e of evidence) {
    if (e.startsWith('witness_')) out.add('secondhand');
    for (const c of s.evidenceCaveats[e] ?? []) out.add(c);
  }
  return [...out];
}

function mushroomView(s: State, id: string, m: Mushroom): MushroomView {
  const v = mushroomViewBase(s, id, m);
  return {
    present: v.present,
    marked: m.marked,
    label: v.label,
    canAbsorb: v.canAbsorb && !m.marked && !OUT_OF_REACH.includes(id), // CR14, CR15
    canTaste: v.canTaste && !OUT_OF_REACH.includes(id), // CR15
    because: v.because,
    caveats: caveatsOf(s, v.because),
    why: whyNot(s, id, m),
  };
}

// CR11: why an action is unavailable (first matching rule), or '' when allowed.
function whyNot(s: State, id: string, m: Mushroom): { absorb: WhyNot; taste: WhyNot } {
  const mk = (reason: string, because: string[]): WhyNot => ({ reason, because, caveats: caveatsOf(s, because) });
  const allowed = mk('', []);
  if (m.consumed) {
    return { absorb: mk('Already eaten', [m.consumedBy]), taste: mk('Already eaten', [m.consumedBy]) };
  }
  if (OUT_OF_REACH.includes(id)) return { absorb: mk('Out of reach', []), taste: mk('Out of reach', []) };
  const tasteId = evidenceId('taste', id, m.life);
  const absorb =
    m.tasted === 'duskcap' ? mk('Known duskcap', [tasteId])
    : m.marked ? mk('Marked to avoid', [])
    : m.tasted === null && s.contradictedBy.length >= 2 ? mk('Too risky untasted', [...s.contradictedBy])
    : allowed;
  const taste = m.tasted !== null ? mk('Already tasted', [tasteId]) : allowed;
  return { absorb, taste };
}

function mushroomViewBase(s: State, id: string, m: Mushroom): Omit<MushroomView, 'caveats' | 'why' | 'marked'> {
  if (m.consumed) return { present: false, label: '', canAbsorb: false, canTaste: false, because: [] };
  const tasteId = evidenceId('taste', id, m.life);
  const tasteCaveats = s.evidenceCaveats[tasteId] ?? [];
  const qualifier = tasteCaveats.includes('taste_faded') ? ' (taste has faded)'
    : tasteCaveats.includes('tasted_in_dark') ? ' (tasted in the dark)'
    : '';
  if (m.tasted === 'glowcap')
    return { present: true, label: qualifier ? `Probably a glowcap${qualifier}` : 'Glowcap', canAbsorb: true, canTaste: false, because: [tasteId] };
  if (m.tasted === 'duskcap')
    return {
      present: true,
      label: qualifier ? `Probably a duskcap${qualifier}` : 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: [tasteId] };
  // CR2: twice bitten — unknown mushrooms are too risky to absorb.
  if (s.contradictedBy.length >= 2)
    return {
      present: true,
      label: 'Too risky — taste first',
      canAbsorb: false,
      canTaste: true,
      because: [...s.contradictedBy],
    };
  const bs = beliefState(s);
  const label =
    bs === 'none' ? 'Glowing mushroom'
    : bs === 'probably_safe' ? 'Probably a glowcap'
    : bs === 'probably_unsafe' ? 'Probably a duskcap'
    : 'Could be a duskcap — taste first';
  const because =
    bs === 'probably_safe' ? [...s.supportedBy]
    : bs === 'probably_unsafe' ? [...s.contradictedBy]
    : bs === 'uncertain' ? [...s.contradictedBy] // CR4: cite the counterexample only
    : [];
  return { present: true, label, canAbsorb: true, canTaste: true, because };
}

function fail(msg: string): never {
  throw new Error(msg);
}

function lookup(s: State, ev: Record<string, unknown>): [string, Mushroom, Kind] {
  const id = ev.id;
  const kind = ev.kind;
  if (typeof id !== 'string' || !Object.hasOwn(s.mushrooms, id)) fail(`unknown mushroom ${String(id)}`);
  if (typeof kind !== 'string' || !KINDS.includes(kind)) fail(`unknown kind ${String(kind)}`);
  const m = s.mushrooms[id]!;
  if (m.consumed) fail(`mushroom ${id} consumed`);
  return [id, m, kind as Kind];
}

// Record an accepted observation: remember it (forgetting the oldest beyond
// MEMORY), then update the trust decision.
function observe(s: State, evId: string, kind: Kind, caveats: string[]): void {
  if (caveats.length > 0) s.evidenceCaveats[evId] = caveats;
  pushBounded(s.journal, { what: kind === 'glowcap' ? ('+' as const) : ('-' as const), because: [evId] }, MAX_JOURNAL);
  s.memory.push(evId);
  (kind === 'glowcap' ? s.supportedBy : s.contradictedBy).push(evId);
  while (s.memory.length > MEMORY) {
    const old = s.memory.shift()!;
    s.supportedBy = s.supportedBy.filter((e) => e !== old);
    s.contradictedBy = s.contradictedBy.filter((e) => e !== old);
  }
  if (kind === 'glowcap') {
    // CR7 counts accepted observations; only the first two matter.
    if (s.supportSinceContradiction.length < 2) s.supportSinceContradiction.push(evId);
    if (s.decision.state === 'none' && beliefState(s) === 'probably_safe') {
      commit(s, [evId]);
    } else if (s.decision.state === 'reopened' && s.supportSinceContradiction.length >= 2) {
      commit(s, s.supportSinceContradiction.slice(0, 2));
    }
  } else {
    s.supportSinceContradiction = [];
    if (s.decision.state !== 'none') {
      s.decision.state = 'reopened';
      pushBounded(s.decision.reopenedBy, evId, MAX_REOPENED_BY);
      pushBounded(s.decision.history, { change: 'reopened' as const, because: [evId] }, MAX_HISTORY);
      pushBounded(s.journal, { what: 'R' as const, because: [evId] }, MAX_JOURNAL);
    }
  }
}

function commit(s: State, basis: string[]): void {
  s.decision.state = 'committed';
  s.decision.basis = basis;
  s.decision.reopenedBy = [];
  s.decision.caveats = caveatsOf(s, basis);
  pushBounded(s.decision.history, { change: 'committed' as const, because: [...basis] }, MAX_HISTORY);
  pushBounded(s.journal, { what: 'C' as const, because: [...basis] }, MAX_JOURNAL);
}

// CR13: drop caveats and fade clocks of evidence nothing can cite any more
// (not remembered, not cited by the journal, not the current taste of a present mushroom).
function prune(s: State): void {
  const live = new Set(s.memory);
  for (const j of s.journal) for (const e of j.because) live.add(e);
  for (const [id, m] of Object.entries(s.mushrooms)) {
    if (!m.consumed && m.tasted !== null) live.add(evidenceId('taste', id, m.life));
  }
  for (const e of Object.keys(s.evidenceCaveats)) if (!live.has(e)) delete s.evidenceCaveats[e];
  for (const e of Object.keys(s.tasteAges)) if (!live.has(e)) delete s.tasteAges[e];
}

// Lower a timer by dt, not below 0; snap float residue to 0.
function countDown(t: number, dt: number): number {
  const r = t - dt;
  return r <= EPS ? 0 : r;
}

function apply(s: State, ev: Record<string, unknown>): void {
  switch (ev.type) {
    case 'absorb': {
      const [id, m, kind] = lookup(s, ev);
      if (!mushroomView(s, id, m).canAbsorb) fail(`cannot absorb ${id}`);
      if (m.tasted !== null && m.tasted !== kind) fail(`kind contradicts taste of ${id}`);
      consume(m, evidenceId('absorb', id, m.life));
      if (kind === 'glowcap') s.glow = 30;
      else s.heavy = s.heavy > 0 ? Math.min(30, s.heavy + 20) : 20; // CR8: heaviness stacks
      observe(s, evidenceId('absorb', id, m.life), kind, []);
      return;
    }
    case 'witness': {
      // CR9: another creature eats it; the slime's timers are untouched.
      const [id, m, kind] = lookup(s, ev);
      if (m.tasted !== null && m.tasted !== kind) fail(`kind contradicts taste of ${id}`);
      consume(m, evidenceId('witness', id, m.life));
      observe(s, evidenceId('witness', id, m.life), kind, []);
      return;
    }
    case 'mark':
    case 'unmark': {
      // CR14: a player mark is not evidence.
      const id = ev.id;
      if (typeof id !== 'string' || !Object.hasOwn(s.mushrooms, id)) fail(`unknown mushroom ${String(id)}`);
      const m = s.mushrooms[id]!;
      if (ev.type === 'mark') {
        if (OUT_OF_REACH.includes(id)) fail(`mushroom ${id} is out of reach`);
        if (m.consumed) fail(`mushroom ${id} consumed`);
        if (m.marked) fail(`mushroom ${id} already marked`);
        m.marked = true;
      } else {
        if (!m.marked) fail(`mushroom ${id} not marked`);
        m.marked = false;
      }
      return;
    }
    case 'taste': {
      const [id, m, kind] = lookup(s, ev);
      if (!mushroomView(s, id, m).canTaste) fail(`cannot taste ${id}`);
      m.tasted = kind;
      const tasteId = evidenceId('taste', id, m.life);
      observe(s, tasteId, kind, s.glow > 0 ? [] : ['tasted_in_dark']);
      s.tasteAges[tasteId] = 0;
      return;
    }
    case 'tick': {
      const dt = ev.dt;
      if (typeof dt !== 'number' || !Number.isFinite(dt) || dt < 0 || dt > 0.1) fail(`dt out of range`);
      s.glow = countDown(s.glow, dt);
      s.heavy = countDown(s.heavy, dt);
      // CR10: consumed mushrooms with lives left regrow after 45 s.
      for (const m of Object.values(s.mushrooms)) {
        if (!m.consumed || m.life >= MAX_LIVES) continue;
        m.sinceConsumed += dt;
        if (m.sinceConsumed >= REGROW_SECONDS - EPS) {
          m.consumed = false;
          m.tasted = null;
          m.life += 1;
          m.sinceConsumed = 0;
          m.consumedBy = '';
        }
      }
      for (const evId of Object.keys(s.tasteAges)) {
        const age = s.tasteAges[evId]! + dt;
        if (age >= TASTE_FADE_SECONDS - EPS) {
          delete s.tasteAges[evId];
          s.evidenceCaveats[evId] = [...(s.evidenceCaveats[evId] ?? []), 'taste_faded'];
        } else {
          s.tasteAges[evId] = age;
        }
      }
      return;
    }
    default:
      fail(`unknown event type ${String(ev.type)}`);
  }
}

const OBSERVATION_TEXT: Record<string, [string, string]> = {
  absorb: ['Absorbed the # mushroom: it gave light.', 'Absorbed the # mushroom: it made me heavy.'],
  taste: ['Tasted the # mushroom: sweet.', 'Tasted the # mushroom: bitter.'],
  witness: ['Saw a creature eat the # mushroom: it glowed.', 'Saw a creature eat the # mushroom: it grew heavy.'],
};

function journalText(j: State['journal'][number]): string {
  if (j.what === 'C') return 'I trust glowing mushrooms now.';
  if (j.what === 'R') return 'I no longer trust glowing mushrooms.';
  const m = EVIDENCE_RE.exec(j.because[0]!);
  if (!m) fail(`bad evidence ${j.because[0]}`);
  return OBSERVATION_TEXT[m[1]!]![j.what === '+' ? 0 : 1].replace('#', m[2]!);
}

function render(s: State) {
  const bs = beliefState(s);
  const mushrooms: Record<string, MushroomView> = {};
  for (const [id, m] of Object.entries(s.mushrooms)) mushrooms[id] = mushroomView(s, id, m);
  const text =
    bs === 'probably_safe' ? 'Glowing mushrooms give you light.'
    : bs === 'probably_unsafe' ? 'Glowing mushrooms make you heavy.'
    : bs === 'uncertain' ? 'Not every glowing mushroom is safe. Taste before absorbing.'
    : '';
  const n =
    bs === 'probably_safe' ? note(s.supportedBy.length)
    : bs === 'probably_unsafe' ? note(s.contradictedBy.length)
    : '';
  return {
    slime: {
      glowing: s.glow > 0,
      heavy: s.heavy > 0,
      heavySeconds: s.heavy > 0 ? Math.max(1, Math.ceil(s.heavy - EPS)) : 0,
    },
    mushrooms,
    belief: { state: bs, text, note: n, supportedBy: [...s.supportedBy], contradictedBy: [...s.contradictedBy],
      caveats: caveatsOf(s, [...s.supportedBy, ...s.contradictedBy]),
    },
    decision: {
      state: s.decision.state,
      basis: [...s.decision.basis],
      reopenedBy: [...s.decision.reopenedBy],
      caveats: [...s.decision.caveats],
      history: s.decision.history.map((h) => ({ change: h.change, because: [...h.because] })),
    },
    journal: s.journal.map((j) => ({ text: journalText(j), because: [...j.because], caveats: caveatsOf(s, j.because) })),
  };
}

// ---------------------------------------------------------------------------
// CR12: compact save format.
// Evidence ids become short tokens: prefix letter (a/t/w) + level index, plus
// "_<life>" from life 2 (e.g. absorb_cave -> "a0", taste_grove_3 -> "t3_3").
// Caveats become one letter each, in order (d/f/s).

const PREFIXES: Record<string, string> = { absorb: 'a', taste: 't', witness: 'w' };
const PREFIX_OF: Record<string, string> = { a: 'absorb', t: 'taste', w: 'witness' };
const CAVEAT_CODES: Record<string, string> = { tasted_in_dark: 'd', taste_faded: 'f', secondhand: 's' };
const CAVEAT_OF: Record<string, string> = { d: 'tasted_in_dark', f: 'taste_faded', s: 'secondhand' };
const EVIDENCE_RE = /^(absorb|taste|witness)_([a-z]+)(?:_(\d+))?$/;
const TOKEN_RE = /^([atw])(\d+)(?:_(\d+))?$/;

function enc(evId: string): string {
  const m = EVIDENCE_RE.exec(evId);
  if (!m) fail(`cannot encode evidence ${evId}`);
  const idx = LEVEL.indexOf(m[2]!);
  if (idx < 0) fail(`cannot encode evidence ${evId}`);
  return `${PREFIXES[m[1]!]}${idx}${m[3] ? `_${m[3]}` : ''}`;
}

function dec(token: unknown): string {
  if (typeof token !== 'string') fail('bad save: evidence token');
  const m = TOKEN_RE.exec(token);
  const id = m ? LEVEL[Number(m[2])] : undefined;
  if (!m || id === undefined) fail(`bad save: evidence token ${token}`);
  return `${PREFIX_OF[m[1]!]}_${id}${m[3] ? `_${m[3]}` : ''}`;
}

const encCaveats = (cs: string[]): string => cs.map((c) => CAVEAT_CODES[c] ?? fail(`cannot encode caveat ${c}`)).join('');
const decCaveats = (code: unknown): string[] => {
  if (typeof code !== 'string') fail('bad save: caveats');
  return [...code].map((c) => CAVEAT_OF[c] ?? fail(`bad save: caveat ${c}`));
};
const encList = (xs: string[]): string => xs.map(enc).join(' ');
const decList = (x: unknown): string[] => {
  if (typeof x !== 'string') fail('bad save: list');
  return x === '' ? [] : x.split(' ').map(dec);
};
const num = (x: unknown): number => {
  if (typeof x !== 'number' || !Number.isFinite(x)) fail('bad save: number');
  return x;
};

const SAVE_VERSION = 5;
const DECISION_STATES = ['none', 'committed', 'reopened'] as const;
const TASTED = [null, 'glowcap', 'duskcap'] as const;

function encode(s: State): unknown {
  return {
    v: SAVE_VERSION,
    g: s.glow,
    h: s.heavy,
    // per level mushroom: [consumed, tasted, life, sinceConsumed, consumedBy, marked]
    m: LEVEL.map((id) => {
      const m = s.mushrooms[id]!;
      return [m.consumed ? 1 : 0, TASTED.indexOf(m.tasted), m.life, m.sinceConsumed, m.consumedBy ? enc(m.consumedBy) : '', m.marked ? 1 : 0];
    }),
    // remembered observations in order, each prefixed + (glowcap) or - (duskcap)
    o: s.memory.map((e) => `${s.supportedBy.includes(e) ? '+' : '-'}${enc(e)}`).join(' '),
    // "token:codes" separated by spaces.
    k: Object.entries(s.evidenceCaveats)
      .map(([e, cs]) => `${enc(e)}:${encCaveats(cs)}`)
      .join(' '),
    t: Object.entries(s.tasteAges).map(([e, age]) => [enc(e), age]),
    d: [
      DECISION_STATES.indexOf(s.decision.state),
      encList(s.decision.basis),
      encList(s.decision.reopenedBy),
      encCaveats(s.decision.caveats),
      s.decision.history.map((h) => `${h.change === 'committed' ? 'c' : 'r'}${encList(h.because)}`),
    ],
    r: encList(s.supportSinceContradiction),
    j: s.journal.map((e) => `${e.what}${encList(e.because)}`),
  };
}

function decode(saved: unknown): State {
  const o = saved as Record<string, unknown> | null;
  if (o === null || typeof o !== 'object' || o.v !== SAVE_VERSION) fail('bad save: version');
  const ms = o.m as unknown[];
  if (!Array.isArray(ms) || ms.length !== LEVEL.length) fail('bad save: mushrooms');
  const mushrooms: Record<string, Mushroom> = {};
  LEVEL.forEach((id, i) => {
    const row = ms[i] as unknown[];
    if (!Array.isArray(row) || row.length !== 6) fail('bad save: mushroom');
    const tasted = TASTED[num(row[1])];
    if (tasted === undefined) fail('bad save: tasted');
    mushrooms[id] = {
      consumed: row[0] === 1,
      tasted,
      life: num(row[2]),
      sinceConsumed: num(row[3]),
      consumedBy: row[4] === '' ? '' : dec(row[4]),
      marked: row[5] === 1,
    };
  });
  const pairs = (x: unknown): unknown[][] => {
    if (!Array.isArray(x) || !x.every((p) => Array.isArray(p) && p.length === 2)) fail('bad save: pairs');
    return x as unknown[][];
  };
  const evidenceCaveats: Record<string, string[]> = {};
  if (typeof o.k !== 'string') fail('bad save: caveats');
  for (const entry of o.k === '' ? [] : o.k.split(' ')) {
    const [e, cs] = entry.split(':');
    evidenceCaveats[dec(e)] = decCaveats(cs);
  }
  const tasteAges: Record<string, number> = {};
  for (const [e, age] of pairs(o.t)) tasteAges[dec(e)] = num(age);
  const d = o.d as unknown[];
  if (!Array.isArray(d) || d.length !== 5 || !Array.isArray(d[4])) fail('bad save: decision');
  const state = DECISION_STATES[num(d[0])];
  if (state === undefined) fail('bad save: decision state');
  const history = (d[4] as unknown[]).map((h) => {
    if (typeof h !== 'string' || (h[0] !== 'c' && h[0] !== 'r')) fail('bad save: history');
    return { change: h[0] === 'c' ? ('committed' as const) : ('reopened' as const), because: decList(h.slice(1)) };
  });
  if (typeof o.o !== 'string') fail('bad save: memory');
  const memory: string[] = [];
  const supportedBy: string[] = [];
  const contradictedBy: string[] = [];
  for (const item of o.o === '' ? [] : o.o.split(' ')) {
    const ev = dec(item.slice(1));
    memory.push(ev);
    if (item[0] === '+') supportedBy.push(ev);
    else if (item[0] === '-') contradictedBy.push(ev);
    else fail('bad save: memory item');
  }
  return {
    glow: num(o.g),
    heavy: num(o.h),
    mushrooms,
    memory,
    supportedBy,
    contradictedBy,
    evidenceCaveats,
    tasteAges,
    decision: {
      state,
      basis: decList(d[1]),
      reopenedBy: decList(d[2]),
      caveats: decCaveats(d[3]),
      history,
    },
    supportSinceContradiction: decList(o.r),
    journal: (Array.isArray(o.j) ? o.j : fail('bad save: journal')).map((e: unknown) => {
      if (typeof e !== 'string' || !['+', '-', 'C', 'R'].includes(e[0] ?? '')) fail('bad save: journal entry');
      return { what: e[0] as '+' | '-' | 'C' | 'R', because: decList(e.slice(1)) };
    }),
  };
}

export function createPolicy(saved?: unknown) {
  let state: State = saved === undefined ? initial() : decode(saved);
  return {
    dispatch(event: Record<string, unknown>): void {
      if (event === null || typeof event !== 'object') fail('event must be an object');
      const next = structuredClone(state);
      apply(next, event);
      prune(next);
      state = next;
    },
    view() {
      return render(state);
    },
    save(): unknown {
      return encode(state);
    },
  };
}
