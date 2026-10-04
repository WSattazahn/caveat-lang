// CR12/CR13: compact save format for the Glowcap policy.
//
// An evidence reference is a short code <verb><spot>.<life>; a retained
// evidence entry adds its kind and one-letter caveat flags. Only unfaded
// tastes keep a clock. State is bounded (CR13), so the save is too.
// Numbers round-trip exactly through JSON.
import type { Evidence, JournalEntry, Kind, Slot, State } from './glowcap.ts';
import { LEVEL, observationText, TRUST_TEXT } from './glowcap.ts';

const VERBS = ['absorb', 'taste', 'witness'];
const CAVEAT_FLAGS: Record<string, string> = { tasted_in_dark: 'd', taste_faded: 'f', secondhand: 's' };
const DECISION_STATES = ['none', 'committed', 'reopened'] as const;

interface Saved {
  v: 4;
  g: number; // glow timer
  h: number; // heavy timer
  e: [string, string, number?][]; // retained evidence: [ref, kind+flags, age of an unfaded taste]
  w: string[]; // remembered window
  s: [number, number, number, string, string, number][]; // per spot: life, consumed, sinceConsumed, taste ref, consumedBy ref ('' = none), marked
  r: string[]; // CR7 count
  d: [number, string[], string[], string, [number, string[]][]]; // state, basis, reopenedBy, caveat flags, history
  j: [number, string[]][]; // journal: 0 observation, 1 committed, 2 reopened; cited refs
}

function ref(id: string): string {
  const [verb, spot, life] = id.split('_');
  return `${VERBS.indexOf(verb)}${LEVEL.indexOf(spot)}.${life ?? 1}`;
}

function unref(code: string): string {
  const m = /^(\d)(\d+)\.(\d+)$/.exec(code);
  if (m === null) throw new Error(`bad save: reference ${code}`);
  const verb = VERBS[Number(m[1])];
  const spot = LEVEL[Number(m[2])];
  if (verb === undefined || spot === undefined) throw new Error(`bad save: reference ${code}`);
  return m[3] === '1' ? `${verb}_${spot}` : `${verb}_${spot}_${m[3]}`;
}

function flagsOf(caveats: readonly string[]): string {
  return caveats.map((c) => CAVEAT_FLAGS[c] ?? '').join('');
}

function caveatsFromFlags(flags: string): string[] {
  const out: string[] = [];
  for (const ch of flags) {
    const name = Object.keys(CAVEAT_FLAGS).find((k) => CAVEAT_FLAGS[k] === ch);
    if (name === undefined) throw new Error(`bad save: caveat flag ${ch}`);
    out.push(name);
  }
  return out;
}

export function encodeSave(st: State): Saved {
  const d = st.decision;
  return {
    v: 4,
    g: st.glow,
    h: st.heavy,
    e: st.evidence.map((ev): [string, string, number?] => {
      const kf = `${ev.kind === 'glowcap' ? 'g' : 'd'}${flagsOf(ev.caveats)}`;
      return ev.age === undefined ? [ref(ev.id), kf] : [ref(ev.id), kf, ev.age];
    }),
    w: st.window.map(ref),
    s: LEVEL.map((id): [number, number, number, string, string, number] => {
      const sl = st.slots[id];
      return [
        sl.life,
        sl.consumed ? 1 : 0,
        sl.sinceConsumed,
        sl.taste === undefined ? '' : ref(sl.taste),
        sl.consumedBy === undefined ? '' : ref(sl.consumedBy),
        sl.marked ? 1 : 0,
      ];
    }),
    r: st.sinceContradiction.map(ref),
    d: [
      DECISION_STATES.indexOf(d.state),
      d.basis.map(ref),
      d.reopenedBy.map(ref),
      flagsOf(d.caveats),
      d.history.map((h): [number, string[]] => [h.change === 'committed' ? 1 : 2, h.because.map(ref)]),
    ],
    j: st.journal.map((e): [number, string[]] => [
      e.text === TRUST_TEXT.committed ? 1 : e.text === TRUST_TEXT.reopened ? 2 : 0,
      e.because.map(ref),
    ]),
  };
}

export function decodeSave(raw: unknown): State {
  const sv = raw as Saved;
  if (sv === null || typeof sv !== 'object' || sv.v !== 4) throw new Error('bad save');
  const evidence: Evidence[] = sv.e.map(([code, kf, age]) => {
    const kind: Kind = kf[0] === 'g' ? 'glowcap' : 'duskcap';
    const ev: Evidence = { id: unref(code), kind, caveats: caveatsFromFlags(kf.slice(1)) };
    if (typeof age === 'number') ev.age = age;
    return ev;
  });
  const slots: Record<string, Slot> = {};
  LEVEL.forEach((spot, k) => {
    const [life, consumed, sinceConsumed, taste, consumedBy, marked] = sv.s[k];
    const slot: Slot = { life, consumed: consumed === 1, sinceConsumed };
    if (taste !== '') slot.taste = unref(taste);
    if (consumedBy !== '') slot.consumedBy = unref(consumedBy);
    if (marked === 1) slot.marked = true;
    slots[spot] = slot;
  });
  const [ds, basis, reopenedBy, flags, history] = sv.d;
  const journal: JournalEntry[] = sv.j.map(([code, refs]) => {
    const because = refs.map(unref);
    if (code === 1) return { text: TRUST_TEXT.committed, because };
    if (code === 2) return { text: TRUST_TEXT.reopened, because };
    const ev = evidence.find((x) => x.id === because[0]);
    if (ev === undefined) throw new Error('bad save: journal cites unknown evidence');
    return { text: observationText(ev), because };
  });
  return {
    ids: [...LEVEL],
    slots,
    evidence,
    window: sv.w.map(unref),
    glow: sv.g,
    heavy: sv.h,
    sinceContradiction: sv.r.map(unref),
    decision: {
      state: DECISION_STATES[ds],
      basis: basis.map(unref),
      reopenedBy: reopenedBy.map(unref),
      caveats: caveatsFromFlags(flags),
      history: history.map(([c, b]) => ({ change: c === 1 ? ('committed' as const) : ('reopened' as const), because: b.map(unref) })),
    },
    journal,
  };
}
