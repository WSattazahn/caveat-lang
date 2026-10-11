// Glowcap beat policy.

export type Kind = 'glowcap' | 'duskcap';

const LEVEL: readonly string[] = ['cave', 'pool', 'ruin', 'grove', 'pit'];
// CR15: mushrooms only other creatures can reach.
const OUT_OF_REACH: readonly string[] = ['pit'];
const KINDS: readonly string[] = ['glowcap', 'duskcap'];
const GLOW_SECONDS = 30;
const HEAVY_SECONDS = 20;
const HEAVY_MAX = 30;
const MAX_DT = 0.1;
const REGROW_SECONDS = 45;
// CR10: declared capacity of lives per mushroom (spec requires at least 64).
export const MAX_LIVES = 1024;
const DARK = 'tasted_in_dark';
const FADED = 'taste_faded';
const SECONDHAND = 'secondhand';
// CR18: a doubted witness; the caveat doubles as the doubt flag.
const DOUBTED = 'doubted';
const FADE_SECONDS = 60;
// Tolerance so that e.g. 600 ticks of 0.1 s count as a full minute despite float rounding.
const EPS = 1e-9;
// CR13: bounded memory.
const MEMORY = 6;
const DECISION_MEMORY = 6;
// CR16: journal length.
const JOURNAL_MEMORY = 6;

interface MushroomState {
  consumed: boolean;
  // CR10: 1-based life number of the current (or last consumed) mushroom.
  life: number;
  // CR10: seconds (sum of tick dt) since this life was consumed.
  consumedAge: number;
  // CR11: evidence that consumed the current life (null while present).
  consumedBy: string | null;
  tasted: Kind | null;
  // CR19: the current life has been retasted (the retaste now identifies it).
  retasted: boolean;
  // CR14: marked by the player to avoid absorbing.
  marked: boolean;
}

interface State {
  glow: number;
  heavy: number;
  mushrooms: Record<string, MushroomState>;
  // CR13: the six most recent observations, oldest first. glow = supports the belief.
  memory: { ev: string; glow: boolean }[];
  // CR3: caveats carried by each evidence id still referenced (pruned otherwise).
  evidenceCaveats: Record<string, string[]>;
  // CR5: seconds elapsed (sum of tick dt) since each taste.
  tasteAge: Record<string, number>;
  // CR7: first two glowcap observations since the most recent contradiction.
  sinceContradiction: string[];
  // CR16: journal entries, oldest first (caveats are derived at render time).
  journal: { text: string; because: string[] }[];
  // caveats: frozen at commit time (CR5).
  decision: { state: 'none' | 'committed' | 'reopened'; basis: string[]; reopenedBy: string[]; caveats: string[]; history: HistoryEntry[] };
}

export interface HistoryEntry {
  change: 'committed' | 'reopened';
  because: string[];
}

export type BeliefState = 'none' | 'probably_safe' | 'probably_unsafe' | 'uncertain';

export interface MushroomView {
  present: boolean;
  marked: boolean;
  regrowsIn: number;
  fadesIn: number;
  canRetaste: boolean;
  label: string;
  canAbsorb: boolean;
  canTaste: boolean;
  because: string[];
  caveats: string[];
  why: { absorb: WhyView; taste: WhyView };
}

export interface WhyView {
  reason: string;
  because: string[];
  caveats: string[];
}

export interface View {
  slime: { glowing: boolean; glowSeconds: number; heavy: boolean; heavySeconds: number };
  mushrooms: Record<string, MushroomView>;
  belief: { state: BeliefState; text: string; note: string; supportedBy: string[]; contradictedBy: string[]; caveats: string[] };
  decision: { state: 'none' | 'committed' | 'reopened'; basis: string[]; reopenedBy: string[]; caveats: string[]; history: HistoryEntry[] };
  journal: { text: string; because: string[]; caveats: string[] }[];
}

export type GlowEvent =
  | { type: 'absorb'; id: string; kind: string }
  | { type: 'taste'; id: string; kind: string }
  | { type: 'witness'; id: string; kind: string }
  | { type: 'mark'; id: string }
  | { type: 'unmark'; id: string }
  | { type: 'doubt'; evidence: string }
  | { type: 'undoubt'; evidence: string }
  | { type: 'retaste'; id: string; kind: string }
  | { type: 'tick'; dt: number };

function initialState(): State {
  const mushrooms: Record<string, MushroomState> = {};
  for (const id of LEVEL) mushrooms[id] = { consumed: false, life: 1, consumedAge: 0, consumedBy: null, tasted: null, retasted: false, marked: false };
  return {
    glow: 0,
    heavy: 0,
    mushrooms,
    memory: [],
    evidenceCaveats: {},
    tasteAge: {},
    sinceContradiction: [],
    journal: [],
    decision: { state: 'none', basis: [], reopenedBy: [], caveats: [], history: [] },
  };
}

function clone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v)) as T;
}

function isDoubted(s: State, ev: string): boolean {
  return (s.evidenceCaveats[ev] ?? []).includes(DOUBTED);
}

// CR18: the belief counts remembered, undoubted observations only.
function counted(s: State): { ev: string; glow: boolean }[] {
  return s.memory.filter((o) => !isDoubted(s, o.ev));
}

function supportedBy(s: State): string[] {
  return counted(s).filter((o) => o.glow).map((o) => o.ev);
}

function contradictedBy(s: State): string[] {
  return counted(s).filter((o) => !o.glow).map((o) => o.ev);
}

function beliefState(s: State): BeliefState {
  const c = counted(s);
  const sup = c.some((o) => o.glow);
  const con = c.some((o) => !o.glow);
  if (sup && con) return 'uncertain';
  if (sup) return 'probably_safe';
  if (con) return 'probably_unsafe';
  return 'none';
}

function note(n: number): string {
  return n === 1 ? 'Based on one observation.' : `Based on ${n} observations.`;
}

function fail(msg: string): never {
  throw new Error(msg);
}

function checkMushroom(s: State, e: { id?: unknown; kind?: unknown }): { m: MushroomState; id: string; kind: Kind } {
  if (typeof e.id !== 'string' || !Object.hasOwn(s.mushrooms, e.id)) fail(`unknown mushroom ${String(e.id)}`);
  if (typeof e.kind !== 'string' || !KINDS.includes(e.kind)) fail(`unknown kind ${String(e.kind)}`);
  const m = s.mushrooms[e.id as string]!;
  if (m.consumed) fail(`mushroom ${e.id as string} already consumed`);
  return { m, id: e.id as string, kind: e.kind as Kind };
}

// Rules for one mushroom, shared by the view and by absorb validation.
// CR10: evidence id for the current life of a mushroom.
type Source = 'absorb' | 'taste' | 'witness' | 'retaste';

function evidenceId(prefix: Source, id: string, life: number): string {
  return life === 1 ? `${prefix}_${id}` : `${prefix}_${id}_${life}`;
}

// CR19: the taste evidence that identifies a mushroom's current life (retaste wins).
function identifyingTaste(s: State, id: string): string {
  const m = s.mushrooms[id]!;
  return evidenceId(m.retasted ? 'retaste' : 'taste', id, m.life);
}

function canRetaste(s: State, id: string): boolean {
  const m = s.mushrooms[id]!;
  if (OUT_OF_REACH.includes(id) || m.consumed || m.tasted === null || m.retasted) return false;
  return (s.evidenceCaveats[evidenceId('taste', id, m.life)] ?? []).includes(FADED);
}

function caveatsOf(s: State, evidence: readonly string[]): string[] {
  const out = new Set<string>();
  for (const ev of evidence) for (const c of s.evidenceCaveats[ev] ?? []) out.add(c);
  return [...out];
}

// Whole seconds rounded up (with float tolerance), 0 when nothing remains.
function secondsUp(remaining: number): number {
  return remaining > EPS ? Math.max(1, Math.ceil(remaining - EPS)) : 0;
}

function mushroomView(s: State, id: string): MushroomView {
  const v = mushroomRule(s, id);
  const m = s.mushrooms[id]!;
  const marked = m.marked;
  // CR17: countdowns derived from existing timers.
  const regrowsIn = m.consumed && m.life < MAX_LIVES ? secondsUp(REGROW_SECONDS - m.consumedAge) : 0;
  const tasteAge = m.consumed || m.tasted === null ? undefined : s.tasteAge[identifyingTaste(s, id)];
  const fadesIn = tasteAge === undefined ? 0 : secondsUp(FADE_SECONDS - tasteAge);
  const reachable = !OUT_OF_REACH.includes(id);
  // CR14: a mark only blocks absorbing. CR15: out-of-reach mushrooms allow neither action.
  return {
    ...v,
    marked,
    regrowsIn,
    fadesIn,
    canRetaste: canRetaste(s, id),
    canAbsorb: v.canAbsorb && !marked && reachable,
    canTaste: v.canTaste && reachable,
    caveats: caveatsOf(s, v.because),
    why: whyNot(s, id),
  };
}

// CR11: why an action is not allowed (first match), or reason '' when allowed.
function whyNot(s: State, id: string): { absorb: WhyView; taste: WhyView } {
  const m = s.mushrooms[id]!;
  const mk = (reason: string, because: string[]): WhyView => ({ reason, because, caveats: caveatsOf(s, because) });
  const allowed = mk('', []);
  const tasteEv = identifyingTaste(s, id);
  if (m.consumed) {
    const by = m.consumedBy !== null ? [m.consumedBy] : [];
    return { absorb: mk('Already eaten', by), taste: mk('Already eaten', [...by]) };
  }
  if (OUT_OF_REACH.includes(id)) return { absorb: mk('Out of reach', []), taste: mk('Out of reach', []) };
  let absorb = allowed;
  if (m.tasted === 'duskcap') absorb = mk('Known duskcap', [tasteEv]);
  else if (m.marked) absorb = mk('Marked to avoid', []);
  else if (m.tasted === null && contradictedBy(s).length >= 2) absorb = mk('Too risky untasted', contradictedBy(s));
  const taste = m.tasted !== null ? mk('Already tasted', [tasteEv]) : mk('', []);
  return { absorb, taste };
}

function mushroomRule(s: State, id: string): Omit<MushroomView, 'caveats' | 'why' | 'marked' | 'regrowsIn' | 'fadesIn' | 'canRetaste'> {
  const m = s.mushrooms[id]!;
  const bs = beliefState(s);
  const sup = supportedBy(s);
  const con = contradictedBy(s);
  const tasteEv = identifyingTaste(s, id);
  const tc = s.evidenceCaveats[tasteEv] ?? [];
  const dark = tc.includes(DARK);
  const faded = tc.includes(FADED);
  const qual = faded ? ' (taste has faded)' : dark ? ' (tasted in the dark)' : '';
  if (m.consumed) return { present: false, label: '', canAbsorb: false, canTaste: false, because: [] };
  if (m.tasted === 'glowcap')
    return { present: true, label: qual ? `Probably a glowcap${qual}` : 'Glowcap', canAbsorb: true, canTaste: false, because: [tasteEv] };
  if (m.tasted === 'duskcap')
    return { present: true, label: qual ? `Probably a duskcap${qual}` : 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: [tasteEv] };
  // CR2: twice bitten.
  if (con.length >= 2) return { present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: [...con] };
  if (bs === 'none') return { present: true, label: 'Glowing mushroom', canAbsorb: true, canTaste: true, because: [] };
  if (bs === 'probably_safe') return { present: true, label: 'Probably a glowcap', canAbsorb: true, canTaste: true, because: [...sup] };
  if (bs === 'probably_unsafe') return { present: true, label: 'Probably a duskcap', canAbsorb: true, canTaste: true, because: [...con] };
  // CR4: cite the counterexample only.
  return { present: true, label: 'Could be a duskcap — taste first', canAbsorb: true, canTaste: true, because: [...con] };
}

function pushBounded<T>(list: T[], item: T, max: number): void {
  list.push(item);
  while (list.length > max) list.shift();
}

function commit(s: State, basis: string[]): HistoryEntry {
  s.decision.state = 'committed';
  s.decision.basis = [...basis];
  s.decision.reopenedBy = [];
  s.decision.caveats = caveatsOf(s, basis);
  const entry: HistoryEntry = { change: 'committed', because: [...basis] };
  pushBounded(s.decision.history, entry, DECISION_MEMORY);
  return entry;
}

// Adds one observation to the (bounded) memory and updates the trust decision.
function record(s: State, ev: string, kind: Kind): HistoryEntry | null {
  const glow = kind === 'glowcap';
  pushBounded(s.memory, { ev, glow }, MEMORY);
  if (glow) {
    if (s.sinceContradiction.length < 2) s.sinceContradiction.push(ev);
    if (s.decision.state === 'none') {
      // CR13: never committed: commit when the belief (after forgetting) is probably_safe; basis is this observation.
      if (beliefState(s) === 'probably_safe') return commit(s, [ev]);
    } else if (s.decision.state === 'reopened' && s.sinceContradiction.length >= 2) {
      // CR7: trust recovers on the second glowcap observation since the latest contradiction.
      return commit(s, s.sinceContradiction.slice(0, 2));
    }
    return null;
  } else {
    s.sinceContradiction = [];
    if (s.decision.state !== 'none') {
      s.decision.state = 'reopened';
      pushBounded(s.decision.reopenedBy, ev, DECISION_MEMORY);
      const entry: HistoryEntry = { change: 'reopened', because: [ev] };
      pushBounded(s.decision.history, entry, DECISION_MEMORY);
      return entry;
    }
    return null;
  }
}

// CR16: journal an accepted observation and any trust change it caused.
const JOURNAL_TEXT: Record<string, Record<Kind, string>> = {
  absorb: { glowcap: 'Absorbed the <id> mushroom: it gave light.', duskcap: 'Absorbed the <id> mushroom: it made me heavy.' },
  taste: { glowcap: 'Tasted the <id> mushroom: sweet.', duskcap: 'Tasted the <id> mushroom: bitter.' },
  retaste: { glowcap: 'Tasted the <id> mushroom again: sweet.', duskcap: 'Tasted the <id> mushroom again: bitter.' },
  witness: { glowcap: 'Saw a creature eat the <id> mushroom: it glowed.', duskcap: 'Saw a creature eat the <id> mushroom: it grew heavy.' },
};

function observe(s: State, source: Source, id: string, ev: string, kind: Kind): void {
  const change = record(s, ev, kind);
  pushBounded(s.journal, { text: JOURNAL_TEXT[source]![kind].replace('<id>', id), because: [ev] }, JOURNAL_MEMORY);
  if (change !== null) {
    const text = change.change === 'committed' ? 'I trust glowing mushrooms now.' : 'I no longer trust glowing mushrooms.';
    pushBounded(s.journal, { text, because: [...change.because] }, JOURNAL_MEMORY);
  }
}

// CR13: drop caveats and taste clocks of evidence nothing can cite any more.
// Evidence ids are never reused (lives only grow), so this cannot change behaviour.
function prune(s: State): void {
  const keep = new Set<string>(s.memory.map((o) => o.ev));
  for (const j of s.journal) for (const ev of j.because) keep.add(ev);
  for (const id of LEVEL) {
    const m = s.mushrooms[id]!;
    if (m.consumedBy !== null) keep.add(m.consumedBy);
    if (!m.consumed && m.tasted !== null) {
      keep.add(evidenceId('taste', id, m.life));
      if (m.retasted) keep.add(evidenceId('retaste', id, m.life));
    }
  }
  for (const ev of Object.keys(s.evidenceCaveats)) if (!keep.has(ev)) delete s.evidenceCaveats[ev];
  for (const ev of Object.keys(s.tasteAge)) if (!keep.has(ev)) delete s.tasteAge[ev];
}

// Applies an event to a state in place. Throws (before or after partial mutation)
// on rejection; the caller discards the working copy in that case.
function apply(s: State, e: GlowEvent): void {
  applyEvent(s, e);
  prune(s);
}

function applyEvent(s: State, e: GlowEvent): void {
  if (e === null || typeof e !== 'object') fail('event must be an object');
  switch (e.type) {
    case 'tick': {
      const dt = e.dt;
      if (typeof dt !== 'number' || !Number.isFinite(dt) || dt < 0 || dt > MAX_DT) fail(`dt out of range: ${String(dt)}`);
      s.glow = Math.max(0, s.glow - dt);
      s.heavy = Math.max(0, s.heavy - dt);
      for (const id of LEVEL) {
        const m = s.mushrooms[id]!;
        if (!m.consumed || m.life >= MAX_LIVES) continue;
        m.consumedAge += dt;
        if (m.consumedAge >= REGROW_SECONDS - EPS) {
          // CR10: a stranger grows in its place.
          m.consumed = false;
          m.consumedAge = 0;
          m.consumedBy = null;
          m.life += 1;
          m.tasted = null;
          m.retasted = false;
        }
      }
      for (const ev of Object.keys(s.tasteAge)) {
        const age = s.tasteAge[ev]! + dt;
        s.tasteAge[ev] = age;
        const cav = s.evidenceCaveats[ev] ?? [];
        if (age >= FADE_SECONDS - EPS) {
          if (!cav.includes(FADED)) s.evidenceCaveats[ev] = [...cav, FADED];
          // Once faded, the age no longer matters; drop it to keep saves small.
          delete s.tasteAge[ev];
        }
      }
      return;
    }
    case 'taste': {
      const { m } = checkMushroom(s, e);
      if (OUT_OF_REACH.includes(e.id)) fail(`mushroom ${e.id} is out of reach`);
      if (m.tasted !== null) fail(`mushroom ${e.id} already tasted`);
      m.tasted = e.kind as Kind;
      const ev = evidenceId('taste', e.id, m.life);
      s.evidenceCaveats[ev] = s.glow > 0 ? [] : [DARK];
      s.tasteAge[ev] = 0;
      observe(s, 'taste', e.id, ev, e.kind as Kind);
      return;
    }
    case 'absorb': {
      const { m, id, kind } = checkMushroom(s, e);
      if (!mushroomView(s, id).canAbsorb) fail(`cannot absorb ${id}`);
      if (m.tasted !== null && m.tasted !== kind) fail(`kind contradicts taste of ${id}`);
      m.consumed = true;
      m.marked = false;
      m.consumedAge = 0;
      const ev = evidenceId('absorb', id, m.life);
      m.consumedBy = ev;
      s.evidenceCaveats[ev] = [];
      if (kind === 'glowcap') s.glow = GLOW_SECONDS;
      // CR8: heaviness stacks up to 30 s.
      else s.heavy = s.heavy > 0 ? Math.min(HEAVY_MAX, s.heavy + HEAVY_SECONDS) : HEAVY_SECONDS;
      observe(s, 'absorb', id, ev, kind);
      return;
    }
    case 'witness': {
      // CR9: another creature eats it; no effect on the slime's timers, no canAbsorb check.
      const { m, id, kind } = checkMushroom(s, e);
      if (m.tasted !== null && m.tasted !== kind) fail(`kind contradicts taste of ${id}`);
      m.consumed = true;
      m.marked = false;
      m.consumedAge = 0;
      const ev = evidenceId('witness', id, m.life);
      m.consumedBy = ev;
      s.evidenceCaveats[ev] = [SECONDHAND];
      observe(s, 'witness', id, ev, kind);
      return;
    }
    case 'mark':
    case 'unmark': {
      // CR14: marks are not evidence and change no timer.
      const id = (e as { id?: unknown }).id;
      if (typeof id !== 'string' || !Object.hasOwn(s.mushrooms, id)) fail(`unknown mushroom ${String(id)}`);
      const m = s.mushrooms[id]!;
      if (e.type === 'mark') {
        if (OUT_OF_REACH.includes(id)) fail(`mushroom ${id} is out of reach`);
        if (m.consumed) fail(`mushroom ${id} already consumed`);
        if (m.marked) fail(`mushroom ${id} already marked`);
        m.marked = true;
      } else {
        if (!m.marked) fail(`mushroom ${id} not marked`);
        m.marked = false;
      }
      return;
    }
    case 'retaste': {
      // CR19: refresh a faded taste once per life.
      const { m, id, kind } = checkMushroom(s, e);
      if (OUT_OF_REACH.includes(id)) fail(`mushroom ${id} is out of reach`);
      if (m.tasted === null) fail(`mushroom ${id} has not been tasted`);
      if (m.retasted) fail(`mushroom ${id} already retasted`);
      if (!canRetaste(s, id)) fail(`taste of ${id} has not faded`);
      if (kind !== m.tasted) fail(`kind contradicts taste of ${id}`);
      m.retasted = true;
      const ev = evidenceId('retaste', id, m.life);
      s.evidenceCaveats[ev] = s.glow > 0 ? [] : [DARK];
      s.tasteAge[ev] = 0;
      observe(s, 'retaste', id, ev, kind);
      return;
    }
    case 'doubt':
    case 'undoubt': {
      // CR18/CR20: (un)doubt a remembered witness. Not an observation: forgets nothing,
      // changes no timer or decision; journaled (CR20).
      const ev = (e as { evidence?: unknown }).evidence;
      const parsed = typeof ev === 'string' ? /^witness_([a-z]+)(?:_\d+)?$/.exec(ev) : null;
      if (typeof ev !== 'string' || parsed === null) fail(`not a witness: ${String(ev)}`);
      if (!s.memory.some((o) => o.ev === ev)) fail(`${ev} is not remembered`);
      const name = parsed[1]!;
      if (e.type === 'doubt') {
        if (isDoubted(s, ev)) fail(`${ev} is already doubted`);
        if (!s.memory.some((o) => o.ev !== ev && !isDoubted(s, o.ev))) fail('at least one undoubted observation must remain');
        s.evidenceCaveats[ev] = [...(s.evidenceCaveats[ev] ?? []), DOUBTED];
        pushBounded(s.journal, { text: `I doubt what I saw at the ${name} mushroom.`, because: [ev] }, JOURNAL_MEMORY);
      } else {
        if (!isDoubted(s, ev)) fail(`${ev} is not doubted`);
        s.evidenceCaveats[ev] = (s.evidenceCaveats[ev] ?? []).filter((c) => c !== DOUBTED);
        pushBounded(s.journal, { text: `I believe what I saw at the ${name} mushroom again.`, because: [ev] }, JOURNAL_MEMORY);
      }
      return;
    }
    default:
      fail(`unknown event type ${String((e as { type?: unknown }).type)}`);
  }
}

function render(s: State): View {
  const bs = beliefState(s);
  const sup = supportedBy(s);
  const con = contradictedBy(s);
  const belief = {
    state: bs,
    text:
      bs === 'probably_safe'
        ? 'Glowing mushrooms give you light.'
        : bs === 'probably_unsafe'
          ? 'Glowing mushrooms make you heavy.'
          : bs === 'uncertain'
            ? 'Not every glowing mushroom is safe. Taste before absorbing.'
            : '',
    note: bs === 'probably_safe' ? note(sup.length) : bs === 'probably_unsafe' ? note(con.length) : '',
    supportedBy: sup,
    contradictedBy: con,
    caveats: caveatsOf(s, [...sup, ...con]),
  };
  const mushrooms: Record<string, MushroomView> = {};
  for (const id of LEVEL) mushrooms[id] = mushroomView(s, id);
  return {
    slime: {
      glowing: s.glow > 0,
      glowSeconds: s.glow > 0 ? Math.max(1, Math.ceil(s.glow - EPS)) : 0,
      heavy: s.heavy > 0,
      heavySeconds: s.heavy > 0 ? Math.max(1, Math.ceil(s.heavy - EPS)) : 0,
    },
    mushrooms,
    belief,
    decision: { state: s.decision.state, basis: [...s.decision.basis], reopenedBy: [...s.decision.reopenedBy], caveats: [...s.decision.caveats],
      history: s.decision.history.map((h) => ({ change: h.change, because: [...h.because] })),
    },
    journal: s.journal.map((j) => ({ text: j.text, because: [...j.because], caveats: caveatsOf(s, j.because) })),
  };
}

// ---- CR12/CR13: compact save format ----
// Evidence ids are written as `<source letter><mushroom index>[_<life>]`, caveats as
// letter codes. Everything in the state is bounded (CR13), so the save is too.

const SAVE_VERSION = 7;
const PREFIX_CODE: Record<string, string> = { absorb: 'a', taste: 't', witness: 'w', retaste: 'r' };
const CODE_PREFIX: Record<string, Source> = { a: 'absorb', t: 'taste', w: 'witness', r: 'retaste' };
const CAVEAT_CODE: Record<string, string> = { [DARK]: 'd', [FADED]: 'f', [SECONDHAND]: 's', [DOUBTED]: 'x' };
const CODE_CAVEAT: Record<string, string> = { d: DARK, f: FADED, s: SECONDHAND, x: DOUBTED };
const DECISION_CODES = ['none', 'committed', 'reopened'] as const;
const TASTE_CODES = [null, 'glowcap', 'duskcap'] as const;

interface Saved {
  v: number;
  g: number; // glow timer
  h: number; // heavy timer
  m: [number, number, number, string, number, number, number][]; // life, consumed, consumedAge, consumedBy code|'', tasted code, marked, retasted
  o: string[]; // memory: '+'|'-' then evidence code
  c: Record<string, string>; // evidence code -> caveat codes
  t: Record<string, number>; // evidence code -> taste age
  sc: string[];
  d: [number, string[], string[], string, [number, string[]][]]; // state, basis, reopenedBy, caveat codes, history
  j: [string, string[]][]; // journal: text, because
}

function encodeEv(ev: string): string {
  const m = /^(absorb|taste|witness|retaste)_([a-z]+)(?:_(\d+))?$/.exec(ev);
  const idx = m ? LEVEL.indexOf(m[2]!) : -1;
  if (!m || idx < 0) fail(`bad evidence id ${ev}`);
  return `${PREFIX_CODE[m[1]!]}${idx}${m[3] ? `_${m[3]}` : ''}`;
}

function decodeEv(code: string): string {
  const m = /^([atwr])(\d+)(?:_(\d+))?$/.exec(code);
  const id = m ? LEVEL[Number(m[2])] : undefined;
  if (!m || id === undefined) fail(`bad saved evidence ${code}`);
  return evidenceId(CODE_PREFIX[m[1]!]!, id, m[3] ? Number(m[3]) : 1);
}

function encodeCaveats(cs: readonly string[]): string {
  return cs.map((c) => CAVEAT_CODE[c] ?? fail(`unknown caveat ${c}`)).join('');
}

function decodeCaveats(code: string): string[] {
  return [...code].map((c) => CODE_CAVEAT[c] ?? fail(`unknown caveat code ${c}`));
}

function encodeState(s: State): Saved {
  const c: Record<string, string> = {};
  for (const [ev, cs] of Object.entries(s.evidenceCaveats)) if (cs.length > 0) c[encodeEv(ev)] = encodeCaveats(cs);
  const t: Record<string, number> = {};
  for (const [ev, age] of Object.entries(s.tasteAge)) t[encodeEv(ev)] = age;
  return {
    v: SAVE_VERSION,
    g: s.glow,
    h: s.heavy,
    m: LEVEL.map((id) => {
      const m = s.mushrooms[id]!;
      return [m.life, m.consumed ? 1 : 0, m.consumedAge, m.consumedBy === null ? '' : encodeEv(m.consumedBy), TASTE_CODES.indexOf(m.tasted), m.marked ? 1 : 0, m.retasted ? 1 : 0];
    }),
    o: s.memory.map((o) => `${o.glow ? '+' : '-'}${encodeEv(o.ev)}`),
    c,
    t,
    sc: s.sinceContradiction.map(encodeEv),
    d: [
      DECISION_CODES.indexOf(s.decision.state),
      s.decision.basis.map(encodeEv),
      s.decision.reopenedBy.map(encodeEv),
      encodeCaveats(s.decision.caveats),
      s.decision.history.map((h) => [h.change === 'committed' ? 1 : 2, h.because.map(encodeEv)]),
    ],
    j: s.journal.map((j) => [j.text, j.because.map(encodeEv)]),
  };
}

function decodeState(raw: unknown): State {
  const v = raw as Saved;
  if (v === null || typeof v !== 'object' || v.v !== SAVE_VERSION) fail('unsupported save');
  const s = initialState();
  s.glow = v.g;
  s.heavy = v.h;
  LEVEL.forEach((id, i) => {
    const [life, consumed, consumedAge, consumedBy, tasted, marked, retasted] = v.m[i] ?? fail('bad saved mushroom');
    s.mushrooms[id] = {
      consumed: consumed === 1,
      life,
      consumedAge,
      consumedBy: consumedBy === '' ? null : decodeEv(consumedBy),
      tasted: TASTE_CODES[tasted] ?? null,
      marked: marked === 1,
      retasted: retasted === 1,
    };
  });
  s.memory = v.o.map((o) => ({ ev: decodeEv(o.slice(1)), glow: o[0] === '+' }));
  // Every referenced evidence has a caveat list (possibly empty).
  for (const ev of s.memory.map((o) => o.ev)) s.evidenceCaveats[ev] = [];
  for (const m of Object.values(s.mushrooms)) if (m.consumedBy !== null) s.evidenceCaveats[m.consumedBy] = [];
  LEVEL.forEach((id) => {
    const m = s.mushrooms[id]!;
    if (!m.consumed && m.tasted !== null) {
      s.evidenceCaveats[evidenceId('taste', id, m.life)] = [];
      if (m.retasted) s.evidenceCaveats[evidenceId('retaste', id, m.life)] = [];
    }
  });
  s.journal = v.j.map(([text, because]) => ({ text, because: because.map(decodeEv) }));
  for (const j of s.journal) for (const ev of j.because) s.evidenceCaveats[ev] ??= [];
  for (const [code, cs] of Object.entries(v.c)) s.evidenceCaveats[decodeEv(code)] = decodeCaveats(cs);
  for (const [code, age] of Object.entries(v.t)) s.tasteAge[decodeEv(code)] = age;
  s.sinceContradiction = v.sc.map(decodeEv);
  const [state, basis, reopenedBy, cav, history] = v.d;
  s.decision = {
    state: DECISION_CODES[state] ?? fail('bad decision state'),
    basis: basis.map(decodeEv),
    reopenedBy: reopenedBy.map(decodeEv),
    caveats: decodeCaveats(cav),
    history: history.map(([c, b]) => ({ change: c === 1 ? 'committed' : 'reopened', because: b.map(decodeEv) })),
  };
  return s;
}

export function createPolicy(saved?: unknown) {
  let state: State = saved === undefined ? initialState() : decodeState(saved);
  return {
    dispatch(event: GlowEvent): void {
      const next = clone(state);
      apply(next, event);
      state = next;
    },
    view(): View {
      return render(state);
    },
    save(): unknown {
      return encodeState(state);
    },
  };
}
