// Glowcap or duskcap, written the way Vessel would write it in TypeScript.
// See ../PROTOCOL.md for the behaviour this implements.

type Kind = 'glowcap' | 'duskcap';
type BeliefState = 'none' | 'probably_safe' | 'probably_unsafe' | 'uncertain';

const MUSHROOMS = ['cave', 'pool', 'ruin'] as const;
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

  function mushroom(id: string): Mushroom {
    const found = mushrooms.get(id);
    if (!found) throw new Error(`Unknown mushroom ${id}`);
    return found;
  }

  function beliefState(): BeliefState {
    if (supportedBy.length && contradictedBy.length) return 'uncertain';
    if (supportedBy.length) return 'probably_safe';
    if (contradictedBy.length) return 'probably_unsafe';
    return 'none';
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
        target.consumed = true;
        const evidence = `absorb_${event.id}`;
        if (kind === 'glowcap') {
          glow = GLOW_SECONDS;
          supportedBy.push(evidence);
          if (!trust && beliefState() === 'probably_safe') trust = { basis: [evidence], reopenedBy: [] };
        } else {
          heavy = HEAVY_SECONDS;
          contradictedBy.push(evidence);
          trust?.reopenedBy.push(evidence);
        }
        return;
      }
      case 'taste': {
        const target = mushroom(event.id);
        const kind = kindOf(event.kind);
        if (target.consumed) throw new Error(`${event.id} is already consumed`);
        if (target.tasted) throw new Error(`${event.id} was already tasted`);
        target.tasted = kind;
        return;
      }
      default:
        throw new Error(`Unknown event ${(event as { type: unknown }).type}`);
    }
  }

  function mushroomView(id: string, belief: BeliefState) {
    const { consumed, tasted } = mushroom(id);
    if (consumed) return { present: false, label: '', canAbsorb: false, canTaste: false, because: [] as string[] };
    if (tasted === 'glowcap') return { present: true, label: 'Glowcap', canAbsorb: true, canTaste: false, because: [`taste_${id}`] };
    if (tasted === 'duskcap') return { present: true, label: 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: [`taste_${id}`] };
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
      },
      decision: trust
        ? { state: trust.reopenedBy.length ? 'reopened' : 'committed', basis: [...trust.basis], reopenedBy: [...trust.reopenedBy] }
        : { state: 'none', basis: [] as string[], reopenedBy: [] as string[] },
    };
  }

  return { dispatch, view };
}
