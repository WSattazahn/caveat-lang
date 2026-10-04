// Glowcap beat policy.
// State is plain JSON data; every dispatch works on a deep copy and only
// replaces the live state if the event is accepted (atomic rollback).
// The view is derived from state. The decision is stored, because its basis
// and caveats are frozen at commit time.
//
// CR13: the belief sees only a window of the six most recent observations.
// Facts that outlive the window (a mushroom's own taste, what consumed it, the
// decision's basis) are kept with their owners.

export type Kind = 'glowcap' | 'duskcap';
type Verb = 'absorb' | 'taste' | 'witness';

const KINDS: readonly string[] = ['glowcap', 'duskcap'];
const LEVEL: readonly string[] = ['cave', 'pool', 'ruin', 'grove', 'pit']; // order matters for saves: append only
const OUT_OF_REACH: readonly string[] = ['pit']; // CR15: only other creatures can eat these
const GLOW_SECONDS = 30;
const HEAVY_SECONDS = 20;
const HEAVY_MAX = 30; // CR8
const MAX_DT = 0.1;
const TOO_RISKY_AFTER = 2; // CR2
const DARK = 'tasted_in_dark'; // CR3
const FADED = 'taste_faded'; // CR5
const FADE_SECONDS = 60; // CR5
const RECOVER_AFTER = 2; // CR7
const SECONDHAND = 'secondhand'; // CR9
const REGROW_SECONDS = 45; // CR10
/** CR10: declared capacity of lives per mushroom (the request requires at least 64). */
export const MAX_LIVES = 1024;
const MEMORY = 6; // CR13: remembered observations
const MAX_REOPENED_BY = 6; // CR13
const MAX_HISTORY = 6; // CR13
const MAX_JOURNAL = 6; // CR16

// One observation (absorption, taste (CR3) or witness (CR9)).
interface Evidence {
  id: string; // absorb_<id>[_<life>] / taste_… / witness_…
  kind: Kind;
  caveats: string[];
  age?: number; // CR5: seconds of ticks since a taste, while it has not faded
}

interface MushroomState {
  life: number; // CR10: 1-based life number
  consumed: boolean;
  sinceConsumed: number; // CR10: seconds of ticks since consumed (0 while present)
  taste: Evidence | null; // this life's taste; outlives the belief's memory (CR13)
  consumedBy: Evidence | null; // CR11: what consumed this life
  marked: boolean; // CR14: player's mark; not evidence
}

type DecisionState = 'none' | 'committed' | 'reopened';

export interface HistoryEntry {
  change: 'committed' | 'reopened';
  because: string[];
}

interface DecisionData {
  state: DecisionState;
  basis: string[];
  reopenedBy: string[]; // at most MAX_REOPENED_BY
  caveats: string[]; // frozen at commit (CR5)
  history: HistoryEntry[]; // at most MAX_HISTORY
  recovery: string[]; // CR7: glowcap observations since the most recent contradiction (capped at RECOVER_AFTER)
}

// CR16: a journal entry keeps its own copies of the evidence it cites, so
// their caveats stay current (fades) even after the belief forgets them.
interface JournalData {
  type: 'observed' | 'committed' | 'reopened';
  evidence: Evidence[];
}

interface State {
  journal: JournalData[]; // CR16, oldest first, at most MAX_JOURNAL
  glow: number;
  heavy: number;
  mushrooms: Record<string, MushroomState>;
  memory: Evidence[]; // CR13: remembered observations, oldest first
  decision: DecisionData;
}

export type BeliefState = 'none' | 'probably_safe' | 'probably_unsafe' | 'uncertain';

export interface WhyNot {
  reason: string;
  because: string[];
  caveats: string[];
}

export interface MushroomView {
  present: boolean;
  marked: boolean; // CR14
  label: string;
  canAbsorb: boolean;
  canTaste: boolean;
  because: string[];
  caveats: string[];
  why: { absorb: WhyNot; taste: WhyNot }; // CR11
}

export interface BeliefView {
  state: BeliefState;
  text: string;
  note: string;
  supportedBy: string[];
  contradictedBy: string[];
  caveats: string[];
}

export interface DecisionView {
  state: DecisionState;
  basis: string[];
  reopenedBy: string[];
  caveats: string[];
  history: HistoryEntry[]; // CR6, ordered
}

export interface JournalEntry {
  text: string;
  because: string[];
  caveats: string[];
}

export interface View {
  journal: JournalEntry[]; // CR16, ordered
  slime: { glowing: boolean; heavy: boolean; heavySeconds: number };
  mushrooms: Record<string, MushroomView>;
  belief: BeliefView;
  decision: DecisionView;
}

export interface Policy {
  dispatch(event: unknown): void;
  view(): View;
  save(): unknown;
}

function initialState(): State {
  const mushrooms: Record<string, MushroomState> = {};
  for (const id of LEVEL) mushrooms[id] = { life: 1, consumed: false, sinceConsumed: 0, taste: null, consumedBy: null, marked: false };
  return {
    glow: 0,
    heavy: 0,
    mushrooms,
    memory: [],
    journal: [],
    decision: { state: 'none', basis: [], reopenedBy: [], caveats: [], history: [], recovery: [] },
  };
}

function clone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v)) as T;
}

function fail(msg: string): never {
  throw new Error(msg);
}

/** CR10: evidence id for a mushroom's life. */
function evidenceId(verb: Verb, id: string, life: number): string {
  return life === 1 ? `${verb}_${id}` : `${verb}_${id}_${life}`;
}

/** Union of the caveats carried by the cited evidence. */
function caveatsOf(cited: readonly Evidence[]): string[] {
  const out = new Set<string>();
  for (const e of cited) for (const c of e.caveats) out.add(c);
  return [...out];
}

function pushCapped<T>(list: T[], item: T, cap: number): void {
  list.push(item);
  while (list.length > cap) list.shift();
}

// --- belief ------------------------------------------------------------------

interface Belief {
  view: BeliefView;
  support: Evidence[];
  contra: Evidence[];
}

function belief(s: State): Belief {
  const support = s.memory.filter((e) => e.kind === 'glowcap');
  const contra = s.memory.filter((e) => e.kind === 'duskcap');
  const supportedBy = support.map((e) => e.id);
  const contradictedBy = contra.map((e) => e.id);
  const caveats = caveatsOf(s.memory);
  const note = (n: number) => (n === 1 ? 'Based on one observation.' : `Based on ${n} observations.`);
  let view: BeliefView;
  if (support.length && contra.length) {
    view = { state: 'uncertain', text: 'Not every glowing mushroom is safe. Taste before absorbing.', note: '', supportedBy, contradictedBy, caveats };
  } else if (support.length) {
    view = { state: 'probably_safe', text: 'Glowing mushrooms give you light.', note: note(support.length), supportedBy, contradictedBy, caveats };
  } else if (contra.length) {
    view = { state: 'probably_unsafe', text: 'Glowing mushrooms make you heavy.', note: note(contra.length), supportedBy, contradictedBy, caveats };
  } else {
    view = { state: 'none', text: '', note: '', supportedBy, contradictedBy, caveats };
  }
  return { view, support, contra };
}

// --- mushroom view -----------------------------------------------------------

function twiceBitten(b: Belief): boolean {
  return b.contra.length >= TOO_RISKY_AFTER; // CR2, remembered contradictions (CR13)
}

/** CR11: why each action is unavailable ('' / [] when allowed). */
function whyNot(id: string, m: MushroomState, b: Belief): { absorb: WhyNot; taste: WhyNot } {
  const w = (reason: string, because: Evidence[]): WhyNot => ({ reason, because: because.map((e) => e.id), caveats: caveatsOf(because) });
  if (m.consumed) {
    const by = m.consumedBy ? [m.consumedBy] : [];
    return { absorb: w('Already eaten', by), taste: w('Already eaten', by) };
  }
  if (OUT_OF_REACH.includes(id)) return { absorb: w('Out of reach', []), taste: w('Out of reach', []) }; // CR15
  let absorb = w('', []);
  if (m.taste?.kind === 'duskcap') absorb = w('Known duskcap', [m.taste]);
  else if (m.marked) absorb = w('Marked to avoid', []); // CR14
  else if (!m.taste && twiceBitten(b)) absorb = w('Too risky untasted', b.contra);
  const taste = m.taste ? w('Already tasted', [m.taste]) : w('', []);
  return { absorb, taste };
}

function mushroomView(s: State, id: string, b: Belief): MushroomView {
  const m = s.mushrooms[id]!;
  const why = whyNot(id, m, b);
  const reachable = !OUT_OF_REACH.includes(id); // CR15
  const mk = (present: boolean, label: string, canAbsorb: boolean, canTaste: boolean, because: Evidence[]): MushroomView => ({
    present,
    marked: m.marked,
    label,
    canAbsorb: canAbsorb && !m.marked && reachable, // CR14: a marked mushroom is never absorbed
    canTaste: canTaste && reachable,
    because: because.map((e) => e.id),
    caveats: caveatsOf(because),
    why,
  });
  if (m.consumed) return mk(false, '', false, false, []);
  if (m.taste) {
    const cav = m.taste.caveats;
    const qualifier = cav.includes(FADED) ? ' (taste has faded)' : cav.includes(DARK) ? ' (tasted in the dark)' : null;
    if (m.taste.kind === 'glowcap') return mk(true, qualifier ? `Probably a glowcap${qualifier}` : 'Glowcap', true, false, [m.taste]);
    return mk(true, qualifier ? `Probably a duskcap${qualifier}` : 'Duskcap — avoid', false, false, [m.taste]);
  }
  if (twiceBitten(b)) return mk(true, 'Too risky — taste first', false, true, b.contra);
  switch (b.view.state) {
    case 'none':
      return mk(true, 'Glowing mushroom', true, true, []);
    case 'probably_safe':
      return mk(true, 'Probably a glowcap', true, true, b.support);
    case 'probably_unsafe':
      return mk(true, 'Probably a duskcap', true, true, b.contra);
    case 'uncertain':
      return mk(true, 'Could be a duskcap — taste first', true, true, b.contra); // CR4: cite the counterexample
  }
}

const JOURNAL_TEXT: Record<Verb, Record<Kind, string>> = {
  absorb: { glowcap: 'Absorbed the <id> mushroom: it gave light.', duskcap: 'Absorbed the <id> mushroom: it made me heavy.' },
  taste: { glowcap: 'Tasted the <id> mushroom: sweet.', duskcap: 'Tasted the <id> mushroom: bitter.' },
  witness: { glowcap: 'Saw a creature eat the <id> mushroom: it glowed.', duskcap: 'Saw a creature eat the <id> mushroom: it grew heavy.' },
};

function journalText(j: JournalData): string {
  if (j.type === 'committed') return 'I trust glowing mushrooms now.';
  if (j.type === 'reopened') return 'I no longer trust glowing mushrooms.';
  const ev = j.evidence[0] ?? fail('empty journal entry');
  const [, verb, mid] = ID_RE.exec(ev.id) ?? fail(`bad evidence id ${ev.id}`);
  return JOURNAL_TEXT[verb as Verb][ev.kind].replace('<id>', mid!);
}

function render(s: State): View {
  const b = belief(s);
  const mushrooms: Record<string, MushroomView> = {};
  for (const id of LEVEL) mushrooms[id] = mushroomView(s, id, b);
  const d = s.decision;
  return {
    journal: s.journal.map((j) => ({ text: journalText(j), because: j.evidence.map((e) => e.id), caveats: caveatsOf(j.evidence) })),
    slime: { glowing: s.glow > 0, heavy: s.heavy > 0, heavySeconds: s.heavy > 0 ? Math.ceil(s.heavy) : 0 },
    mushrooms,
    belief: b.view,
    decision: { state: d.state, basis: [...d.basis], reopenedBy: [...d.reopenedBy], caveats: [...d.caveats], history: clone(d.history) },
  };
}

// --- events ------------------------------------------------------------------

function asRecord(event: unknown): Record<string, unknown> {
  if (typeof event !== 'object' || event === null) fail('event must be an object');
  return event as Record<string, unknown>;
}

function target(s: State, e: Record<string, unknown>): { id: string; kind: Kind; m: MushroomState } {
  const id = e['id'];
  const kind = e['kind'];
  if (typeof id !== 'string' || !LEVEL.includes(id)) fail(`unknown mushroom: ${String(id)}`);
  if (typeof kind !== 'string' || !KINDS.includes(kind)) fail(`unknown kind: ${String(kind)}`);
  const m = s.mushrooms[id]!;
  if (m.consumed) fail(`mushroom consumed: ${id}`);
  return { id, kind: kind as Kind, m };
}

/** Record an accepted observation, forget beyond the window, and update trust. */
function observe(s: State, ev: Evidence): void {
  pushCapped(s.memory, ev, MEMORY);
  pushCapped(s.journal, { type: 'observed', evidence: [clone(ev)] }, MAX_JOURNAL); // CR16
  const d = s.decision;
  const commit = (basis: Evidence[]): void => {
    d.state = 'committed';
    d.basis = basis.map((e) => e.id);
    d.reopenedBy = [];
    d.caveats = caveatsOf(basis);
    pushCapped(d.history, { change: 'committed', because: [...d.basis] }, MAX_HISTORY);
    pushCapped(s.journal, { type: 'committed', evidence: clone(basis) }, MAX_JOURNAL);
  };
  // CR7 count: accepted observations, remembered or not.
  if (ev.kind === 'duskcap') d.recovery = [];
  else if (d.recovery.length < RECOVER_AFTER) d.recovery.push(ev.id);

  if (d.state === 'none') {
    if (belief(s).view.state === 'probably_safe') commit([ev]); // basis: the observation that made it safe
  } else if (ev.kind === 'duskcap') {
    d.state = 'reopened';
    pushCapped(d.reopenedBy, ev.id, MAX_REOPENED_BY);
    pushCapped(d.history, { change: 'reopened', because: [ev.id] }, MAX_HISTORY);
    pushCapped(s.journal, { type: 'reopened', evidence: [clone(ev)] }, MAX_JOURNAL);
  } else if (d.state === 'reopened' && d.recovery.length === RECOVER_AFTER && d.recovery[RECOVER_AFTER - 1] === ev.id) {
    // CR7: the second glowcap observation since the most recent contradiction recommits.
    const basis = d.recovery.map((id) => findObservation(s, id) ?? fail(`lost evidence ${id}`));
    commit(basis);
  }
}

/** The caveats of a recovery basis: the observations are recent, but look beyond memory to be safe. */
function findObservation(s: State, id: string): Evidence | undefined {
  for (const e of s.memory) if (e.id === id) return e;
  for (const m of Object.values(s.mushrooms)) {
    if (m.taste?.id === id) return m.taste;
    if (m.consumedBy?.id === id) return m.consumedBy;
  }
  return undefined;
}

function apply(s: State, event: unknown): void {
  const e = asRecord(event);
  switch (e['type']) {
    case 'absorb':
    case 'witness': {
      const verb: Verb = e['type'];
      const { id, kind, m } = target(s, e);
      if (verb === 'absorb' && !mushroomView(s, id, belief(s)).canAbsorb) fail(`cannot absorb ${id}`);
      if (m.taste && m.taste.kind !== kind) fail(`kind contradicts taste of ${id}`);
      const ev: Evidence = { id: evidenceId(verb, id, m.life), kind, caveats: verb === 'witness' ? [SECONDHAND] : [] };
      m.consumed = true;
      m.sinceConsumed = 0;
      m.consumedBy = clone(ev);
      m.marked = false; // CR14: eating clears the mark
      if (verb === 'absorb') {
        if (kind === 'glowcap') s.glow = GLOW_SECONDS;
        else s.heavy = s.heavy > 0 ? Math.min(HEAVY_MAX, s.heavy + HEAVY_SECONDS) : HEAVY_SECONDS; // CR8: stacks
      }
      // CR9: a witness leaves the slime's timers alone and ignores canAbsorb.
      observe(s, ev);
      return;
    }
    case 'taste': {
      const { id, kind, m } = target(s, e);
      if (OUT_OF_REACH.includes(id)) fail(`out of reach: ${id}`); // CR15
      if (m.taste) fail(`already tasted: ${id}`);
      const ev: Evidence = { id: evidenceId('taste', id, m.life), kind, caveats: s.glow > 0 ? [] : [DARK], age: 0 };
      m.taste = clone(ev);
      observe(s, ev);
      return;
    }
    case 'mark':
    case 'unmark': {
      // CR14: a mark is not evidence and touches nothing else.
      const id = e['id'];
      if (typeof id !== 'string' || !LEVEL.includes(id)) fail(`unknown mushroom: ${String(id)}`);
      const m = s.mushrooms[id]!;
      if (e['type'] === 'mark') {
        if (m.consumed) fail(`mushroom consumed: ${id}`);
        if (OUT_OF_REACH.includes(id)) fail(`out of reach: ${id}`); // CR15
        if (m.marked) fail(`already marked: ${id}`);
        m.marked = true;
      } else {
        if (!m.marked) fail(`not marked: ${id}`);
        m.marked = false;
      }
      return;
    }
    case 'tick': {
      const dt = e['dt'];
      if (typeof dt !== 'number' || !Number.isFinite(dt) || dt < 0 || dt > MAX_DT) fail(`dt out of range: ${String(dt)}`);
      s.glow = Math.max(0, s.glow - dt);
      s.heavy = Math.max(0, s.heavy - dt);
      // CR5: tastes fade, wherever they are kept.
      const fade = (ev: Evidence | null): void => {
        if (!ev || ev.age === undefined) return;
        ev.age += dt;
        if (ev.age >= FADE_SECONDS) {
          if (!ev.caveats.includes(FADED)) ev.caveats.push(FADED);
          delete ev.age;
        }
      };
      for (const ev of s.memory) fade(ev);
      for (const m of Object.values(s.mushrooms)) fade(m.taste);
      for (const j of s.journal) for (const ev of j.evidence) fade(ev);
      // CR10: regrowth, unless the mushroom has used its last life.
      for (const m of Object.values(s.mushrooms)) {
        if (!m.consumed || m.life >= MAX_LIVES) continue;
        m.sinceConsumed += dt;
        if (m.sinceConsumed >= REGROW_SECONDS) {
          m.life += 1;
          m.consumed = false;
          m.sinceConsumed = 0;
          m.taste = null;
          m.consumedBy = null;
          m.marked = false;
        }
      }
      return;
    }
    default:
      fail(`unknown event type: ${String(e['type'])}`);
  }
}

// ---------------------------------------------------------------------------
// CR12: compact save format (v2).
// An observation is a short string, e.g. "t1.2dD@12.5" = taste of LEVEL[1] in
// life 2, duskcap, caveat tasted_in_dark, 12.5 s since tasting (while
// unfaded). Bare references (decision lists) drop kind, caveats and age:
// "t1.2". Everything is bounded (CR13), so the save is bounded too.

const SAVE_VERSION = 4; // v3 adds the CR14 mark, v4 the CR16 journal; v2/v3 saves still load
const VERB_CODES: Record<string, Verb> = { a: 'absorb', t: 'taste', w: 'witness' };
const CAVEAT_CODES: Record<string, string> = { D: DARK, F: FADED, S: SECONDHAND };
const ID_RE = /^(absorb|taste|witness)_([a-z]+)(?:_(\d+))?$/;
const REF_RE = /^([atw])(\d+)\.(\d+)$/;
const OBS_RE = /^([atw]\d+\.\d+)([gd])([DFS]*)(?:@(.+))?$/;

interface SaveV2 {
  v: number;
  t: [number, number]; // glow, heavy
  m: [number, number, number, string, string, number?][]; // per LEVEL: life, consumed, sinceConsumed, taste obs, consumedBy obs, marked
  e: string[]; // memory
  d: { s: string; b: string[]; r: string[]; c: string; h: string[]; k: string[] };
  j?: string[]; // CR16 journal: type char + space-separated observation codes
}

function codeOf(map: Record<string, string>, value: string): string {
  for (const [k, v] of Object.entries(map)) if (v === value) return k;
  return fail(`cannot encode ${value}`);
}

function refOf(id: string): string {
  const [, verb, mid, life] = ID_RE.exec(id) ?? fail(`bad evidence id ${id}`);
  const i = LEVEL.indexOf(mid!);
  if (i < 0) fail(`bad mushroom ${mid}`);
  return `${verb![0]}${i}.${life ?? 1}`;
}

function idOf(ref: string): string {
  const [, v, mi, life] = REF_RE.exec(ref) ?? fail(`bad reference ${ref}`);
  return evidenceId(VERB_CODES[v!] ?? fail(`bad verb ${v}`), LEVEL[Number(mi)] ?? fail(`bad mushroom ${mi}`), Number(life));
}

function encodeCaveats(cs: string[]): string {
  return cs.map((c) => codeOf(CAVEAT_CODES, c)).join('');
}

function decodeCaveats(codes: string): string[] {
  return [...codes].map((c) => CAVEAT_CODES[c] ?? fail(`bad caveat ${c}`));
}

function encodeObs(ev: Evidence | null): string {
  if (!ev) return '';
  return `${refOf(ev.id)}${ev.kind[0]}${encodeCaveats(ev.caveats)}${ev.age === undefined ? '' : `@${ev.age}`}`;
}

function decodeObs(code: string): Evidence | null {
  if (code === '') return null;
  const [, ref, k, cs, age] = OBS_RE.exec(code) ?? fail(`bad observation ${code}`);
  const kind: Kind = k === 'g' ? 'glowcap' : 'duskcap';
  const ev: Evidence = { id: idOf(ref!), kind, caveats: decodeCaveats(cs!) };
  if (age !== undefined) {
    ev.age = Number(age);
    if (!Number.isFinite(ev.age)) fail(`bad age ${age}`);
  }
  return ev;
}

function encode(s: State): SaveV2 {
  const d = s.decision;
  return {
    v: SAVE_VERSION,
    t: [s.glow, s.heavy],
    m: LEVEL.map((id) => {
      const x = s.mushrooms[id]!;
      return [x.life, x.consumed ? 1 : 0, x.sinceConsumed, encodeObs(x.taste), encodeObs(x.consumedBy), x.marked ? 1 : 0];
    }),
    e: s.memory.map(encodeObs),
    j: s.journal.map((j) => `${j.type[0]}${j.evidence.map(encodeObs).join(' ')}`),
    d: {
      s: d.state[0]!,
      b: d.basis.map(refOf),
      r: d.reopenedBy.map(refOf),
      c: encodeCaveats(d.caveats),
      h: d.history.map((h) => `${h.change[0]}${h.because.map(refOf).join(',')}`),
      k: d.recovery.map(refOf),
    },
  };
}

function decode(saved: unknown): State {
  const x = saved as SaveV2;
  if (typeof x !== 'object' || x === null || ![2, 3, SAVE_VERSION].includes(x.v)) fail('unsupported save');
  const num = (n: unknown): number => (typeof n === 'number' && Number.isFinite(n) ? n : fail(`bad number ${String(n)}`));
  const mushrooms: Record<string, MushroomState> = {};
  LEVEL.forEach((mid, i) => {
    // A save from before a mushroom was added has no entry for it: it starts fresh.
    const [life, consumed, sinceConsumed, taste, by, marked] = x.m[i] ?? [1, 0, 0, '', '', 0];
    mushrooms[mid] = {
      life: num(life),
      consumed: consumed === 1,
      sinceConsumed: num(sinceConsumed),
      taste: decodeObs(taste),
      consumedBy: decodeObs(by),
      marked: marked === 1 && consumed !== 1,
    };
  });
  const states: Record<string, DecisionState> = { n: 'none', c: 'committed', r: 'reopened' };
  return {
    glow: num(x.t[0]),
    heavy: num(x.t[1]),
    mushrooms,
    memory: x.e.map((c) => decodeObs(c) ?? fail('empty observation')),
    journal: (x.j ?? []).map((code): JournalData => {
      const types: Record<string, JournalData['type']> = { o: 'observed', c: 'committed', r: 'reopened' };
      const type = types[code[0] ?? ''] ?? fail(`bad journal entry ${code}`);
      return { type, evidence: code.slice(1).split(' ').map((c) => decodeObs(c) ?? fail(`bad journal entry ${code}`)) };
    }),
    decision: {
      state: states[x.d.s] ?? fail(`bad decision ${x.d.s}`),
      basis: x.d.b.map(idOf),
      reopenedBy: x.d.r.map(idOf),
      caveats: decodeCaveats(x.d.c),
      history: x.d.h.map((part) => ({
        change: part[0] === 'c' ? 'committed' : part[0] === 'r' ? 'reopened' : fail(`bad history ${part}`),
        because: part.slice(1).split(',').map(idOf),
      })),
      recovery: x.d.k.map(idOf),
    },
  };
}

export function createPolicy(saved?: unknown): Policy {
  let state: State = saved === undefined ? initialState() : decode(clone(saved));
  return {
    dispatch(event: unknown): void {
      const next = clone(state);
      apply(next, event);
      state = next;
    },
    view(): View {
      return render(state);
    },
    save(): unknown {
      return encode(state);
    },
  };
}
