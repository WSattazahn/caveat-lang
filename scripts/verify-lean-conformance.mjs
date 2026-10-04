import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cpSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadRuntimeFromDirectory } from '../kit/lib/node.mjs';
import { cases, renderCase, validateCase, SCHEMA, STATES, GENERATOR_SEED } from './lean-conformance-cases.mjs';
import { assertFreshRuntime } from './runtime-build-fingerprint.mjs';
import { leanRunnerDecoderCases } from './lean-runner-decoder-cases.mjs';
import { lateCases, renderLateCase, validateLateCase, LATE_SCHEMA, LATE_STATES, LATE_CAVEATS, COMMITMENTS } from './lean-late-qualification-cases.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const suffix = process.platform === 'win32' ? '.exe' : '';
const digest = value => createHash('sha256').update(value).digest('hex');
export class InfrastructureFailure extends Error {}
export class SemanticMismatch extends Error {
  constructor(caseId, runner, difference) {
    super(caseId + ': ' + runner + ' differs at ' + difference.path);
    Object.assign(this, { caseId, runner, difference });
  }
}

export function firstDifference(expected, actual, path = '') {
  if (Object.is(expected, actual)) return null;
  if (expected === null || actual === null || typeof expected !== 'object' || typeof actual !== 'object') {
    return { path: path || '/', expected, actual };
  }
  if (Array.isArray(expected) !== Array.isArray(actual)) return { path: path || '/', expected, actual };
  const left = Array.isArray(expected) ? Object.keys(expected) : Object.keys(expected).sort();
  const right = Array.isArray(actual) ? Object.keys(actual) : Object.keys(actual).sort();
  if (JSON.stringify(left) !== JSON.stringify(right)) return { path: path || '/', expected, actual };
  for (const key of left) {
    const difference = firstDifference(expected[key], actual[key], path + '/' + key.replaceAll('~', '~0').replaceAll('/', '~1'));
    if (difference) return difference;
  }
  return null;
}

function keys(value, expected, label) {
  if (!value || Array.isArray(value) || typeof value !== 'object' ||
    JSON.stringify(Object.keys(value).sort()) !== JSON.stringify([...expected].sort())) {
    throw new InfrastructureFailure('Malformed ' + label);
  }
}
function strings(value, label) {
  if (!Array.isArray(value) || value.some(item => typeof item !== 'string')) throw new InfrastructureFailure('Malformed ' + label);
  return value;
}
function provenance(value) {
  keys(value, ['evidence', 'caveats'], 'provenance');
  return Object.fromEntries(['evidence', 'caveats'].map(kind => [kind, [...new Set(strings(value[kind], kind))].sort()]));
}
function number(value) {
  if (!Number.isSafeInteger(value) || Object.is(value, -0) || Math.abs(value) > 1e9) throw new InfrastructureFailure('Numeric result outside registered exact-integer domain');
  return value;
}
// Ungrounded citation is the only classified refusal reachable in this fragment.

function outcome(value, index) {
  const kind = value?.outcome;
  keys(value, kind === 'rejected' ? ['outcome', 'origin', 'code'] : ['outcome'], 'model outcome');
  if (index === 0) {
    if (kind === 'initial') return { outcome: 'initial' };
    throw new InfrastructureFailure('Only frame zero may be initial, and it must be initial');
  }
  if (kind === 'accepted') return { outcome: 'accepted' };
  if (index > 1 && kind === 'rejected' && value.origin === 'evaluation' && value.code === 'ungrounded_citation') {
    return { outcome: 'rejected', origin: value.origin, code: value.code };
  }
  // A crash/fatal is not a killed semantic mutant in this deliberately nonfatal fragment.
  throw new InfrastructureFailure('Unexpected fatal, initial, seed refusal or malformed outcome in the registered fragment');
}

function orderedRecords(frame) {
  if (!Array.isArray(frame.decision_journal) || frame.decision_journal.length !== 0 || !Array.isArray(frame.effects)) {
    throw new InfrastructureFailure('Malformed or unsupported ordered records');
  }
  for (const effect of frame.effects) {
    keys(effect, ['kind', 'evidence'], 'reveal effect');
    if (effect.kind !== 'reveal' || !['ea', 'eb', 'eg'].includes(effect.evidence)) {
      throw new InfrastructureFailure('Unsupported effect in registered fragment');
    }
  }
}

export function modelTrace(value, fixture) {
  keys(value, ['schema', 'id', 'frames'], 'model document');
  if (value.schema !== SCHEMA || value.id !== fixture.id || !Array.isArray(value.frames) ||
    value.frames.length !== fixture.steps.length + 2) throw new InfrastructureFailure('Wrong model identity/frame count');
  return {
    schema: SCHEMA, id: fixture.id,
    frames: value.frames.map((frame, index) => {
      keys(frame, ['outcome', 'states', 'observations', 'decision_journal', 'effects'], 'model frame');
      keys(frame.states, STATES, 'model state names');
      const states = Object.fromEntries(STATES.map(name => {
        const state = frame.states[name];
        keys(state, ['value', 'lineage', 'grounds'], 'model state');
        return [name, { value: number(state.value), lineage: provenance(state.lineage), grounds: provenance(state.grounds) }];
      }));
      orderedRecords(frame);
      return { outcome: outcome(frame.outcome, index), states,
        observations: strings(frame.observations, 'observations'), decision_journal: frame.decision_journal, effects: frame.effects };
    }),
  };
}

function captureShape(record, names = STATES) {
  const snapshot = record.snapshot;
  const view = record.view;
  if (!snapshot || Array.isArray(snapshot) || snapshot.schema !== 'caveat-reactive/0.1' ||
    !view || Array.isArray(view) || view.schema !== 'caveat-reactive-view/0.1' || typeof record.save !== 'string') {
    throw new InfrastructureFailure('Malformed runtime snapshot/view/save capture');
  }
  let save;
  try { save = JSON.parse(record.save); } catch { throw new InfrastructureFailure('Malformed save JSON'); }
  if (!save || Array.isArray(save) || save.schema !== 'caveat-reactive-save/0.1' ||
    typeof snapshot.source_id !== 'string' || save.source_id !== snapshot.source_id ||
    !Number.isSafeInteger(snapshot.sequence) || snapshot.sequence < 0 ||
    save.sequence !== snapshot.sequence || view.sequence !== snapshot.sequence) {
    throw new InfrastructureFailure('Inconsistent capture identity/sequence');
  }
  keys(snapshot.values, names, 'runtime state names');
  keys(snapshot.qualified_values, names, 'runtime qualified state names');
  keys(snapshot.value_grounds, names, 'runtime grounds state names');
  for (const name of names) {
    const tracked = snapshot.qualified_values[name];
    keys(tracked, ['value', 'provenance'], 'runtime tracked state');
    if (!Object.is(number(tracked.value), number(snapshot.values[name]))) {
      throw new InfrastructureFailure('Runtime numeric projections disagree for ' + name);
    }
  }
  for (const field of ['decision_journal', 'effects']) {
    if (firstDifference(snapshot[field], view[field])) throw new InfrastructureFailure('Snapshot/view disagree on ' + field);
  }
}

function rawOutcome(record, index) {
  const value = record.outcome;
  const kind = value?.outcome;
  if (kind === 'accepted') {
    keys(value, ['schema', 'outcome', 'snapshot'], 'accepted runtime outcome');
    if (firstDifference(record.snapshot, value.snapshot)) throw new InfrastructureFailure('Accepted outcome snapshot differs from capture');
  } else if (kind === 'rejected') {
    keys(value, ['schema', 'outcome', 'origin', 'code', 'message'], 'rejected runtime outcome');
    if (typeof value.message !== 'string') throw new InfrastructureFailure('Malformed rejection message');
  } else {
    throw new InfrastructureFailure('Unexpected fatal or malformed runtime outcome');
  }
  if (value.schema !== 'caveat-dispatch/0.1') throw new InfrastructureFailure('Unexpected dispatch schema');
  return outcome(kind === 'accepted' ? { outcome: kind }
    : { outcome: kind, origin: value.origin, code: value.code }, index);
}

export function runtimeTrace(value, fixture) {
  keys(value, ['initial', 'events'], 'native document');
  keys(value.initial, ['snapshot', 'save', 'view'], 'initial runtime record');
  if (!Array.isArray(value.events) || value.events.length !== fixture.steps.length + 1) throw new InfrastructureFailure('Wrong native event count');
  const records = [value.initial, ...value.events];
  return modelTrace({
    schema: SCHEMA, id: fixture.id,
    frames: records.map((record, index) => {
      keys(record, index === 0 ? ['snapshot', 'save', 'view'] : ['snapshot', 'save', 'view', 'outcome'], 'runtime record');
      captureShape(record);
      const s = record.snapshot;
      // Neutral-only source has no ledger until seed. It must be present after
      // seed; silently defaulting a missing later ledger would hide bad output.
      if (index > 0 && !Object.hasOwn(s, 'observations')) throw new InfrastructureFailure('Missing neutral observation ledger');
      return { outcome: index === 0 ? { outcome: 'initial' } : rawOutcome(record, index),
        states: Object.fromEntries(STATES.map(name => [name, {
          value: number(s.values[name]), lineage: provenance(s.qualified_values[name].provenance),
          grounds: provenance(s.value_grounds[name]),
        }])),
        observations: Object.hasOwn(s, 'observations') ? s.observations : [], decision_journal: s.decision_journal, effects: s.effects,
      };
    }),
  }, fixture);
}

// Late qualification: one modeled step from the runtime's own state before it.
// Commitment creation is not modeled; the projection is states, commitment
// bases and grounds, observations, and an accepted step's effects.
function lateState(value, label) {
  keys(value, ['states', 'commitments', 'observations'], label);
  keys(value.states, LATE_STATES, label + ' state names');
  const states = Object.fromEntries(LATE_STATES.map(name => {
    const state = value.states[name];
    keys(state, ['value', 'lineage', 'grounds'], label + ' state');
    return [name, { value: number(state.value), lineage: provenance(state.lineage), grounds: provenance(state.grounds) }];
  }));
  const commitments = value.commitments;
  if (!commitments || Array.isArray(commitments) || typeof commitments !== 'object' ||
    Object.keys(commitments).some(name => !COMMITMENTS.includes(name))) throw new InfrastructureFailure('Malformed ' + label + ' commitments');
  return { states, observations: strings(value.observations, 'observations'),
    commitments: Object.fromEntries(Object.keys(commitments).sort().map(name => {
      const commitment = commitments[name];
      keys(commitment, ['basis', 'grounds'], label + ' commitment');
      keys(commitment.basis, ['value', 'evidence', 'caveats'], label + ' commitment basis');
      const basisValue = commitment.basis.value === null ? null : number(commitment.basis.value);
      const basis = provenance({ evidence: commitment.basis.evidence, caveats: commitment.basis.caveats });
      return [name, { basis: { value: basisValue, ...basis }, grounds: provenance(commitment.grounds) }];
    })) };
}

function lateFrame(frame, label) {
  const kind = frame?.outcome?.outcome;
  keys(frame, kind === 'accepted' ? ['outcome', 'states', 'commitments', 'observations', 'effects']
    : ['outcome', 'states', 'commitments', 'observations'], label);
  let outcomeValue;
  if (kind === 'accepted') {
    keys(frame.outcome, ['outcome'], label + ' outcome');
    outcomeValue = { outcome: 'accepted' };
  } else if (kind === 'rejected' && frame.outcome.origin === 'evaluation' && frame.outcome.code === 'unobserved_evidence') {
    keys(frame.outcome, ['outcome', 'origin', 'code'], label + ' outcome');
    outcomeValue = { outcome: 'rejected', origin: 'evaluation', code: 'unobserved_evidence' };
  } else {
    // Unobserved evidence is the only classified refusal reachable in this fragment.
    throw new InfrastructureFailure('Unexpected fatal or unregistered outcome in the late-qualification fragment');
  }
  const { states, commitments, observations } = lateState(
    { states: frame.states, commitments: frame.commitments, observations: frame.observations }, label);
  const result = { outcome: outcomeValue, states, commitments, observations };
  if (kind === 'accepted') {
    if (!Array.isArray(frame.effects)) throw new InfrastructureFailure('Malformed ' + label + ' effects');
    result.effects = frame.effects.map(effect => {
      keys(effect, ['kind', 'evidence', 'caveat'], label + ' effect');
      if (effect.kind !== 'qualify' || typeof effect.evidence !== 'string' || typeof effect.caveat !== 'string') {
        throw new InfrastructureFailure('Unsupported effect in the late-qualification fragment');
      }
      return { kind: 'qualify', evidence: effect.evidence, caveat: effect.caveat };
    });
  }
  return result;
}

export function lateModelFrame(value, fixture) {
  keys(value, ['schema', 'id', 'after'], 'late model document');
  if (value.schema !== LATE_SCHEMA || value.id !== fixture.id) throw new InfrastructureFailure('Wrong late model identity');
  return lateFrame(value.after, 'late model frame');
}

function lateRuntimeState(record) {
  captureShape(record, LATE_STATES);
  const s = record.snapshot;
  const names = Object.keys(s.commitment_bases ?? {});
  if (!s.commitment_bases || !s.commitment_grounds ||
    JSON.stringify(names.sort()) !== JSON.stringify(Object.keys(s.commitment_grounds).sort())) {
    throw new InfrastructureFailure('Runtime commitment bases and grounds disagree');
  }
  return {
    states: Object.fromEntries(LATE_STATES.map(name => [name, { value: s.values[name],
      lineage: s.qualified_values[name].provenance, grounds: s.value_grounds[name] }])),
    commitments: Object.fromEntries(names.map(name => [name, {
      basis: { value: s.commitment_bases[name].value, ...s.commitment_bases[name].provenance },
      grounds: s.commitment_grounds[name] }])),
    observations: s.observations,
  };
}

/** The runtime's projection after a qualification event. */
export function lateRuntimeFrame(record) {
  keys(record, ['snapshot', 'save', 'view', 'outcome'], 'late runtime record');
  const value = record.outcome;
  if (value?.schema !== 'caveat-dispatch/0.1') throw new InfrastructureFailure('Unexpected dispatch schema');
  if (value.outcome === 'accepted') {
    keys(value, ['schema', 'outcome', 'snapshot'], 'accepted runtime outcome');
    if (firstDifference(record.snapshot, value.snapshot)) throw new InfrastructureFailure('Accepted outcome snapshot differs from capture');
  } else if (value.outcome === 'rejected') {
    keys(value, ['schema', 'outcome', 'origin', 'code', 'message'], 'rejected runtime outcome');
  } else {
    throw new InfrastructureFailure('Unexpected fatal or malformed runtime outcome');
  }
  const state = lateRuntimeState(record);
  const frame = { ...state, outcome: value.outcome === 'accepted' ? { outcome: 'accepted' }
    : { outcome: value.outcome, origin: value.origin, code: value.code } };
  if (value.outcome === 'accepted') frame.effects = record.snapshot.effects;
  return lateFrame(frame, 'late runtime frame');
}

/** The Lean request for qualification `index`, from the runtime record before it. */
export function lateRequest(fixture, index, beforeRecord) {
  validateLateCase(fixture);
  const q = fixture.qualifications[index];
  if (!q) throw new InfrastructureFailure('Unknown qualification step');
  const before = lateState(lateRuntimeState(beforeRecord), 'late runtime state');
  return { schema: LATE_SCHEMA, id: fixture.id, before, qualification: {
    evidence: q.evidence, caveat: q.caveat, qualifiers: [...LATE_CAVEATS[q.caveat]], guard: q.guard } };
}

const commitmentMutationAnchor = '        // Current values only. Commitment bases and grounds, reading\n' +
  '        // archives and the journal record what was known then.\n';
const basisMutation = '        for basis in Arc::make_mut(&mut self.commitment_bases).values_mut() {\n' +
  '            if basis.provenance.evidence.contains(evidence) {\n' +
  '                basis.provenance.merge(&added)?;\n' +
  '            }\n' +
  '        }\n';
const commitmentMutations = {
  basis: '        // Deliberate mutation: the late caveat reaches recorded commitment bases.\n' + basisMutation,
  // Grounds must stay within the basis and the journal must repeat the grounds,
  // so this mutant rewrites all three and restore validation still accepts its
  // saves: only the comparison with the model can catch it.
  records: '        // Deliberate mutation: the late caveat reaches every commitment record.\n' + basisMutation +
    '        for (name, grounds) in Arc::make_mut(&mut self.commitment_grounds).iter_mut() {\n' +
    '            if grounds.evidence.contains(evidence) {\n' +
    '                grounds.merge(&added)?;\n' +
    '                for entry in Arc::make_mut(&mut self.journal).iter_mut() {\n' +
    '                    if entry.change == "committed" && &entry.commitment == name {\n' +
    '                        entry.caveats = grounds.caveats.iter().cloned().collect();\n' +
    '                    }\n' +
    '                }\n' +
    '            }\n' +
    '        }\n',
};
export function commitmentQualificationMutation(source, field) {
  if (!Object.hasOwn(commitmentMutations, field)) throw new InfrastructureFailure('Unregistered commitment mutation');
  const normalized = source.replaceAll('\r\n', '\n');
  if (normalized.split(commitmentMutationAnchor).length !== 2) {
    throw new InfrastructureFailure('Late-qualification mutation anchor must occur exactly once');
  }
  return normalized.replace(commitmentMutationAnchor, commitmentMutationAnchor + commitmentMutations[field]);
}

// The registered discrepancy: exactly the commitment records (bases, and for
// the records mutant grounds) that include the evidence gain the late caveat
// and its qualifiers, nothing else.
export function verifyLateMutation(id, expected, actual, qualification) {
  const difference = firstDifference(expected, actual);
  if (!difference) throw new InfrastructureFailure('Semantic mutant survived: ' + id);
  try {
    const fields = { 'qualify-reaches-commitment-basis': ['basis'],
      'qualify-reaches-commitment-records': ['basis', 'grounds'] }[id];
    if (!fields) throw new Error('Unknown semantic mutation: ' + id);
    assert.deepEqual(expected.outcome, { outcome: 'accepted' });
    const intended = structuredClone(expected);
    const added = [qualification.caveat, ...LATE_CAVEATS[qualification.caveat]];
    const reached = new Set();
    for (const commitment of Object.values(intended.commitments)) {
      for (const field of fields) {
        const target = commitment[field];
        if (!target.evidence.includes(qualification.evidence)) continue;
        const caveats = [...new Set([...target.caveats, ...added])].sort();
        if (caveats.length !== target.caveats.length) reached.add(field);
        target.caveats = caveats;
      }
    }
    assert.deepEqual([...reached].sort(), [...fields].sort(), 'the witness has a commitment record the caveat would newly reach');
    assert.deepEqual(actual, intended, 'the complete projection must have exactly the intended semantic change');
  } catch (error) {
    throw new InfrastructureFailure('Mutation did not reach its registered discrepancy: ' + id + ': ' + error.message);
  }
  return difference;
}

export function compare(expected, actual, caseId, runner) {
  const difference = firstDifference(expected, actual);
  if (difference) throw new SemanticMismatch(caseId, runner, difference);
}

const mutationAnchor = [
  '                let Some(tracked) = numbers(name)? else {',
  '                    return Ok(None);',
  '                };',
  '                let value = finite(tracked.value)?;',
  '                provenance.borrow_mut().merge(&tracked.provenance)?;',
].join('\n');

export function dependencyDropMutation(source, name = "a") {
  if (!["a", "b", "g"].includes(name)) throw new InfrastructureFailure("Unregistered metadata-drop state");
  const normalized = source.replaceAll('\r\n', '\n');
  if (normalized.split(mutationAnchor).length !== 2) throw new InfrastructureFailure('Dependency-drop mutation anchor must occur exactly once');
  return normalized.replace(mutationAnchor, mutationAnchor.replace(
    'provenance.borrow_mut().merge(&tracked.provenance)?;',
    'if name != ' + JSON.stringify(name) + ' { provenance.borrow_mut().merge(&tracked.provenance)?; }'));
}

const branchMutationAnchor = "            Node::If(condition, yes, no) => {\n                if condition\n                    .evaluate_values(numbers, predicate, qualify, history, identifiers, remaining)?\n                    .boolean()?\n                {\n                    yes.evaluate_values(\n                        numbers,\n                        predicate,\n                        qualify,\n                        history,\n                        identifiers,\n                        remaining,\n                    )\n                } else {\n                    no.evaluate_values(numbers, predicate, qualify, history, identifiers, remaining)\n                }\n            }\n";
const branchMutationReplacement = "            Node::If(condition, yes, no) => {\n                let chosen = condition\n                    .evaluate_values(numbers, predicate, qualify, history, identifiers, remaining)?\n                    .boolean()?;\n                // Deliberate mutation: evaluate both branches and retain the selected number.\n                let yes_value = yes.evaluate_values(numbers, predicate, qualify, history, identifiers, remaining)?;\n                let no_value = no.evaluate_values(numbers, predicate, qualify, history, identifiers, remaining)?;\n                if chosen { Ok(yes_value) } else { Ok(no_value) }\n            }\n";
export function unselectedBranchMutation(source) {
  const normalized = source.replaceAll('\r\n', '\n');
  if (normalized.split(branchMutationAnchor).length !== 2) {
    throw new InfrastructureFailure('Untaken-branch mutation anchor must occur exactly once');
  }
  return normalized.replace(branchMutationAnchor, branchMutationReplacement);
}

function capture(session) { return { snapshot: session.snapshot(), save: session.save(), view: session.view() }; }
export async function wasmTrace(runtime, source, events) {
  const session = runtime.open(source);
  try {
    const result = { initial: capture(session), events: [] };
    for (const event of events) {
      const before = capture(session);
      const twin = runtime.restore(source, before.save);
      try {
        assert.deepEqual(capture(twin), before, 'WASM prefix restoration');
        const dispatched = session.dispatch(event);
        const resumed = twin.dispatch(event);
        assert.deepEqual(dispatched, resumed, 'WASM continued dispatch');
        const after = capture(session);
        assert.deepEqual(capture(twin), after, 'WASM resumed state');
        if (dispatched.outcome === 'rejected') assert.deepEqual(after, before, 'WASM rejected event rollback');
        result.events.push({ outcome: dispatched, ...after });
      } finally { twin.close(); }
    }
    const final = capture(session);
    const restored = runtime.restore(source, final.save);
    try { assert.deepEqual(capture(restored), final, 'WASM final prefix restoration'); } finally { restored.close(); }
    return result;
  } catch (error) {
    throw new InfrastructureFailure('WASM capture/rollback/restore failed: ' + error.message);
  } finally { session.close(); }
}

// These are narrow assertions about the registered mutation witnesses, not an
// alternate evaluator. An unrelated mismatch never earns semantic-control credit.
export function verifyMutation(id, expected, actual) {
  const difference = firstDifference(expected, actual);
  if (!difference) throw new InfrastructureFailure('Semantic mutant survived: ' + id);
  const intended = structuredClone(expected);
  const remove = (value, evidence, caveat) => {
    value.evidence = value.evidence.filter(name => name !== evidence);
    value.caveats = value.caveats.filter(name => name !== caveat);
  };
  try {
    switch (id) {
      case 'drop-both-channels': {
        const x = intended.frames[2].states.x;
        assert.equal(intended.frames[2].outcome.outcome, 'accepted');
        assert.equal(x.value, 0, 'the accepted cancellation witness is zero');
        for (const field of ['lineage', 'grounds']) {
          assert(x[field].evidence.includes('ea') && x[field].caveats.includes('ca'));
          remove(x[field], 'ea', 'ca');
        }
        for (const kind of ['evidence', 'caveats']) {
          assert(actual.frames[2].states.x.grounds[kind].every(name => actual.frames[2].states.x.lineage[kind].includes(name)));
        }
        break;
      }
      case 'branch-condition-loss':
      case 'branch-selected-loss':
      case 'branch-unselected-injection': {
        assert.equal(intended.frames.length, 3, 'registered branch witnesses have one modeled step');
        const frame = intended.frames[2];
        assert.equal(frame.outcome.outcome, 'accepted');
        const yes = intended.frames[1].states.g.value !== 0;
        const selected = yes ? ['ea', 'ca'] : ['eb', 'cb'];
        const unused = yes ? ['eb', 'cb'] : ['ea', 'ca'];
        const x = frame.states.x;
        assert.equal(x.value, yes ? 3 : -2);
        for (const field of ['lineage', 'grounds']) {
          assert.deepEqual(x[field], { evidence: [selected[0], 'eg'].sort(), caveats: [selected[1], 'cg'].sort() });
          if (id === 'branch-unselected-injection') {
            x[field].evidence.push(unused[0]); x[field].evidence.sort();
            x[field].caveats.push(unused[1]); x[field].caveats.sort();
          } else {
            const lost = id === 'branch-condition-loss' ? ['eg', 'cg'] : selected;
            remove(x[field], ...lost);
          }
        }
        break;
      }
      case 'skip-guard-loss': {
        const x = intended.frames[3].states.x;
        assert.equal(intended.frames[3].outcome.outcome, 'accepted');
        assert(x.lineage.evidence.includes('eg') && x.lineage.caveats.includes('cg'));
        remove(x.lineage, 'eg', 'cg');
        break;
      }
      case 'guard-into-grounds': {
        const x = intended.frames[2].states.x;
        assert.equal(intended.frames[2].outcome.outcome, 'accepted');
        assert(x.lineage.evidence.includes('eg') && x.lineage.caveats.includes('cg'));
        assert(!x.grounds.evidence.includes('eg') && !x.grounds.caveats.includes('cg'));
        x.grounds.evidence = [...x.grounds.evidence, 'eg'].sort();
        x.grounds.caveats = [...x.grounds.caveats, 'cg'].sort();
        break;
      }
      case 'bypass-citation': {
        assert.deepEqual(intended.frames[2].outcome, { outcome: 'rejected', origin: 'evaluation', code: 'ungrounded_citation' });
        const admitted = structuredClone(intended.frames[1].states.a);
        admitted.grounds = { evidence: [], caveats: [] };
        intended.frames[2].outcome = { outcome: 'accepted' };
        intended.frames[2].effects = [];
        for (const frame of intended.frames.slice(2)) frame.states.x = structuredClone(admitted);
        break;
      }
      case 'reorder-observations': {
        for (const frame of intended.frames.slice(1)) {
          assert.deepEqual(frame.observations, ['eg', 'ea', 'eb']);
          frame.observations = ['ea', 'eg', 'eb'];
        }
        assert.deepEqual(intended.frames[1].effects, ['eg', 'ea', 'eb'].map(evidence => ({ kind: 'reveal', evidence })));
        intended.frames[1].effects = ['ea', 'eg', 'eb'].map(evidence => ({ kind: 'reveal', evidence }));
        break;
      }
      default: throw new Error('Unknown semantic mutation: ' + id);
    }
    assert.deepEqual(actual, intended, 'the complete projection must have exactly the intended semantic change');
  } catch (error) {
    throw new InfrastructureFailure('Mutation did not reach its registered discrepancy: ' + id + ': ' + error.message);
  }
  return difference;
}

export function assertSourceHashesUnchanged(before, after) {
  const difference = firstDifference(before, after);
  if (difference) throw new InfrastructureFailure('Conformance inputs changed during execution at ' + difference.path);
}

function filesBelow(path) {
  return readdirSync(path, { withFileTypes: true }).filter(entry => entry.name !== '.lake').flatMap(entry =>
    entry.isDirectory() ? filesBelow(join(path, entry.name)) : [join(path, entry.name)]);
}

export function checkedOutput(result, label) {
  if (result.error || result.signal || result.status !== 0) {
    throw new InfrastructureFailure("Command failed: " + label + "\n" + (result.stderr ?? result.error?.message ?? ""));
  }
  return result.stdout;
}

export function checkedDecoderRejection(result, control) {
  if (result.error || result.signal || result.status !== 1 || result.stdout !== '' ||
    typeof result.stderr !== 'string' || !result.stderr.includes(control.diagnostic)) {
    throw new InfrastructureFailure('Decoder control did not produce its registered refusal: ' + control.id);
  }
}

export async function verifyConformance({ caseId } = {}) {
  const output = join(root, 'test-results', 'lean-conformance');
  mkdirSync(output, { recursive: true });
  const report = { schema: 'caveat-conformance-report/0.1', startedAt: new Date().toISOString(),
    status: 'failed', mode: caseId ? 'single-case replay' : 'complete gate', fragment: SCHEMA, generatorSeed: GENERATOR_SEED, commands: [], cases: [], mutations: [], decoderControls: [],
    lateCases: [],
    scope: 'bounded conditional expressions and guarded assignments/citations, and one late-qualification step from the runtime\'s own state; sampled correspondence, not Rust refinement' };
  writeFileSync(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  function commandResult(command, args, options = {}) {
    const result = spawnSync(command, args, { cwd: root, encoding: 'utf8',
      timeout: 600000, maxBuffer: 32 * 1024 * 1024, ...options });
    const stem = 'command-' + String(report.commands.length + 1).padStart(3, '0');
    writeFileSync(join(output, stem + '.stdout'), result.stdout ?? '');
    writeFileSync(join(output, stem + '.stderr'), result.stderr ?? '');
    report.commands.push({ command, args,
      exitCode: result.status, signal: result.signal, error: result.error?.message,
      stdinSha256: options.input ? digest(options.input) : undefined, logs: stem });
    return result;
  }
  function run(command, args, options = {}) {
    return checkedOutput(commandResult(command, args, options), command + ' ' + args.join(' '));
  }
  function jsonCommand(command, args, input) {
    const text = run(command, args, { input: JSON.stringify(input) });
    try { return JSON.parse(text); } catch { throw new InfrastructureFailure('Runner emitted malformed JSON: ' + command); }
  }
  let sourceHashes;
  try {
    report.rustc = run('rustc', ['--version', '--verbose']).trim();
    try { report.runtimeBuild = await assertFreshRuntime(root); } catch (error) {
      throw new InfrastructureFailure('WASM build is not current: ' + error.message);
    }
    report.revision = run('git', ['rev-parse', 'HEAD']).trim();
    report.workingTree = run('git', ['status', '--short']).trim();
    sourceHashes = () => {
      const sourcePaths = [
        ...filesBelow(join(root, 'runtime', 'src')),
        ...filesBelow(join(root, 'proofs', 'lean')).filter(path => !path.includes('.lake')),
        ...filesBelow(join(root, 'kit', 'lib')),
        ...['runtime/Cargo.toml', 'runtime/Cargo.lock', 'runtime/prelude.cav',
          'runtime/examples/lean_conformance.rs', 'scripts/lean-conformance-cases.mjs',
          'scripts/verify-lean-conformance.mjs', 'scripts/verify-lean-conformance.test.mjs', 'scripts/verify-lean.mjs',
          'scripts/lean-late-qualification-cases.mjs',
          'scripts/runtime-build-fingerprint.mjs', 'scripts/build-web.mjs', 'scripts/lean-runner-decoder-cases.mjs',
          'package.json', 'rust-toolchain.toml', 'dist/pkg-reactive/caveat_runtime.js',
          'dist/pkg-reactive/caveat_runtime_bg.wasm', 'dist/build-info.json'].map(path => join(root, path)),
      ];
        return Object.fromEntries(sourcePaths.sort().map(path => [relative(root, path).replaceAll('\\', '/'), digest(readFileSync(path))]));
    };
    report.sources = sourceHashes();
    // Building/auditing the model is mandatory before executing it as an oracle.
    run(process.execPath, ['scripts/verify-lean.mjs']);
    const proofReport = readFileSync(join(root, 'test-results/lean-verification/report.json'));
    report.proofs = JSON.parse(proofReport);
    if (report.proofs.status !== 'passed') throw new InfrastructureFailure('Proof gate did not pass');
    run('cargo', ['build', '--locked', '--manifest-path', 'runtime/Cargo.toml',
      '--target-dir', join(root, 'runtime/target'), '--no-default-features', '--example', 'lean_conformance']);
    const native = join(root, 'runtime/target/debug/examples/lean_conformance' + suffix);
    const lean = join(root, 'proofs/lean/.lake/build/bin/caveat_compare' + suffix);
    report.executables = { nativeSha256: digest(readFileSync(native)), leanSha256: digest(readFileSync(lean)) };
    const decoderControls = leanRunnerDecoderCases();
    if (!Array.isArray(decoderControls) || !decoderControls.length ||
      new Set(decoderControls.map(control => control.id)).size !== decoderControls.length) {
      throw new InfrastructureFailure('Decoder controls must be nonempty and have unique IDs');
    }
    for (const control of decoderControls) {
      keys(control, ['id', 'input', 'diagnostic'], 'decoder control');
      if (typeof control.id !== 'string' || !/^[a-z][a-z0-9-]+$/.test(control.id) ||
        typeof control.input !== 'string' || typeof control.diagnostic !== 'string' || !control.diagnostic) {
        throw new InfrastructureFailure('Malformed decoder control');
      }
      const inputPath = join(output, 'decoder-' + control.id + '.json');
      writeFileSync(inputPath, control.input);
      checkedDecoderRejection(commandResult(lean, [], { input: control.input }), control);
      report.decoderControls.push({ id: control.id, inputPath: relative(root, inputPath).replaceAll('\\', '/'),
        inputSha256: digest(control.input), diagnostic: control.diagnostic,
        result: 'refused as specified', logs: report.commands.at(-1).logs });
    }
    const runtime = await loadRuntimeFromDirectory(join(root, 'dist/pkg-reactive'));
    const fullCorpus = cases();
    const corpus = caseId ? fullCorpus.filter(item => item.id === caseId) : fullCorpus;
    const fullLateCorpus = lateCases();
    const lateCorpus = caseId ? fullLateCorpus.filter(item => item.id === caseId) : fullLateCorpus;
    if (!corpus.length && !lateCorpus.length) throw new InfrastructureFailure("Unknown or empty corpus/case ID");
    if (new Set([...fullCorpus, ...fullLateCorpus].map(item => item.id)).size !== fullCorpus.length + fullLateCorpus.length) {
      throw new InfrastructureFailure('Duplicate fixture IDs');
    }
    if (new Set(corpus.map(item => item.id)).size !== corpus.length) throw new InfrastructureFailure('Duplicate fixture IDs');
    const leanResults = new Map();
    const sources = new Map();
    let accepted = 0, rejected = 0;
    for (const fixture of corpus) {
      validateCase(fixture);
      const rendered = renderCase(fixture);
      sources.set(fixture.id, rendered);
      const inputPath = join(output, fixture.id + '.input.json');
      writeFileSync(inputPath, JSON.stringify(fixture, null, 2) + '\n');
      writeFileSync(join(output, fixture.id + '.cav'), rendered.source);
      const expected = modelTrace(jsonCommand(lean, [], fixture), fixture);
      leanResults.set(fixture.id, expected);
      const nativeRaw = jsonCommand(native, [], { schema: 'caveat-native-trace/0.1', ...rendered });
      const wasmRaw = await wasmTrace(runtime, rendered.source, rendered.events);
      const nativeProjection = runtimeTrace(nativeRaw, fixture);
      const wasmProjection = runtimeTrace(wasmRaw, fixture);
      compare(expected, nativeProjection, fixture.id, 'native');
      compare(expected, wasmProjection, fixture.id, 'WASM');
      compare(nativeRaw, wasmRaw, fixture.id, 'native/WASM complete captures');
      const outcomes = expected.frames.slice(2).map(frame => frame.outcome.outcome);
      accepted += outcomes.filter(value => value === 'accepted').length;
      rejected += outcomes.filter(value => value === 'rejected').length;
      report.cases.push({ id: fixture.id, sourceSha256: digest(rendered.source),
        inputSha256: digest(JSON.stringify(fixture)), events: rendered.events, outcomes,
        replay: 'npm run verify:lean-conformance -- --case ' + fixture.id });
    }
    if (!caseId && (!accepted || !rejected)) throw new InfrastructureFailure('Corpus must exercise accepted and rejected events');
    report.events = { accepted, rejected };

    // Each qualification step: the model, given the runtime's state before the
    // step, must predict the runtime's state after it on both runtimes.
    const lateSources = new Map();
    let lateAccepted = 0, lateRejected = 0;
    async function lateTraces(fixture, rendered) {
      const nativeRaw = jsonCommand(native, [], { schema: 'caveat-native-trace/0.1', source: rendered.source, events: rendered.events });
      if (!Array.isArray(nativeRaw?.events) || nativeRaw.events.length !== rendered.events.length) {
        throw new InfrastructureFailure('Wrong native event count: ' + fixture.id);
      }
      const records = [nativeRaw.initial, ...nativeRaw.events];
      for (const record of records.slice(1, rendered.qualifyFrom + 1)) {
        if (record?.outcome?.outcome !== 'accepted') throw new InfrastructureFailure('Setup event refused: ' + fixture.id);
      }
      return { nativeRaw, records };
    }
    function lateStep(fixture, rendered, records, index) {
      const position = rendered.qualifyFrom + index + 1;
      const request = lateRequest(fixture, index, records[position - 1]);
      return { request, expected: lateModelFrame(jsonCommand(lean, [], request), fixture),
        actual: lateRuntimeFrame(records[position]) };
    }
    for (const fixture of lateCorpus) {
      validateLateCase(fixture);
      const rendered = renderLateCase(fixture);
      lateSources.set(fixture.id, rendered);
      writeFileSync(join(output, fixture.id + '.input.json'), JSON.stringify(fixture, null, 2) + '\n');
      writeFileSync(join(output, fixture.id + '.cav'), rendered.source);
      const { nativeRaw, records } = await lateTraces(fixture, rendered);
      const wasmRaw = await wasmTrace(runtime, rendered.source, rendered.events);
      compare(nativeRaw, wasmRaw, fixture.id, 'native/WASM complete captures');
      const wasmRecords = [wasmRaw.initial, ...wasmRaw.events];
      const outcomes = [];
      for (const index of fixture.qualifications.keys()) {
        const { expected, actual } = lateStep(fixture, rendered, records, index);
        compare(expected, actual, fixture.id + '/learn' + index, 'native');
        compare(expected, lateRuntimeFrame(wasmRecords[rendered.qualifyFrom + index + 1]), fixture.id + '/learn' + index, 'WASM');
        outcomes.push(expected.outcome.outcome);
      }
      lateAccepted += outcomes.filter(value => value === 'accepted').length;
      lateRejected += outcomes.filter(value => value === 'rejected').length;
      report.lateCases.push({ id: fixture.id, sourceSha256: digest(rendered.source),
        inputSha256: digest(JSON.stringify(fixture)), events: rendered.events, outcomes,
        replay: 'npm run verify:lean-conformance -- --case ' + fixture.id });
    }
    if (!caseId && (!lateAccepted || !lateRejected)) throw new InfrastructureFailure('Late corpus must exercise accepted and rejected steps');
    report.lateEvents = { accepted: lateAccepted, rejected: lateRejected };
    if (caseId) {
      report.status = "passed";
      report.summary = "Case replay passed: " + caseId + "; full-corpus mutation gate was not run.";
      return;
    }
    function semanticMutation(id, fixtureId, runner, actualRaw) {
      const fixture = corpus.find(item => item.id === fixtureId);
      const expected = leanResults.get(fixtureId);
      const actual = runtimeTrace(actualRaw, fixture);
      const difference = verifyMutation(id, expected, actual);
      report.mutations.push({ id, fixtureId, runner, difference });
      return { expected, actual };
    }

    // A genuine compiled Rust mutant drops the same state-read metadata in both
    // lineage and grounds; no output projection or shadow evaluator fabricates it.
    const mutantWorkspace = mkdtempSync(join(output, 'runtime-mutant-'));
    const mutantRoot = join(mutantWorkspace, 'runtime');
    mkdirSync(mutantRoot);
    // Compile-time source includes remain reachable even without game exports.
    for (const file of ['game/the_door_round2.cav', 'web/the_door_round2.cav']) {
      const destination = join(mutantWorkspace, file);
      mkdirSync(dirname(destination), { recursive: true });
      cpSync(join(root, file), destination);
    }
    for (const path of ['src', 'Cargo.toml', 'Cargo.lock', 'prelude.cav']) {
      cpSync(join(root, 'runtime', path), join(mutantRoot, path), { recursive: true });
    }
    mkdirSync(join(mutantRoot, 'examples'));
    cpSync(join(root, 'runtime/examples/lean_conformance.rs'), join(mutantRoot, 'examples/lean_conformance.rs'));
    // Each mutant changes exactly one file; every other mutable file is restored first.
    const originals = Object.fromEntries(['reactive_expr.rs', 'reactive.rs'].map(file =>
      [file, readFileSync(join(root, 'runtime/src', file), 'utf8')]));
    const mutantTarget = join(root, 'runtime/target/lean-mutants');
    report.runtimeMutations = [];
    function buildMutant(id, variant, file, transform) {
      for (const [name, text] of Object.entries(originals)) writeFileSync(join(mutantRoot, 'src', name), text);
      const mutationFile = join(mutantRoot, 'src', file);
      writeFileSync(mutationFile, transform(originals[file]));
      const recordedSource = join(output, 'mutation-' + variant + '.rs');
      cpSync(mutationFile, recordedSource);
      const receipt = { id, variant, file: 'runtime/src/' + file, originalSha256: digest(originals[file]),
        mutatedSha256: digest(readFileSync(mutationFile)), path: relative(root, recordedSource).replaceAll('\\', '/') };
      run('cargo', ['build', '--locked', '--manifest-path', join(mutantRoot, 'Cargo.toml'),
        '--target-dir', mutantTarget, '--no-default-features', '--example', 'lean_conformance']);
      const mutant = join(mutantTarget, 'debug/examples/lean_conformance' + suffix);
      receipt.executableSha256 = digest(readFileSync(mutant));
      report.runtimeMutations.push(receipt);
      return mutant;
    }
    function compiledMutation(id, variant, transform, witnesses) {
      const mutant = buildMutant(id, variant, 'reactive_expr.rs', transform);
      for (const fixtureId of witnesses) {
        semanticMutation(id, fixtureId, 'compiled Rust mutation / ' + variant,
          jsonCommand(mutant, [], { schema: 'caveat-native-trace/0.1', ...sources.get(fixtureId) }));
      }
    }
    compiledMutation('drop-both-channels', 'drop-eager-a', source => dependencyDropMutation(source), ['eager-cancellation']);
    compiledMutation('branch-condition-loss', 'drop-condition-g', source => dependencyDropMutation(source, 'g'), ['branch-true', 'branch-false']);
    compiledMutation('branch-selected-loss', 'drop-selected-a', source => dependencyDropMutation(source, 'a'), ['branch-true']);
    compiledMutation('branch-selected-loss', 'drop-selected-b', source => dependencyDropMutation(source, 'b'), ['branch-false']);
    compiledMutation('branch-unselected-injection', 'evaluate-unselected', unselectedBranchMutation, ['branch-true', 'branch-false']);
    // A runtime whose late qualification also reached recorded commitments must be caught.
    for (const [id, field] of [['qualify-reaches-commitment-basis', 'basis'], ['qualify-reaches-commitment-records', 'records']]) {
      const mutant = buildMutant(id, 'commitment-' + field, 'reactive.rs', source => commitmentQualificationMutation(source, field));
      for (const fixtureId of ['late-commitment-basis', 'late-guard-evidence']) {
        const fixture = lateCorpus.find(item => item.id === fixtureId);
        const rendered = lateSources.get(fixtureId);
        const raw = jsonCommand(mutant, [], { schema: 'caveat-native-trace/0.1', source: rendered.source, events: rendered.events });
        if (!Array.isArray(raw?.events)) throw new InfrastructureFailure('Malformed mutant trace: ' + fixtureId);
        const { expected, actual } = lateStep(fixture, rendered, [raw.initial, ...raw.events], 0);
        const difference = verifyLateMutation(id, expected, actual, fixture.qualifications[0]);
        report.mutations.push({ id, fixtureId, runner: 'compiled Rust mutation / commitment-' + field, difference });
      }
    }
    // Separate source-translation mutations drive the unmodified real runtime.
    const translations = [
      ['skip-guard-loss', 'skipped-guard', source => source.replace('when g != 0', 'when false')],
      ['guard-into-grounds', 'successful-guard', source => source.replace('set x = a;', 'set x = a + g - g;')],
      ['bypass-citation', 'invalid-citation', source => source.replace('because b;', 'because nothing;')],
      ['reorder-observations', 'eager-cancellation', source => source.replace(
        'on seed reveal eg;\non seed reveal ea;', 'on seed reveal ea;\non seed reveal eg;')],
    ];
    for (const [id, fixtureId, mutate] of translations) {
      const original = sources.get(fixtureId);
      const source = mutate(original.source);
      if (source === original.source) throw new InfrastructureFailure('Mutation did not apply: ' + id);
      const changed = { schema: 'caveat-native-trace/0.1', source, events: original.events };
      semanticMutation(id, fixtureId, 'source translation / native', jsonCommand(native, [], changed));
      const wasmChanged = await wasmTrace(runtime, source, original.events);
      semanticMutation(id, fixtureId, 'source translation / WASM', wasmChanged);
      writeFileSync(join(output, id + '.mutant.cav'), source);
    }
    report.status = 'passed';
    report.summary = 'Conformance passed: ' + corpus.length + ' cases, ' + accepted + ' accepted / ' + rejected +
      ' rejected modeled steps; ' + lateCorpus.length + ' late-qualification cases, ' + lateAccepted + ' accepted / ' + lateRejected +
      ' rejected qualification steps; native/WASM full captures and continuation agree; 10 semantic mutation families detected.';
  } catch (error) {
    report.error = { name: error.constructor.name, message: error.message,
      caseId: error.caseId, runner: error.runner, difference: error.difference };
    throw error;
  } finally {
    let integrityError;
    if (sourceHashes && report.sources) {
      try {
        try { await assertFreshRuntime(root); } catch (error) {
          throw new InfrastructureFailure('WASM build changed during execution: ' + error.message);
        }
        const after = sourceHashes();
        assertSourceHashesUnchanged(report.sources, after);
        report.sourceIntegrity = { checkedAt: new Date().toISOString(), unchanged: true };
      } catch (error) {
        report.sourceIntegrity = { checkedAt: new Date().toISOString(), unchanged: false, message: error.message };
        if (report.status === 'passed') integrityError = error;
        report.status = 'failed';
        report.error ??= { name: error.constructor.name, message: error.message };
      }
    }
    report.finishedAt = new Date().toISOString();
    writeFileSync(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
    if (integrityError) throw integrityError;
    if (report.status === 'passed') console.log(report.summary);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  if (args.length !== 0 && !(args.length === 2 && args[0] === '--case')) {
    console.error('Usage: node scripts/verify-lean-conformance.mjs [--case ID]'); process.exitCode = 1;
  } else {
    verifyConformance({ caseId: args[1] }).catch(error => { console.error(error); process.exitCode = 1; });
  }
}
