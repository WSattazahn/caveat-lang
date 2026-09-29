// Per-class statistics from a run kept with --keep-samples: joins every
// per-event sample to its event's class (inputs/event-classes.json) by
// position, and summarizes each (target, workload, engine, mode, operation,
// group) per run, then across runs.
//
//   node experiments/performance-0.1/analyze.mjs RESULTS_DIR [--classes=FILE]
//
// Groups: "all"; every class; for Glowcap streams the owner's categories
// (idle tick, state-changing tick, observation (class)); for the decision
// workloads (ledger, Trail Rescue) every signature (event, class, effects);
// for a single-episode workload played several times (the ledger) every
// position in the episode, which shows how cost grows as the session's
// graph and journal grow. Writes RESULTS_DIR/classes.json. Statistics use
// lib/stats.mjs, the same quantile rule as every other number here.
import { readdir, readFile, writeFile } from 'node:fs/promises';
import { gunzipSync } from 'node:zlib';
import path from 'node:path';
import { harnessDirectory, loadManifest, loadWorkload } from './lib/workloads.mjs';
import { spread, summarize, toMicroseconds } from './lib/stats.mjs';
import { CLASS_ORDER, glowcapCategory } from './lib/classes.mjs';

const directory = path.resolve(process.argv[2] ?? '');
const classesFile = process.argv.find((value) => value.startsWith('--classes='))?.slice(10)
  ?? path.join(harnessDirectory, 'inputs', 'event-classes.json');
const report = JSON.parse(await readFile(path.join(directory, 'results.json'), 'utf8'));
const labelled = JSON.parse(await readFile(classesFile, 'utf8'));
const byWorkload = new Map(labelled.workloads.map((workload) => [workload.id, workload]));
const manifest = await loadManifest();
const DECISION_WORKLOADS = new Set(['ledger-session', 'trail-rescue-scenarios']);

// Per workload, for each event position within one repeat: its groups.
const groupsCache = new Map();
async function groupsOf(workloadId) {
  if (groupsCache.has(workloadId)) return groupsCache.get(workloadId);
  const classes = byWorkload.get(workloadId);
  const reportWorkload = report.workloads.find((candidate) => candidate.id === workloadId);
  if (!classes) throw new Error(`no classes for ${workloadId} in ${classesFile}`);
  if (classes.streamSha256 !== reportWorkload.streamSha256 || classes.programSha256 !== reportWorkload.program.sha256) {
    throw new Error(`classes for ${workloadId} do not match its stream or program`);
  }
  if (reportWorkload.truncatedTo) throw new Error('analyze.mjs needs full streams (not a smoke run)');
  const loaded = await loadWorkload(manifest, workloadId);
  const names = loaded.episodes.flat().map(([event]) => event);
  const flatClasses = classes.classes.flat();
  const flatSignatures = classes.signatures.flat();
  if (names.length !== flatClasses.length) throw new Error(`${workloadId}: ${names.length} events but ${flatClasses.length} labels`);
  const single = loaded.episodes.length === 1 && loaded.repeats > 1;
  const groups = flatClasses.map((eventClass, position) => {
    const list = ['all', `class: ${eventClass}`];
    if (workloadId.startsWith('glowcap')) list.push(`category: ${glowcapCategory(names[position], eventClass)}`);
    if (DECISION_WORKLOADS.has(workloadId)) list.push(`signature: ${flatSignatures[position]}`);
    if (single) list.push(`position: ${String(position).padStart(2, '0')} ${flatSignatures[position]}`);
    return list;
  });
  groupsCache.set(workloadId, groups);
  return groups;
}

// Sample files are NNNN-LABEL-WORKLOAD-ENGINE-MODE.json.gz, in run order.
const labels = report.targets.map((target) => target.label).sort((a, b) => b.length - a.length);
const workloadIds = report.workloads.map((workload) => workload.id).sort((a, b) => b.length - a.length);
function parseName(file) {
  const rest = file.replace(/\.json\.gz$/, '').replace(/^\d+-/, '');
  const label = labels.find((candidate) => rest.startsWith(`${candidate}-`));
  const afterLabel = rest.slice(label.length + 1);
  const workload = workloadIds.find((candidate) => afterLabel.startsWith(`${candidate}-`));
  const afterWorkload = afterLabel.slice(workload.length + 1);
  const engine = afterWorkload.startsWith('native-') ? 'native' : 'wasm';
  return { label, workload, engine, mode: afterWorkload.slice(engine.length + 1) };
}

const NOT_PER_EVENT = new Set(['lifecycle', 'micro', 'adapter-resume', 'probe.lifecycle', 'probe.overhead']);
const files = (await readdir(path.join(directory, 'samples'))).filter((file) => file.endsWith('.json.gz')).sort();
const seen = new Map();
const out = {};
for (const file of files) {
  const { label, workload, engine, mode } = parseName(file);
  if (NOT_PER_EVENT.has(mode)) continue;
  const key = `${label}|${workload}|${engine}|${mode}`;
  const repeat = seen.get(key) ?? 0;
  seen.set(key, repeat + 1);
  const groups = await groupsOf(workload);
  const raw = JSON.parse(gunzipSync(await readFile(path.join(directory, 'samples', file))).toString('utf8'));
  for (const [op, { unit, rounds }] of Object.entries(raw.ops)) {
    const values = {};
    for (const round of rounds) {
      const micro = toMicroseconds(round, unit);
      if (micro.length % groups.length !== 0) throw new Error(`${file} ${op}: ${micro.length} samples is not a whole number of repeats of ${groups.length} events`);
      for (let position = 0; position < micro.length; position++) {
        const value = micro[position];
        if (value === null || value === undefined) continue;
        for (const group of groups[position % groups.length]) (values[group] ??= []).push(value);
      }
    }
    const summaries = Object.fromEntries(Object.entries(values).map(([group, list]) => [group, summarize(list)]));
    ((((((out[label] ??= {})[workload] ??= {})[engine] ??= {})[mode] ??= {})[op] ??= { runs: [] }).runs[repeat] = summaries);
  }
}

// Across runs: the spread of each group's median, p95 and IQR, and its count.
for (const byTarget of Object.values(out)) {
  for (const byWorkloadOut of Object.values(byTarget)) {
    for (const byEngine of Object.values(byWorkloadOut)) {
      for (const byMode of Object.values(byEngine)) {
        for (const entry of Object.values(byMode)) {
          const groups = new Set(entry.runs.flatMap((run) => Object.keys(run ?? {})));
          entry.acrossRuns = Object.fromEntries([...groups].map((group) => {
            const per = entry.runs.map((run) => run?.[group]).filter(Boolean);
            return [group, {
              runs: per.length,
              nPerRun: per.map((summary) => summary.n),
              median: spread(per.map((summary) => summary.median)),
              p95: spread(per.map((summary) => summary.p95)),
              p25: spread(per.map((summary) => summary.p25)),
              p75: spread(per.map((summary) => summary.p75)),
              iqr: spread(per.map((summary) => summary.iqr)),
              min: spread(per.map((summary) => summary.min)),
              max: spread(per.map((summary) => summary.max)),
            }];
          }));
        }
      }
    }
  }
}

await writeFile(path.join(directory, 'classes.json'), `${JSON.stringify({
  schema: 'caveat-performance-classes-results/0.2',
  classesFile: path.relative(directory, classesFile).replace(/\\/g, '/'),
  classesRuntimes: labelled.runtimes,
  classOrder: CLASS_ORDER,
  counts: Object.fromEntries(labelled.workloads.map((workload) => [workload.id, { perRepeat: workload.countsPerRepeat, repeats: workload.repeats, bySignature: workload.bySignature }])),
  results: out,
})}\n`);
console.log(`wrote ${path.join(directory, 'classes.json')}`);
