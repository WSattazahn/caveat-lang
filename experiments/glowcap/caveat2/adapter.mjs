// Presents glowcap.cav through the experiment's common interface. Rules, text
// and explanations stay in the source; this only maps events in and reads the
// snapshot's bindings, grounded explanations and graph out.
import { readFile } from 'node:fs/promises';
import init, { WebReactiveSession } from '../../../dist/pkg/caveat_runtime.js';

const source = await readFile(new URL('./glowcap.cav', import.meta.url), 'utf8');
export const ready = init({ module_or_path: await readFile(new URL('../../../dist/pkg/caveat_runtime_bg.wasm', import.meta.url)) });

// An unknown kind maps outside the declared 0..1 bound, so the source rejects it.
const dusk = (kind) => (kind === 'glowcap' ? 0 : kind === 'duskcap' ? 1 : -1);

export function createPolicy() {
  const session = new WebReactiveSession(source);
  let snapshot = JSON.parse(session.snapshot());
  const mushrooms = snapshot.world.entities.filter((entity) => entity.kind === 'mushroom').map((entity) => entity.id);

  function dispatch(event) {
    let name;
    let payload;
    if (event.type === 'tick') [name, payload] = ['tick', { dt: event.dt }];
    else if (event.type === 'absorb' || event.type === 'taste') [name, payload] = [`${event.type}_${event.id}`, { dusk: dusk(event.kind) }];
    else throw new Error(`Unknown event ${event.type}`);
    snapshot = JSON.parse(session.dispatch(name, JSON.stringify(payload)));
  }

  function view() {
    const { bindings, binding_explanations: cites, relations, commitment_bases: bases, commitments } = snapshot;
    const bearing = (relation) => relations.filter((r) => r.relation === relation && r.to === 'glowing_is_safe').map((r) => r.from);
    const series = snapshot.decision_series.trust;
    const revision = (id) => commitments.find((c) => c.action === id);
    const current = series.current ? revision(series.current) : null;
    // Evidence in the order it was observed, for lists compared in order.
    const seen = new Map();
    relations.forEach((r, index) => { if (r.to === 'glowing_is_safe' && !seen.has(r.from)) seen.set(r.from, index); });
    const ordered = (ids) => [...ids].sort((a, b) => seen.get(a) - seen.get(b));
    // A revision's basis also holds its predecessor's basis and reopening
    // witnesses; subtract them to recover what it was committed on.
    const ownBasis = (rev) => {
      const inherited = new Set(rev.previous ? [...bases[rev.previous].provenance.evidence, ...revision(rev.previous).reopened_by] : []);
      return ordered(bases[rev.id].provenance.evidence.filter((id) => !inherited.has(id)));
    };
    const shown = cites.decision?.basis;
    return {
      slime: { ...bindings.slime },
      mushrooms: Object.fromEntries(mushrooms.map((id) => [id, { ...bindings[id], because: cites[id].label.evidence, caveats: cites[id].label.caveats }])),
      belief: { ...bindings.belief, supportedBy: bearing('supports'), contradictedBy: bearing('opposes'), caveats: cites.belief.state.caveats },
      decision: {
        state: bindings.decision.state,
        basis: shown?.evidence ?? [],
        reopenedBy: current?.reopened_by ?? [],
        caveats: shown?.caveats ?? [],
        // Bindings are primitive, so the history is read from the series' records.
        history: series.revisions.flatMap((rev) => [
          { change: 'committed', because: ownBasis(rev) },
          ...revision(rev.id).reopened_by.map((evidence) => ({ change: 'reopened', because: [evidence] })),
        ]),
      },
    };
  }

  return { dispatch, view, free: () => session.free() };
}
