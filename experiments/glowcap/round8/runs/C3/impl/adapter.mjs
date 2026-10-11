// Host glue: loads impl/glowcap.cav, forwards events, projects the view.
// Every rule, text and citation is in the Caveat source; this file only
// renames payload fields and reshapes what the runtime reports.
import { readFileSync } from 'node:fs';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const source = readFileSync(new URL('./glowcap.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();
const MUSHROOMS = runtime.interface(source).events
  .find((e) => e.name === 'absorb').parameters.find((p) => p.name === 'target').members;

// The host's `id` and `kind` are the program's `target` and `sort`.
const RENAMED = { id: 'target', kind: 'sort' };
function translate({ type, ...rest }) {
  // An evidence id `witness_pool_2` names its source, mushroom and life.
  const parts = typeof rest.evidence === 'string' && /^([a-z]+)_([a-z]+)(?:_(\d+))?$/.exec(rest.evidence);
  if ((type === 'doubt' || type === 'undoubt') && parts) return [type, { via: parts[1], target: parts[2], life: Number(parts[3] ?? 1) }];
  return [type, Object.fromEntries(Object.entries(rest).map(([k, v]) => [RENAMED[k] ?? k, v]))];
}

// A renewed occurrence `absorb_cave@2` is the view's `absorb_cave_2`.
const named = (list) => list.map((e) => e.replace(/@(\d+)$/, '_$1'));
const explained = (v, target, property) => v.binding_explanations?.[target]?.[property];
const cited = (v, target, property) => named(explained(v, target, property)?.evidence ?? []);
const caveated = (v, ...pairs) =>
  [...new Set(pairs.flatMap(([t, p]) => explained(v, t, p)?.caveats ?? []))];

function project(v) {
  const b = v.bindings;
  const mushrooms = {};
  for (const m of MUSHROOMS) {
    mushrooms[m] = {
      present: b[m].present, label: b[m].label,
      canAbsorb: b[m].canAbsorb, canTaste: b[m].canTaste, marked: b[m].marked, canRetaste: b[m].canRetaste,
      regrowsIn: b[m].regrowsIn, fadesIn: b[m].fadesIn,
      because: cited(v, m, 'label'),
      caveats: caveated(v, [m, 'label']),
      why: {
        absorb: { reason: b[m].whyAbsorb, because: cited(v, m, 'whyAbsorb'), caveats: caveated(v, [m, 'whyAbsorb']) },
        taste: { reason: b[m].whyTaste, because: cited(v, m, 'whyTaste'), caveats: caveated(v, [m, 'whyTaste']) },
      },
    };
  }
  // The journal holds the latest changes (the program keeps six).
  const journal = v.decision_journal.filter((e) => e.decision === 'trust');
  const current = v.decision_series.trust?.current;
  const grounds = current ? v.commitment_grounds[current] : undefined;
  return {
    slime: { glowing: b.slime.glowing, heavy: b.slime.heavy, heavySeconds: b.slime.heavySeconds, glowSeconds: b.slime.glowSeconds },
    mushrooms,
    belief: {
      state: b.belief.state, text: b.belief.text, note: b.belief.note,
      supportedBy: cited(v, 'belief', 'supportedBy'),
      contradictedBy: cited(v, 'belief', 'contradictedBy'),
      caveats: caveated(v, ['belief', 'supportedBy'], ['belief', 'contradictedBy']),
    },
    // The program keeps the journal as six records, oldest first: rN is the
    // text, cN cites what it is about and dN adds `doubted` when that applies.
    journal: [1, 2, 3, 4, 5, 6].filter((n) => b.diary[`r${n}`] !== '')
      .map((n) => ({
        text: b.diary[`r${n}`],
        because: cited(v, 'diary', `c${n}`),
        caveats: caveated(v, ['diary', `c${n}`], ['diary', `d${n}`]),
      })),
    decision: {
      state: b.decision.state,
      basis: grounds ? named(grounds.evidence.filter((e) => !grounds.inherited?.includes(e))) : [],
      caveats: grounds ? [...grounds.caveats] : [],
      history: journal.map((e) => ({ change: e.change, because: named(e.because) })),
      reopenedBy: journal.filter((e) => e.change === 'reopened' && e.commitment === current)
        .flatMap((e) => named(e.because)),
    },
  };
}

export function createPolicy(saved) {
  const session = saved === undefined ? runtime.open(source) : runtime.restore(source, JSON.stringify(saved));
  let current = project(session.view());
  return {
    dispatch(event) {
      const [name, payload] = translate(event);
      const outcome = session.dispatchView(name, payload);
      if (outcome.outcome !== 'accepted') {
        throw new Error(`${name} refused: ${outcome.origin}/${outcome.code}: ${outcome.message}`);
      }
      current = project(outcome.view);
    },
    view() { return structuredClone(current); },
    save() { return JSON.parse(session.save()); },
  };
}
