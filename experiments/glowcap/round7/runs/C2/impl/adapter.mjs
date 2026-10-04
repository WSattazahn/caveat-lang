// Host glue: translates host events into the program's events and the
// program's view into the required view shape. The game's rules live in
// glowcap.cav; this file only renames and projects.
import { readFileSync } from 'node:fs';
import zlib from 'node:zlib';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const source = readFileSync(new URL('./glowcap.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();
export const ready = Promise.resolve();

// Mushroom ids, as the program declares them (static world).
const probe = runtime.open(source);
const MUSHROOMS = probe.snapshot().world.entities.filter((e) => e.kind === 'mushroom').map((e) => e.id);
probe.close();

function translate(event) {
  const { type, ...rest } = event ?? {};
  if (type === 'absorb' || type === 'taste' || type === 'witness') {
    if (Object.keys(rest).some((k) => k !== 'id' && k !== 'kind')) throw new Error(`unexpected field in ${type}`);
    return [type, { target: rest.id, sort: rest.kind }];
  }
  if (type === 'mark' || type === 'unmark') {
    if (Object.keys(rest).some((k) => k !== 'id')) throw new Error(`unexpected field in ${type}`);
    return [type, { target: rest.id }];
  }
  if (type === 'tick') return ['tick', rest];
  throw new Error(`unknown event type ${JSON.stringify(type)}`);
}

// An evidence occurrence `absorb_cave@2` is named `absorb_cave_2` in the view.
const named = (list) => list.map((name) => name.replace(/@(\d+)$/, '_$1'));
// What a displayed value cites (its explanation). Evidence is listed in the
// order it was observed (its stance relation's place in the graph), and caveats
// in the order they were attached to that evidence (their `qualifies` relations).
const observedAt = (v, e) => v.relations.findIndex((r) => r.from === e && (r.relation === 'supports' || r.relation === 'opposes'));
function cites(v, target, prop, field = 'evidence') {
  const { evidence = [], caveats = [] } = v.binding_explanations[target]?.[prop] ?? {};
  const ordered = [...evidence].sort((a, b) => observedAt(v, a) - observedAt(v, b));
  if (field === 'evidence') return named(ordered);
  const attached = ordered.flatMap((e) => v.relations.filter((r) => r.relation === 'qualifies' && r.to === e).map((r) => r.from));
  return [...new Set([...attached.filter((c) => caveats.includes(c)), ...caveats])];
}

// The view shows at most the six most recent decision changes and reopenings.
const SHOWN = 6;

const why = (v, id, prop) => ({
  reason: v.bindings[id][prop], because: cites(v, id, prop), caveats: cites(v, id, prop, 'caveats'),
});

function project(v) {
  const b = v.bindings;
  const mushrooms = {};
  for (const id of MUSHROOMS) {
    mushrooms[id] = {
      present: b[id].present,
      marked: b[id].marked,
      label: b[id].label,
      canAbsorb: b[id].can_absorb,
      canTaste: b[id].can_taste,
      because: cites(v, id, 'label'),
      caveats: cites(v, id, 'label', 'caveats'),
      why: { absorb: why(v, id, 'why_absorb'), taste: why(v, id, 'why_taste') },
    };
  }
  const journal = v.decision_journal.filter((e) => e.decision === 'trust');
  // The current commitment: the latest 'committed' entry, and what followed it.
  const at = journal.map((e) => e.change).lastIndexOf('committed');
  const current = at >= 0 ? journal[at] : null;
  const grounds = current ? v.commitment_grounds[current.commitment] : { evidence: [], caveats: [] };
  // The journal's six slots, newest first in the program; shown oldest first.
  const journal6 = [6, 5, 4, 3, 2, 1].map((k) => `journal_${k}`).filter((slot) => b[slot].text !== '')
    .map((slot) => ({ text: b[slot].text, because: cites(v, slot, 'text'), caveats: cites(v, slot, 'text', 'caveats') }));
  return {
    slime: { glowing: b.slime.glowing, heavy: b.slime.heavy, heavySeconds: b.slime.heavy_seconds },
    mushrooms,
    belief: {
      state: b.belief.state,
      text: b.belief.text,
      note: b.belief.note,
      supportedBy: cites(v, 'belief', 'supported'),
      contradictedBy: cites(v, 'belief', 'contradicted'),
      caveats: cites(v, 'belief', 'observations', 'caveats'),
    },
    journal: journal6,
    decision: {
      state: b.decision.state,
      basis: current ? named(current.because) : [],
      caveats: [...grounds.caveats],
      reopenedBy: at >= 0 ? named(journal.slice(at + 1).flatMap((e) => e.because)).slice(-SHOWN) : [],
      // The runtime's decision journal for trust, in order.
      history: journal.slice(-SHOWN).map((e) => ({ change: e.change, because: named(e.because) })),
    },
  };
}

// Saving. The runtime's own save keeps the whole evidence history, so it grows
// with play and soon passes the 4096-byte bound. The save is therefore the
// accepted events, replayed into a fresh session on resume (the runtime is
// deterministic). Runs of equal ticks are counted, and observations are bounded
// by the program's 64 lives per mushroom, so the deflated log stays small.
const pack = (log) => zlib.deflateRawSync(JSON.stringify(log)).toString('base64');
const unpack = (text) => JSON.parse(zlib.inflateRawSync(Buffer.from(text, 'base64')).toString('utf8'));

export function createPolicy(saved) {
  const session = runtime.open(source);
  const log = [];   // [name, payload, times]
  let current = project(session.view());
  function apply(name, payload) {
    const outcome = session.dispatchView(name, payload);
    if (outcome.outcome !== 'accepted') {
      throw new Error(`${name} rejected (${outcome.origin}/${outcome.code}): ${outcome.message}`);
    }
    current = project(outcome.view);
    const last = log[log.length - 1];
    if (last && name === 'tick' && last[0] === 'tick' && last[1].dt === payload.dt) last[2] += 1;
    else log.push([name, payload, 1]);
  }
  if (saved !== undefined) {
    for (const [name, payload, times] of unpack(saved.replay)) {
      for (let n = 0; n < times; n += 1) apply(name, payload);
    }
  }
  return {
    dispatch(event) {
      apply(...translate(event));
    },
    view() {
      return structuredClone(current);
    },
    save() {
      return { replay: pack(log) };
    },
  };
}
