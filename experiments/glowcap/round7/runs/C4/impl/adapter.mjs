// Host glue: translates events into the Caveat program's events and its view
// into the required shape. Every game decision is made in glowcap.cav.
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
import { deflateRawSync, inflateRawSync } from 'node:zlib';

const source = await readFile(new URL('./glowcap.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();
const { MUSHROOMS, CAVEATS } = (() => {
  const s = runtime.open(source);
  const snap = s.snapshot();
  s.close();
  return {
    MUSHROOMS: snap.world.entities.filter((e) => e.kind === 'mushroom').map((e) => e.id),
    // Caveats in the order the program declares them (the snapshot sorts its
    // symbols by name), for lists compared in order.
    CAVEATS: [...source.matchAll(/^caveat\s+(\w+)/gm)].map((m) => m[1]),
  };
})();

function translate(event) {
  const { type, ...p } = event ?? {};
  if (type === 'absorb' || type === 'taste' || type === 'witness') {
    if (Object.keys(p).sort().join() !== 'id,kind') throw new Error(`bad ${type} payload`);
    return [type, { target: p.id, sort: p.kind }];
  }
  if (type === 'mark' || type === 'unmark') {
    if (Object.keys(p).join() !== 'id') throw new Error(`bad ${type} payload`);
    return [type, { target: p.id }];
  }
  if (type === 'tick') {
    if (Object.keys(p).join() !== 'dt') throw new Error('bad tick payload');
    return ['tick', { dt: p.dt }];
  }
  throw new Error(`unknown event ${type}`);
}

// Evidence occurrence `absorb_cave@2` (the mushroom's second life) is named `absorb_cave_2`.
const name = (e) => e.replace(/@(\d+)$/, '_$1');
const names = (list) => (list ?? []).map(name);
// The view shows at most this many recent reopenings and history entries.
const KEEP = 6;
const evidence = (expl, target, prop) => names(expl[target]?.[prop]?.evidence);
const caveats = (expl, target, prop) => [...(expl[target]?.[prop]?.caveats ?? [])];

// Ordered lists: evidence in the runtime's observation order, caveats in declaration order.
const byIndex = (order) => (a, b) => order.indexOf(a) - order.indexOf(b);

function project(v, observations) {
  const b = v.bindings, x = v.binding_explanations;
  const mushrooms = {};
  for (const id of MUSHROOMS) {
    mushrooms[id] = {
      present: b[id].present,
      marked: b[id].marked,
      label: b[id].label,
      canAbsorb: b[id].can_absorb,
      canTaste: b[id].can_taste,
      because: evidence(x, id, 'label'),
      caveats: caveats(x, id, 'label'),
      why: {
        absorb: { reason: b[id].why_absorb, because: evidence(x, id, 'why_absorb'), caveats: caveats(x, id, 'why_absorb') },
        taste: { reason: b[id].why_taste, because: evidence(x, id, 'why_taste'), caveats: caveats(x, id, 'why_taste') },
      },
    };
  }
  const series = v.decision_series.trust;
  const current = series?.current;
  const journal = v.decision_journal.filter((e) => e.decision === 'trust');
  const committed = journal.find((e) => e.commitment === current && e.change === 'committed');
  const grounds = current ? v.commitment_grounds[current] : undefined;
  const journal_ = [1, 2, 3, 4, 5, 6]
    .map((k) => {
      const why = x.journal?.[`text_${k}`];
      return {
        text: b.journal[`text_${k}`],
        because: names([...(why?.evidence ?? [])].sort(byIndex(observations))),
        caveats: [...(why?.caveats ?? [])].sort(byIndex(CAVEATS)),
      };
    })
    .filter((e) => e.text !== '');
  return {
    journal: journal_,
    slime: { glowing: b.slime.glowing, heavy: b.slime.heavy, heavySeconds: b.slime.heavy_seconds },
    mushrooms,
    belief: {
      state: b.belief.state,
      text: b.belief.text,
      note: b.belief.note,
      supportedBy: evidence(x, 'belief', 'supported'),
      contradictedBy: evidence(x, 'belief', 'contradicted'),
      caveats: caveats(x, 'belief', 'evidence'),
    },
    decision: {
      state: b.decision.state,
      basis: names(committed?.because ?? grounds?.evidence),
      caveats: [...(grounds?.caveats ?? [])],
      reopenedBy: journal
        .filter((e) => e.commitment === current && e.change === 'reopened')
        .flatMap((e) => names(e.because))
        .slice(-KEEP),
      history: journal.slice(-KEEP).map((e) => ({ change: e.change, because: names(e.because) })),
    },
  };
}

export function createPolicy(saved) {
  // A save is the runtime's save text, compressed to keep stored saves small.
  const session = saved === undefined
    ? runtime.open(source)
    : runtime.restore(source, inflateRawSync(Buffer.from(saved.caveat, 'base64')).toString('utf8'));
  let observations = session.snapshot().observations ?? [];
  let view = project(session.view(), observations);
  return {
    dispatch(event) {
      const [name, payload] = translate(event);
      const outcome = session.dispatchView(name, payload);
      if (outcome.outcome !== 'accepted') {
        throw new Error(`${name} rejected (${outcome.origin}/${outcome.code}): ${outcome.message}`);
      }
      if (outcome.view.effects.some((e) => e.kind === 'reveal')) observations = session.snapshot().observations ?? [];
      view = project(outcome.view, observations);
    },
    view: () => structuredClone(view),
    save: () => ({ caveat: deflateRawSync(session.save(), { level: 9 }).toString('base64') }),
  };
}
