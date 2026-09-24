// Independent model of the ferry task. It imports nothing from the runtime and
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

// Instruments: the event and its parameter, the reading stream, the caveat its
// evidence carries, the per-session cap, the largest reading that still
// supports a safe crossing, and whether its readings are wind (the decision's
// value and grounds).
export const MAST = { event: 'gust', param: 'kt', stream: 'wind', caveat: 'sheltered_mast', cap: 10, safe: 30, wind: true };
export const BUOY = { event: 'wave', param: 'm', stream: 'swell', caveat: 'buoy_drift', cap: 10, safe: 1.5, wind: false };
export const CALM = 30;
export const DECISIONS = 4;

export function initialState() {
  return { sequence: 0, readings: [], order: [], caveatsOf: {}, observations: [], warned: false,
    decisions: [], journal: [], grounds: {} };
}

function observe(state, id, relation, caveat) {
  state.order.push(id);
  state.caveatsOf[id] = [caveat];
  state.observations.push({ from: id, to: 'crossing_safe', relation });
}

export function current(state) { return state.decisions.at(-1) ?? null; }
function mayGo(state) {
  const decision = current(state);
  return decision !== null && !decision.reopened && decision.value <= CALM;
}

function reopen(state, because, event) {
  const decision = current(state);
  decision.reopened = true;
  state.journal.push({ decision: 'crossing', commitment: decision.id, change: 'reopened', sequence: state.sequence,
    event, elapsed: 0, value: decision.value, because: [because], caveats: [...state.caveatsOf[because]] });
}

export function record(state, instrument, value) {
  const count = state.readings.filter(reading => reading.stream === instrument.stream).length;
  if (count >= instrument.cap) return false;
  const id = `${instrument.stream}@${count + 1}`;
  state.readings.push({ id, stream: instrument.stream, wind: instrument.wind, value });
  const safe = value <= instrument.safe;
  observe(state, id, safe ? 'supports' : 'opposes', instrument.caveat);
  if (!safe && mayGo(state)) reopen(state, id, instrument.event);
  return true;
}

function warning(state) {
  if (state.warned) return false;
  state.warned = true;
  observe(state, 'storm_warning', 'opposes', 'regional_forecast');
  if (mayGo(state)) reopen(state, 'storm_warning', 'warning');
  return true;
}

function decide(state) {
  const winds = state.readings.filter(reading => reading.wind);
  const waves = state.readings.filter(reading => reading.stream === 'swell');
  if (winds.length < 2 || waves.length < 2) return false;
  const decision = current(state);
  if (decision && !decision.reopened) return false;
  if (state.decisions.length >= DECISIONS) return false;
  const value = Math.max(...winds.map(reading => reading.value));
  const evidence = winds.map(reading => reading.id);
  const grounds = { evidence: lexical(evidence), caveats: lexical(evidence.flatMap(id => state.caveatsOf[id])) };
  const id = `crossing@${state.decisions.length + 1}`;
  state.decisions.push({ id, value, reopened: false });
  state.grounds[id] = clone(grounds);
  state.journal.push({ decision: 'crossing', commitment: id, change: 'committed', sequence: state.sequence,
    event: 'decide', elapsed: 0, value, because: state.order.filter(entry => evidence.includes(entry)),
    caveats: [...grounds.caveats] });
  return true;
}

// A task: its events, its instruments, and what its hud shows.
// `consequences` names every caveat the task declares, with its consequence;
// the program must declare each of them so.
export function makeTask({ signatures, instruments, hud, consequences }) {
  const step = (state, record_) => {
    if (record_?.resume === true) return { accepted: true, state };
    if (!record_ || typeof record_.event !== 'string' || !Object.hasOwn(signatures, record_.event)) return { accepted: false, state };
    if (!validPayload(signatures[record_.event], record_.payload)) return { accepted: false, state };
    const next = clone(state);
    next.sequence++;
    let accepted;
    const instrument = instruments.find(entry => entry.event === record_.event);
    if (instrument) accepted = record(next, instrument, record_.payload[instrument.param]);
    else if (record_.event === 'warning') accepted = warning(next);
    else accepted = decide(next);
    return accepted ? { accepted: true, state: next } : { accepted: false, state };
  };
  const project = state => {
    const qualifications = state.order.flatMap(id => state.caveatsOf[id].map(caveat => ({ from: caveat, to: id, relation: 'qualifies' })));
    qualifications.sort(compareRelation);
    return { hud: hud(state), decision_journal: clone(state.journal), commitment_grounds: clone(state.grounds),
      observations: clone(state.observations), qualifications, consequences: clone(consequences) };
  };
  return { initial: initialState, step, project, projectActual: snapshot => projectActual(snapshot, Object.keys(consequences)) };
}

export function count(state, streams) {
  return state.readings.filter(reading => streams.includes(reading.stream)).length;
}

export function largest(state, streams) {
  const values = state.readings.filter(reading => streams.includes(reading.stream)).map(reading => reading.value);
  return values.length ? Math.max(...values) : -1;
}

export function decisionText(state) {
  const decision = current(state);
  if (!decision) return 'none';
  if (decision.reopened) return 'review';
  return decision.value <= CALM ? 'go' : 'hold';
}

export const phase1 = makeTask({
  signatures: { gust: { kt: [0, 90] }, wave: { m: [0, 6] }, warning: {}, decide: {} },
  instruments: [MAST, BUOY],
  consequences: { sheltered_mast: 'material', buoy_drift: 'low', regional_forecast: 'low' },
  hud: state => ({
    gusts: count(state, ['wind']),
    waves: count(state, ['swell']),
    strongest: largest(state, ['wind']),
    highest: largest(state, ['swell']),
    warned: state.warned ? 1 : 0,
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
// supports/opposes relations to `crossing_safe`, in the runtime's order;
// qualifications are the `qualifies` relations on observed evidence;
// consequences are those of the task's caveats, as declared.
export function projectActual(snapshot, caveats) {
  const triple = ({ from, to, relation }) => ({ from, to, relation });
  const relations = snapshot.relations ?? [];
  const observations = relations.filter(r => (r.relation === 'supports' || r.relation === 'opposes') && r.to === 'crossing_safe').map(triple);
  const observed = new Set(observations.map(r => r.from));
  const qualifications = relations.filter(r => r.relation === 'qualifies' && observed.has(r.to)).map(triple).sort(compareRelation);
  const journal = (snapshot.decision_journal ?? []).map(entry => Object.fromEntries(
    ['decision', 'commitment', 'change', 'sequence', 'event', 'elapsed', 'value', 'because', 'caveats'].map(key => [key, entry[key]])));
  const declared = new Map((snapshot.symbols ?? []).filter(symbol => symbol.kind === 'caveat').map(symbol => [symbol.name, symbol.consequence]));
  const consequences = Object.fromEntries(caveats.map(name => [name, declared.get(name) ?? null]));
  return { hud: clone(snapshot.bindings?.hud ?? null), decision_journal: journal,
    commitment_grounds: clone(snapshot.commitment_grounds ?? {}), observations, qualifications, consequences };
}
