// Independent model of the pond task. It imports nothing from the runtime and
// predicts, from the task text alone, what a correct program shows: admission,
// the hud, relations, qualifications, commitment grounds and decision journal.
// The change request's model extends this one (see change-oracle.mjs).

const clone = value => structuredClone(value);
const lexical = values => [...new Set(values)].sort();

export const input = (event, payload = {}) => ({ event, payload });
export const resume = () => ({ resume: true });

// A payload is valid when it has exactly the declared parameters, each a
// finite number within its inclusive range.
export function validPayload(signature, payload) {
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) return false;
  const names = Object.keys(signature);
  if (Object.keys(payload).length !== names.length) return false;
  return names.every(name => {
    const value = payload[name];
    const [low, high] = signature[name];
    return Object.hasOwn(payload, name) && typeof value === 'number' && Number.isFinite(value) && value >= low && value <= high;
  });
}

// Instruments: the reading stream, its evidence caveat, its per-session cap.
export const AUGER = { event: 'measure', stream: 'thickness', caveat: 'single_hole', cap: 8 };
export const THIN = 10;

export function initialState() {
  return { sequence: 0, readings: [], order: [], caveatsOf: {}, observations: [], cracked: false,
    decisions: [], journal: [], grounds: {} };
}

function observe(state, id, relation, caveat) {
  state.order.push(id);
  state.caveatsOf[id] = [caveat];
  state.observations.push({ from: id, to: 'ice_safe', relation });
}

export function current(state) { return state.decisions.at(-1) ?? null; }
function isOpen(state) {
  const decision = current(state);
  return decision !== null && !decision.reopened && decision.value >= THIN;
}

function reopen(state, because, event) {
  const decision = current(state);
  decision.reopened = true;
  state.journal.push({ decision: 'rink', commitment: decision.id, change: 'reopened', sequence: state.sequence,
    event, elapsed: 0, value: decision.value, because: [because], caveats: [...state.caveatsOf[because]] });
}

export function record(state, instrument, cm) {
  const count = state.readings.filter(reading => reading.stream === instrument.stream).length;
  if (count >= instrument.cap) return false;
  const id = `${instrument.stream}@${count + 1}`;
  state.readings.push({ id, stream: instrument.stream, cm });
  observe(state, id, cm >= THIN ? 'supports' : 'opposes', instrument.caveat);
  if (cm < THIN && isOpen(state)) reopen(state, id, instrument.event);
  return true;
}

function crack(state) {
  if (state.cracked) return false;
  state.cracked = true;
  observe(state, 'crack_report', 'opposes', 'secondhand');
  if (isOpen(state)) reopen(state, 'crack_report', 'crack');
  return true;
}

function decide(state) {
  if (state.readings.length < 3) return false;
  const decision = current(state);
  if (decision && !decision.reopened) return false;
  if (state.decisions.length >= 3) return false;
  const value = Math.min(...state.readings.map(reading => reading.cm));
  const evidence = state.readings.map(reading => reading.id);
  const grounds = { evidence: lexical(evidence), caveats: lexical(evidence.flatMap(id => state.caveatsOf[id])) };
  const id = `rink@${state.decisions.length + 1}`;
  state.decisions.push({ id, value, reopened: false });
  state.grounds[id] = clone(grounds);
  state.journal.push({ decision: 'rink', commitment: id, change: 'committed', sequence: state.sequence,
    event: 'decide', elapsed: 0, value, because: state.order.filter(entry => evidence.includes(entry)),
    caveats: [...grounds.caveats] });
  return true;
}

// A task: its events, its instruments, and what its hud shows. `decide` needs
// three readings from any instrument, and the decision's value and grounds
// cover every reading of every instrument.
export function makeTask({ signatures, instruments, hud }) {
  const step = (state, record_) => {
    if (record_?.resume === true) return { accepted: true, state };
    if (!record_ || typeof record_.event !== 'string' || !Object.hasOwn(signatures, record_.event)) return { accepted: false, state };
    if (!validPayload(signatures[record_.event], record_.payload)) return { accepted: false, state };
    const next = clone(state);
    next.sequence++;
    let accepted;
    const instrument = instruments.find(entry => entry.event === record_.event);
    if (instrument) accepted = record(next, instrument, record_.payload.cm);
    else if (record_.event === 'crack') accepted = crack(next);
    else accepted = decide(next);
    return accepted ? { accepted: true, state: next } : { accepted: false, state };
  };
  const project = state => {
    const qualifications = state.order.flatMap(id => state.caveatsOf[id].map(caveat => ({ from: caveat, to: id, relation: 'qualifies' })));
    qualifications.sort(compareRelation);
    return { hud: hud(state), decision_journal: clone(state.journal), commitment_grounds: clone(state.grounds),
      observations: clone(state.observations), qualifications };
  };
  return { initial: initialState, step, project, projectActual };
}

export function thinnest(state, streams) {
  const values = state.readings.filter(reading => streams.includes(reading.stream)).map(reading => reading.cm);
  return values.length ? Math.min(...values) : -1;
}

export function decisionText(state) {
  const decision = current(state);
  if (!decision) return 'none';
  if (decision.reopened) return 'review';
  return decision.value >= THIN ? 'open' : 'closed';
}

export const phase1 = makeTask({
  signatures: { measure: { cm: [0, 60] }, crack: {}, decide: {} },
  instruments: [AUGER],
  hud: state => ({
    readings: state.readings.length,
    thinnest: thinnest(state, ['thickness']),
    cracked: state.cracked ? 1 : 0,
    decision: decisionText(state),
    frozen: current(state)?.value ?? -1,
    revision: state.decisions.length,
  }),
});

export function compareRelation(a, b) {
  for (const key of ['from', 'to', 'relation']) {
    if (a[key] < b[key]) return -1;
    if (a[key] > b[key]) return 1;
  }
  return 0;
}

// The same projection read from a runtime snapshot. Observations are the
// supports/opposes relations to `ice_safe`, in the runtime's order;
// qualifications are the `qualifies` relations on observed evidence.
export function projectActual(snapshot) {
  const triple = ({ from, to, relation }) => ({ from, to, relation });
  const relations = snapshot.relations ?? [];
  const observations = relations.filter(r => (r.relation === 'supports' || r.relation === 'opposes') && r.to === 'ice_safe').map(triple);
  const observed = new Set(observations.map(r => r.from));
  const qualifications = relations.filter(r => r.relation === 'qualifies' && observed.has(r.to)).map(triple).sort(compareRelation);
  const journal = (snapshot.decision_journal ?? []).map(entry => Object.fromEntries(
    ['decision', 'commitment', 'change', 'sequence', 'event', 'elapsed', 'value', 'because', 'caveats'].map(key => [key, entry[key]])));
  return { hud: clone(snapshot.bindings?.hud ?? null), decision_journal: journal,
    commitment_grounds: clone(snapshot.commitment_grounds ?? {}), observations, qualifications };
}
