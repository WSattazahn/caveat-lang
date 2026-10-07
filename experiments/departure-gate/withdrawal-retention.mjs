// Design probe of the CURRENT runtime, not an implementation of collection.
// node experiments/departure-gate/withdrawal-retention.mjs --runtime=dist/pkg-reactive --out=test-results/withdrawal-retention.json
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { loadRuntimeFromDirectory } from '../../kit/lib/node.mjs';
import { pinnedRecords } from './pin-census.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const option = (name, fallback) => process.argv.find((arg) => arg.startsWith(`--${name}=`))?.slice(name.length + 3) ?? fallback;
const runtimeDirectory = path.resolve(option('runtime', 'dist/pkg-reactive'));
const runtime = await loadRuntimeFromDirectory(runtimeDirectory);
const cycles = Number(option('cycles', '1000'));
assert(Number.isInteger(cycles) && cycles >= 60 && cycles <= 10000, 'cycles must be an integer in 60..10000');
const checkpoints = [...new Set([10, 30, 60, 300, cycles].filter((n) => n <= cycles))].sort((a,b) => a-b);
const digest = (text) => createHash('sha256').update(text).digest('hex');
const definitions = [
  { name: 'self' }, { name: 'mutual' }, { name: 'acyclic' },
  { name: 'reachable-chain', initialize: true },
  { name: 'self-own-ground', hold: true }, { name: 'mutual-own-ground', hold: true },
  { name: 'permission-root', initialize: true }, { name: 'later-history-read', initialize: true },
];

// Diagnostic oracle restricted to these fixtures. It does not modify a session,
// prove complete language reachability, or stand in for a production collector.
// Roots include all current history records, declared records, non-withdrawal
// required pins and exact dependencies of independently retained computations.
function draftReachability(save, source) {
  const roots = new Set([...source.matchAll(/\bevidence\s+(\w+)\s+from\b/g)].map((m) => m[1]));
  const retired = new Set(Object.keys(save.retired ?? {}));
  const add = (p) => { for (const name of p?.evidence ?? []) roots.add(name); };
  const census = pinnedRecords(save);
  for (const [kind, names] of Object.entries(census.by_provenance)) {
    if (kind !== 'withdrawal reason') for (const name of names) roots.add(name);
  }
  for (const occurrences of Object.values(save.renewals ?? {})) for (const name of occurrences) {
    if (!retired.has(name)) roots.add(name);
  }
  for (const state of Object.values(save.states)) { add(state.lineage); add(state.grounds); }
  for (const basis of Object.values(save.commitment_bases ?? {})) add(basis.provenance);
  for (const p of Object.values(save.commitment_grounds ?? {})) add(p);
  for (const stream of Object.values(save.reading_streams ?? {})) {
    add(stream.selection_qualifications);
    for (const record of stream.occurrences) if (!retired.has(record.id)) { roots.add(record.id); add(record.provenance); }
  }
  for (const series of Object.values(save.decision_series ?? {})) add(series.selection_qualifications);
  for (const scheduled of save.scheduled_qualifications ?? []) { roots.add(scheduled.evidence); add(scheduled.guard); }
  const withdrawals = new Map((save.withdrawals ?? []).map((entry) => [entry.evidence, entry.because]));
  const reached = new Set(roots);
  const todo = [...roots];
  for (let index = 0; index < todo.length; index++) {
    const reason = withdrawals.get(todo[index]);
    if (reason && !reached.has(reason)) { reached.add(reason); todo.push(reason); }
  }
  const eligible = [...retired].filter((name) => withdrawals.has(name) && !reached.has(name)).sort();
  return {
    scope: 'Fixture-only conservative diagnostic; collection is not implemented',
    independentRootCount: roots.size,
    reachableWithdrawnRetired: [...retired].filter((name) => withdrawals.has(name) && reached.has(name)).length,
    proposedEligibleCount: eligible.length,
    proposedEligibleSha256: digest(JSON.stringify(eligible)),
    firstPairReachable: { 'a@2': reached.has('a@2'), 'b@2': reached.has('b@2') },
  };
}

const reports = [];
for (const definition of definitions) {
  const source = readFileSync(path.join(here, 'withdrawal-fixtures', `${definition.name}.cav`), 'utf8');
  const drained = runtime.open(source), undrained = runtime.open(source);
  const archive = [], evidence = [], observations = {};
  let accepted = 0;
  function event(name, expected = 'accepted') {
    const before = drained.save(), queued = undrained.undrained;
    const one = drained.dispatchView(name, {}), two = undrained.dispatchView(name, {});
    assert.equal(one.outcome, expected, `${definition.name}/${name}: ${JSON.stringify(one)}`);
    assert.deepEqual(one, two, 'drain must not affect outcomes or view');
    if (expected === 'accepted') accepted++;
    else {
      assert.equal(drained.save(), before, `${name} changed a refused transaction`);
      assert.equal(undrained.undrained, queued, `${name} published an archive item`);
    }
    archive.push(...drained.drainArchive());
    assert.equal(drained.undrained, 0);
    return one;
  }
  function checkpoint(label) {
    const text = drained.save();
    assert.equal(text, undrained.save(), 'drain changed save or deterministic execution');
    const restored = runtime.restore(source, text);
    assert.equal(restored.save(), text, 'restore changed current save bytes');
    assert.equal(restored.undrained, 0);
    restored.close();
    const save = JSON.parse(text), census = pinnedRecords(save);
    assert.deepEqual(census.unpinned_non_declarations, []);
    evidence.push({ label, acceptedEvents: accepted, liveSaveBytes: Buffer.byteLength(text), saveSha256: digest(text),
      heldRenewalRecords: Object.values(save.renewals ?? {}).reduce((n, list) => n + list.length, 0),
      withdrawals: (save.withdrawals ?? []).length, retired: census.retired, pinned: census.pinned,
      byProvenanceCounts: Object.fromEntries(Object.entries(census.by_provenance).map(([key,names]) => [key,names.length])),
      drainedRecords: archive.filter((item) => item.kind !== 'provenance').length,
      drainedNodes: archive.filter((item) => item.kind === 'provenance').length,
      undrained: drained.undrained, undrainedTwinItems: undrained.undrained,
      exactRestore: true, drainIndependent: true, draft: draftReachability(save, source) });
  }
  try {
    if (definition.initialize) event('initialize');
    for (let cycle = 1; cycle <= cycles; cycle++) {
      event('cycle');
      if (definition.hold && cycle === 1) event('hold');
      if (checkpoints.includes(cycle)) checkpoint(`cycle-${cycle}`);
    }
    if (definition.hold) {
      event('release_first'); checkpoint('one-independent-ground-remains');
      assert(evidence.at(-1).draft.firstPairReachable['a@2']);
      observations.refused = event('refuse', 'rejected');
      observations.bindingFailure = event('fail', 'rejected');
      checkpoint('refusals-preserve-last-ground');
      event('release_last'); checkpoint('last-independent-ground-released');
      assert.equal(evidence.at(-1).draft.firstPairReachable['a@2'], false);
      if (definition.name.startsWith('mutual')) assert.equal(evidence.at(-1).draft.firstPairReachable['b@2'], false);
    }
    if (definition.name === 'permission-root') {
      event('probe');
      assert.equal(drained.snapshot().values.revoked, 1);
      observations.permissionDenied = event('denied', 'rejected');
      assert.equal(observations.permissionDenied.origin, 'policy');
      assert.equal(observations.permissionDenied.code, 'not_permitted');
      checkpoint('permission-status-and-refusal-preserved');
      assert(evidence.at(-1).draft.firstPairReachable['a@2']);
    }
    if (definition.name === 'later-history-read') {
      assert(evidence.at(-1).draft.firstPairReachable['a@2'], 'later history status needs its withdrawn dependency');
      event('probe');
      assert(drained.snapshot().qualified_values.result.provenance.caveats.includes('withdrawn'));
      checkpoint('later-history-read-still-withdrawn');
      observations.dynamicWithdrawalCaveat = true;
    }
    if (definition.name === 'acyclic') {
      const records = archive.filter((item) => item.kind !== 'provenance');
      assert.equal(records.length, cycles - 1);
      assert(records.every((item) => item.withdrawal.evidence === item.record && item.withdrawal.because === 'reason'));
      observations.everyTransferredWithdrawalRelationshipExact = true;
    }
    if (definition.name === 'reachable-chain') assert(evidence.every((row) => row.draft.proposedEligibleCount === 0));
    assert.deepEqual(undrained.drainArchive(), archive, 'drain schedule changed exact ordered archive');
    reports.push({ name: definition.name, sourceSha256: digest(source), observations, checkpoints: evidence });
  } finally { drained.close(); undrained.close(); }
}

const olderRestores = [];
const baselineDirectory = option('baseline-runtime', null);
if (baselineDirectory) {
  const baseline = await loadRuntimeFromDirectory(path.resolve(baselineDirectory));
  for (const name of ['self', 'mutual', 'reachable-chain']) {
    const source = readFileSync(path.join(here, 'withdrawal-fixtures', `${name}.cav`), 'utf8');
    const older = baseline.open(source), current = runtime.open(source);
    if (name === 'reachable-chain') { older.dispatchView('initialize', {}); current.dispatchView('initialize', {}); }
    for (let n = 0; n < 10; n++) { assert.equal(older.dispatchView('cycle', {}).outcome, 'accepted'); current.dispatchView('cycle', {}); }
    const oldSave = older.save(), restored = runtime.restore(source, oldSave);
    assert.equal(restored.save(), current.save(), 'old save differs after current restoration');
    assert.equal(restored.undrained, 0);
    assert.deepEqual(restored.dispatchView('cycle', {}), current.dispatchView('cycle', {}));
    assert.equal(restored.save(), current.save());
    olderRestores.push({ name, baselineIdentity: baseline.identity, oldSaveBytes: Buffer.byteLength(oldSave), oldSaveSha256: digest(oldSave), restoredAndContinuedIdentically: true });
    older.close(); current.close(); restored.close();
  }
}
const report = { schema: 'caveat-withdrawal-retention-probe/1', measuredAt: new Date().toISOString(),
  scope: 'Current runtime behavior plus fixture-only design eligibility diagnostics; no collection implementation or universal bound proven',
  runtime: runtime.identity, cycles, checkpoints, reports, olderRestores,
  checks: { currentRestore: true, rollback: true, drainIndependence: true, orderedArchiveEquality: true },
  limits: ['Oracle covers only these fixtures; production root/consumer completeness requires separate implementation review.',
    'Save bytes measure serialized live state, not resident heap or collector latency.',
    'Reachable withdrawal chains can grow with play; this proposal does not establish a universal window-only bound.'] };
const output = option('out', null);
if (output) { mkdirSync(path.dirname(path.resolve(output)), { recursive: true }); writeFileSync(output, JSON.stringify(report, null, 2) + '\n'); }
console.log(JSON.stringify(report, null, 2));
