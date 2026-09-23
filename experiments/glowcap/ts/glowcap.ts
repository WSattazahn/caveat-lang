// Glowcap or duskcap, written the way Vessel would write it in TypeScript.
// See ../PROTOCOL.md for the behaviour this implements.

type Kind = 'glowcap' | 'duskcap';
type BeliefState = 'none' | 'probably_safe' | 'probably_unsafe' | 'uncertain';

const MUSHROOMS = ['cave', 'pool', 'ruin', 'grove'] as const;
const GLOW_SECONDS = 30;
const HEAVY_SECONDS = 20;
const HEAVY_MAX_SECONDS = 30;
const TASTE_FADES_AFTER = 60;
const REGROWS_AFTER = 45;

export type GlowcapEvent =
  | { type: 'absorb'; id: string; kind: string }
  | { type: 'taste'; id: string; kind: string }
  | { type: 'witness'; id: string; kind: string }
  | { type: 'tick'; dt: number };

// A mushroom that regrows is a new one: `life` numbers its evidence.
interface Mushroom {
  consumed: boolean;
  tasted: Kind | null;
  life: number;
  consumedAt: number;
  consumedBy: string;
}

interface Trust { basis: string[]; reopenedBy: string[]; caveats: string[] }
interface Change { change: 'committed' | 'reopened'; because: string[] }

// Everything a policy knows, as JSON: save() writes it, createPolicy(saved) resumes.
export interface Saved {
  mushrooms: [string, Mushroom][];
  glow: number;
  heavy: number;
  now: number;
  supportedBy: string[];
  contradictedBy: string[];
  trust: Trust | null;
  history: Change[];
  sinceContradiction: string[];
  caveatsOf: [string, string[]][];
  tastedAt: [string, number][];
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

export function createPolicy(saved?: Saved) {
  const mushrooms = new Map<string, Mushroom>(MUSHROOMS.map((id) => [id, { consumed: false, tasted: null, life: 1, consumedAt: 0, consumedBy: '' }]));
  let glow = 0;
  let heavy = 0;
  const supportedBy: string[] = [];
  const contradictedBy: string[] = [];
  let trust: Trust | null = null;
  const history: Change[] = [];
  // Glowcap observations since the most recent contradiction; two recommit trust.
  let sinceContradiction: string[] = [];
  // Caveats attached to each piece of evidence; anything citing it inherits them.
  const caveatsOf = new Map<string, string[]>();
  let now = 0;
  const tastedAt = new Map<string, number>();

  if (saved) {
    const copy = structuredClone(saved);
    for (const [id, state] of copy.mushrooms) mushrooms.set(id, state);
    ({ glow, heavy, now, trust, sinceContradiction } = copy);
    supportedBy.push(...copy.supportedBy);
    contradictedBy.push(...copy.contradictedBy);
    history.push(...copy.history);
    for (const [id, caveats] of copy.caveatsOf) caveatsOf.set(id, caveats);
    for (const [id, at] of copy.tastedAt) tastedAt.set(id, at);
  }

  function save(): Saved {
    return structuredClone({
      mushrooms: [...mushrooms], glow, heavy, now, supportedBy, contradictedBy, trust, history,
      sinceContradiction, caveatsOf: [...caveatsOf], tastedAt: [...tastedAt],
    });
  }

  function faded(evidence: string): boolean {
    const at = tastedAt.get(evidence);
    return at !== undefined && now - at >= TASTE_FADES_AFTER;
  }

  function caveatsFor(evidence: string[]): string[] {
    return [...new Set(evidence.flatMap((id) => [...(caveatsOf.get(id) ?? []), ...(faded(id) ? ['taste_faded'] : [])]))];
  }

  function mushroom(id: string): Mushroom {
    const found = mushrooms.get(id);
    if (!found) throw new Error(`Unknown mushroom ${id}`);
    return found;
  }

  function evidenceOf(action: string, id: string): string {
    const { life } = mushroom(id);
    return life === 1 ? `${action}_${id}` : `${action}_${id}_${life}`;
  }

  function consume(target: Mushroom, evidence: string): void {
    target.consumed = true;
    target.consumedAt = now;
    target.consumedBy = evidence;
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
      // A decision keeps the caveats its basis carried when it was made.
      sinceContradiction.push(evidence);
      const recovered = trust !== null && trust.reopenedBy.length > 0 && sinceContradiction.length >= 2;
      if ((!trust && beliefState() === 'probably_safe') || recovered) {
        const basis = recovered ? [...sinceContradiction] : [evidence];
        trust = { basis, reopenedBy: [], caveats: caveatsFor(basis) };
        history.push({ change: 'committed', because: [...basis] });
      }
    } else {
      contradictedBy.push(evidence);
      sinceContradiction = [];
      if (trust) {
        trust.reopenedBy.push(evidence);
        history.push({ change: 'reopened', because: [evidence] });
      }
    }
  }

  // Every check runs before the first write, so a rejected event changes nothing.
  function dispatch(event: GlowcapEvent): void {
    switch (event.type) {
      case 'tick': {
        if (!(typeof event.dt === 'number' && event.dt >= 0 && event.dt <= 0.1)) throw new Error(`dt out of range: ${event.dt}`);
        glow = Math.max(0, glow - event.dt);
        heavy = Math.max(0, heavy - event.dt);
        now += event.dt;
        for (const target of mushrooms.values()) {
          if (target.consumed && now - target.consumedAt >= REGROWS_AFTER) Object.assign(target, { consumed: false, tasted: null, life: target.life + 1 });
        }
        return;
      }
      case 'absorb': {
        const target = mushroom(event.id);
        const kind = kindOf(event.kind);
        if (target.consumed) throw new Error(`${event.id} is already consumed`);
        if (target.tasted === 'duskcap') throw new Error(`${event.id} is a known duskcap`);
        if (target.tasted && target.tasted !== kind) throw new Error(`${event.id} tasted as ${target.tasted}`);
        if (tooRisky(target)) throw new Error(`${event.id} is too risky to absorb untasted`);
        const evidence = evidenceOf('absorb', event.id);
        consume(target, evidence);
        if (kind === 'glowcap') glow = GLOW_SECONDS;
        // Heaviness stacks while it lasts, up to a cap.
        else heavy = heavy > 0 ? Math.min(HEAVY_MAX_SECONDS, heavy + HEAVY_SECONDS) : HEAVY_SECONDS;
        learn(evidence, kind);
        return;
      }
      case 'taste': {
        const target = mushroom(event.id);
        const kind = kindOf(event.kind);
        if (target.consumed) throw new Error(`${event.id} is already consumed`);
        if (target.tasted) throw new Error(`${event.id} was already tasted`);
        target.tasted = kind;
        const evidence = evidenceOf('taste', event.id);
        if (glow <= 0) caveatsOf.set(evidence, ['tasted_in_dark']);
        tastedAt.set(evidence, now);
        learn(evidence, kind);
        return;
      }
      // Another creature eats it: the slime learns secondhand.
      case 'witness': {
        const target = mushroom(event.id);
        const kind = kindOf(event.kind);
        if (target.consumed) throw new Error(`${event.id} is already consumed`);
        if (target.tasted && target.tasted !== kind) throw new Error(`${event.id} tasted as ${target.tasted}`);
        const evidence = evidenceOf('witness', event.id);
        consume(target, evidence);
        caveatsOf.set(evidence, ['secondhand']);
        learn(evidence, kind);
        return;
      }
      default:
        throw new Error(`Unknown event ${(event as { type: unknown }).type}`);
    }
  }

  function mushroomView(id: string, belief: BeliefState) {
    const base = labelFor(id, belief);
    return { ...base, caveats: caveatsFor(base.because), why: whyNot(id) };
  }

  // Why each greyed-out action is greyed out, citing what the slime learned.
  function whyNot(id: string) {
    const target = mushroom(id);
    const reason = (text: string, because: string[]) => ({ reason: text, because, caveats: caveatsFor(because) });
    const allowed = reason('', []);
    if (target.consumed) return { absorb: reason('Already eaten', [target.consumedBy]), taste: reason('Already eaten', [target.consumedBy]) };
    const taste = evidenceOf('taste', id);
    if (target.tasted) return { absorb: target.tasted === 'duskcap' ? reason('Known duskcap', [taste]) : allowed, taste: reason('Already tasted', [taste]) };
    return { absorb: tooRisky(target) ? reason('Too risky untasted', [...contradictedBy]) : allowed, taste: allowed };
  }

  function labelFor(id: string, belief: BeliefState) {
    const { consumed, tasted } = mushroom(id);
    if (consumed) return { present: false, label: '', canAbsorb: false, canTaste: false, because: [] as string[] };
    if (tasted) {
      const taste = evidenceOf('taste', id);
      const dark = caveatsOf.get(taste)?.includes('tasted_in_dark') ?? false;
      const qualifier = faded(taste) ? ' (taste has faded)' : dark ? ' (tasted in the dark)' : null;
      const label = tasted === 'glowcap'
        ? qualifier ? `Probably a glowcap${qualifier}` : 'Glowcap'
        : qualifier ? `Probably a duskcap${qualifier}` : 'Duskcap — avoid';
      return { present: true, label, canAbsorb: tasted === 'glowcap', canTaste: false, because: [taste] };
    }
    if (tooRisky(mushroom(id))) return { present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: [...contradictedBy] };
    const guess: Record<BeliefState, [string, string[]]> = {
      none: ['Glowing mushroom', []],
      probably_safe: ['Probably a glowcap', supportedBy],
      probably_unsafe: ['Probably a duskcap', contradictedBy],
      uncertain: ['Could be a duskcap — taste first', contradictedBy],
    };
    const [label, because] = guess[belief];
    return { present: true, label, canAbsorb: true, canTaste: true, because: [...because] };
  }

  function view() {
    const state = beliefState();
    return {
      slime: { glowing: glow > 0, heavy: heavy > 0, heavySeconds: Math.ceil(heavy) },
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
        ? { state: trust.reopenedBy.length ? 'reopened' : 'committed', basis: [...trust.basis], reopenedBy: [...trust.reopenedBy], caveats: [...trust.caveats], history: history.map((entry) => ({ ...entry, because: [...entry.because] })) }
        : { state: 'none', basis: [] as string[], reopenedBy: [] as string[], caveats: [] as string[], history: [] },
    };
  }

  return { dispatch, view, save };
}
