// Host glue for impl/glowcap.cav. It translates host events into the program's
// events and projects the program's view into the required shape. Every game
// decision (what is accepted, labels, belief, trust) is made by the program.
import { readFileSync } from 'node:fs';
import { deflateRawSync, inflateRawSync } from 'node:zlib';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const source = readFileSync(new URL('./glowcap.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();
export const ready = Promise.resolve();

// The mushrooms are the program's entities of kind `mushroom`.
const probe = runtime.open(source);
const MUSHROOMS = probe.snapshot().world.entities.filter((e) => e.kind === 'mushroom').map((e) => e.id);
probe.close();

function toProgramEvent(event) {
  const { type, ...rest } = event ?? {};
  if (type === 'absorb' || type === 'taste' || type === 'retaste' || type === 'witness') {
    const { id, kind, ...extra } = rest;
    return [type, { target: id, sort: kind, ...extra }];
  }
  if (type === 'mark' || type === 'unmark') {
    const { id, ...extra } = rest;
    return [type, { target: id, ...extra }];
  }
  if (type === 'doubt' || type === 'undoubt') {
    // An evidence id names its action, mushroom and life: `witness_pool_2`.
    const { evidence, ...extra } = rest;
    const parts = /^([a-z]+)_([a-z]+)(?:_([0-9]+))?$/.exec(String(evidence));
    if (!parts) return [type, rest]; // the program refuses a payload it does not declare
    return [type, { act: parts[1], target: parts[2], life: parts[3] === undefined ? 1 : Number(parts[3]), ...extra }];
  }
  return [type, rest];
}

// The runtime names a later life's occurrence `absorb_cave@2`; the view calls it `absorb_cave_2`.
const name = (occurrence) => occurrence.replace(/@(\d+)$/, '_$1');
const names = (list) => list.map(name);
const evidence = (explanations, target, property) => names(explanations[target]?.[property]?.evidence ?? []);
const caveats = (explanations, ...cited) => [...new Set(cited.flatMap(([t, p]) => explanations[t]?.[p]?.caveats ?? []))];

function project(view) {
  const b = view.bindings;
  const ex = view.binding_explanations;
  const mushrooms = {};
  for (const id of MUSHROOMS) {
    mushrooms[id] = {
      present: b[id].present,
      marked: b[id].marked,
      canRetaste: b[id].can_retaste,
      regrowsIn: b[id].regrows_in,
      fadesIn: b[id].fades_in,
      label: b[id].label,
      canAbsorb: b[id].can_absorb,
      canTaste: b[id].can_taste,
      because: evidence(ex, id, 'label'),
      caveats: caveats(ex, [id, 'label']),
      why: {
        absorb: { reason: b[id].why_absorb, because: evidence(ex, id, 'why_absorb'), caveats: caveats(ex, [id, 'why_absorb']) },
        taste: { reason: b[id].why_taste, because: evidence(ex, id, 'why_taste'), caveats: caveats(ex, [id, 'why_taste']) },
      },
    };
  }
  const current = view.decision_series.trust?.current ?? null;
  // The journal keeps the six most recent changes (journal window 6); the
  // current commitment's entries are its newest ones.
  const changes = view.decision_journal.filter((e) => e.decision === 'trust');
  const grounds = current ? view.commitment_grounds[current] : null;
  // The program's journal slots, oldest first; empty slots are not shown.
  const journal = [];
  for (let k = 1; k <= 6; k += 1) {
    if (b.journal?.[`text${k}`] === undefined) continue;
    journal.push({ text: b.journal[`text${k}`], because: evidence(ex, 'journal', `cites${k}`), caveats: caveats(ex, ['journal', `cites${k}`]) });
  }
  return {
    journal,
    slime: { glowing: b.slime.glowing, heavy: b.slime.heavy, heavySeconds: b.slime.heavy_seconds, glowSeconds: b.slime.glow_seconds },
    mushrooms,
    belief: {
      state: b.belief.state,
      text: b.belief.text,
      note: b.belief.note,
      supportedBy: evidence(ex, 'belief', 'supported_by'),
      contradictedBy: evidence(ex, 'belief', 'contradicted_by'),
      caveats: caveats(ex, ['belief', 'supported_by'], ['belief', 'contradicted_by']),
    },
    decision: {
      state: b.decision.state,
      // The current commitment's frozen grounds, in the order first observed (View 0.2).
      basis: names(grounds?.evidence ?? []),
      caveats: [...(grounds?.caveats ?? [])],
      reopenedBy: names(changes.filter((e) => e.commitment === current && e.change === 'reopened').flatMap((e) => e.because)),
      history: changes.map((e) => ({ change: e.change, because: names(e.because) })),
    },
  };
}

// A save is the runtime's own save text, deflated and base64-encoded: a host-side
// storage encoding only. The runtime save keeps the full evidence record.
const pack = (text) => ({ caveatSave: deflateRawSync(Buffer.from(text, 'utf8'), { level: 9 }).toString('base64') });
const unpack = (saved) => inflateRawSync(Buffer.from(saved.caveatSave, 'base64')).toString('utf8');

export function createPolicy(saved) {
  const session = saved === undefined ? runtime.open(source) : runtime.restore(source, unpack(saved));
  return {
    dispatch(event) {
      const [eventName, payload] = toProgramEvent(event);
      const outcome = session.dispatch(eventName, payload);
      if (outcome.outcome !== 'accepted') {
        throw new Error(`${eventName} rejected (${outcome.origin}/${outcome.code}): ${outcome.message}`);
      }
    },
    view() {
      return project(session.view({ schema: '0.2' }));
    },
    save() {
      return pack(session.save());
    },
  };
}
