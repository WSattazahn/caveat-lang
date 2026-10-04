// Glowcap beat policy.
import { decodeSave, encodeSave } from './save.ts';

export type Kind = 'glowcap' | 'duskcap';

const KINDS: readonly Kind[] = ['glowcap', 'duskcap'];
export const LEVEL: readonly string[] = ['cave', 'pool', 'ruin', 'grove', 'pit'];
/** CR15: mushrooms only other creatures can reach (witness only). */
const OUT_OF_REACH: readonly string[] = ['pit'];
const GLOW_SECONDS = 30;
const HEAVY_SECONDS = 20;
const HEAVY_MAX_SECONDS = 30;
const TASTE_FADE_SECONDS = 60;
const REGROW_SECONDS = 45;
/** CR10: declared capacity — lives per mushroom (the first plus 63 regrowths). */
export const LIVES_PER_MUSHROOM = 64;
/** CR13: the slime remembers this many observations; reopenedBy and history are bounded alike. */
export const MEMORY = 6;

export interface Evidence {
  id: string; // absorb_<id>[_<life>], taste_<id>[_<life>] or witness_<id>[_<life>]
  kind: Kind;
  caveats: string[];
  age?: number; // tastes only: seconds of ticks since the taste
}

// One spot in the level; each life is a new mushroom in that spot.
export interface Slot {
  life: number; // 1-based
  consumed: boolean;
  sinceConsumed: number; // seconds of ticks since consumed (CR10)
  taste?: string; // evidence id of this life's taste
  consumedBy?: string; // evidence id that consumed this life (CR11)
  marked?: boolean; // CR14: player mark (not evidence)
}

export interface State {
  ids: string[];
  slots: Record<string, Slot>;
  // Retained observations in acceptance order: those remembered (window) or
  // still cited by a mushroom's current life (its taste, what consumed it).
  evidence: Evidence[];
  // CR13: ids of the MEMORY most recent observations, oldest first.
  window: string[];
  glow: number;
  heavy: number;
  // CR7: glowcap observations since the most recent contradiction.
  sinceContradiction: string[];
  // caveats are frozen at commit time (CR5).
  decision: { state: 'none' | 'committed' | 'reopened'; basis: string[]; reopenedBy: string[]; caveats: string[]; history: { change: 'committed' | 'reopened'; because: string[] }[] };
  // CR16: at most MEMORY entries, oldest first; caveats are computed on view.
  journal: JournalEntry[];
}

export interface JournalEntry {
  text: string;
  because: string[];
}

export interface GlowEvent {
  type: string;
  [key: string]: unknown;
}

function initialState(): State {
  return {
    ids: [...LEVEL],
    slots: Object.fromEntries(LEVEL.map((id) => [id, { life: 1, consumed: false, sinceConsumed: 0 }])),
    evidence: [],
    window: [],
    glow: 0,
    heavy: 0,
    sinceContradiction: [],
    decision: { state: 'none', basis: [], reopenedBy: [], caveats: [], history: [] },
    journal: [],
  };
}

function clone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v)) as T;
}

function fail(msg: string): never {
  throw new Error(msg);
}

function evidenceId(s: State, verb: string, id: string): string {
  const life = s.slots[id].life;
  return life === 1 ? `${verb}_${id}` : `${verb}_${id}_${life}`;
}

function tastedKind(s: State, id: string): Kind | undefined {
  const t = s.slots[id].taste;
  return t === undefined ? undefined : s.evidence.find((e) => e.id === t)?.kind;
}

function consume(s: State, id: string, by: string): void {
  s.slots[id].consumed = true;
  s.slots[id].consumedBy = by;
  delete s.slots[id].marked;
  s.slots[id].sinceConsumed = 0;
}

function caveatsOf(s: State, ids: readonly string[]): string[] {
  const out: string[] = [];
  for (const e of s.evidence) {
    if (!ids.includes(e.id)) continue;
    for (const c of e.caveats) if (!out.includes(c)) out.push(c);
  }
  return out;
}

function beliefOf(s: State) {
  const remembered = s.evidence.filter((a) => s.window.includes(a.id));
  const supportedBy = remembered.filter((a) => a.kind === 'glowcap').map((a) => a.id);
  const contradictedBy = remembered.filter((a) => a.kind === 'duskcap').map((a) => a.id);
  const n = (k: number) => (k === 1 ? 'Based on one observation.' : `Based on ${k} observations.`);
  let state: 'none' | 'probably_safe' | 'probably_unsafe' | 'uncertain';
  let text = '';
  let note = '';
  if (supportedBy.length === 0 && contradictedBy.length === 0) state = 'none';
  else if (contradictedBy.length === 0) {
    state = 'probably_safe';
    text = 'Glowing mushrooms give you light.';
    note = n(supportedBy.length);
  } else if (supportedBy.length === 0) {
    state = 'probably_unsafe';
    text = 'Glowing mushrooms make you heavy.';
    note = n(contradictedBy.length);
  } else {
    state = 'uncertain';
    text = 'Not every glowing mushroom is safe. Taste before absorbing.';
  }
  return { state, text, note, supportedBy, contradictedBy, caveats: caveatsOf(s, [...supportedBy, ...contradictedBy]) };
}

interface WhyNot {
  reason: string;
  because: string[];
  caveats: string[];
}

interface MushroomView {
  present: boolean;
  label: string;
  canAbsorb: boolean;
  canTaste: boolean;
  because: string[];
  caveats: string[];
  marked: boolean;
  why: { absorb: WhyNot; taste: WhyNot };
}

function mushroomRow(s: State, id: string, belief: ReturnType<typeof beliefOf>): Omit<MushroomView, 'caveats' | 'why' | 'marked'> {
  const slot = s.slots[id];
  if (slot.consumed) return { present: false, label: '', canAbsorb: false, canTaste: false, because: [] };
  const t = tastedKind(s, id);
  if (t !== undefined && slot.taste !== undefined) {
    const tasteId = slot.taste;
    const cav = caveatsOf(s, [tasteId]);
    const qualifier = cav.includes('taste_faded') ? ' (taste has faded)' : cav.includes('tasted_in_dark') ? ' (tasted in the dark)' : '';
    if (t === 'glowcap')
      return { present: true, label: qualifier ? `Probably a glowcap${qualifier}` : 'Glowcap', canAbsorb: true, canTaste: false, because: [tasteId] };
    return { present: true, label: qualifier ? `Probably a duskcap${qualifier}` : 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: [tasteId] };
  }
  if (belief.contradictedBy.length >= 2)
    return { present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: [...belief.contradictedBy] };
  switch (belief.state) {
    case 'none':
      return { present: true, label: 'Glowing mushroom', canAbsorb: true, canTaste: true, because: [] };
    case 'probably_safe':
      return { present: true, label: 'Probably a glowcap', canAbsorb: true, canTaste: true, because: [...belief.supportedBy] };
    case 'probably_unsafe':
      return { present: true, label: 'Probably a duskcap', canAbsorb: true, canTaste: true, because: [...belief.contradictedBy] };
    default:
      return {
        present: true,
        label: 'Could be a duskcap — taste first',
        canAbsorb: true,
        canTaste: true,
        because: [...belief.contradictedBy], // CR4: cite the counterexample only
      };
  }
}

// CR11: why an action is unavailable (first match), or empty when allowed.
function whyNot(s: State, id: string, belief: ReturnType<typeof beliefOf>) {
  const slot = s.slots[id];
  const make = (reason: string, because: string[]): WhyNot => ({ reason, because, caveats: caveatsOf(s, because) });
  const allowed = make('', []);
  if (slot.consumed) {
    const by = slot.consumedBy === undefined ? [] : [slot.consumedBy];
    return { absorb: make('Already eaten', by), taste: make('Already eaten', by) };
  }
  if (OUT_OF_REACH.includes(id)) return { absorb: make('Out of reach', []), taste: make('Out of reach', []) };
  const t = tastedKind(s, id);
  if (slot.taste !== undefined) {
    const absorb = t === 'duskcap' ? make('Known duskcap', [slot.taste]) : slot.marked ? make('Marked to avoid', []) : allowed;
    return { absorb, taste: make('Already tasted', [slot.taste]) };
  }
  const absorb = slot.marked
    ? make('Marked to avoid', [])
    : belief.contradictedBy.length >= 2
      ? make('Too risky untasted', [...belief.contradictedBy])
      : allowed;
  return { absorb, taste: make('', []) };
}

function mushroomView(s: State, id: string, belief: ReturnType<typeof beliefOf>): MushroomView {
  const row = mushroomRow(s, id, belief);
  const marked = s.slots[id].marked === true;
  return {
    ...row,
    canAbsorb: row.canAbsorb && !marked && !OUT_OF_REACH.includes(id),
    canTaste: row.canTaste && !OUT_OF_REACH.includes(id),
    caveats: caveatsOf(s, row.because),
    marked,
    why: whyNot(s, id, belief),
  };
}

function render(s: State) {
  const belief = beliefOf(s);
  const mushrooms: Record<string, MushroomView> = {};
  for (const id of s.ids) mushrooms[id] = mushroomView(s, id, belief);
  return {
    slime: { glowing: s.glow > 0, heavy: s.heavy > 0, heavySeconds: s.heavy > 0 ? Math.ceil(s.heavy) : 0 },
    mushrooms,
    belief,
    decision: clone(s.decision),
    journal: s.journal.map((j) => ({ text: j.text, because: [...j.because], caveats: caveatsOf(s, j.because) })),
  };
}

function parseKind(v: unknown): Kind {
  if (typeof v !== 'string' || !(KINDS as readonly string[]).includes(v)) fail(`unknown kind ${String(v)}`);
  return v as Kind;
}

function parseId(s: State, v: unknown): string {
  if (typeof v !== 'string' || !s.ids.includes(v)) fail(`unknown id ${String(v)}`);
  return v;
}

function pushBounded<T>(list: T[], item: T): void {
  list.push(item);
  while (list.length > MEMORY) list.shift();
}

const OBSERVATION_TEXT: Record<string, Record<Kind, string>> = {
  absorb: { glowcap: 'Absorbed the # mushroom: it gave light.', duskcap: 'Absorbed the # mushroom: it made me heavy.' },
  taste: { glowcap: 'Tasted the # mushroom: sweet.', duskcap: 'Tasted the # mushroom: bitter.' },
  witness: { glowcap: 'Saw a creature eat the # mushroom: it glowed.', duskcap: 'Saw a creature eat the # mushroom: it grew heavy.' },
};

export function observationText(ev: Evidence): string {
  const [verb, spot] = ev.id.split('_');
  return OBSERVATION_TEXT[verb][ev.kind].replace('#', spot);
}

export const TRUST_TEXT = { committed: 'I trust glowing mushrooms now.', reopened: 'I no longer trust glowing mushrooms.' };

// CR16: journal the decision change just appended to history.
function journalTrust(s: State): void {
  const h = s.decision.history[s.decision.history.length - 1];
  pushBounded(s.journal, { text: TRUST_TEXT[h.change], because: [...h.because] });
}

// Adds one accepted observation (forgetting the oldest beyond MEMORY) and
// updates the trust decision.
function record(s: State, ev: Evidence): void {
  s.evidence.push(ev);
  pushBounded(s.journal, { text: observationText(ev), because: [ev.id] });
  pushBounded(s.window, ev.id);
  const after = beliefOf(s);
  const d = s.decision;
  // CR7 count: accepted glowcap observations since the latest contradiction
  // (only the first two matter).
  if (ev.kind === 'duskcap') s.sinceContradiction = [];
  else if (s.sinceContradiction.length < 2) s.sinceContradiction.push(ev.id);
  if (d.state === 'none') {
    if (after.state === 'probably_safe') {
      d.state = 'committed';
      d.basis = [ev.id];
      d.reopenedBy = [];
      d.caveats = [...ev.caveats];
      pushBounded(d.history, { change: 'committed', because: [ev.id] });
      journalTrust(s);
    }
  } else if (ev.kind === 'duskcap') {
    d.state = 'reopened';
    pushBounded(d.reopenedBy, ev.id);
    pushBounded(d.history, { change: 'reopened', because: [ev.id] });
    journalTrust(s);
  } else if (d.state === 'reopened' && s.sinceContradiction.length >= 2) {
    const basis = s.sinceContradiction.slice(0, 2);
    d.state = 'committed';
    d.basis = basis;
    d.reopenedBy = [];
    d.caveats = caveatsOf(s, basis);
    pushBounded(d.history, { change: 'committed', because: [...basis] });
    journalTrust(s);
  }
}

// Drops evidence nothing can cite any more, so state stays bounded.
function collect(s: State): void {
  const keep = new Set(s.window);
  for (const j of s.journal) for (const id of j.because) keep.add(id);
  for (const id of s.ids) {
    const slot = s.slots[id];
    if (slot.taste !== undefined) keep.add(slot.taste);
    if (slot.consumedBy !== undefined) keep.add(slot.consumedBy);
  }
  s.evidence = s.evidence.filter((e) => keep.has(e.id));
}

function apply(s: State, e: GlowEvent): void {
  switch (e.type) {
    case 'absorb': {
      const id = parseId(s, e.id);
      const kind = parseKind(e.kind);
      if (s.slots[id].consumed) fail(`${id} already consumed`);
      const view = mushroomView(s, id, beliefOf(s));
      if (!view.canAbsorb) fail(`${id} may not be absorbed`);
      const tasted = tastedKind(s, id);
      if (tasted !== undefined && tasted !== kind) fail(`${id} kind contradicts taste`);
      const evId = evidenceId(s, 'absorb', id);
      record(s, { id: evId, kind, caveats: [] });
      consume(s, id, evId);
      if (kind === 'glowcap') s.glow = GLOW_SECONDS;
      else s.heavy = s.heavy > 0 ? Math.min(HEAVY_MAX_SECONDS, s.heavy + HEAVY_SECONDS) : HEAVY_SECONDS;
      return;
    }
    case 'witness': {
      // CR9: another creature eats it; consumed, secondhand evidence, no timers.
      const id = parseId(s, e.id);
      const kind = parseKind(e.kind);
      if (s.slots[id].consumed) fail(`${id} already consumed`);
      const tasted = tastedKind(s, id);
      if (tasted !== undefined && tasted !== kind) fail(`${id} kind contradicts taste`);
      const evId = evidenceId(s, 'witness', id);
      record(s, { id: evId, kind, caveats: ['secondhand'] });
      consume(s, id, evId);
      return;
    }
    case 'mark': {
      const id = parseId(s, e.id);
      if (OUT_OF_REACH.includes(id)) fail(`${id} is out of reach`);
      if (s.slots[id].consumed) fail(`${id} already consumed`);
      if (s.slots[id].marked) fail(`${id} already marked`);
      s.slots[id].marked = true;
      return;
    }
    case 'unmark': {
      const id = parseId(s, e.id);
      if (!s.slots[id].marked) fail(`${id} not marked`);
      delete s.slots[id].marked;
      return;
    }
    case 'taste': {
      const id = parseId(s, e.id);
      const kind = parseKind(e.kind);
      if (s.slots[id].consumed) fail(`${id} already consumed`);
      if (s.slots[id].taste !== undefined) fail(`${id} already tasted`);
      if (OUT_OF_REACH.includes(id)) fail(`${id} is out of reach`);
      const evId = evidenceId(s, 'taste', id);
      s.slots[id].taste = evId;
      record(s, { id: evId, kind, caveats: s.glow > 0 ? [] : ['tasted_in_dark'], age: 0 });
      return;
    }
    case 'tick': {
      const dt = e.dt;
      if (typeof dt !== 'number' || !Number.isFinite(dt) || dt < 0 || dt > 0.1) fail(`dt out of range`);
      s.glow = Math.max(0, s.glow - dt);
      s.heavy = Math.max(0, s.heavy - dt);
      for (const ev of s.evidence) {
        if (ev.age === undefined) continue;
        ev.age += dt;
        if (ev.age >= TASTE_FADE_SECONDS) {
          if (!ev.caveats.includes('taste_faded')) ev.caveats.push('taste_faded');
          delete ev.age; // faded for good; no clock needed any more
        }
      }
      for (const id of s.ids) {
        const slot = s.slots[id];
        if (!slot.consumed || slot.life >= LIVES_PER_MUSHROOM) continue;
        slot.sinceConsumed += dt;
        if (slot.sinceConsumed >= REGROW_SECONDS) s.slots[id] = { life: slot.life + 1, consumed: false, sinceConsumed: 0 };
      }
      return;
    }
    default:
      fail(`unknown event type ${String(e.type)}`);
  }
}

export function createPolicy(saved?: unknown) {
  let state: State = saved === undefined ? initialState() : decodeSave(saved);
  return {
    dispatch(event: GlowEvent): void {
      const next = clone(state);
      apply(next, event);
      collect(next);
      state = next;
    },
    view() {
      return render(state);
    },
    save(): unknown {
      return encodeSave(state);
    },
  };
}
