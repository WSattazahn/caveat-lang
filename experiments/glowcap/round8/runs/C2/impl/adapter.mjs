// Host glue for impl/glowcap.cav. It translates host events into the
// program's events and the program's view into the required view shape.
// Every game decision (what is accepted, labels, belief, trust) is the
// program's; the adapter only renames fields and copies explanations.
import { readFileSync } from 'node:fs';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const source = readFileSync(new URL('./glowcap.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();
const face = runtime.interface(source);
const mushrooms = face.events.find((e) => e.name === 'absorb')
  .parameters.find((p) => p.name === 'target').members;

function toProgram(event) {
  switch (event.type) {
    case 'absorb':
    case 'taste':
    case 'retaste':
    case 'witness':
      return [event.type, { target: event.id, sort: event.kind }];
    case 'tick':
      return ['tick', { dt: event.dt }];
    case 'mark':
    case 'unmark':
      return [event.type, { target: event.id }];
    case 'doubt':
    case 'undoubt':
      return [event.type, doubtPayload(event.evidence)];
    default:
      return [event.type, {}];
  }
}

// An evidence id as the view spells it (see EVIDENCE below): `witness_pool` is
// the pool's first life's witness, `witness_pool_2` its second. The program
// decides whether that is a remembered witness; here it is only spelled out as
// act (1 absorb, 2 taste, 3 witness, 4 retaste), mushroom and life. A spelling no evidence
// can have (an unknown mushroom, an unknown prefix, or `_1`) is passed as
// something the program cannot match (act 0 / life 0); an unknown mushroom is
// refused by the runtime as invalid input.
const ACT = { absorb: 1, taste: 2, witness: 3, retaste: 4 };
function doubtPayload(id) {
  const m = /^([a-z]+)_([a-z]+)(?:_(\d+))?$/.exec(String(id ?? ''));
  if (!m) return { target: mushrooms[0], act: 0, life: 0 };
  const life = m[3] === undefined ? 1 : Number(m[3]) >= 2 && Number(m[3]) <= 99999 ? Number(m[3]) : 0;
  return { target: m[2], act: ACT[m[1]] ?? 0, life };
}

const cited = (explanations, target, property, field) =>
  [...(explanations[target]?.[property]?.[field] ?? [])];
// The program keeps its names short (see the top of glowcap.cav): a_cave@2 is
// the view's absorb_cave_2, and the caveat dark is tasted_in_dark.
const EVIDENCE = { a: 'absorb', t: 'taste', w: 'witness', r: 'retaste' };
const CAVEAT = { dark: 'tasted_in_dark', faded: 'taste_faded', secondhand: 'secondhand', doubted: 'doubted' };
const occurrence = (name) => name
  .replace(/^([atwr])_/, (_, k) => `${EVIDENCE[k]}_`)
  .replace(/@(\d+)$/, '_$1');
// `d` is the program's record that the player doubts a witness; it carries the
// `doubted` caveat onto what cites that witness and is not itself an
// observation, so the view's evidence lists leave it out.
const names = (list) => list.filter((name) => name !== 'd').map(occurrence);
const caveatNames = (list) => list.map((c) => CAVEAT[c] ?? c);
const evidence = (ex, target, property) => names(cited(ex, target, property, 'evidence'));
const caveats = (ex, target, property) => caveatNames(cited(ex, target, property, 'caveats'));

function project(v) {
  const b = v.bindings;
  const ex = v.binding_explanations;
  const out = {
    slime: {
      glowing: b.slime.glowing,
      heavy: b.slime.heavy,
      heavySeconds: b.slime.heavy_seconds,
      glowSeconds: b.slime.glow_seconds,
    },
    mushrooms: {},
    belief: {
      state: b.belief.state,
      text: b.belief.text,
      note: b.belief.note,
      supportedBy: evidence(ex, 'belief', 'supported_by'),
      contradictedBy: evidence(ex, 'belief', 'contradicted_by'),
      caveats: caveats(ex, 'belief', 'cited'),
    },
    decision: { state: b.decision.state, basis: [], reopenedBy: [], caveats: [], history: [] },
  };
  for (const id of mushrooms) {
    out.mushrooms[id] = {
      present: b[id].present,
      marked: b[id].marked,
      label: b[id].label,
      canAbsorb: b[id].can_absorb,
      canTaste: b[id].can_taste,
      canRetaste: b[id].can_retaste,
      regrowsIn: b[id].regrows_in,
      fadesIn: b[id].fades_in,
      because: evidence(ex, id, 'label'),
      caveats: caveats(ex, id, 'label'),
      why: {
        absorb: { reason: b[id].why_absorb, because: evidence(ex, id, 'why_absorb'), caveats: caveats(ex, id, 'why_absorb') },
        taste: { reason: b[id].why_taste, because: evidence(ex, id, 'why_taste'), caveats: caveats(ex, id, 'why_taste') },
      },
    };
  }
  // The program's journal records e1..e6, oldest first: each shows an entry
  // and, after it, the trust entry the same event wrote, when the program shows
  // them. What an entry rests on is the explanation of its `cites` binding.
  const entry = (text, cites) => ({ text: b.journal[text], because: evidence(ex, 'journal', cites), caveats: caveats(ex, 'journal', cites) });
  out.journal = ['e1', 'e2', 'e3', 'e4', 'e5', 'e6']
    .flatMap((e) => [[`${e}_text`, `${e}_cites`], [`${e}_trust`, `${e}_trust_cites`]])
    .filter(([text]) => b.journal?.[text] !== undefined)
    .map(([text, cites]) => entry(text, cites));
  const journal = v.decision_journal.filter((j) => j.decision === 'trust');
  // The revision in force: its frozen grounds (View 0.2 lists them in
  // first-observed order) and the contradictions that reopened it.
  const current = v.decision_series.trust?.current;
  if (current) {
    const grounds = v.commitment_grounds[current];
    out.decision.basis = names(grounds?.evidence ?? []);
    out.decision.caveats = caveatNames(grounds?.caveats ?? []);
    out.decision.reopenedBy = names(journal
      .filter((j) => j.commitment === current && j.change === 'reopened')
      .flatMap((j) => j.because));
  }
  // The journal keeps its six most recent entries (journal window 6).
  out.decision.history = journal.map((j) => ({ change: j.change, because: names(j.because) }));
  return out;
}

export function createPolicy(saved) {
  // A save is the runtime's save as a JSON value (not a string holding JSON).
  const session = saved === undefined ? runtime.open(source) : runtime.restore(source, JSON.stringify(saved));
  const view = () => project(session.view({ schema: '0.2' }));
  let current = view();
  return {
    dispatch(event) {
      const [name, payload] = toProgram(event);
      let outcome;
      try {
        outcome = session.dispatchView(name, payload);
      } catch (error) {
        throw new Error(`event not handled: ${error.message}`);
      }
      if (outcome.outcome !== 'accepted') {
        throw new Error(`rejected (${outcome.origin}/${outcome.code}): ${outcome.message}`);
      }
      current = view();
    },
    view() {
      return structuredClone(current);
    },
    save() {
      return JSON.parse(session.save());
    },
  };
}
