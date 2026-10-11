// Glowcap beat policy.
type Kind = 'glowcap' | 'duskcap';
const KINDS: readonly string[] = ['glowcap', 'duskcap'];
const LEVEL: readonly string[] = ['cave', 'pool', 'ruin', 'grove', 'pit'];
// CR15: mushrooms only other creatures can reach (no absorb, taste or mark).
const OUT_OF_REACH: readonly string[] = ['pit'];
const GLOW_TIME = 30;
const HEAVY_TIME = 20;
const HEAVY_MAX = 30;
const TASTE_FADE = 60;
const REGROW_TIME = 45;
// Declared capacity: each mushroom has at most this many lives (CR10 requires >= 64).
export const MAX_LIVES = 1024;
// CR13: the slime remembers this many observations; reopenedBy and history keep this many entries.
const MEMORY = 6;
const DECISION_KEEP = 6;
// CR16: journal length.
const JOURNAL_KEEP = 6;
const EPS = 1e-9;

// tasted refers to the current life only; consumedAge counts ticks since consumption (for regrowth).
// marked: CR14 player mark (never evidence; cleared on consumption).
// retasted: CR19, the current life has been retasted (the retaste then identifies it).
interface MushroomState { life: number; consumed: boolean; consumedAge: number; consumedBy: string; tasted: Kind | null; retasted: boolean; marked: boolean }
interface HistoryEntry { change: 'committed' | 'reopened'; because: string[] }
interface Observation { ev: string; kind: Kind }
// Journal entry: an observation (its kind; text derived from its evidence id) or a trust change.
interface JournalEntry { entry: Kind | 'committed' | 'reopened' | 'doubt' | 'undoubt'; because: string[] }

interface State {
  glow: number;
  heavy: number;
  mushrooms: Record<string, MushroomState>;
  // remembered observations, oldest first (at most MEMORY)
  memory: Observation[];
  journal: JournalEntry[];
  // caveats carried by each still-referenced evidence id (absent = none)
  caveats: Record<string, string[]>;
  // seconds since each still-referenced taste that has not faded yet
  tasteAges: Record<string, number>;
  decision: {
    state: 'none' | 'committed' | 'reopened';
    basis: string[];
    // caveats the basis carried when committed (frozen)
    basisCaveats: string[];
    reopenedBy: string[];
    history: HistoryEntry[];
    // first glowcap observations (at most 2) accepted since the most recent contradiction (CR7)
    sinceContradiction: string[];
  };
}

export interface GlowEvent { type: string; [k: string]: unknown }

function initial(): State {
  const mushrooms: Record<string, MushroomState> = {};
  for (const id of LEVEL) mushrooms[id] = { life: 1, consumed: false, consumedAge: 0, consumedBy: '', tasted: null, retasted: false, marked: false };
  return {
    glow: 0, heavy: 0, mushrooms, memory: [], journal: [], caveats: {}, tasteAges: {},
    decision: { state: 'none', basis: [], basisCaveats: [], reopenedBy: [], history: [], sinceContradiction: [] },
  };
}

// CR18: a doubted observation carries the 'doubted' caveat and does not count in the belief.
const isDoubted = (s: State, ev: string): boolean => (s.caveats[ev] ?? []).includes('doubted');
const counted = (s: State): Observation[] => s.memory.filter((o) => !isDoubted(s, o.ev));
const sup = (s: State): string[] => counted(s).filter((o) => o.kind === 'glowcap').map((o) => o.ev);
const con = (s: State): string[] => counted(s).filter((o) => o.kind === 'duskcap').map((o) => o.ev);

function beliefState(s: State): 'none' | 'probably_safe' | 'probably_unsafe' | 'uncertain' {
  const hasSup = sup(s).length > 0, hasCon = con(s).length > 0;
  if (hasSup && hasCon) return 'uncertain';
  if (hasSup) return 'probably_safe';
  if (hasCon) return 'probably_unsafe';
  return 'none';
}

type Action = 'absorb' | 'taste' | 'witness' | 'retaste';
const ACTIONS: readonly string[] = ['absorb', 'taste', 'witness', 'retaste'];

// The taste evidence that identifies a mushroom's current life: its retaste if any, else its taste.
function idTaste(s: State, id: string): string {
  const m = s.mushrooms[id];
  return evId(m.retasted ? 'retaste' : 'taste', id, m.life);
}

// CR19: a retaste would be accepted (with the kind the current life was tasted as).
function canRetaste(s: State, id: string): boolean {
  const m = s.mushrooms[id];
  return !OUT_OF_REACH.includes(id) && !m.consumed && m.tasted !== null && !m.retasted
    && (s.caveats[evId('taste', id, m.life)] ?? []).includes('taste_faded');
}

// Evidence id for an observation of a mushroom's given life: absorb_cave, absorb_cave_2, ...
function evId(what: Action, id: string, life: number): string {
  return life === 1 ? `${what}_${id}` : `${what}_${id}_${life}`;
}

function note(n: number): string { return n === 1 ? 'Based on one observation.' : `Based on ${n} observations.`; }

function caveatsOf(s: State, evidence: readonly string[]): string[] {
  const out = new Set<string>();
  for (const ev of evidence) for (const c of s.caveats[ev] ?? []) out.add(c);
  return [...out];
}

interface Why { reason: string; because: string[]; caveats: string[] }
interface MushroomView {
  present: boolean; marked: boolean; canRetaste: boolean; regrowsIn: number; fadesIn: number; label: string; canAbsorb: boolean; canTaste: boolean; because: string[]; caveats: string[];
  why: { absorb: Why; taste: Why };
}

function mushroomView(s: State, id: string): MushroomView {
  const v = mushroomRow(s, id);
  const marked = s.mushrooms[id].marked;
  const reachable = !OUT_OF_REACH.includes(id);
  return {
    present: v.present, marked, canRetaste: canRetaste(s, id), ...countdowns(s, id), label: v.label, canAbsorb: v.canAbsorb && !marked && reachable, canTaste: v.canTaste && reachable,
    because: v.because, caveats: caveatsOf(s, v.because), why: whyNot(s, id),
  };
}

// CR17: read-only countdowns of timers the state already keeps.
function countdowns(s: State, id: string): { regrowsIn: number; fadesIn: number } {
  const m = s.mushrooms[id];
  const regrowsIn = m.consumed && m.life < MAX_LIVES ? ceilSeconds(REGROW_TIME - m.consumedAge) : 0;
  const age = !m.consumed && m.tasted !== null ? s.tasteAges[idTaste(s, id)] : undefined;
  const fadesIn = age !== undefined ? ceilSeconds(TASTE_FADE - age) : 0;
  return { regrowsIn, fadesIn };
}

function why(s: State, reason: string, because: string[]): Why {
  return { reason, because, caveats: caveatsOf(s, because) };
}

// CR11: first matching reason an action is unavailable; '' when it is allowed.
function whyNot(s: State, id: string): { absorb: Why; taste: Why } {
  const m = s.mushrooms[id];
  if (m.consumed) return { absorb: why(s, 'Already eaten', [m.consumedBy]), taste: why(s, 'Already eaten', [m.consumedBy]) };
  if (OUT_OF_REACH.includes(id)) return { absorb: why(s, 'Out of reach', []), taste: why(s, 'Out of reach', []) };
  const tasteEv = idTaste(s, id);
  const contra = con(s);
  const absorb = m.tasted === 'duskcap' ? why(s, 'Known duskcap', [tasteEv])
    : m.marked ? why(s, 'Marked to avoid', [])
    : m.tasted === null && contra.length >= 2 ? why(s, 'Too risky untasted', contra)
    : why(s, '', []);
  const taste = m.tasted !== null ? why(s, 'Already tasted', [tasteEv]) : why(s, '', []);
  return { absorb, taste };
}

function mushroomRow(s: State, id: string) {
  const m = s.mushrooms[id];
  if (m.consumed) return { present: false, label: '', canAbsorb: false, canTaste: false, because: [] as string[] };
  const tasteEv = idTaste(s, id);
  const tc = m.tasted !== null ? (s.caveats[tasteEv] ?? []) : [];
  const qual = tc.includes('taste_faded') ? ' (taste has faded)' : tc.includes('tasted_in_dark') ? ' (tasted in the dark)' : null;
  if (m.tasted === 'glowcap') return { present: true, label: qual !== null ? `Probably a glowcap${qual}` : 'Glowcap', canAbsorb: true, canTaste: false, because: [tasteEv] };
  if (m.tasted === 'duskcap') return { present: true, label: qual !== null ? `Probably a duskcap${qual}` : 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: [tasteEv] };
  const supported = sup(s), contra = con(s);
  if (contra.length >= 2) return { present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: contra };
  switch (beliefState(s)) {
    case 'none': return { present: true, label: 'Glowing mushroom', canAbsorb: true, canTaste: true, because: [] as string[] };
    case 'probably_safe': return { present: true, label: 'Probably a glowcap', canAbsorb: true, canTaste: true, because: supported };
    case 'probably_unsafe': return { present: true, label: 'Probably a duskcap', canAbsorb: true, canTaste: true, because: contra };
    case 'uncertain': return { present: true, label: 'Could be a duskcap — taste first', canAbsorb: true, canTaste: true, because: contra };
  }
}

// Whole seconds rounded up; tolerance absorbs float drift from summed dt.
function ceilSeconds(t: number): number {
  return t > 0 ? Math.max(1, Math.ceil(t - 1e-9)) : 0;
}

function render(s: State) {
  const bs = beliefState(s);
  const supported = sup(s), contra = con(s);
  const belief = {
    state: bs,
    text: bs === 'none' ? '' : bs === 'probably_safe' ? 'Glowing mushrooms give you light.'
      : bs === 'probably_unsafe' ? 'Glowing mushrooms make you heavy.' : 'Not every glowing mushroom is safe. Taste before absorbing.',
    note: bs === 'probably_safe' ? note(supported.length) : bs === 'probably_unsafe' ? note(contra.length) : '',
    supportedBy: supported,
    contradictedBy: contra,
    caveats: caveatsOf(s, [...supported, ...contra]),
  };
  const mushrooms: Record<string, MushroomView> = {};
  for (const id of LEVEL) mushrooms[id] = mushroomView(s, id);
  const d = s.decision;
  return {
    slime: { glowing: s.glow > 0, heavy: s.heavy > 0, heavySeconds: ceilSeconds(s.heavy), glowSeconds: ceilSeconds(s.glow) },
    mushrooms,
    belief,
    journal: s.journal.map((j) => ({ text: journalText(j), because: [...j.because], caveats: caveatsOf(s, j.because) })),
    decision: {
      state: d.state,
      basis: [...d.basis],
      reopenedBy: [...d.reopenedBy],
      caveats: [...d.basisCaveats],
      history: d.history.map((h) => ({ change: h.change, because: [...h.because] })),
    },
  };
}

function fail(msg: string): never { throw new Error(msg); }

function target(s: State, e: GlowEvent): { id: string; kind: Kind; m: MushroomState } {
  const id = e.id, kind = e.kind;
  if (typeof id !== 'string' || !Object.hasOwn(s.mushrooms, id)) fail(`unknown mushroom ${String(id)}`);
  if (typeof kind !== 'string' || !KINDS.includes(kind)) fail(`unknown kind ${String(kind)}`);
  const m = s.mushrooms[id];
  if (m.consumed) fail(`${id} already consumed`);
  return { id, kind: kind as Kind, m };
}

function keepLast<T>(list: T[], n: number): void {
  if (list.length > n) list.splice(0, list.length - n);
}

// Record one accepted observation, forget beyond MEMORY, and update the trust decision.
function observe(s: State, ev: string, kind: Kind, caveats: string[]): void {
  if (caveats.length > 0) s.caveats[ev] = [...caveats];
  pushJournal(s, { entry: kind, because: [ev] });
  s.memory.push({ ev, kind });
  keepLast(s.memory, MEMORY);
  const d = s.decision;
  if (kind === 'duskcap') d.sinceContradiction = [];
  else if (d.sinceContradiction.length < 2) d.sinceContradiction.push(ev);
  if (d.state === 'none') {
    if (beliefState(s) === 'probably_safe') commit(s, [ev]);
  } else if (d.state === 'reopened' && d.sinceContradiction.length >= 2) {
    commit(s, d.sinceContradiction.slice(0, 2));
  } else if (kind === 'duskcap') {
    d.state = 'reopened';
    d.reopenedBy.push(ev);
    keepLast(d.reopenedBy, DECISION_KEEP);
    pushHistory(s, { change: 'reopened', because: [ev] });
  }
}

// Every history entry is also journaled, right after the observation that caused it.
function pushHistory(s: State, entry: HistoryEntry): void {
  s.decision.history.push(entry);
  keepLast(s.decision.history, DECISION_KEEP);
  pushJournal(s, { entry: entry.change, because: [...entry.because] });
}

function pushJournal(s: State, entry: JournalEntry): void {
  s.journal.push(entry);
  keepLast(s.journal, JOURNAL_KEEP);
}

const OBS_TEXT: Record<Action, Record<Kind, string>> = {
  absorb: { glowcap: 'Absorbed the %s mushroom: it gave light.', duskcap: 'Absorbed the %s mushroom: it made me heavy.' },
  taste: { glowcap: 'Tasted the %s mushroom: sweet.', duskcap: 'Tasted the %s mushroom: bitter.' },
  witness: { glowcap: 'Saw a creature eat the %s mushroom: it glowed.', duskcap: 'Saw a creature eat the %s mushroom: it grew heavy.' },
  retaste: { glowcap: 'Tasted the %s mushroom again: sweet.', duskcap: 'Tasted the %s mushroom again: bitter.' },
};

function journalText(e: JournalEntry): string {
  if (e.entry === 'committed') return 'I trust glowing mushrooms now.';
  if (e.entry === 'reopened') return 'I no longer trust glowing mushrooms.';
  const { what, mi } = parseEv(e.because[0]);
  if (e.entry === 'doubt') return `I doubt what I saw at the ${LEVEL[mi]} mushroom.`;
  if (e.entry === 'undoubt') return `I believe what I saw at the ${LEVEL[mi]} mushroom again.`;
  return OBS_TEXT[what][e.entry].replace('%s', LEVEL[mi]);
}

function commit(s: State, basis: string[]): void {
  const d = s.decision;
  d.state = 'committed';
  d.basis = basis;
  d.basisCaveats = caveatsOf(s, basis);
  d.reopenedBy = [];
  pushHistory(s, { change: 'committed', because: [...basis] });
}

function consume(m: MushroomState, by: string): void {
  m.consumed = true;
  m.marked = false;
  m.consumedBy = by;
  m.consumedAge = 0;
}

// Evidence still shown somewhere with its caveats: remembered observations,
// the current taste of each present mushroom, and what consumed each consumed one.
function referenced(s: State): Set<string> {
  const refs = new Set(s.memory.map((o) => o.ev));
  for (const j of s.journal) for (const ev of j.because) refs.add(ev);
  LEVEL.forEach((id) => {
    const m = s.mushrooms[id];
    if (m.consumed) refs.add(m.consumedBy);
    else if (m.tasted !== null) refs.add(idTaste(s, id));
  });
  return refs;
}

// Drop caveats and fade clocks nothing can show any more, so state stays bounded.
function prune(s: State): void {
  const refs = referenced(s);
  for (const ev of Object.keys(s.caveats)) if (!refs.has(ev)) delete s.caveats[ev];
  for (const ev of Object.keys(s.tasteAges)) if (!refs.has(ev)) delete s.tasteAges[ev];
}

function apply(s: State, e: GlowEvent): void {
  if (e === null || typeof e !== 'object') fail('event must be an object');
  switch (e.type) {
    case 'absorb': {
      const { id, kind, m } = target(s, e);
      if (!mushroomView(s, id).canAbsorb) fail(`${id} cannot be absorbed`);
      if (m.tasted !== null && m.tasted !== kind) fail(`${id} kind contradicts its taste`);
      const ev = evId('absorb', id, m.life);
      consume(m, ev);
      if (kind === 'glowcap') s.glow = GLOW_TIME;
      else s.heavy = s.heavy > 0 ? Math.min(HEAVY_MAX, s.heavy + HEAVY_TIME) : HEAVY_TIME;
      observe(s, ev, kind, []);
      break;
    }
    case 'witness': {
      const { id, kind, m } = target(s, e);
      if (m.tasted !== null && m.tasted !== kind) fail(`${id} kind contradicts its taste`);
      const ev = evId('witness', id, m.life);
      consume(m, ev);
      observe(s, ev, kind, ['secondhand']);
      break;
    }
    case 'taste': {
      const { id, kind, m } = target(s, e);
      if (OUT_OF_REACH.includes(id)) fail(`${id} is out of reach`);
      if (m.tasted !== null) fail(`${id} already tasted`);
      m.tasted = kind;
      const ev = evId('taste', id, m.life);
      s.tasteAges[ev] = 0;
      observe(s, ev, kind, s.glow > 0 ? [] : ['tasted_in_dark']);
      break;
    }
    case 'retaste': {
      const { id, kind, m } = target(s, e);
      if (!canRetaste(s, id)) fail(`${id} cannot be retasted`);
      if (m.tasted !== kind) fail(`${id} kind contradicts its taste`);
      m.retasted = true;
      const ev = evId('retaste', id, m.life);
      s.tasteAges[ev] = 0;
      observe(s, ev, kind, s.glow > 0 ? [] : ['tasted_in_dark']);
      break;
    }
    case 'mark':
    case 'unmark': {
      const id = e.id;
      if (typeof id !== 'string' || !Object.hasOwn(s.mushrooms, id)) fail(`unknown mushroom ${String(id)}`);
      const m = s.mushrooms[id];
      if (e.type === 'mark') {
        if (m.consumed) fail(`${id} already consumed`);
        if (m.marked) fail(`${id} already marked`);
        if (OUT_OF_REACH.includes(id)) fail(`${id} is out of reach`);
        m.marked = true;
      } else {
        if (!m.marked) fail(`${id} is not marked`);
        m.marked = false;
      }
      break;
    }
    case 'doubt': {
      const ev = e.evidence;
      const obs = typeof ev === 'string' ? s.memory.find((o) => o.ev === ev) : undefined;
      if (obs === undefined || !obs.ev.startsWith('witness_')) fail(`${String(ev)} is not a remembered witness`);
      if (isDoubted(s, obs.ev)) fail(`${obs.ev} is already doubted`);
      if (!counted(s).some((o) => o.ev !== obs.ev)) fail('at least one undoubted observation must remain');
      s.caveats[obs.ev] = [...(s.caveats[obs.ev] ?? []), 'doubted'];
      pushJournal(s, { entry: 'doubt', because: [obs.ev] });
      break;
    }
    case 'undoubt': {
      const ev = e.evidence;
      const obs = typeof ev === 'string' ? s.memory.find((o) => o.ev === ev) : undefined;
      if (obs === undefined || !obs.ev.startsWith('witness_')) fail(`${String(ev)} is not a remembered witness`);
      if (!isDoubted(s, obs.ev)) fail(`${obs.ev} is not doubted`);
      s.caveats[obs.ev] = (s.caveats[obs.ev] ?? []).filter((c) => c !== 'doubted');
      pushJournal(s, { entry: 'undoubt', because: [obs.ev] });
      break;
    }
    case 'tick': {
      const dt = e.dt;
      if (typeof dt !== 'number' || !Number.isFinite(dt) || dt < 0 || dt > 0.1) fail(`dt out of range`);
      s.glow = Math.max(0, s.glow - dt);
      s.heavy = Math.max(0, s.heavy - dt);
      for (const ev of Object.keys(s.tasteAges)) {
        const age = s.tasteAges[ev] + dt;
        if (age >= TASTE_FADE - EPS) {
          s.caveats[ev] = [...(s.caveats[ev] ?? []), 'taste_faded'];
          delete s.tasteAges[ev];
        } else s.tasteAges[ev] = age;
      }
      for (const id of LEVEL) {
        const m = s.mushrooms[id];
        if (!m.consumed || m.life >= MAX_LIVES) continue;
        m.consumedAge += dt;
        if (m.consumedAge >= REGROW_TIME - EPS) {
          m.life += 1;
          m.consumed = false;
          m.consumedAge = 0;
          m.consumedBy = '';
          m.tasted = null;
          m.retasted = false;
        }
      }
      break;
    }
    default: fail(`unknown event type ${String(e.type)}`);
  }
  prune(s);
}

// ---- Save format (CR12) ----
// Compact and bounded. An evidence reference is a token: action letter (a/t/w/e=retaste),
// mushroom index, life in base 36. A full token also carries the kind (G/D) and
// '*' for a taste in the dark. Caveats are rederived: witness = secondhand,
// '*' = tasted_in_dark, a taste without a saved fade clock = taste_faded.
// '!' marks a doubted witness (CR18).
const SAVE_VERSION = 7;
const TOKEN = /^([atwe])(\d)([0-9a-z]+)([GD])?(\*)?(!)?$/;

interface Saved {
  v: number;
  g: number; // glow timer
  h: number; // heavy timer
  m: [number, number, string, string, number][]; // per LEVEL: [life, consumedAge, current taste token, consumedBy token, marked 0/1]
  o: string; // remembered observations, full tokens, oldest first
  j: string; // journal: ';'-separated journalToken()s
  f: [string, number][]; // unfaded referenced tastes: [ref token, age]
  d: { st: string; b: string; bc: string[]; r: string; h: string; sc: string };
}

function parseEv(ev: string): { what: Action; mi: number; life: number } {
  const parts = ev.split('_');
  const what = parts[0] as Action;
  const mi = LEVEL.indexOf(parts[1]);
  const life = parts.length > 2 ? Number(parts[2]) : 1;
  if (!ACTIONS.includes(what) || mi < 0 || !Number.isInteger(life)) fail(`bad evidence id ${ev}`);
  return { what, mi, life };
}

const ACTION_LETTER: Record<Action, string> = { absorb: 'a', taste: 't', witness: 'w', retaste: 'e' };

function refToken(ev: string): string {
  const { what, mi, life } = parseEv(ev);
  return `${ACTION_LETTER[what]}${mi}${life.toString(36)}`;
}

function fullToken(s: State, ev: string, kind: Kind | null): string {
  const cav = s.caveats[ev] ?? [];
  const k = kind === null ? '' : kind === 'glowcap' ? 'G' : 'D';
  return `${refToken(ev)}${k}${cav.includes('tasted_in_dark') ? '*' : ''}${cav.includes('doubted') ? '!' : ''}`;
}

function readToken(t: string): { ev: string; what: Action; mi: number; life: number; kind: Kind | null; dark: boolean; doubted: boolean } {
  const r = TOKEN.exec(t);
  if (r === null) fail(`bad token ${t}`);
  const what: Action = r[1] === 'a' ? 'absorb' : r[1] === 't' ? 'taste' : r[1] === 'e' ? 'retaste' : 'witness';
  const mi = Number(r[2]);
  const life = parseInt(r[3], 36);
  if (mi >= LEVEL.length || !(life >= 1)) fail(`bad token ${t}`);
  const kind: Kind | null = r[4] === 'G' ? 'glowcap' : r[4] === 'D' ? 'duskcap' : null;
  return { ev: evId(what, LEVEL[mi], life), what, mi, life, kind, dark: r[5] === '*', doubted: r[6] === '!' };
}

const refs = (evs: readonly string[]) => evs.map(refToken).join(' ');
const unrefs = (str: string) => (str === '' ? [] : str.split(' ').map((t) => readToken(t).ev));

// Journal entry token: an observation is its full token; a trust change, doubt or undoubt
// is c/r/d/u followed by ','-joined full tokens of its evidence.
function journalToken(s: State, e: JournalEntry): string {
  if (e.entry === 'glowcap' || e.entry === 'duskcap') return fullToken(s, e.because[0], e.entry);
  const kind: Kind | null = e.entry === 'committed' ? 'glowcap' : e.entry === 'reopened' ? 'duskcap' : null;
  return `${e.entry[0]}${e.because.map((ev) => fullToken(s, ev, kind)).join(',')}`;
}

const JOURNAL_PREFIX: Record<string, JournalEntry['entry']> = { c: 'committed', r: 'reopened', d: 'doubt', u: 'undoubt' };

function encode(s: State): Saved {
  const d = s.decision;
  return {
    v: SAVE_VERSION,
    g: s.glow,
    h: s.heavy,
    m: LEVEL.map((id) => {
      const m = s.mushrooms[id];
      const taste = !m.consumed && m.tasted !== null ? fullToken(s, idTaste(s, id), m.tasted) : '';
      return [m.life, m.consumedAge, taste, m.consumed ? fullToken(s, m.consumedBy, null) : '', m.marked ? 1 : 0];
    }),
    o: s.memory.map((o) => fullToken(s, o.ev, o.kind)).join(' '),
    j: s.journal.map((e) => journalToken(s, e)).join(';'),
    f: Object.entries(s.tasteAges).map(([ev, age]) => [refToken(ev), age]),
    d: {
      st: d.state,
      b: refs(d.basis),
      bc: [...d.basisCaveats],
      r: refs(d.reopenedBy),
      h: d.history.map((e) => `${e.change === 'committed' ? 'c' : 'r'}${refs(e.because).replace(/ /g, ',')}`).join(';'),
      sc: refs(d.sinceContradiction),
    },
  };
}

function decode(raw: unknown): State {
  const sv = raw as Saved;
  if (sv === null || typeof sv !== 'object' || sv.v !== SAVE_VERSION) fail('unrecognised save');
  const s = initial();
  s.glow = sv.g;
  s.heavy = sv.h;
  const unfaded = new Map(sv.f.map(([t, age]) => [readToken(t).ev, age] as const));
  // Restore the caveats an evidence token implies.
  const restore = (t: ReturnType<typeof readToken>) => {
    const cav: string[] = [];
    if (t.what === 'witness') cav.push('secondhand');
    if (t.dark) cav.push('tasted_in_dark');
    if (t.what === 'taste' || t.what === 'retaste') {
      const age = unfaded.get(t.ev);
      if (age !== undefined) s.tasteAges[t.ev] = age;
      else cav.push('taste_faded');
    }
    if (t.doubted) cav.push('doubted');
    if (cav.length > 0) s.caveats[t.ev] = cav;
  };
  LEVEL.forEach((id, i) => {
    const [life, consumedAge, taste, consumedBy, marked] = sv.m[i];
    const m = s.mushrooms[id];
    m.marked = marked === 1;
    m.life = life;
    m.consumedAge = consumedAge;
    if (taste !== '') {
      const t = readToken(taste);
      m.tasted = t.kind;
      m.retasted = t.what === 'retaste';
      restore(t);
    }
    if (consumedBy !== '') {
      const t = readToken(consumedBy);
      m.consumed = true;
      m.consumedBy = t.ev;
      restore(t);
    }
  });
  if (sv.o !== '') {
    for (const tok of sv.o.split(' ')) {
      const t = readToken(tok);
      if (t.kind === null) fail(`bad token ${tok}`);
      s.memory.push({ ev: t.ev, kind: t.kind });
      restore(t);
    }
  }
  if (sv.j !== '') {
    for (const part of sv.j.split(';')) {
      const prefixed = JOURNAL_PREFIX[part[0]];
      if (prefixed !== undefined) {
        const toks = part.slice(1).split(',').map(readToken);
        toks.forEach(restore);
        s.journal.push({ entry: prefixed, because: toks.map((t) => t.ev) });
      } else {
        const t = readToken(part);
        if (t.kind === null) fail(`bad token ${part}`);
        restore(t);
        s.journal.push({ entry: t.kind, because: [t.ev] });
      }
    }
  }
  const d = s.decision;
  d.state = sv.d.st as State['decision']['state'];
  d.basis = unrefs(sv.d.b);
  d.basisCaveats = [...sv.d.bc];
  d.reopenedBy = unrefs(sv.d.r);
  d.history = sv.d.h === '' ? [] : sv.d.h.split(';').map((e) => ({
    change: e[0] === 'c' ? 'committed' as const : 'reopened' as const,
    because: unrefs(e.slice(1).replace(/,/g, ' ')),
  }));
  d.sinceContradiction = unrefs(sv.d.sc);
  return s;
}

export function createPolicy(saved?: unknown) {
  let state: State = saved === undefined ? initial() : decode(saved);
  return {
    dispatch(event: GlowEvent): void {
      const next = structuredClone(state);
      apply(next, event);
      state = next;
    },
    view() { return render(state); },
    save(): unknown { return encode(state); },
  };
}
