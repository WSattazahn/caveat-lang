// Host glue: loads impl/glowcap.cav, translates host events into the
// program's events, and projects the program's view into the required shape.
// Every game decision (what is allowed, what is shown, why) is in the source.
import { readFile } from 'node:fs/promises';
import { brotliCompressSync, brotliDecompressSync } from 'node:zlib';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const source = await readFile(new URL('./glowcap.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();
// The mushrooms are the program's `mushroom` entities, as its absorb event declares them.
const mushrooms = (() => {
  const probe = runtime.open(source);
  try {
    const absorb = probe.snapshot().events.find((e) => e.name === 'absorb');
    return absorb.parameters.find((p) => p.name === 'target').domain.entity.members;
  } finally { probe.close(); }
})();

export const ready = Promise.resolve();

const translate = {
  absorb: (e) => ['absorb', { target: e.id, sort: e.kind }],
  witness: (e) => ['witness', { target: e.id, sort: e.kind }],
  taste: (e) => ['taste', { target: e.id, sort: e.kind }],
  mark: (e) => ['mark', { target: e.id }],
  unmark: (e) => ['unmark', { target: e.id }],
  tick: (e) => ['tick', { dt: e.dt }],
};

// The runtime names a renewed occurrence `absorb_cave@2`; the view calls that life `absorb_cave_2`.
const life = (evidence) => evidence.replace(/@(\d+)$/, '_$1');

const rank = (order, caveat) => { const at = order.indexOf(caveat); return at < 0 ? order.length : at; };

function project(view) {
  const b = view.bindings;
  const explained = (target, property) => view.binding_explanations[target]?.[property] ?? { evidence: [], caveats: [] };
  const cites = (target, property) => explained(target, property).evidence.map(life);
  // Caveats are listed in the order they came to qualify the cited evidence
  // (the graph's relation order); the runtime reports them as sorted sets.
  const caveatsOf = (...cited) => {
    const evidence = new Set(cited.flatMap(([t, p]) => explained(t, p).evidence));
    const caveats = new Set(cited.flatMap(([t, p]) => explained(t, p).caveats));
    const order = view.relations.filter((r) => r.relation === 'qualifies' && evidence.has(r.to)).map((r) => r.from);
    return [...caveats].sort((x, y) => rank(order, x) - rank(order, y));
  };
  const current = view.decision_series.trust?.current ?? null;
  const reopenedBy = view.decision_journal
    .filter((j) => j.commitment === current && j.change === 'reopened')
    .flatMap((j) => j.because.map(life))
    .slice(-6);   // the view keeps the six most recent
  return {
    slime: { glowing: b.slime.glowing, heavy: b.slime.heavy, heavySeconds: b.slime.heavySeconds },
    mushrooms: Object.fromEntries(mushrooms.map((id) => [id, {
      present: b[id].present,
      marked: b[id].marked,
      label: b[id].label,
      canAbsorb: b[id].canAbsorb,
      canTaste: b[id].canTaste,
      because: cites(id, 'label'),
      caveats: caveatsOf([id, 'label']),
      why: Object.fromEntries(['absorb', 'taste'].map((action) => [action, {
        reason: b[id][`why.${action}`],
        because: cites(id, `why.${action}`),
        caveats: caveatsOf([id, `why.${action}`]),
      }])),
    }])),
    belief: {
      state: b.belief.state,
      text: b.belief.text,
      note: b.belief.note,
      supportedBy: cites('belief', 'supportedBy'),
      contradictedBy: cites('belief', 'contradictedBy'),
      caveats: caveatsOf(['belief', 'supportedBy'], ['belief', 'contradictedBy']),
    },
    // The program's journal slots e1 (oldest) .. e6; an empty text is an unused slot.
    journal: [1, 2, 3, 4, 5, 6].map((n) => `e${n}`).filter((slot) => b.journal[slot] !== '')
      .map((slot) => ({ text: b.journal[slot], because: cites('journal', slot), caveats: caveatsOf(['journal', slot]) })),
    decision: {
      state: b.decision.state,
      basis: current ? view.commitment_grounds[current].evidence.map(life) : [],
      reopenedBy,
      caveats: current ? [...view.commitment_grounds[current].caveats] : [],
      // The runtime's decision journal, in order; the view keeps the six most recent.
      history: view.decision_journal.filter((j) => j.decision === 'trust')
        .map((j) => ({ change: j.change, because: j.because.map(life) }))
        .slice(-6),
    },
  };
}

// A save is the runtime's exact save text, stored compressed: the runtime keeps
// the whole evidence and decision history, which is larger than the 4096-byte
// budget once it is written out as plain JSON.
const pack = (text) => ({ brotli: brotliCompressSync(Buffer.from(text, 'utf8')).toString('base64') });
const unpack = (saved) => brotliDecompressSync(Buffer.from(saved.brotli, 'base64')).toString('utf8');

export function createPolicy(saved) {
  const session = saved === undefined ? runtime.open(source) : runtime.restore(source, unpack(saved));
  let view = session.view();
  return {
    dispatch(event) {
      const make = translate[event?.type];
      if (!make) throw new Error(`unknown event type ${JSON.stringify(event?.type)}`);
      const [name, payload] = make(event);
      const outcome = session.dispatchView(name, payload);
      if (outcome.outcome !== 'accepted') {
        throw new Error(`${name} rejected (${outcome.origin}/${outcome.code}): ${outcome.message}`);
      }
      view = outcome.view;
    },
    view: () => project(view),
    save: () => pack(session.save()),
  };
}
