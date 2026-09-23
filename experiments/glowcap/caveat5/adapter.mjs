// Presents glowcap.cav through the experiment's common interface, on the lean
// reactive runtime. The host sends names; rules, text and explanations stay in
// the source; this reads the per-event view.
import { readFile } from 'node:fs/promises';
import init, { WebReactiveSession } from '../../../dist/pkg-reactive/caveat_runtime.js';

const source = await readFile(new URL('./glowcap.cav', import.meta.url), 'utf8');
export const ready = init({ module_or_path: await readFile(new URL('../../../dist/pkg-reactive/caveat_runtime_bg.wasm', import.meta.url)) });

export function createPolicy() {
  const session = new WebReactiveSession(source);
  const mushrooms = JSON.parse(session.snapshot()).world.entities.filter((e) => e.kind === 'mushroom').map((e) => e.id);
  let shown = JSON.parse(session.view());

  function dispatch(event) {
    const payload = event.type === 'tick' ? { dt: event.dt } : { target: event.id, sort: event.kind };
    shown = JSON.parse(session.dispatch_view(event.type, JSON.stringify(payload)));
  }

  // The runtime names a regrown mushroom's evidence absorb_cave@2; the
  // protocol calls it absorb_cave_2.
  const named = (evidence) => evidence.map((name) => name.replace('@', '_'));

  function view() {
    const { bindings, binding_explanations: cites, relations, commitments, commitment_grounds: grounds, decision_journal: journal, decision_series: series } = shown;
    const bearing = (relation) => named(relations.filter((r) => r.relation === relation && r.to === 'glowing_is_safe').map((r) => r.from));
    const current = series.trust.current;
    const trust = commitments.find((c) => c.action === current);
    return {
      slime: { ...bindings.slime },
      mushrooms: Object.fromEntries(mushrooms.map((id) => {
        const { whyAbsorb, whyTaste, ...mushroom } = bindings[id];
        const why = (reason, cited) => ({ reason, because: named(cited.evidence), caveats: cited.caveats });
        return [id, { ...mushroom, because: named(cites[id].label.evidence), caveats: cites[id].label.caveats,
          why: { absorb: why(whyAbsorb, cites[id].whyAbsorb), taste: why(whyTaste, cites[id].whyTaste) } }];
      })),
      belief: { ...bindings.belief, supportedBy: bearing('supports'), contradictedBy: bearing('opposes'), caveats: cites.belief.state.caveats },
      decision: { state: bindings.decision.state, basis: named(grounds[current]?.evidence ?? []), reopenedBy: named(trust?.reopened_by ?? []), caveats: grounds[current]?.caveats ?? [],
        history: journal.filter((entry) => entry.decision === 'trust').map(({ change, because }) => ({ change, because: named(because) })) },
    };
  }

  return { dispatch, view, free: () => session.free() };
}
