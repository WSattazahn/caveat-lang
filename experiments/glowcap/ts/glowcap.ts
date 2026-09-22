// Glowcap or duskcap, written the way Vessel would write it in TypeScript.
// See ../PROTOCOL.md for the behaviour this implements.

type Kind = 'glowcap' | 'duskcap';
type BeliefState = 'none' | 'probably_safe' | 'probably_unsafe' | 'uncertain';

const MUSHROOMS = ['cave', 'pool', 'ruin', 'grove'] as const;
const GLOW_SECONDS = 30;
const HEAVY_SECONDS = 20;

export type GlowcapEvent =
  | { type: 'absorb'; id: string; kind: string }
  | { type: 'taste'; id: string; kind: string }
  | { type: 'tick'; dt: number };

interface Mushroom {
  consumed: boolean;
  tasted: Kind | null;
}

const BELIEF_TEXT: Record<BeliefState, string> = {
  none: '',
  probably_safe: 'Glowing mushrooms give you light.',
  probably_unsafe: 'Glowing mushrooms make you heavy.',
  uncertain: 'Not every glowing mushroom is safe. Taste before absorbing.',
};

function kindOf(kind: string): Kind {
  if (kind !== 'glowcap' && kind !== 'duskcap') throw new Error(`Unknown mushroom kind ${kind}`);
  return kind;
}

function basedOn(count: number): string {
  return count === 1 ? 'Based on one observation.' : `Based on ${count} observations.`;
}

export function createPolicy() {
  const mushrooms = new Map<string, Mushroom>(MUSHROOMS.map((id) => [id, { consumed: false, tasted: null }]));
  let glow = 0;
  let heavy = 0;
  const supportedBy: string[] = [];
  const contradictedBy: string[] = [];
  let trust: { basis: string[]; reopenedBy: string[] } | null = null;
  // Caveats attached to each piece of evidence; anything citing it inherits them.
  const caveatsOf = new Map<string, string[]>();

  function caveatsFor(evidence: string[]): string[] {
    return [...new Set(evidence.flatMap((id) => caveatsOf.get(id) ?? []))];
  }

  function mushroom(id: string): Mushroom {
    const found = mushrooms.get(id);
    if (!found) throw new Error(`Unknown mushroom ${id}`);
    return found;
  }

  // Twice bitten: after two contradictions, only a mushroom known to be a
  // glowcap may be absorbed.
  function tooRisky(target: Mushroom): boolean {
    return target.tasted === null && contradictedBy.length >= 2;
  }

  function beliefState(): BeliefState {
    if (supportedBy.length && contradictedBy.length) return 'uncertain';
    if (supportedBy.length) return 'probably_safe';
    if (contradictedBy.length) return 'probably_unsafe';
    return 'none';
  }

  // Absorptions and tastes both bear on the belief and the trust decision.
  function learn(evidence: string, kind: Kind): void {
    if (kind === 'glowcap') {
      supportedBy.push(evidence);
      if (!trust && beliefState() === 'probably_safe') trust = { basis: [evidence], reopenedBy: [] };
    } else {
      contradictedBy.push(evidence);
      trust?.reopenedBy.push(evidence);
    }
  }

  // Every check runs before the first write, so a rejected event changes nothing.
  function dispatch(event: GlowcapEvent): void {
    switch (event.type) {
      case 'tick': {
        if (!(typeof event.dt === 'number' && event.dt >= 0 && event.dt <= 0.1)) throw new Error(`dt out of range: ${event.dt}`);
        glow = Math.max(0, glow - event.dt);
        heavy = Math.max(0, heavy - event.dt);
        return;
      }
      case 'absorb': {
        const target = mushroom(event.id);
        const kind = kindOf(event.kind);
        if (target.consumed) throw new Error(`${event.id} is already consumed`);
        if (target.tasted === 'duskcap') throw new Error(`${event.id} is a known duskcap`);
        if (target.tasted && target.tasted !== kind) throw new Error(`${event.id} tasted as ${target.tasted}`);
        if (tooRisky(target)) throw new Error(`${event.id} is too risky to absorb untasted`);
        target.consumed = true;
        if (kind === 'glowcap') glow = GLOW_SECONDS;
        else heavy = HEAVY_SECONDS;
        learn(`absorb_${event.id}`, kind);
        return;
      }
      case 'taste': {
        const target = mushroom(event.id);
        const kind = kindOf(event.kind);
        if (target.consumed) throw new Error(`${event.id} is already consumed`);
        if (target.tasted) throw new Error(`${event.id} was already tasted`);
        target.tasted = kind;
        const evidence = `taste_${event.id}`;
        if (glow <= 0) caveatsOf.set(evidence, ['tasted_in_dark']);
        learn(evidence, kind);
        return;
      }
      default:
        throw new Error(`Unknown event ${(event as { type: unknown }).type}`);
    }
  }

  function mushroomView(id: string, belief: BeliefState) {
    const base = labelFor(id, belief);
    return { ...base, caveats: caveatsFor(base.because) };
  }

  function labelFor(id: string, belief: BeliefState) {
    const { consumed, tasted } = mushroom(id);
    if (consumed) return { present: false, label: '', canAbsorb: false, canTaste: false, because: [] as string[] };
    if (tasted) {
      const taste = `taste_${id}`;
      const dark = caveatsOf.get(taste)?.includes('tasted_in_dark') ?? false;
      const label = tasted === 'glowcap'
        ? dark ? 'Probably a glowcap (tasted in the dark)' : 'Glowcap'
        : dark ? 'Probably a duskcap (tasted in the dark)' : 'Duskcap — avoid';
      return { present: true, label, canAbsorb: tasted === 'glowcap', canTaste: false, because: [taste] };
    }
    if (tooRisky(mushroom(id))) return { present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: [...contradictedBy] };
    const guess: Record<BeliefState, [string, string[]]> = {
      none: ['Glowing mushroom', []],
      probably_safe: ['Probably a glowcap', supportedBy],
      probably_unsafe: ['Probably a duskcap', contradictedBy],
      uncertain: ['Could be a duskcap — taste first', [...supportedBy, ...contradictedBy]],
    };
    const [label, because] = guess[belief];
    return { present: true, label, canAbsorb: true, canTaste: true, because: [...because] };
  }

  function view() {
    const state = beliefState();
    return {
      slime: { glowing: glow > 0, heavy: heavy > 0 },
      mushrooms: Object.fromEntries(MUSHROOMS.map((id) => [id, mushroomView(id, state)])),
      belief: {
        state,
        text: BELIEF_TEXT[state],
        note: state === 'probably_safe' ? basedOn(supportedBy.length) : state === 'probably_unsafe' ? basedOn(contradictedBy.length) : '',
        supportedBy: [...supportedBy],
        contradictedBy: [...contradictedBy],
        caveats: caveatsFor([...supportedBy, ...contradictedBy]),
      },
      decision: trust
        ? { state: trust.reopenedBy.length ? 'reopened' : 'committed', basis: [...trust.basis], reopenedBy: [...trust.reopenedBy], caveats: caveatsFor(trust.basis) }
        : { state: 'none', basis: [] as string[], reopenedBy: [] as string[], caveats: [] as string[] },
    };
  }

  return { dispatch, view };
}
