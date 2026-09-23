// Converts Trail Rescue's 24 frozen scenarios and Glowcap's 38 cr12 scenarios
// into Scenarios 0.1 files, as acceptance checks for the format and runner.
// The original harnesses, adapters and scenario files are read, never changed.
//
//   node experiments/scenario-conversion/convert.mjs           write the files
//   node experiments/scenario-conversion/convert.mjs --check   fail on drift
//
// The originals assert on each page adapter's projected view. Each converted
// assertion targets the raw snapshot fields that the adapter projects:
// - derived: the expected value comes from the original expectation, through
//   the adapter's naming (report_stone -> seen_report_stone, absorb_cave_2 ->
//   absorb_cave@2). Lists the adapter treats as sets become $set.
// - located: where the adapter looks something up (the current decision, the
//   observation acquired i-th, which relations bear on a claim), the location
//   is read from a reference run of the same history and asserted too, so a
//   converted file never checks less than the original.
// Expected rejections are classified from the program's declared event
// signatures: a payload outside them is an input refusal, anything else a
// policy rejection. Events the page adapter refuses before calling Caveat are
// host-input checks; they are listed in the report, not converted.
import { readFile, writeFile } from 'node:fs/promises';
import { isDeepStrictEqual } from 'node:util';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import { SCENARIOS as GLOWCAP_SCENARIOS, applies } from '../glowcap/scenarios.mjs';
import { createPolicyFromSession } from '../../web/trail-rescue-policy.js';
import * as caveat5 from '../glowcap/caveat5/adapter.mjs';
import { loadRuntimeFromDirectory } from '../../kit/lib/node.mjs';
import { formatFileReport, runScenarioFile, validateScenarioFile } from '../../kit/lib/scenarios.mjs';

const root = new URL('../../', import.meta.url);
const here = new URL('./', import.meta.url);
const TRAIL_SOURCE = 'game/trail_rescue.cav';
const GLOWCAP_SOURCE = 'experiments/glowcap/caveat5/glowcap.cav';
const TRAIL_CONTRACT = 'experiments/trail-rescue/scenarios.json';
const GLOWCAP_ADAPTER = 'experiments/glowcap/caveat5/adapter.mjs';

const namespace = await import(new URL('dist/pkg-reactive/caveat_runtime.js', root).href);
await namespace.default({ module_or_path: await readFile(new URL('dist/pkg-reactive/caveat_runtime_bg.wasm', root)) });
const { WebReactiveSession } = namespace;
await caveat5.ready;

const trailSource = await readFile(new URL(TRAIL_SOURCE, root), 'utf8');
const glowcapSource = await readFile(new URL(GLOWCAP_SOURCE, root), 'utf8');
const trailContract = JSON.parse(await readFile(new URL(TRAIL_CONTRACT, root), 'utf8'));
const sha256 = text => createHash('sha256').update(text).digest('hex');
const clone = value => JSON.parse(JSON.stringify(value));
const isObject = value => value !== null && typeof value === 'object' && !Array.isArray(value);

// The glowcap payload mapping is reproduced here; refuse to run if the adapter
// no longer contains it verbatim.
const adapterText = await readFile(new URL(GLOWCAP_ADAPTER, root), 'utf8');
for (const line of [
  "const payload = event.type === 'tick' ? { dt: event.dt } : { target: event.id, sort: event.kind };",
  'shown = JSON.parse(session.dispatch_view(event.type, JSON.stringify(payload)));',
  "const named = (evidence) => evidence.map((name) => name.replace('@', '_'));",
]) {
  if (!adapterText.includes(line)) throw new Error(`${GLOWCAP_ADAPTER} changed; review the conversion (missing: ${line})`);
}
const glowcapPayload = event => (event.type === 'tick' ? { dt: event.dt } : { target: event.id, sort: event.kind });

export class ConversionError extends Error {}
const refuse = message => { throw new ConversionError(message); };

// ------------------------------------------------------------ shared helpers

class Expectations {
  constructor(counts) { this.map = new Map(); this.counts = counts; }
  add(pointer, value, kind) {
    this.map.set(pointer, this.map.has(pointer) ? merge(this.map.get(pointer), value, pointer) : value);
    this.counts[kind] += 1;
  }
  get size() { return this.map.size; }
  step() { return { expect: Object.fromEntries(this.map) }; }
}

const isMatcher = value => isObject(value) && Object.keys(value).length === 1 && Object.keys(value)[0].startsWith('$');

function merge(a, b, at) {
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) refuse(`conflicting list lengths at ${at}`);
    return a.map((item, index) => merge(item, b[index], `${at}/${index}`));
  }
  if (isObject(a) && isObject(b) && !isMatcher(a) && !isMatcher(b)) {
    const out = { ...a };
    for (const [key, value] of Object.entries(b)) out[key] = key in out ? merge(out[key], value, `${at}/${key}`) : value;
    return out;
  }
  if (isDeepStrictEqual(a, b)) return a;
  return refuse(`conflicting expectations at ${at}`);
}

// Classify an expected rejection from the program's declared events.
export function classifyRejection(events, name, payload) {
  if (!isObject(payload)) return { origin: 'input', code: 'payload_invalid' };
  const signature = events.find(event => event.name === name);
  if (!signature) return { origin: 'input', code: 'unknown_event' };
  const expected = signature.parameters.map(parameter => parameter.name).sort();
  if (!isDeepStrictEqual(Object.keys(payload).sort(), expected)) return { origin: 'input', code: 'payload_invalid' };
  for (const parameter of signature.parameters) {
    const value = payload[parameter.name];
    const members = parameter.domain?.entity?.members ?? parameter.domain?.member?.members;
    if (members && typeof value === 'string') {
      if (!members.includes(value)) return { origin: 'input', code: 'payload_invalid' };
      continue;
    }
    if (typeof value !== 'number') return { origin: 'input', code: 'payload_invalid' };
    if (!(value >= parameter.min && value <= parameter.max)) return { origin: 'input', code: 'bound_exceeded' };
  }
  return true;
}

const pointerSegments = pointer => pointer.split('/').slice(1).map(key => key.replace(/~1/g, '/').replace(/~0/g, '~'));
const atPointer = (value, pointer) => pointerSegments(pointer).reduce((node, key) => node?.[key], value);

function multisetEqual(a, b) {
  return a.length === b.length && isDeepStrictEqual([...a].map(String).sort(), [...b].map(String).sort());
}

// ------------------------------------------------------------ Trail Rescue

const trailProbe = new WebReactiveSession(trailSource);
const trailSnapshot = JSON.parse(trailProbe.snapshot());
trailProbe.free();
const trailEntities = trailSnapshot.world.entities;
const TUNNELS = trailEntities.filter(entity => entity.kind === 'tunnel').map(entity => entity.id);
const OBSERVATIONS = trailEntities.filter(entity => entity.kind === 'observation').map(entity => entity.id);
const TRAIL_CAVEATS = ['secondhand', 'stale'];
{
  // The adapter keeps only these caveats; that filter changes nothing only if
  // the program declares no others.
  const declared = trailSnapshot.symbols.filter(symbol => symbol.kind === 'caveat').map(symbol => symbol.name);
  if (!multisetEqual(declared, TRAIL_CAVEATS)) refuse(`trail_rescue.cav declares caveats ${declared}`);
  if (!multisetEqual(Object.keys(trailSnapshot.decision_series), ['route'])) refuse('trail_rescue.cav has decision series other than route');
}
const TRAIL_EVIDENCE = new Set(trailSnapshot.symbols.filter(symbol => symbol.kind === 'evidence').map(symbol => symbol.name));
const seen = id => (OBSERVATIONS.includes(id) && TRAIL_EVIDENCE.has(`seen_${id}`) ? `seen_${id}` : refuse(`unknown observation ${JSON.stringify(id)}`));
const tunnelValue = tunnel => (TUNNELS.includes(tunnel) ? TUNNELS.indexOf(tunnel) + 1 : refuse(`unknown tunnel ${JSON.stringify(tunnel)}`));
const caveatSet = values => {
  if (!Array.isArray(values)) refuse('caveats must be a list');
  return { $set: values };
};

function trailHistoryEntry(entry) {
  const extra = Object.keys(entry).filter(key => !['change', 'tunnel', 'at', 'because', 'caveats'].includes(key));
  if (extra.length) refuse(`unexpected history fields ${extra}`);
  const mapped = { decision: 'route' };
  if ('change' in entry) mapped.change = entry.change;
  if ('tunnel' in entry) mapped.value = tunnelValue(entry.tunnel);
  if ('at' in entry) mapped.elapsed = entry.at;
  if ('because' in entry) mapped.because = entry.because.map(seen);
  if ('caveats' in entry) mapped.caveats = caveatSet(entry.caveats);
  return mapped;
}

function trailDecisionLocation(ex, raw) {
  const current = raw.decision_series.route.current;
  ex.add('/decision_series/route/current', current, 'located');
  ex.add('/decision_journal', raw.decision_journal.map(entry => ({ commitment: entry.commitment, change: entry.change })), 'located');
  const find = change => raw.decision_journal.findIndex(entry => entry.commitment === current && entry.change === change);
  return { committed: current === null ? -1 : find('committed'), reopened: current === null ? -1 : find('reopened') };
}

function trailObservationAt(ex, raw, index) {
  const id = OBSERVATIONS.find(candidate => raw.bindings[candidate].observed && raw.bindings[candidate].order === index + 1);
  if (!id) refuse(`no observation acquired at position ${index + 1}`);
  ex.add(`/bindings/${id}/observed`, true, 'located');
  ex.add(`/bindings/${id}/order`, index + 1, 'located');
  return id;
}

function trailEvidenceField(ex, id, field, value) {
  if (['tunnel', 'method', 'condition', 'at'].includes(field)) ex.add(`/bindings/${id}/${field}`, value, 'derived');
  else if (field === 'caveats') ex.add(`/binding_explanations/${id}/condition/caveats`, caveatSet(value), 'derived');
  else refuse(`unexpected evidence field ${field}`);
}

function mapTrail(ex, pointer, value, raw) {
  const segments = pointerSegments(pointer);
  const [head] = segments;
  if (!segments.length) {
    for (const [key, item] of Object.entries(value)) mapTrail(ex, `/${key}`, item, raw);
    return;
  }
  if (['now', 'scoutsRemaining', 'outcome', 'canRescue'].includes(head) && segments.length === 1) {
    ex.add(`/bindings/hud/${head}`, value, 'derived');
  } else if (head === 'tunnels') {
    if (segments.length === 1) {
      if (!multisetEqual(Object.keys(value), TUNNELS)) refuse('tunnel list differs from the program');
      for (const [tunnel, item] of Object.entries(value)) mapTrail(ex, `/tunnels/${tunnel}`, item, raw);
    } else if (segments.length === 2) {
      for (const [field, item] of Object.entries(value)) mapTrail(ex, `/tunnels/${segments[1]}/${field}`, item, raw);
    } else if (segments.length === 3) {
      const [, tunnel, field] = segments;
      if (!TUNNELS.includes(tunnel)) refuse(`unknown tunnel ${tunnel}`);
      const cites = { clearBy: 'clearCount', blockedBy: 'blockedCount', because: 'status' };
      if (['status', 'canScout', 'canReport', 'canPlan'].includes(field)) ex.add(`/bindings/${tunnel}/${field}`, value, 'derived');
      else if (field in cites) ex.add(`/binding_explanations/${tunnel}/${cites[field]}/evidence`, { $set: value.map(seen) }, 'derived');
      else if (field === 'caveats') ex.add(`/binding_explanations/${tunnel}/status/caveats`, caveatSet(value), 'derived');
      else refuse(`unexpected tunnel field ${field}`);
    } else refuse(`unsupported pointer ${pointer}`);
  } else if (head === 'evidence') {
    if (segments.length === 1) {
      const ids = value.map(item => item.id);
      for (const id of OBSERVATIONS) {
        ex.add(`/bindings/${id}/observed`, ids.includes(id), 'derived');
        if (ids.includes(id)) ex.add(`/bindings/${id}/order`, ids.indexOf(id) + 1, 'derived');
      }
      value.forEach(item => { for (const [field, fieldValue] of Object.entries(item)) if (field !== 'id') trailEvidenceField(ex, item.id, field, fieldValue); });
    } else {
      const index = Number(segments[1]);
      if (segments.length === 2) {
        let id = value.id;
        if (id === undefined) id = trailObservationAt(ex, raw, index);
        else { seen(id); ex.add(`/bindings/${id}/observed`, true, 'derived'); ex.add(`/bindings/${id}/order`, index + 1, 'derived'); }
        for (const [field, fieldValue] of Object.entries(value)) if (field !== 'id') trailEvidenceField(ex, id, field, fieldValue);
      } else if (segments.length === 3) {
        if (segments[2] === 'id') {
          seen(value);
          ex.add(`/bindings/${value}/observed`, true, 'derived');
          ex.add(`/bindings/${value}/order`, index + 1, 'derived');
        } else {
          trailEvidenceField(ex, trailObservationAt(ex, raw, index), segments[2], value);
        }
      } else refuse(`unsupported pointer ${pointer}`);
    }
  } else if (head === 'decision') {
    if (segments.length === 1) {
      for (const [field, item] of Object.entries(value)) mapTrail(ex, `/decision/${field}`, item, raw);
      return;
    }
    const field = segments[1];
    if (field === 'state' && segments.length === 2) ex.add('/bindings/decision/state', value, 'derived');
    else if (field === 'history' && segments.length === 2) ex.add('/decision_journal', value.map(trailHistoryEntry), 'derived');
    else if (field === 'history' && segments.length === 3) ex.add(`/decision_journal/${Number(segments[2])}`, trailHistoryEntry(value), 'derived');
    else if (['tunnel', 'basis', 'caveats', 'reopenedBy'].includes(field) && segments.length === 2) {
      const { committed, reopened } = trailDecisionLocation(ex, raw);
      const entry = field === 'reopenedBy' ? reopened : committed;
      const empty = field === 'tunnel' ? value === null : value.length === 0;
      if (entry === -1) { if (!empty) refuse(`no journal entry for ${field}`); return; }
      if (field === 'tunnel') ex.add(`/decision_journal/${entry}/value`, value === null ? refuse('tunnel is null but a basis exists') : tunnelValue(value), 'derived');
      else if (field === 'caveats') ex.add(`/decision_journal/${entry}/caveats`, caveatSet(value), 'derived');
      else ex.add(`/decision_journal/${entry}/because`, value.map(seen), 'derived');
    } else refuse(`unsupported pointer ${pointer}`);
  } else {
    refuse(`unsupported pointer ${pointer}`);
  }
}

function trailRecorder() {
  const log = { last: null, session: null };
  class Recorder {
    constructor(source, inner) {
      this.inner = inner ?? new WebReactiveSession(source);
      log.session = this;
    }
    static restore(source, saved) { return new Recorder(source, WebReactiveSession.restore(source, saved)); }
    snapshot() { return this.inner.snapshot(); }
    view() { return this.inner.view(); }
    save() { return this.inner.save(); }
    free() { this.inner.free(); }
    dispatch_view(event, payload) {
      log.last = { event, payload };
      return this.inner.dispatch_view(event, payload);
    }
  }
  return { Recorder, log };
}

export function convertTrailScenario(scenario, { initialView = trailContract.initialView, mutate = false } = {}) {
  const counts = { derived: 0, located: 0 };
  const hostOnly = [];
  const steps = [];
  const { Recorder, log } = trailRecorder();
  let policy = createPolicyFromSession(Recorder, trailSource);
  const policies = [policy];
  const events = JSON.parse(log.session.snapshot()).events;
  const expectStep = expectations => {
    const ex = new Expectations(counts);
    const raw = JSON.parse(log.session.snapshot());
    for (const [pointer, value] of expectations) {
      if (!mutate && !isDeepStrictEqual(pointer ? atPointer(policy.view(), pointer) : policy.view(), value)) {
        refuse(`${scenario.id}: the original expectation ${pointer || '(view)'} does not hold on the reference run`);
      }
      mapTrail(ex, pointer, value, raw);
    }
    if (ex.size) steps.push(ex.step());
  };
  try {
    expectStep([['', initialView]]);
    scenario.steps.forEach((step, index) => {
      if (step.op === 'resume') {
        policy = createPolicyFromSession(Recorder, trailSource, clone(policy.save()));
        policies.push(policy);
        steps.push({ resume: true });
      } else if (step.op === 'dispatch') {
        log.last = null;
        let refusal = null;
        try { policy.dispatch(step.event); } catch (error) { refusal = error; }
        if (!mutate && Boolean(refusal) !== Boolean(step.reject)) refuse(`${scenario.id} step ${index + 1}: reference outcome differs from the original`);
        if (!log.last) {
          if (!step.reject) refuse(`${scenario.id} step ${index + 1}: accepted without reaching Caveat`);
          hostOnly.push({ scenario: scenario.id, step: index + 1, event: step.event, adapter: String(refusal?.message ?? refusal) });
        } else {
          const send = { send: log.last.event, payload: JSON.parse(log.last.payload) };
          if (step.reject) send.rejected = classifyRejection(events, send.send, send.payload);
          steps.push(send);
        }
      } else refuse(`unknown step ${step.op}`);
      if (step.expect) expectStep(Object.entries(step.expect));
    });
  } finally {
    for (const item of policies) item.free();
  }
  return { scenario: { id: scenario.id, title: scenario.title, steps }, counts, hostOnly };
}

// ------------------------------------------------------------ Glowcap

// Glowcap's harness comparison: listed members only; lists compared after
// sorting, which makes lists of names sets and keeps lists of objects in order.
export function glowcapCompare(actual, expected, at = '', failures = []) {
  for (const [key, want] of Object.entries(expected)) {
    const got = actual?.[key];
    const where = at ? `${at}.${key}` : key;
    if (Array.isArray(want)) {
      if (!(Array.isArray(got) && JSON.stringify([...got].sort()) === JSON.stringify([...want].sort()))) failures.push(where);
    } else if (want && typeof want === 'object') {
      glowcapCompare(got, want, where, failures);
    } else if (got !== want) {
      failures.push(where);
    }
  }
  return failures;
}

const glowcapProbe = new WebReactiveSession(glowcapSource);
const glowcapSnapshot = JSON.parse(glowcapProbe.snapshot());
glowcapProbe.free();
if (!multisetEqual(Object.keys(glowcapSnapshot.decision_series), ['trust'])) refuse('glowcap.cav has decision series other than trust');
const MUSHROOMS = glowcapSnapshot.world.entities.filter(entity => entity.kind === 'mushroom').map(entity => entity.id);
const CLAIM = 'glowing_is_safe';

// The adapter shows absorb_cave@2 as absorb_cave_2. Invert through the
// program's declared evidence names and that occurrence rule, falling back to
// names present in the reference snapshot, and refuse anything ambiguous.
const GLOWCAP_EVIDENCE = new Set(glowcapSnapshot.symbols.filter(symbol => symbol.kind === 'evidence').map(symbol => symbol.name));
function glowcapNames(raw) {
  const seenNames = new Map();
  const visit = value => {
    if (typeof value === 'string') {
      if (/^[a-z][a-z0-9_]*(@\d+)?$/.test(value)) {
        const shown = value.replace('@', '_');
        if (seenNames.has(shown) && seenNames.get(shown) !== value) refuse(`ambiguous name ${shown}`);
        seenNames.set(shown, value);
      }
    } else if (Array.isArray(value)) value.forEach(visit);
    else if (isObject(value)) Object.values(value).forEach(visit);
  };
  visit([raw.binding_explanations, raw.relations, raw.decision_journal, raw.commitments, raw.commitment_grounds]);
  return shown => {
    const occurrence = /^(.+)_(\d+)$/.exec(shown);
    const candidates = new Set();
    if (GLOWCAP_EVIDENCE.has(shown)) candidates.add(shown);
    if (occurrence && GLOWCAP_EVIDENCE.has(occurrence[1])) candidates.add(`${occurrence[1]}@${occurrence[2]}`);
    if (seenNames.has(shown)) candidates.add(seenNames.get(shown));
    if (candidates.size > 1) refuse(`ambiguous name ${JSON.stringify(shown)}`);
    return candidates.size ? [...candidates][0] : refuse(`unknown name ${JSON.stringify(shown)}`);
  };
}

function glowcapLeaves(value, path = []) {
  if (isObject(value)) return Object.entries(value).flatMap(([key, item]) => glowcapLeaves(item, [...path, key]));
  return [[path, value]];
}

function mapGlowcap(ex, path, value, raw, raw2name) {
  const [head, second, third, fourth, fifth] = path;
  const names = list => (Array.isArray(list) ? list.map(raw2name) : refuse(`${path.join('.')} must be a list`));
  const set = list => (Array.isArray(list) ? { $set: list } : refuse(`${path.join('.')} must be a list`));
  if (head === 'slime' && path.length === 2) {
    ex.add(`/bindings/slime/${second}`, value, 'derived');
  } else if (head === 'mushrooms' && MUSHROOMS.includes(second)) {
    const why = { absorb: 'whyAbsorb', taste: 'whyTaste' };
    if (['label', 'present', 'canAbsorb', 'canTaste'].includes(third) && path.length === 3) ex.add(`/bindings/${second}/${third}`, value, 'derived');
    else if (third === 'because' && path.length === 3) ex.add(`/binding_explanations/${second}/label/evidence`, { $set: names(value) }, 'derived');
    else if (third === 'caveats' && path.length === 3) ex.add(`/binding_explanations/${second}/label/caveats`, set(value), 'derived');
    else if (third === 'why' && fourth in why && path.length === 5) {
      if (fifth === 'reason') ex.add(`/bindings/${second}/${why[fourth]}`, value, 'derived');
      else if (fifth === 'because') ex.add(`/binding_explanations/${second}/${why[fourth]}/evidence`, { $set: names(value) }, 'derived');
      else if (fifth === 'caveats') ex.add(`/binding_explanations/${second}/${why[fourth]}/caveats`, set(value), 'derived');
      else refuse(`unsupported ${path.join('.')}`);
    } else refuse(`unsupported ${path.join('.')}`);
  } else if (head === 'belief' && path.length === 2) {
    if (['state', 'note', 'text'].includes(second)) ex.add(`/bindings/belief/${second}`, value, 'derived');
    else if (second === 'caveats') ex.add('/binding_explanations/belief/state/caveats', set(value), 'derived');
    else if (second === 'supportedBy' || second === 'contradictedBy') {
      const relation = second === 'supportedBy' ? 'supports' : 'opposes';
      ex.add('/relations', raw.relations.map(item => ({ relation: item.relation, to: item.to })), 'located');
      const bearing = raw.relations.map((item, index) => [item, index]).filter(([item]) => item.relation === relation && item.to === CLAIM);
      const want = names(value);
      if (!multisetEqual(want, bearing.map(([item]) => item.from))) refuse(`${second} differs from the reference relations`);
      // Same multiset as expected: assert each bearing relation's source in
      // the order the relations were added.
      for (const [item, index] of bearing) ex.add(`/relations/${index}/from`, item.from, 'derived');
    } else refuse(`unsupported ${path.join('.')}`);
  } else if (head === 'decision' && path.length === 2) {
    const current = raw.decision_series.trust.current;
    if (second === 'state') ex.add('/bindings/decision/state', value, 'derived');
    else if (second === 'history') {
      ex.add('/decision_journal', value.map(entry => {
        const extra = Object.keys(entry).filter(key => !['change', 'because'].includes(key));
        if (extra.length) refuse(`unexpected history fields ${extra}`);
        return { decision: 'trust', change: entry.change, because: names(entry.because) };
      }), 'derived');
    } else if (second === 'basis') {
      ex.add('/decision_series/trust/current', current, 'located');
      ex.add('/decision_journal', raw.decision_journal.map(entry => ({ commitment: entry.commitment, change: entry.change })), 'located');
      const index = current === null ? -1 : raw.decision_journal.findIndex(entry => entry.commitment === current && entry.change === 'committed');
      if (index === -1) { if (value.length) refuse('basis without a committed entry'); return; }
      ex.add(`/decision_journal/${index}/because`, { $set: names(value) }, 'derived');
    } else if (second === 'reopenedBy') {
      ex.add('/decision_series/trust/current', current, 'located');
      ex.add('/commitments', raw.commitments.map(item => ({ action: item.action })), 'located');
      const index = raw.commitments.findIndex(item => item.action === current);
      if (index === -1) { if (value.length) refuse('reopenedBy without a commitment'); return; }
      ex.add(`/commitments/${index}/reopened_by`, { $set: names(value) }, 'derived');
    } else if (second === 'caveats') {
      ex.add('/decision_series/trust/current', current, 'located');
      if (current === null || !raw.commitment_grounds[current]) {
        if (value.length) refuse('caveats without grounds');
        if (current !== null) ex.add(`/commitment_grounds/${current}`, { $absent: true }, 'located');
        return;
      }
      ex.add(`/commitment_grounds/${current}/caveats`, set(value), 'derived');
    } else refuse(`unsupported ${path.join('.')}`);
  } else {
    refuse(`unsupported ${path.join('.')}`);
  }
}

export function convertGlowcapScenario(scenario, { mutate = false } = {}) {
  const counts = { derived: 0, located: 0 };
  const steps = [];
  let policy = caveat5.createPolicy();
  let raw = new WebReactiveSession(glowcapSource);
  const events = glowcapSnapshot.events;
  const dispatchBoth = event => {
    let refused = false;
    try { policy.dispatch(event); } catch { refused = true; }
    const outcome = JSON.parse(raw.dispatch_outcome(event.type, JSON.stringify(glowcapPayload(event))));
    if ((outcome.outcome === 'rejected') !== refused) refuse(`${scenario.id}: adapter and raw session disagree`);
    return refused;
  };
  try {
    scenario.steps.forEach((step, index) => {
      const [kind] = step;
      if (kind === 'expect') {
        if (!mutate && glowcapCompare(policy.view(), step[1]).length) refuse(`${scenario.id} step ${index + 1}: the original expectation does not hold on the reference run`);
        const snapshot = JSON.parse(raw.snapshot());
        const ex = new Expectations(counts);
        const names = glowcapNames(snapshot);
        for (const [path, value] of glowcapLeaves(step[1])) mapGlowcap(ex, path, value, snapshot, names);
        steps.push(ex.step());
      } else if (kind === 'resume') {
        steps.push({ size: { save: { max: 4096 } } }, { resume: true });
        const next = caveat5.createPolicy(JSON.parse(JSON.stringify(policy.save())));
        policy.free();
        policy = next;
        const restored = WebReactiveSession.restore(glowcapSource, raw.save());
        raw.free();
        raw = restored;
      } else if (kind === 'reject') {
        const event = step[1];
        const payload = glowcapPayload(event);
        const refused = dispatchBoth(event);
        if (!mutate && !refused) refuse(`${scenario.id} step ${index + 1}: expected rejection was accepted`);
        steps.push({ send: event.type, payload, rejected: classifyRejection(events, event.type, payload) });
      } else {
        const event = kind === 'tick' ? { type: 'tick', dt: step[1] } : { type: kind, id: step[1], kind: step[2] };
        const times = kind === 'tick' ? step[2] ?? 1 : 1;
        for (let repetition = 0; repetition < times; repetition++) {
          if (dispatchBoth(event) && !mutate) refuse(`${scenario.id} step ${index + 1}: event was rejected`);
        }
        const send = { send: event.type, payload: glowcapPayload(event) };
        if (times > 1) send.repeat = times;
        steps.push(send);
      }
    });
  } finally {
    policy.free();
    raw.free();
  }
  return { scenario: { id: scenario.id, title: scenario.title, steps }, counts };
}

// ------------------------------------------------------------ files

export const GLOWCAP_PHASE = 'cr12';
export const glowcapScenarios = () => GLOWCAP_SCENARIOS.filter(scenario => applies(scenario, GLOWCAP_PHASE));
export const trailScenarios = () => trailContract.scenarios;

export function buildTrail() {
  const converted = trailScenarios().map(scenario => convertTrailScenario(scenario));
  const doc = {
    schema: 'caveat-scenarios/0.1',
    source: `../../${TRAIL_SOURCE}`,
    note: `Generated by experiments/scenario-conversion/convert.mjs from ${TRAIL_CONTRACT}. Do not edit.`,
    scenarios: converted.map(item => item.scenario),
  };
  return { doc: validateScenarioFile(doc), converted };
}

export function buildGlowcap() {
  const converted = glowcapScenarios().map(scenario => convertGlowcapScenario(scenario));
  const doc = {
    schema: 'caveat-scenarios/0.1',
    source: `../../${GLOWCAP_SOURCE}`,
    note: `Generated by experiments/scenario-conversion/convert.mjs from experiments/glowcap/scenarios.mjs, phase ${GLOWCAP_PHASE}. Do not edit.`,
    scenarios: converted.map(item => item.scenario),
  };
  return { doc: validateScenarioFile(doc), converted };
}

export const FILES = { trail: 'trail-rescue.scenarios.json', glowcap: 'glowcap-cr12.scenarios.json' };
export const text = doc => `${JSON.stringify(doc, null, 2)}\n`;

export async function runConverted(doc, runtime) {
  return runScenarioFile(doc, {
    runtime,
    readSource: async relative => readFile(new URL(relative, here), 'utf8'),
  });
}

async function main() {
  const check = process.argv.includes('--check');
  const trail = buildTrail();
  const glowcap = buildGlowcap();
  const runtime = await loadRuntimeFromDirectory();
  const suites = {};
  let drift = false;
  let failed = false;
  for (const [key, built, from] of [['trail', trail, TRAIL_CONTRACT], ['glowcap', glowcap, 'experiments/glowcap/scenarios.mjs']]) {
    const file = new URL(FILES[key], here);
    const generated = text(built.doc);
    if (check) {
      let existing = null;
      try { existing = await readFile(file, 'utf8'); } catch { /* missing */ }
      if (existing !== generated) { drift = true; console.error(`DRIFT ${FILES[key]} differs from a fresh conversion`); }
    } else {
      await writeFile(file, generated);
    }
    const result = await runConverted(built.doc, runtime);
    if (result.failed) failed = true;
    console.log(formatFileReport({ ...result, file: FILES[key] }));
    const totals = built.converted.reduce((sum, item) => ({ derived: sum.derived + item.counts.derived, located: sum.located + item.counts.located }), { derived: 0, located: 0 });
    suites[key] = {
      file: FILES[key],
      from,
      scenarios: built.doc.scenarios.length,
      events: result.scenarios.reduce((sum, item) => sum + item.events, 0),
      rejected: result.scenarios.reduce((sum, item) => sum + item.rejected, 0),
      resumes: result.scenarios.reduce((sum, item) => sum + item.resumes, 0),
      assertions: totals,
      hostOnlySteps: built.converted.flatMap(item => item.hostOnly ?? []),
      passed: result.passed,
      failed: result.failed,
    };
  }
  const report = {
    schema: 1,
    runtime: runtime.identity,
    sources: {
      [TRAIL_SOURCE]: sha256(trailSource), [TRAIL_CONTRACT]: sha256(JSON.stringify(trailContract)),
      [GLOWCAP_SOURCE]: sha256(glowcapSource), [GLOWCAP_ADAPTER]: sha256(adapterText),
    },
    suites,
  };
  if (!check) await writeFile(new URL('conversion-report.json', here), text(report));
  if (drift || failed) process.exitCode = 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();
