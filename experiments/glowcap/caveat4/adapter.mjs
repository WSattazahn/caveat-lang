// Presents glowcap.cav through the experiment's common interface, on the lean
// reactive runtime. The host sends names; rules, text and explanations stay in
// the source; this reads the per-event view.
import { readFile } from 'node:fs/promises';
import init, { WebReactiveSession } from '../../../dist/pkg-reactive/caveat_runtime.js';

const source = await readFile(new URL('./glowcap.cav', import.meta.url), 'utf8');
export const ready = init({ module_or_path: await readFile(new URL('../../../dist/pkg-reactive/caveat_runtime_bg.wasm', import.meta.url)) });

export function createPolicy(saved) {
  const session = new WebReactiveSession(source);
  let shown = JSON.parse(session.view());
  // Each mushroom's declared lives, in order; the current one is the first not yet regrown.
  const lives = new Map();
  for (const { id, kind } of JSON.parse(session.snapshot()).world.entities) {
    const base = id.replace(/_\d+$/, '');
    if (kind === 'mushroom') lives.set(base, [...(lives.get(base) ?? []), id]);
  }
  const liveOf = (id) => {
    if (!lives.has(id)) throw new Error(`Unknown mushroom ${id}`);
    return lives.get(id).find((life) => !shown.bindings[life].regrown) ?? lives.get(id).at(-1);
  };

  // The runtime cannot restore a session, so a save is the accepted events,
  // runs of equal ticks folded together, replayed on resume.
  const log = [];

  function dispatch(event) {
    const payload = event.type === 'tick' ? { dt: event.dt } : { target: liveOf(event.id), sort: event.kind };
    shown = JSON.parse(session.dispatch_view(event.type, JSON.stringify(payload)));
    const last = log.at(-1);
    if (event.type === 'tick' && last?.type === 'tick' && last.dt === event.dt) last.times += 1;
    else log.push(event.type === 'tick' ? { type: 'tick', dt: event.dt, times: 1 } : { type: event.type, id: event.id, kind: event.kind });
  }

  function view() {
    const { bindings, binding_explanations: cites, relations, commitments, commitment_grounds: grounds, decision_journal: journal, decision_series: series } = shown;
    const bearing = (relation) => relations.filter((r) => r.relation === relation && r.to === 'glowing_is_safe').map((r) => r.from);
    const current = series.trust.current;
    const trust = commitments.find((c) => c.action === current);
    return {
      slime: { ...bindings.slime },
      mushrooms: Object.fromEntries([...lives.keys()].map((id) => {
        const life = liveOf(id);
        const { regrown, whyAbsorb, whyTaste, ...mushroom } = bindings[life];
        const why = (reason, cited) => ({ reason, because: cited.evidence, caveats: cited.caveats });
        return [id, { ...mushroom, because: cites[life].label.evidence, caveats: cites[life].label.caveats,
          why: { absorb: why(whyAbsorb, cites[life].whyAbsorb), taste: why(whyTaste, cites[life].whyTaste) } }];
      })),
      belief: { ...bindings.belief, supportedBy: bearing('supports'), contradictedBy: bearing('opposes'), caveats: cites.belief.state.caveats },
      decision: { state: bindings.decision.state, basis: grounds[current]?.evidence ?? [], reopenedBy: trust?.reopened_by ?? [], caveats: grounds[current]?.caveats ?? [],
        history: journal.filter((entry) => entry.decision === 'trust').map(({ change, because }) => ({ change, because })) },
    };
  }

  for (const { times = 1, ...event } of saved?.log ?? []) for (let i = 0; i < times; i += 1) dispatch(event);

  return { dispatch, view, save: () => ({ log: structuredClone(log) }), free: () => session.free() };
}
