#!/usr/bin/env node
// Finite paired computation evidence for the collector draft; no runtime edits.
import {createHash} from 'node:crypto';
import {mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const root = fileURLToPath(new URL('../..', import.meta.url));
const usage = 'node experiments/departure-gate/current-computation.mjs --baseline PKG_REACTIVE_DIR [--candidate PKG_REACTIVE_DIR] [--population FILE] [--output FILE] [--runs N] [--events N]';
const options = {};
for (let i = 2; i < process.argv.length; i += 2) {
  const flag = process.argv[i];
  if (!['--baseline', '--candidate', '--population', '--output', '--runs', '--events'].includes(flag) || !process.argv[i + 1] || options[flag]) throw new Error(usage);
  options[flag] = process.argv[i + 1];
}
if (!options['--baseline']) throw new Error(usage);
const baselineDirectory = resolve(root, options['--baseline']);
const candidateDirectory = resolve(root, options['--candidate'] ?? 'dist/pkg-reactive');
const populationPath = resolve(root, options['--population'] ?? 'test-results/rc16-window-population.txt');
const output = resolve(root, options['--output'] ?? 'test-results/collector-current-computation.json');
const runs = Number(options['--runs'] ?? 32);
const events = Number(options['--events'] ?? 400);
if (![runs, events].every(value => Number.isSafeInteger(value) && value > 0)) throw new Error(usage);
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const expected = {
  baseline: {revision: 'f5ec8294efe2be24705f234ef75e5f5459aa5e89', wasm: 'daab2c71023777a3a9e5dc72f7882c653147c4acb445470368cecd76ed30bc59'},
  candidate: {revision: 'c1fe15cfcb1cac6c069ba5b00f1c7238df7e9853', wasm: '0554b21cbc24e9d094975ea53be05075ee1a1233a8e6719b4373f00cf8235bdb'},
};

// Object keys canonicalized, array order retained, and numeric negative zero
// preserved. This is also valid payload JSON, unlike JSON.stringify(-0).
function canonical(value) {
  if (typeof value === 'number') {
    if (!Number.isFinite(value)) throw new Error('Non-finite value in comparison');
    return Object.is(value, -0) ? '-0' : String(value);
  }
  if (value === null || typeof value === 'boolean' || typeof value === 'string') return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (value && typeof value === 'object') return `{${Object.keys(value).sort().map(key => `${JSON.stringify(key)}:${canonical(value[key])}`).join(',')}}`;
  throw new Error(`Unrepresentable comparison value: ${typeof value}`);
}
function firstDifference(left, right, path = '$') {
  if (Object.is(left, right)) return null;
  if (!left || !right || typeof left !== 'object' || typeof right !== 'object' || Array.isArray(left) !== Array.isArray(right)) {
    return {path, baseline: canonical(left), candidate: canonical(right)};
  }
  const keys = [...new Set([...Object.keys(left), ...Object.keys(right)])].sort();
  for (const key of keys) {
    if (!(key in left) || !(key in right)) return {path: `${path}[${JSON.stringify(key)}]`, baseline_present: key in left, candidate_present: key in right};
    const difference = firstDifference(left[key], right[key], `${path}[${JSON.stringify(key)}]`);
    if (difference) return difference;
  }
  return {path, baseline: canonical(left), candidate: canonical(right)};
}
const map = (object, convert) => Object.fromEntries(Object.entries(object ?? {}).map(([key, value]) => [key, convert(value, key)]));
const caveats = provenance => provenance?.caveats ?? [];
function ownGrounds(provenance) {
  // Actual runtime definition: Provenance::own_evidence(), reactive_expr.rs.
  // The input is the separate *_grounds field, never all value lineage.
  const inherited = new Set(provenance?.inherited ?? []);
  return {evidence: (provenance?.evidence ?? []).filter(name => !inherited.has(name)), caveats: caveats(provenance)};
}
function projection(snapshot) {
  const revisionSeries = new Map();
  const currentCommitments = new Set();
  for (const [name, series] of Object.entries(snapshot.decision_series)) {
    for (const revision of series.revisions) revisionSeries.set(revision.id, name);
    if (series.current !== null) currentCommitments.add(series.current);
  }
  for (const record of snapshot.commitments) if (!revisionSeries.has(record.action)) currentCommitments.add(record.action);
  const currentMap = values => Object.fromEntries(Object.entries(values).filter(([name]) => currentCommitments.has(name)));
  return {
    schema: snapshot.schema, source_id: snapshot.source_id,
    sequence: snapshot.sequence, last_event: snapshot.last_event, elapsed: snapshot.elapsed,
    values: snapshot.values, bindings: snapshot.bindings,
    qualified_value_caveats: map(snapshot.qualified_values, value => caveats(value.provenance)),
    binding_caveats: map(snapshot.binding_qualifications, properties => map(properties, caveats)),
    binding_explanation_caveats: map(snapshot.binding_explanations, properties => map(properties, caveats)),
    value_own_grounds: map(snapshot.value_grounds, ownGrounds),
    commitments: snapshot.commitments.map(record => ({...record, status: revisionSeries.has(record.action) && snapshot.decision_series[revisionSeries.get(record.action)].current !== record.action ? 'superseded' : record.open ? 'reopened' : 'in force'})),
    decision_series: map(snapshot.decision_series, series => ({limit: series.limit, current: series.current, revisions: series.revisions, selection_caveats: caveats(series.selection_qualifications)})),
    current_commitment_own_grounds: map(currentMap(snapshot.commitment_grounds), ownGrounds),
    current_commitment_bases: map(currentMap(snapshot.commitment_bases), basis => ({value: basis.value, caveats: caveats(basis.provenance)})),
    commitment_permissions: snapshot.commitment_permissions ?? {},
    budget: snapshot.budget,
    renewals: map(snapshot.renewals, renewal => ({limit: renewal.limit, current: renewal.occurrences.at(-1) ?? null})),
    reading_streams: map(snapshot.reading_streams, stream => ({
      template: stream.template, limit: stream.limit, current: stream.current,
      selection_caveats: caveats(stream.selection_qualifications),
      live_occurrences: stream.occurrences.filter(record => !Object.hasOwn(snapshot.retired ?? {}, record.id)).map(({provenance, ...record}) => ({...record, caveats: caveats(provenance)})),
    })),
    identifiers: snapshot.identifiers ?? [],
    cues: snapshot.cues, cue_caveats: snapshot.cue_qualifications.map(caveats),
    scheduled_qualifications: snapshot.scheduled_qualifications.map(({guard, ...scheduled}) => ({...scheduled, guard_caveats: caveats(guard)})),
    clock: snapshot.clock, controls: snapshot.controls, world: snapshot.world,
    scenes: snapshot.scenes, labels: snapshot.labels,
  };
}

const mask64 = (1n << 64n) - 1n;
class Seed {
  constructor(run) { this.state = (0x9E3779B97F4A7C15n ^ (BigInt(run + 1) * 0xD1B54A32D192ED03n)) & mask64; }
  below(bound) {
    if (!Number.isSafeInteger(bound) || bound < 1) throw new Error('Empty parameter domain/event set');
    this.state ^= this.state >> 12n;
    this.state = (this.state ^ (this.state << 25n)) & mask64;
    this.state ^= this.state >> 27n;
    return Number(((this.state * 0x2545F4914F6CDD1Dn) & mask64) % BigInt(bound));
  }
}
function numeric(seed, min, max) {
  switch (seed.below(6)) {
    case 0: return min;
    case 1: return max;
    case 2: {
      const value = min + (max - min) * (seed.below(1001) / 1000);
      // Rust f64::round: nearest integer, ties away from zero (including -0).
      const magnitude = Math.abs(value);
      const whole = Math.floor(magnitude);
      const rounded = whole + (magnitude - whole >= 0.5 ? 1 : 0);
      return value < 0 || Object.is(value, -0) ? -rounded : rounded;
    }
    default: return min + (max - min) * (seed.below(1_000_001) / 1_000_000);
  }
}
function payload(seed, parameters) {
  return `{${parameters.map(parameter => {
    const domain = parameter.domain;
    const members = domain?.entity?.members ?? domain?.member?.members;
    const value = members ? members[seed.below(members.length)] : domain?.identifier ? `id${seed.below(12)}` : numeric(seed, parameter.min, parameter.max);
    return `${JSON.stringify(parameter.name)}:${canonical(value)}`;
  }).join(',')}}`;
}
async function load(directory, name) {
  const modulePath = join(directory, 'caveat_runtime.js');
  const wasmPath = join(directory, 'caveat_runtime_bg.wasm');
  const buildInfoPath = join(dirname(directory), 'build-info.json');
  const wasm = readFileSync(wasmPath);
  const infoBytes = readFileSync(buildInfoPath);
  const info = JSON.parse(infoBytes);
  const artifact = {directory, revision: info.revision, clean: info.clean, compiled: info.compiled, wasm_sha256: sha256(wasm), glue_sha256: sha256(readFileSync(modulePath)), build_info_sha256: sha256(infoBytes), runtime_source_fingerprint: info.runtimeSourceFingerprint};
  if (info.revision !== expected[name].revision || artifact.wasm_sha256 !== expected[name].wasm || !info.clean || !info.compiled) throw new Error(`${name} is not the pinned clean compiled artifact`);
  for (const [suffix, actual] of [['caveat_runtime.js', artifact.glue_sha256], ['caveat_runtime_bg.wasm', artifact.wasm_sha256]]) {
    if (info.runtimeArtifacts[`pkg-reactive/${suffix}`] !== actual) throw new Error(`${name} artifact disagrees with build-info`);
  }
  const module = await import(`${pathToFileURL(modulePath).href}?computation-sweep=${name}`);
  await module.default({module_or_path: wasm});
  return {module, artifact, modulePath, wasmPath, buildInfoPath};
}
const populationBytes = readFileSync(populationPath);
const population = populationBytes.toString('utf8').split(/\r?\n/).map(line => line.trim()).filter(Boolean);
if (population.length !== 15 || new Set(population).size !== population.length || population.some(name => !name.startsWith('experiments/departure-gate/') || !name.endsWith('.cav') || name.includes('..'))) throw new Error('Expected the 15 unique registered windowed fixtures');
const sources = population.map(path => ({path, source: readFileSync(join(root, path), 'utf8'), sha256: sha256(readFileSync(join(root, path)))}));
mkdirSync(dirname(output), {recursive: true});
const report = {
  schema: 'caveat-collector-current-computation/1', started_at: new Date().toISOString(), status: 'running',
  node: process.version, platform: process.platform, architecture: process.arch,
  script_sha256: sha256(readFileSync(fileURLToPath(import.meta.url))),
  command: [process.execPath, fileURLToPath(import.meta.url), ...process.argv.slice(2)],
  population_file: populationPath, population_sha256: sha256(populationBytes),
  population: sources.map(({path, sha256}) => ({path, sha256})), runs_per_fixture: runs, attempts_per_run: events,
  seed: 'runtime/examples/outcome_sweep.rs xorshift64*; seed 0x9E3779B97F4A7C15 XOR ((run+1)*0xD1B54A32D192ED03), u64 wrapping; identical event/parameter selection and numeric rounding, JSON numeric text may differ from Rust serde_json but values preserve negative zero',
  projection: {
    initial: 'Full initial snapshots including event signatures compare exactly before every run.',
    after_every_attempt: [
      '$.schema, $.source_id, $.sequence, $.last_event, $.elapsed; $.values; all nested $.bindings values with types and negative zero preserved',
      '$.qualified_values[*].provenance.caveats; $.binding_qualifications[*][*].caveats; $.binding_explanations[*][*].caveats',
      '$.value_grounds[*] own evidence and caveats. Own evidence mirrors runtime Provenance::own_evidence(): evidence entries absent from inherited, on the separate grounds field.',
      'All $.commitments fields, plus derived status: superseded if a series revision differs from series.current, otherwise reopened when record.open, otherwise in force. This mirrors kit explain status.',
      '$.decision_series[*].limit/current/revisions plus selection_qualifications.caveats; $.commitment_grounds[current] own evidence+caveats; $.commitment_bases[current].value/provenance.caveats. Current means series.current or a standalone commitment.',
      'All $.commitment_permissions; $.budget; $.renewals[*].limit plus current occurrence (last occurrences element)',
      '$.reading_streams[*].template/limit/current, selection_qualifications.caveats, and all occurrence fields except provenance for records absent from $.retired, plus each such provenance.caveats',
      '$.identifiers; $.cues; $.cue_qualifications[*].caveats; all $.scheduled_qualifications fields except guard lineage, plus guard.caveats',
      '$.clock, $.controls, $.world, $.scenes, $.labels',
    ],
    outcome: 'All dispatch result fields except accepted snapshot; includes refusal origin/code/message and any source span. Rejected attempts additionally compare current snapshots. Fatal exceptions fail.',
    exclusions_after_initial: [
      '$.retired; noncurrent $.renewals[*].occurrences; retired $.reading_streams[*].occurrences; $.decision_journal; $.observations; $.withdrawals; $.symbols; $.relations; $.effects',
      'Full lineage evidence/inherited/departed/archive_ref in provenance holders; noncurrent $.commitment_bases and $.commitment_grounds; raw observation/examination/reopening/predicate qualifications; raw binding explanation evidence',
      '$.events and $.windows after load are not repeated in the projection (the full initial snapshot is compared). Archive buffers are counted and drained independently, not compared for equality.',
    ],
    comparison: 'Canonical JSON with sorted object keys, array order unchanged, exact finite IEEE-754 numeric values including negative zero; no numeric tolerances or removed differing fields.',
  },
  limits: ['Finite fixture/event coverage only; no proof for arbitrary programs or unselected provenance/history fields.', 'Paired WASM executions compare current runtime builds, not incremental versus forced full evaluation within one build; native debug audits cover that separate property.', 'Pinned hashes describe the recorded Windows-built artifacts; rebuilds with different bytes require an explicit reviewed identity update.', 'Archive drained after every attempt; alternate drain schedules and save/restore are covered elsewhere, not by this sweep.'],
  totals: {fixtures_completed: 0, runs: 0, initial_snapshot_comparisons: 0, event_attempts: 0, accepted: 0, refused: 0, current_projection_comparisons: 0, baseline_departures: 0, candidate_departures: 0}, rows: [],
};
function persist() { writeFileSync(output, `${JSON.stringify(report, null, 2)}\n`); }
let context = {};
try {
  const baseline = await load(baselineDirectory, 'baseline');
  const candidate = await load(candidateDirectory, 'candidate');
  report.baseline = baseline.artifact;
  report.candidate = candidate.artifact;
  persist();
  const check = (left, right, name) => {
    const a = canonical(left), b = canonical(right);
    if (a !== b) {
      report.mismatch = {...context, comparison: name, ...firstDifference(left, right)};
      throw new Error(`Mismatch in ${name}: ${report.mismatch.path}`);
    }
    return a;
  };
  for (const fixture of sources) {
    for (let run = 0; run < runs; run++) {
      context = {fixture: fixture.path, run, step: 'load'};
      let a, b;
      try {
        a = new baseline.module.WebReactiveSession(fixture.source);
        b = new candidate.module.WebReactiveSession(fixture.source);
        const initialA = JSON.parse(a.snapshot()), initialB = JSON.parse(b.snapshot());
        check(initialA, initialB, 'full_initial_snapshot');
        report.totals.initial_snapshot_comparisons++;
        const seed = new Seed(run);
        const inputDigest = createHash('sha256'), resultDigest = createHash('sha256');
        const row = {fixture: fixture.path, run, accepted: 0, refused: 0, baseline_departures: 0, candidate_departures: 0, baseline_archive_items: 0, candidate_archive_items: 0};
        for (let step = 0; step < events; step++) {
          const event = initialA.events[seed.below(initialA.events.length)];
          const body = payload(seed, event.parameters);
          context = {fixture: fixture.path, run, step, event: event.name, payload: body};
          const resultA = JSON.parse(a.dispatch_outcome(event.name, body));
          const resultB = JSON.parse(b.dispatch_outcome(event.name, body));
          const {snapshot: acceptedA, ...outcomeA} = resultA;
          const {snapshot: acceptedB, ...outcomeB} = resultB;
          const outcomeText = check(outcomeA, outcomeB, 'dispatch_outcome');
          const snapshotA = acceptedA ?? JSON.parse(a.snapshot());
          const snapshotB = acceptedB ?? JSON.parse(b.snapshot());
          const projected = check(projection(snapshotA), projection(snapshotB), 'current_computation');
          inputDigest.update(event.name).update('\0').update(body).update('\n');
          resultDigest.update(outcomeText).update('\0').update(projected).update('\n');
          if (resultA.outcome === 'accepted') {
            row.accepted++;
            row.baseline_departures += snapshotA.effects.filter(effect => effect.kind === 'depart').length;
            row.candidate_departures += snapshotB.effects.filter(effect => effect.kind === 'depart').length;
          } else row.refused++;
          row.baseline_archive_items += JSON.parse(a.drain_archive()).length;
          row.candidate_archive_items += JSON.parse(b.drain_archive()).length;
          report.totals.event_attempts++;
          report.totals.current_projection_comparisons++;
        }
        row.input_sha256 = inputDigest.digest('hex');
        row.equal_outcome_and_projection_sha256 = resultDigest.digest('hex');
        for (const key of ['accepted', 'refused', 'baseline_departures', 'candidate_departures']) report.totals[key] += row[key];
        report.totals.runs++;
        report.rows.push(row);
      } finally { a?.free(); b?.free(); }
    }
    report.totals.fixtures_completed++;
    persist();
    console.log(`${fixture.path}: ${runs} runs x ${events} attempts agree (${report.totals.event_attempts} comparisons total)`);
  }
  for (const artifact of [baseline, candidate]) {
    if (sha256(readFileSync(artifact.wasmPath)) !== artifact.artifact.wasm_sha256 || sha256(readFileSync(artifact.modulePath)) !== artifact.artifact.glue_sha256 || sha256(readFileSync(artifact.buildInfoPath)) !== artifact.artifact.build_info_sha256) throw new Error('Artifact files changed during sweep');
  }
  for (const fixture of sources) if (sha256(readFileSync(join(root, fixture.path))) !== fixture.sha256) throw new Error(`Fixture changed during sweep: ${fixture.path}`);
  if (sha256(readFileSync(populationPath)) !== report.population_sha256 || sha256(readFileSync(fileURLToPath(import.meta.url))) !== report.script_sha256) throw new Error('Harness/population changed during sweep');
  report.status = 'passed';
} catch (error) {
  report.status = 'failed';
  report.failure = {context, message: error?.message ?? String(error), stack: error?.stack ?? null};
  process.exitCode = 1;
  console.error(report.failure.message);
} finally {
  report.finished_at = new Date().toISOString();
  persist();
  console.log(output);
}
