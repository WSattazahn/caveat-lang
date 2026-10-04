// @ts-check
// Host glue for impl/glowcap.cav. It translates host events into the
// program's events and projects the program's view into the required shape.
// Every rule, label, decision and citation comes from the program: labels and
// flags from its bindings, `because` lists from its binding explanations, the
// trust decision's basis, its caveats, reopenings and history from its decision journal.
import { readFile } from 'node:fs/promises';
import { deflateRawSync, inflateRawSync } from 'node:zlib';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
import { typed } from 'caveat-lang/types';
/** @typedef {import('./glowcap.js').Program} Program */
/** @typedef {import('./glowcap.js').EventName} EventName */
/** @typedef {Program['events']['absorb']['target']} Mushroom */
/** @typedef {[EventName, Program['events'][EventName]]} Dispatch */

const source = await readFile(new URL('./glowcap.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();

// The mushrooms are the program's `mushroom` entities, read from the
// signature of its absorb event.
/** @type {Mushroom[]} */
const mushrooms = (() => {
  const probe = runtime.open(source);
  const absorb = probe.snapshot().events.find((e) => e.name === 'absorb');
  probe.close();
  return absorb.parameters.find((p) => p.name === 'target').domain.entity.members;
})();

/** @returns {Dispatch} */
function translate(event) {
  const { type, ...payload } = event ?? {};
  if (type === 'absorb' || type === 'taste' || type === 'witness') {
    const { id, kind, ...rest } = payload;
    if (Object.keys(rest).length) throw new Error(`unexpected fields for ${type}`);
    return [type, { target: id, sort: kind }];
  }
  if (type === 'mark' || type === 'unmark') {
    const { id, ...rest } = payload;
    if (Object.keys(rest).length) throw new Error(`unexpected fields for ${type}`);
    return [type, { target: id }];
  }
  if (type === 'tick') return ['tick', payload];
  throw new Error(`unknown event type ${type}`);
}

// A renewed evidence occurrence is named `absorb_cave@2` by the runtime; the
// host names a mushroom's later lives `absorb_cave_2`.
const evidenceName = (name) => name.replace(/@(\d+)$/, '_$1');
const names = (list) => list.map(evidenceName);
const explained = (view, target, property, part) => [...(view.binding_explanations?.[target]?.[property]?.[part] ?? [])];
const cited = (view, target, property) => names(explained(view, target, property, 'evidence'));
// Caveats are listed in the order they came to qualify the cited evidence: the
// order of the runtime's `qualifies` relations (relations keep insertion order).
function inOrder(view, evidence, caveats) {
  const first = (caveat) => {
    const i = view.relations.findIndex((r) => r.relation === 'qualifies' && r.from === caveat && evidence.includes(r.to));
    return i < 0 ? Infinity : i;
  };
  return [...caveats].sort((a, b) => first(a) - first(b));
}
const caveatsOf = (view, target, property) =>
  inOrder(view, explained(view, target, property, 'evidence'), explained(view, target, property, 'caveats'));

// The view shows at most the six most recent reopenings and journal entries.
const SHOWN = 6;

/** @param {import('./glowcap.js').View} view */
function project(view) {
  const b = view.bindings;
  const journal = view.decision_journal.filter((entry) => entry.decision === 'trust');
  let current = -1;
  journal.forEach((entry, i) => { if (entry.change === 'committed') current = i; });
  const basis = current < 0 ? [] : names(journal[current].because);
  const basisCaveats = current < 0 ? [] : inOrder(view, journal[current].because, journal[current].caveats);
  const reopenedBy = current < 0 ? [] : journal.slice(current + 1)
    .filter((entry) => entry.change === 'reopened').flatMap((entry) => names(entry.because)).slice(-SHOWN);
  return {
    slime: { glowing: b.slime.glowing, heavy: b.slime.heavy, heavySeconds: b.slime.heavy_seconds },
    mushrooms: Object.fromEntries(mushrooms.map((m) => [m, {
      present: b[m].present,
      marked: b[m].marked,
      label: b[m].label,
      canAbsorb: b[m].can_absorb,
      canTaste: b[m].can_taste,
      because: cited(view, m, 'label'),
      caveats: caveatsOf(view, m, 'label'),
      why: {
        absorb: { reason: b[m].why_absorb, because: cited(view, m, 'why_absorb'), caveats: caveatsOf(view, m, 'why_absorb') },
        taste: { reason: b[m].why_taste, because: cited(view, m, 'why_taste'), caveats: caveatsOf(view, m, 'why_taste') },
      },
    }])),
    belief: {
      state: b.belief.state,
      text: b.belief.text,
      note: b.belief.note,
      supportedBy: cited(view, 'belief', 'supported'),
      contradictedBy: cited(view, 'belief', 'contradicted'),
      caveats: caveatsOf(view, 'belief', 'cited'),
    },
    journal: ['j1', 'j2', 'j3', 'j4', 'j5', 'j6'].filter((j) => b[j].shown).map((j) => ({
      text: b[j].text,
      because: cited(view, j, 'text'),
      caveats: caveatsOf(view, j, 'text'),
    })),
    decision: {
      state: b.decision.state,
      basis,
      reopenedBy,
      caveats: basisCaveats,
      history: journal.slice(-SHOWN).map((entry) => ({ change: entry.change, because: names(entry.because) })),
    },
  };
}

// A save is the runtime's save text, deflated and base64-encoded into one JSON
// string. The runtime keeps the session's whole evidence archive, which grows
// with play; compression keeps the stored text small, it does not bound it.
const pack = (text) => deflateRawSync(Buffer.from(text, 'utf8'), { level: 9 }).toString('base64');
const unpack = (packed) => inflateRawSync(Buffer.from(packed, 'base64')).toString('utf8');

export function createPolicy(saved) {
  /** @type {import('./glowcap.js').Session} */
  const session = typed(saved === undefined ? runtime.open(source) : runtime.restore(source, unpack(saved)));
  return {
    dispatch(event) {
      const [name, payload] = translate(event);
      const outcome = session.dispatchView(name, payload);
      if (outcome.outcome !== 'accepted') {
        throw new Error(`${name} refused (${outcome.origin}/${outcome.code}): ${outcome.message}`);
      }
    },
    view() {
      return project(session.view());
    },
    save() {
      return pack(session.save());
    },
  };
}
