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
    const trust = commitments.find((c) => c.action === 'trust');
    return {
      slime: { ...bindings.slime },
      mushrooms: Object.fromEntries(mushrooms.map((id) => [id, { ...bindings[id], because: cites[id].label.evidence, caveats: cites[id].label.caveats }])),
      belief: { ...bindings.belief, supportedBy: bearing('supports'), contradictedBy: bearing('opposes'), caveats: cites.belief.state.caveats },
      decision: {
        state: bindings.decision.state,
        basis: bases.trust?.provenance.evidence ?? [],
        reopenedBy: trust?.reopened_by ?? [],
        caveats: bases.trust?.provenance.caveats ?? [],
      },
    };
  }

  return { dispatch, view, free: () => session.free() };
}
