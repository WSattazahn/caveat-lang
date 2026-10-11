// Glowcap beat policy. State is plain JSON data; the view is derived from it.

type Kind = 'glowcap' | 'duskcap';

interface MushroomState {
  consumed: boolean;
  tasted: Kind | null; // taste of the current life
  life: number; // 1-based life number of the current (or last) mushroom here
  consumedFor: number; // seconds of ticks since consumed (meaningful while consumed)
  consumedBy: string | null; // evidence that consumed the current life
  marked: boolean; // CR14: player's mark; never evidence
  retasted: boolean; // CR19: the current life has been retasted
}

// One observation about glowing mushrooms.
interface Evidence {
  id: string; // absorb_<mushroom>[_<life>], taste_... or witness_...
  kind: Kind; // glowcap supports the belief, duskcap contradicts it
  caveats: string[];
  age?: number; // unfaded tastes only: seconds of ticks since the taste
}

interface State {
  glow: number;
  heavy: number;
  mushrooms: Record<string, MushroomState>;
  // Every observation something still cites (the remembered window, a mushroom's
  // current-life taste, what consumed a mushroom). Bounded: pruned after changes.
  evidence: Evidence[];
  // CR13: ids of the remembered observations, oldest first (at most MEMORY).
  window: string[];
  // CR7: glowcap observations accepted since the most recent contradiction (at most RECOVERY_COUNT).
  recovery: string[];
  decision: {
    state: 'none' | 'committed' | 'reopened';
    basis: string[];
    reopenedBy: string[]; // at most DECISION_KEEP
    caveats: string[]; // caveats the basis carried when committed (frozen)
    history: { change: 'committed' | 'reopened'; because: string[] }[]; // at most DECISION_KEEP
  };
  // CR16: journal entries, oldest first (at most JOURNAL_KEEP). An observation entry
  // records the observed kind; its text is derived from the evidence id.
  journal: JournalEntry[];
}

type Change = 'committed' | 'reopened';

// CR20: doubts and undoubts are journaled too.
type Reconsider = 'doubt' | 'undoubt';

interface JournalEntry {
  entry: Kind | Change | Reconsider;
  because: string[];
}

const LEVEL = ['cave', 'pool', 'ruin', 'grove', 'pit'];
// CR15: mushrooms only other creatures can reach (never tasted, absorbed or marked).
const OUT_OF_REACH = ['pit'];
const REGROW_SECONDS = 45;
// CR10: declared capacity of lives per mushroom spot (spec requires at least 64).
export const MAX_LIVES = 1024;
const GLOW_SECONDS = 30;
const HEAVY_SECONDS = 20;
const HEAVY_MAX_SECONDS = 30;
const TASTED_IN_DARK = 'tasted_in_dark';
const SECONDHAND = 'secondhand';
const TASTE_FADED = 'taste_faded';
const DOUBTED = 'doubted';
const TASTE_FADE_SECONDS = 60;
const RECOVERY_COUNT = 2;
const MEMORY = 6;
const DECISION_KEEP = 6;
const JOURNAL_KEEP = 6;

function initialState(): State {
  const mushrooms: Record<string, MushroomState> = {};
  for (const id of LEVEL) mushrooms[id] = { consumed: false, tasted: null, life: 1, consumedFor: 0, consumedBy: null, marked: false, retasted: false };
  return {
    glow: 0,
    heavy: 0,
    mushrooms,
    evidence: [],
    window: [],
    recovery: [],
    decision: { state: 'none', basis: [], reopenedBy: [], caveats: [], history: [] },
    journal: [],
  };
}

function clone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v)) as T;
}

function isKind(k: unknown): k is Kind {
  return k === 'glowcap' || k === 'duskcap';
}

// Evidence id for the mushroom's current life: first life has no suffix.
type Verb = 'absorb' | 'taste' | 'witness' | 'retaste';

function evidenceId(s: State, verb: Verb, id: string): string {
  const life = s.mushrooms[id].life;
  return life === 1 ? `${verb}_${id}` : `${verb}_${id}_${life}`;
}

// CR19: the taste that identifies the current life (the retaste once there is one).
function identifyingTaste(s: State, id: string): string {
  return evidenceId(s, s.mushrooms[id].retasted ? 'retaste' : 'taste', id);
}

function find(s: State, id: string): Evidence | undefined {
  return s.evidence.find((x) => x.id === id);
}

function consume(m: MushroomState, by: string): void {
  m.consumed = true;
  m.consumedFor = 0;
  m.consumedBy = by;
  m.marked = false; // CR14: eating clears the mark
}

function isDoubted(e: Evidence): boolean {
  return e.caveats.includes(DOUBTED);
}

// The belief's evidence: remembered (CR13), undoubted (CR18) observations only.
function split(s: State) {
  const counted = s.window
    .map((id) => find(s, id))
    .filter((e): e is Evidence => e !== undefined && !isDoubted(e));
  const supportedBy = counted.filter((e) => e.kind === 'glowcap').map((e) => e.id);
  const contradictedBy = counted.filter((e) => e.kind === 'duskcap').map((e) => e.id);
  return { supportedBy, contradictedBy, all: counted.map((e) => e.id) };
}

// Union of the caveats carried by the cited evidence, in first-seen order.
function caveatsOf(s: State, ids: string[]): string[] {
  const out: string[] = [];
  for (const id of ids) {
    const e = find(s, id);
    if (!e) continue;
    for (const c of e.caveats) if (!out.includes(c)) out.push(c);
  }
  return out;
}

type BeliefState = 'none' | 'probably_safe' | 'probably_unsafe' | 'uncertain';

function beliefState(sup: number, con: number): BeliefState {
  if (sup === 0 && con === 0) return 'none';
  if (con === 0) return 'probably_safe';
  if (sup === 0) return 'probably_unsafe';
  return 'uncertain';
}

function currentBelief(s: State): BeliefState {
  const { supportedBy, contradictedBy } = split(s);
  return beliefState(supportedBy.length, contradictedBy.length);
}

function observations(n: number): string {
  return n === 1 ? 'Based on one observation.' : `Based on ${n} observations.`;
}

interface Why {
  reason: string;
  because: string[];
  caveats: string[];
}

interface MushroomRow {
  present: boolean;
  marked: boolean;
  canRetaste: boolean;
  regrowsIn: number;
  fadesIn: number;
  label: string;
  canAbsorb: boolean;
  canTaste: boolean;
  because: string[];
  caveats: string[];
  why: { absorb: Why; taste: Why };
}

// A refusal: reason and the evidence it cites. null means the action is allowed.
type Refusal = { reason: string; because: string[] } | null;

function why(s: State, r: Refusal): Why {
  if (r === null) return { reason: '', because: [], caveats: [] };
  return { reason: r.reason, because: [...r.because], caveats: caveatsOf(s, r.because) };
}

// CR19: a faded first taste of the current life may be refreshed once.
function canRetaste(s: State, id: string): boolean {
  const m = s.mushrooms[id];
  if (OUT_OF_REACH.includes(id) || m.consumed || m.tasted === null || m.retasted) return false;
  const taste = find(s, evidenceId(s, 'taste', id));
  return taste !== undefined && taste.caveats.includes(TASTE_FADED);
}

// CR17: whole seconds rounded up; 0 when the timer is not running.
function countdowns(s: State, id: string): { regrowsIn: number; fadesIn: number } {
  const m = s.mushrooms[id];
  const regrowsIn = m.consumed && m.life < MAX_LIVES ? Math.ceil(REGROW_SECONDS - m.consumedFor) : 0;
  let fadesIn = 0;
  if (!m.consumed && m.tasted !== null) {
    const taste = find(s, identifyingTaste(s, id));
    if (taste?.age !== undefined) fadesIn = Math.ceil(TASTE_FADE_SECONDS - taste.age);
  }
  return { regrowsIn, fadesIn };
}

// canAbsorb/canTaste are derived from the refusals, so they can never disagree with `why`.
function row(s: State, id: string, label: string, because: string[], absorb: Refusal, taste: Refusal): MushroomRow {
  const m = s.mushrooms[id];
  return {
    present: !m.consumed,
    marked: m.marked,
    canRetaste: canRetaste(s, id),
    ...countdowns(s, id),
    label,
    canAbsorb: absorb === null,
    canTaste: taste === null,
    because: [...because],
    caveats: caveatsOf(s, because),
    why: { absorb: why(s, absorb), taste: why(s, taste) },
  };
}

// The single source of truth for what the HUD shows and allows for one mushroom.
function mushroomRow(s: State, id: string): MushroomRow {
  const m = s.mushrooms[id];
  if (m.consumed) {
    const eaten: Refusal = { reason: 'Already eaten', because: m.consumedBy === null ? [] : [m.consumedBy] };
    return row(s, id, '', [], eaten, eaten);
  }
  // CR15: out of reach comes directly after Already eaten, for both actions.
  const outOfReach: Refusal = OUT_OF_REACH.includes(id) ? { reason: 'Out of reach', because: [] } : null;
  // CR14: a mark refuses absorbing after `Known duskcap` and before the twice-bitten check.
  const marked: Refusal = m.marked ? { reason: 'Marked to avoid', because: [] } : null;
  if (m.tasted !== null) {
    const because = [identifyingTaste(s, id)];
    const cav = caveatsOf(s, because);
    const tasted: Refusal = { reason: 'Already tasted', because };
    // CR5: the faded label wins over the dark label.
    const qualifier = cav.includes(TASTE_FADED) ? ' (taste has faded)' : cav.includes(TASTED_IN_DARK) ? ' (tasted in the dark)' : null;
    if (m.tasted === 'glowcap') {
      return row(s, id, qualifier ? `Probably a glowcap${qualifier}` : 'Glowcap', because, marked, tasted);
    }
    const known: Refusal = { reason: 'Known duskcap', because };
    return row(s, id, qualifier ? `Probably a duskcap${qualifier}` : 'Duskcap — avoid', because, known, tasted);
  }
  // Unknown mushroom: labelled from the belief.
  const { supportedBy, contradictedBy } = split(s);
  let label: string;
  let because: string[];
  let absorb: Refusal = marked;
  if (contradictedBy.length >= 2) {
    // CR2: twice bitten (remembered contradictions only, CR13).
    label = 'Too risky — taste first';
    because = contradictedBy;
    absorb = absorb ?? { reason: 'Too risky untasted', because: contradictedBy };
  } else {
    const bs = beliefState(supportedBy.length, contradictedBy.length);
    if (bs === 'none') [label, because] = ['Glowing mushroom', []];
    else if (bs === 'probably_safe') [label, because] = ['Probably a glowcap', supportedBy];
    else if (bs === 'probably_unsafe') [label, because] = ['Probably a duskcap', contradictedBy];
    // CR4: cite the counterexample only.
    else [label, because] = ['Could be a duskcap — taste first', contradictedBy];
  }
  if (outOfReach) return row(s, id, label, because, outOfReach, outOfReach);
  return row(s, id, label, because, absorb, null);
}

function viewOf(s: State) {
  const { supportedBy, contradictedBy, all } = split(s);
  const bs = beliefState(supportedBy.length, contradictedBy.length);
  let text = '';
  let note = '';
  if (bs === 'probably_safe') {
    text = 'Glowing mushrooms give you light.';
    note = observations(supportedBy.length);
  } else if (bs === 'probably_unsafe') {
    text = 'Glowing mushrooms make you heavy.';
    note = observations(contradictedBy.length);
  } else if (bs === 'uncertain') {
    text = 'Not every glowing mushroom is safe. Taste before absorbing.';
  }

  const mushrooms: Record<string, MushroomRow> = {};
  for (const id of Object.keys(s.mushrooms)) mushrooms[id] = mushroomRow(s, id);

  return {
    slime: {
      glowing: s.glow > 0,
      glowSeconds: s.glow > 0 ? Math.ceil(s.glow) : 0,
      heavy: s.heavy > 0,
      heavySeconds: s.heavy > 0 ? Math.ceil(s.heavy) : 0,
    },
    mushrooms,
    belief: { state: bs, text, note, supportedBy, contradictedBy, caveats: caveatsOf(s, all) },
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

const OBSERVATION_TEXT: Record<string, Record<Kind, string>> = {
  absorb: { glowcap: 'Absorbed the <id> mushroom: it gave light.', duskcap: 'Absorbed the <id> mushroom: it made me heavy.' },
  taste: { glowcap: 'Tasted the <id> mushroom: sweet.', duskcap: 'Tasted the <id> mushroom: bitter.' },
  witness: { glowcap: 'Saw a creature eat the <id> mushroom: it glowed.', duskcap: 'Saw a creature eat the <id> mushroom: it grew heavy.' },
  retaste: { glowcap: 'Tasted the <id> mushroom again: sweet.', duskcap: 'Tasted the <id> mushroom again: bitter.' },
};

function journalText(j: JournalEntry): string {
  if (j.entry === 'committed') return 'I trust glowing mushrooms now.';
  if (j.entry === 'reopened') return 'I no longer trust glowing mushrooms.';
  const [verb, name] = j.because[0].split('_');
  if (j.entry === 'doubt') return `I doubt what I saw at the ${name} mushroom.`;
  if (j.entry === 'undoubt') return `I believe what I saw at the ${name} mushroom again.`;
  return OBSERVATION_TEXT[verb][j.entry].replace('<id>', name);
}

function pushBounded<T>(list: T[], item: T, max: number): void {
  list.push(item);
  while (list.length > max) list.shift();
}

function commit(s: State, basis: string[]): Change {
  const d = s.decision;
  d.state = 'committed';
  d.basis = [...basis];
  d.reopenedBy = [];
  d.caveats = caveatsOf(s, basis);
  pushBounded(d.history, { change: 'committed', because: [...basis] }, DECISION_KEEP);
  return 'committed';
}

// Drops evidence nothing cites any more.
function prune(s: State): void {
  const keep = new Set<string>(s.window);
  for (const j of s.journal) for (const id of j.because) keep.add(id);
  for (const id of Object.keys(s.mushrooms)) {
    const m = s.mushrooms[id];
    if (m.consumedBy !== null) keep.add(m.consumedBy);
    if (!m.consumed && m.tasted !== null) keep.add(identifyingTaste(s, id));
  }
  s.evidence = s.evidence.filter((e) => keep.has(e.id));
}

// Records an accepted observation, forgets beyond the window, updates trust and the journal.
function observe(s: State, e: Evidence): void {
  const change = decide(s, e);
  pushBounded(s.journal, { entry: e.kind, because: [e.id] }, JOURNAL_KEEP);
  if (change !== null) {
    const because = change === 'committed' ? [...s.decision.basis] : [e.id];
    pushBounded(s.journal, { entry: change, because }, JOURNAL_KEEP);
  }
  prune(s);
}

// Adds the observation to memory and updates the trust decision; returns the history change, if any.
function decide(s: State, e: Evidence): Change | null {
  s.evidence.push(e);
  pushBounded(s.window, e.id, MEMORY);
  const d = s.decision;
  if (e.kind === 'duskcap') s.recovery = [];
  else if (s.recovery.length < RECOVERY_COUNT) s.recovery.push(e.id);

  if (d.state === 'none') {
    // Base rule (restated by CR13): the observation after which the belief is safe commits.
    if (currentBelief(s) === 'probably_safe') return commit(s, [e.id]);
  } else if (e.kind === 'duskcap') {
    d.state = 'reopened';
    pushBounded(d.reopenedBy, e.id, DECISION_KEEP);
    pushBounded(d.history, { change: 'reopened', because: [e.id] }, DECISION_KEEP);
    return 'reopened';
  } else if (d.state === 'reopened' && s.recovery.length >= RECOVERY_COUNT && s.recovery[RECOVERY_COUNT - 1] === e.id) {
    // CR7: the second glowcap observation since the most recent contradiction recommits.
    return commit(s, s.recovery);
  }
  return null;
}

function rememberedWitness(s: State, evId: unknown): Evidence {
  const e = typeof evId === 'string' && /^witness_/.test(evId) && s.window.includes(evId) ? find(s, evId) : undefined;
  if (e === undefined) throw new Error(`${String(evId)} is not a remembered witness`);
  return e;
}

function journal(s: State, entry: JournalEntry): void {
  pushBounded(s.journal, entry, JOURNAL_KEEP);
  prune(s);
}

function requireMushroom(s: State, id: unknown): MushroomState {
  if (typeof id !== 'string' || !Object.prototype.hasOwnProperty.call(s.mushrooms, id)) {
    throw new Error(`unknown mushroom: ${String(id)}`);
  }
  return s.mushrooms[id];
}

// Applies an event to s in place; throws on rejection (caller discards s).
function apply(s: State, event: Record<string, unknown>): void {
  switch (event.type) {
    case 'absorb': {
      const id = event.id as string;
      const m = requireMushroom(s, event.id);
      const kind = event.kind;
      if (!isKind(kind)) throw new Error(`unknown kind: ${String(kind)}`);
      if (m.consumed) throw new Error(`${id} already consumed`);
      if (!mushroomRow(s, id).canAbsorb) throw new Error(`${id} may not be absorbed`);
      if (m.tasted !== null && m.tasted !== kind) throw new Error(`${id} was tasted as ${m.tasted}`);
      const evId = evidenceId(s, 'absorb', id);
      consume(m, evId);
      if (kind === 'glowcap') s.glow = GLOW_SECONDS;
      // CR8: heaviness stacks while heavy, capped.
      else s.heavy = s.heavy > 0 ? Math.min(HEAVY_MAX_SECONDS, s.heavy + HEAVY_SECONDS) : HEAVY_SECONDS;
      observe(s, { id: evId, kind, caveats: [] });
      return;
    }
    case 'taste': {
      const id = event.id as string;
      const m = requireMushroom(s, event.id);
      const kind = event.kind;
      if (!isKind(kind)) throw new Error(`unknown kind: ${String(kind)}`);
      if (m.consumed) throw new Error(`${id} already consumed`);
      if (m.tasted !== null) throw new Error(`${id} already tasted`);
      if (!mushroomRow(s, id).canTaste) throw new Error(`${id} may not be tasted`);
      m.tasted = kind;
      const caveats = s.glow > 0 ? [] : [TASTED_IN_DARK];
      observe(s, { id: evidenceId(s, 'taste', id), kind, caveats, age: 0 });
      return;
    }
    case 'retaste': {
      const id = event.id as string;
      const m = requireMushroom(s, event.id);
      const kind = event.kind;
      if (!isKind(kind)) throw new Error(`unknown kind: ${String(kind)}`);
      if (!canRetaste(s, id)) throw new Error(`${id} cannot be retasted now`);
      if (m.tasted !== kind) throw new Error(`${id} was tasted as ${String(m.tasted)}`);
      const caveats = s.glow > 0 ? [] : [TASTED_IN_DARK];
      const evId = evidenceId(s, 'retaste', id);
      m.retasted = true;
      observe(s, { id: evId, kind, caveats, age: 0 });
      return;
    }
    case 'witness': {
      // CR9: another creature eats the mushroom; the slime's timers are untouched
      // and its own canAbsorb does not apply.
      const id = event.id as string;
      const m = requireMushroom(s, event.id);
      const kind = event.kind;
      if (!isKind(kind)) throw new Error(`unknown kind: ${String(kind)}`);
      if (m.consumed) throw new Error(`${id} already consumed`);
      if (m.tasted !== null && m.tasted !== kind) throw new Error(`${id} was tasted as ${m.tasted}`);
      const evId = evidenceId(s, 'witness', id);
      consume(m, evId);
      observe(s, { id: evId, kind, caveats: [SECONDHAND] });
      return;
    }
    case 'mark': {
      const m = requireMushroom(s, event.id);
      if (m.consumed) throw new Error(`${String(event.id)} already consumed`);
      if (m.marked) throw new Error(`${String(event.id)} already marked`);
      if (OUT_OF_REACH.includes(event.id as string)) throw new Error(`${String(event.id)} is out of reach`);
      m.marked = true;
      return;
    }
    case 'unmark': {
      const m = requireMushroom(s, event.id);
      if (!m.marked) throw new Error(`${String(event.id)} is not marked`);
      m.marked = false;
      return;
    }
    case 'doubt': {
      // CR18: not an observation; changes only the belief (via the caveat), never the decision.
      const e = rememberedWitness(s, event.evidence);
      if (isDoubted(e)) throw new Error(`${e.id} is already doubted`);
      const othersUndoubted = s.window.some((id) => {
        const other = find(s, id);
        return id !== e.id && other !== undefined && !isDoubted(other);
      });
      if (!othersUndoubted) throw new Error('at least one remembered observation must stay undoubted');
      e.caveats.push(DOUBTED);
      journal(s, { entry: 'doubt', because: [e.id] });
      return;
    }
    case 'undoubt': {
      // CR20: only a remembered doubted witness; a forgotten doubt is permanent.
      const e = rememberedWitness(s, event.evidence);
      if (!isDoubted(e)) throw new Error(`${e.id} is not doubted`);
      e.caveats = e.caveats.filter((c) => c !== DOUBTED);
      journal(s, { entry: 'undoubt', because: [e.id] });
      return;
    }
    case 'tick': {
      const dt = event.dt;
      if (typeof dt !== 'number' || !Number.isFinite(dt) || dt < 0 || dt > 0.1) {
        throw new Error(`dt out of range: ${String(dt)}`);
      }
      s.glow = Math.max(0, s.glow - dt);
      s.heavy = Math.max(0, s.heavy - dt);
      for (const e of s.evidence) {
        if (e.age === undefined) continue;
        e.age += dt;
        if (e.age >= TASTE_FADE_SECONDS) {
          // Fading is permanent, so the clock is no longer needed.
          e.caveats.push(TASTE_FADED);
          delete e.age;
        }
      }
      // CR10: consumed mushrooms regrow as strangers until their lives run out.
      for (const m of Object.values(s.mushrooms)) {
        if (!m.consumed || m.life >= MAX_LIVES) continue;
        m.consumedFor += dt;
        if (m.consumedFor >= REGROW_SECONDS) {
          m.consumed = false;
          m.tasted = null;
          m.life += 1;
          m.consumedFor = 0;
          m.consumedBy = null;
          m.marked = false;
          m.retasted = false;
        }
      }
      prune(s);
      return;
    }
    default:
      throw new Error(`unknown event type: ${String(event.type)}`);
  }
}

// ---- CR12/CR13: compact save format ----
// The save is a compact encoding of State (not a replay log). Every list in State
// is bounded (CR13), so the save is bounded for play of any length. Floats are
// stored as-is; JSON round-trips them exactly.
//
// Evidence id code: '<verb a|t|w|r><level index>[.<life>]'          e.g. 't1.3' = taste_pool_3
// Evidence code:    '<id code without life><kind g|d><flags d=dark f=faded x=doubted>[.<life>]' e.g. 't1gd.3'
//
// { v: 8, g: glow, h: heavy,
//   m: [[consumed 0|1, tasted 'g'|'d'|'', life, consumedFor, consumedBy id code | '', marked 0|1, retasted 0|1], ...in LEVEL order],
//   e: [evidence code, ...], a: [[index in e, age], ...] for unfaded tastes,
//   w: [id code, ...] remembered window, r: [id code, ...] recovery,
//   d: [state 'n'|'c'|'r', basis, reopenedBy, caveats, [['c'|'r', ids], ...history]],
//   j: [['g'|'d'|'c'|'r'|'x'|'u', ids], ...journal] }

const SAVE_VERSION = 8;
const VERBS = { a: 'absorb', t: 'taste', w: 'witness', r: 'retaste' } as const;

interface Saved {
  v: number;
  g: number;
  h: number;
  m: [number, string, number, number, string, number, number][];
  e: string[];
  a: [number, number][];
  w: string[];
  r: string[];
  d: [string, string[], string[], string[], [string, string[]][]];
  j: [string, string[]][];
}

const JOURNAL_CODES: Record<string, Kind | Change | Reconsider> = {
  g: 'glowcap',
  d: 'duskcap',
  c: 'committed',
  r: 'reopened',
  x: 'doubt',
  u: 'undoubt',
};
const JOURNAL_LETTERS: Record<Kind | Change | Reconsider, string> = {
  glowcap: 'g',
  duskcap: 'd',
  committed: 'c',
  reopened: 'r',
  doubt: 'x',
  undoubt: 'u',
};

function parseId(id: string): { verb: string; level: number; life: string } {
  const match = /^(absorb|taste|witness|retaste)_([a-z]+)(?:_(\d+))?$/.exec(id);
  if (!match || !LEVEL.includes(match[2])) throw new Error(`bad evidence id ${id}`);
  return { verb: match[1][0], level: LEVEL.indexOf(match[2]), life: match[3] ? `.${match[3]}` : '' };
}

function idCode(id: string): string {
  const p = parseId(id);
  return `${p.verb}${p.level}${p.life}`;
}

function idFromParts(verb: string, level: string, life: string | undefined): string {
  const v = VERBS[verb as keyof typeof VERBS];
  const name = LEVEL[Number(level)];
  if (v === undefined || name === undefined) throw new Error('bad evidence code');
  return `${v}_${name}${life ? `_${life}` : ''}`;
}

function idFromCode(code: string): string {
  const match = /^([atwr])(\d+)(?:\.(\d+))?$/.exec(code);
  if (!match) throw new Error(`bad evidence id code ${code}`);
  return idFromParts(match[1], match[2], match[3]);
}

function encode(s: State): Saved {
  const a: [number, number][] = [];
  const e = s.evidence.map((ev, i) => {
    const p = parseId(ev.id);
    let flags = '';
    if (ev.caveats.includes(TASTED_IN_DARK)) flags += 'd';
    if (ev.caveats.includes(TASTE_FADED)) flags += 'f';
    if (ev.caveats.includes(DOUBTED)) flags += 'x';
    if (ev.age !== undefined) a.push([i, ev.age]);
    return `${p.verb}${p.level}${ev.kind[0]}${flags}${p.life}`;
  });
  const m = LEVEL.map((id): [number, string, number, number, string, number, number] => {
    const ms = s.mushrooms[id];
    return [
      ms.consumed ? 1 : 0,
      ms.tasted ? ms.tasted[0] : '',
      ms.life,
      ms.consumedFor,
      ms.consumedBy === null ? '' : idCode(ms.consumedBy),
      ms.marked ? 1 : 0,
      ms.retasted ? 1 : 0,
    ];
  });
  const d = s.decision;
  return {
    v: SAVE_VERSION,
    g: s.glow,
    h: s.heavy,
    m,
    e,
    a,
    w: s.window.map(idCode),
    r: s.recovery.map(idCode),
    d: [d.state[0], d.basis.map(idCode), d.reopenedBy.map(idCode), [...d.caveats], d.history.map((h): [string, string[]] => [h.change[0], h.because.map(idCode)])],
    j: s.journal.map((x): [string, string[]] => [JOURNAL_LETTERS[x.entry], x.because.map(idCode)]),
  };
}

function decode(raw: unknown): State {
  const sv = raw as Saved;
  if (typeof sv !== 'object' || sv === null || sv.v !== SAVE_VERSION) throw new Error('unrecognised save');
  const kindOf = (c: string): Kind => (c === 'g' ? 'glowcap' : 'duskcap');
  const evidence: Evidence[] = sv.e.map((code) => {
    const match = /^([atwr])(\d+)([gd])([dfx]*)(?:\.(\d+))?$/.exec(code);
    if (!match) throw new Error(`bad evidence code ${code}`);
    const caveats: string[] = [];
    if (match[1] === 'w') caveats.push(SECONDHAND);
    if (match[4].includes('d')) caveats.push(TASTED_IN_DARK);
    if (match[4].includes('f')) caveats.push(TASTE_FADED);
    if (match[4].includes('x')) caveats.push(DOUBTED);
    return { id: idFromParts(match[1], match[2], match[5]), kind: kindOf(match[3]), caveats };
  });
  for (const [i, age] of sv.a) evidence[i].age = age;
  const mushrooms: Record<string, MushroomState> = {};
  LEVEL.forEach((name, i) => {
    const [consumed, tasted, life, consumedFor, by, marked, retasted] = sv.m[i];
    mushrooms[name] = {
      consumed: consumed === 1,
      tasted: tasted === '' ? null : kindOf(tasted),
      life,
      consumedFor,
      consumedBy: by === '' ? null : idFromCode(by),
      marked: marked === 1,
      retasted: retasted === 1,
    };
  });
  const [st, basis, reopenedBy, caveats, history] = sv.d;
  const states = { n: 'none', c: 'committed', r: 'reopened' } as const;
  return {
    glow: sv.g,
    heavy: sv.h,
    mushrooms,
    evidence,
    window: sv.w.map(idFromCode),
    recovery: sv.r.map(idFromCode),
    decision: {
      state: states[st as keyof typeof states],
      basis: basis.map(idFromCode),
      reopenedBy: reopenedBy.map(idFromCode),
      caveats: [...caveats],
      history: history.map(([c, because]) => ({
        change: c === 'c' ? ('committed' as const) : ('reopened' as const),
        because: because.map(idFromCode),
      })),
    },
    journal: sv.j.map(([c, because]) => {
      const entry = JOURNAL_CODES[c];
      if (entry === undefined) throw new Error(`bad journal code ${c}`);
      return { entry, because: because.map(idFromCode) };
    }),
  };
}

export function createPolicy(saved?: unknown) {
  let state: State = saved === undefined ? initialState() : decode(saved);
  return {
    dispatch(event: unknown): void {
      if (typeof event !== 'object' || event === null) throw new Error('event must be an object');
      const next = clone(state);
      apply(next, event as Record<string, unknown>);
      state = next;
    },
    view() {
      return viewOf(state);
    },
    save(): unknown {
      return encode(state);
    },
  };
}
