// Measurement-only performance baseline for the reactive runtime.
//
//   node experiments/performance-0.1/run.mjs --target=LABEL=KIND:DIR [--target=...]
//        [--suite=baseline|smoke|view-path|view-path-smoke] [--repeats=N] [--engines=native,wasm]
//        [--workloads=ID,...] [--modes=MODE,...] [--affinity=0xMASK|none]
//        [--affinity-set=0xMASK,0xMASK,...] [--monitor-interval=SECONDS]
//        [--monitor-lead=SECONDS] [--job-guard]
//        [--priority=high|abovenormal|normal] [--build=auto|always|never]
//        [--out=DIR] [--keep-samples] [--keep-work]
//
// KIND is tree (a checkout: native and WebAssembly), package (an unpacked npm
// package: WebAssembly and kit) or runtime (a WebAssembly directory). Every
// mode runs in its own process, pinned and prioritized; each repeat runs every
// target back to back for one mode before the next mode, rotating which target
// goes first, so drift over a run affects every target alike. With
// --affinity-set every target runs once under each mask, as the target
// LABEL@MASK, and the masks rotate with the targets. Writes results.json
// (summaries), summary.md and run.log to --out. See README.md.
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdir, readFile, rm, writeFile, appendFile } from 'node:fs/promises';
import { gzipSync } from 'node:zlib';
import path from 'node:path';
import { summarizeRounds, spread, toMicroseconds } from './lib/stats.mjs';
import { harnessDirectory, loadManifest, loadWorkload, repositoryRoot, sha256 } from './lib/workloads.mjs';
import { parseTarget, prepareTarget } from './lib/targets.mjs';
import { coresDuring, machine, load, startMonitor, summarizeMonitor, toolchain } from './lib/environment.mjs';
import { maskLabel, maskOf, maskProcessors, parseMask, parseMaskSet, runPinned } from './lib/pinning.mjs';
import { SUITES } from './lib/suites.mjs';
import { JOB_GUARD, covering, describeGuard, judge, settled, toSamples } from './lib/guard.mjs';
import { renderSummary } from './lib/report.mjs';

const argv = process.argv.slice(2);
const option = (name) => argv.filter((arg) => arg.startsWith(`--${name}=`)).map((arg) => arg.slice(name.length + 3));
const single = (name, fallback = null) => option(name).at(-1) ?? fallback;
const list = (name) => single(name)?.split(',').filter(Boolean) ?? null;

const suiteName = single('suite', 'baseline');
const suite = SUITES[suiteName];
if (!suite) throw new Error(`unknown suite ${suiteName}; one of ${Object.keys(SUITES).join(', ')}`);
const targets = option('target').map(parseTarget);
if (!targets.length) throw new Error('give at least one --target=LABEL=tree:DIR|package:DIR|runtime:DIR');
if (new Set(targets.map((target) => target.label)).size !== targets.length) throw new Error('target labels must be unique');
const repeats = Number(single('repeats', suite.smoke ? '1' : '3'));
const engines = list('engines') ?? ['native', 'wasm'];
const onlyWorkloads = list('workloads');
const onlyModes = list('modes');
const priority = single('priority', 'high');
const build = single('build', 'auto');
const keepSamples = argv.includes('--keep-samples');
const runId = `${new Date().toISOString().replace(/[:.]/g, '-').replace('Z', '')}-${suiteName}`;
const out = path.resolve(single('out', path.join(harnessDirectory, 'results', runId)));
const work = path.join(out, 'work');
await mkdir(work, { recursive: true });

const logFile = path.join(out, 'run.log');
async function log(message) {
  const line = `[${new Date().toISOString()}] ${message}`;
  console.log(line);
  await appendFile(logFile, `${line}\n`);
}

// ---- environment and targets ------------------------------------------------

const startedAt = new Date().toISOString();
await log(`run ${runId}: suite ${suiteName}, ${repeats} repeat(s), engines ${engines.join(',')}, out ${out}`);
const environment = { machine: await machine(work) };
const affinityOption = single('affinity', 'auto');
const affinitySet = parseMaskSet(single('affinity-set'));
let affinity = null;
if (affinityOption === 'auto') {
  // Four performance cores, leaving logical processors 0 and 1 (where the
  // system tends to service interrupts) alone. The performance cores are not
  // interchangeable on every part (RESULTS.md, core placement): record the
  // mask with every result and compare only like with like.
  const cores = (environment.machine.performanceCores ?? []).filter((core) => core >= 2).slice(0, 4);
  if (cores.length) affinity = maskOf(cores);
} else {
  affinity = parseMask(affinityOption);
}
const masks = affinitySet ?? [affinity];
const monitored = [...new Set(masks.flatMap((mask) => maskProcessors(mask) ?? []))].sort((a, b) => a - b);
environment.pinning = {
  affinityMask: affinitySet ? null : maskLabel(affinity) === 'none' ? null : maskLabel(affinity),
  logicalProcessors: affinitySet ? null : maskProcessors(affinity),
  affinitySet: affinitySet ? affinitySet.map((mask) => ({ mask: maskLabel(mask), logicalProcessors: maskProcessors(mask) })) : null,
  priority,
  how: process.platform === 'win32' ? 'cmd /c start /b /wait /<priority> /affinity <mask>' : (affinity === null ? 'none' : 'taskset'),
};
const firstTree = targets.find((target) => target.kind === 'tree')?.directory ?? repositoryRoot;
environment.toolchain = toolchain(firstTree);
environment.harness = { revision: spawnSync('git', ['rev-parse', 'HEAD'], { cwd: repositoryRoot, encoding: 'utf8' }).stdout.trim() };
environment.loadBefore = await load(work);
await log(`machine ${environment.machine.cpuModel}; pinning ${affinitySet ? `set ${affinitySet.map(maskLabel).join(',')}` : `${environment.pinning.affinityMask ?? 'none'} (${environment.pinning.logicalProcessors?.join(',') ?? '-'})`}, priority ${priority}`);
await log(`load before: ${JSON.stringify(environment.loadBefore)}`);
// A measurement needs mains power and the High performance plan; a smoke run
// or --allow-any-power skips the check.
const HIGH_PERFORMANCE = '8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c';
if (process.platform === 'win32' && !suite.smoke && !argv.includes('--allow-any-power')) {
  const windows = environment.machine.windows ?? {};
  const onMains = windows.batteryStatus === null || windows.batteryStatus === undefined || windows.batteryStatus === 2;
  if (!onMains || !String(windows.powerScheme ?? '').includes(HIGH_PERFORMANCE)) {
    await log(`STOPPED: battery status ${windows.batteryStatus} (2 = mains), power scheme ${windows.powerScheme}; a measurement needs mains power and High performance`);
    process.exit(2);
  }
}

const built = [];
for (const target of targets) {
  built.push(await prepareTarget(target, { build, engines, log: (message) => log(message) }));
  const last = built.at(-1);
  await log(`${last.label}: ${last.kind} ${last.directory}; wasm ${last.runtimeFiles?.['caveat_runtime_bg.wasm']?.sha256 ?? '-'}; native ${last.native?.binary ?? '-'}`);
}
// One entry per (target, mask): with --affinity-set a target runs under each
// mask as LABEL@MASK.
const prepared = affinitySet
  ? built.flatMap((target) => affinitySet.map((mask) => ({ ...target, label: `${target.label}@${maskLabel(mask)}`, baseLabel: target.label, affinity: mask })))
  : built.map((target) => ({ ...target, baseLabel: target.label, affinity }));

// ---- plans ------------------------------------------------------------------

const manifest = await loadManifest();
const perEventModes = (mode) => !['lifecycle', 'micro', 'adapter-resume'].includes(mode);
const workloads = new Map();
const jobs = [];
for (const job of suite.jobs) {
  if (onlyWorkloads && !onlyWorkloads.includes(job.workload)) continue;
  const workload = await loadWorkload(manifest, job.workload, {
    maxEvents: suite.maxEvents,
    repeat: suite.repeatCap ? Math.min(suite.repeatCap, manifest.workloads.find((w) => w.id === job.workload).episodes.repeat ?? 1) : null,
  });
  workloads.set(job.workload, workload);
  for (const engine of engines) {
    for (const mode of job[engine] ?? []) {
      if (onlyModes && !onlyModes.includes(mode)) continue;
      jobs.push({ workload: job.workload, engine, mode });
    }
  }
}
await log(`${jobs.length} jobs x ${prepared.length} target(s) x ${repeats} repeat(s)`);

function planFor(target, workload, engine, mode, jobWork) {
  const published = mode === 'published-method';
  const common = {
    schema: 'caveat-performance-plan/0.1',
    workload: workload.id,
    mode,
    program: workload.program.source,
    episodes: workload.episodes,
    repeats: workload.repeats,
    warmup: published ? 0 : suite.warmup,
    rounds: published ? suite.publishedRounds : suite.rounds,
    lifecycleSamples: suite.lifecycleSamples,
    lifecycleWarmup: suite.lifecycleWarmup,
    resumeRuns: suite.resumeRuns,
    microSamples: suite.microSamples,
  };
  if (engine === 'native') return common;
  let adapter = null;
  if (workload.adapter) {
    const file = target.adapterFile ?? path.join(repositoryRoot, workload.adapter.path);
    adapter = { file, events: workload.adapter.events };
  }
  return { ...common, runtimeDir: target.runtimeDir, kitLib: target.kitLib, adapter, workDir: jobWork };
}

// ---- running ------------------------------------------------------------------


const needs = { native: (target) => target.native, wasm: (target) => target.runtimeDir };
const adapterModes = new Set(['published-method', 'adapter', 'adapter-resume']);
const kitModes = new Set(['kit', 'kit.read', 'kit.dispatchView', 'kit.dispatchView.pieces']);
const results = {};
const failures = [];
const skipped = [];
const correctness = [];
const finalsSeen = [];
let counter = 0;

// Load during the timed jobs (builds are over by now). With --job-guard every
// timed job is judged by its own load as it ends (lib/guard.mjs), and one that
// overlapped a burst is discarded and run again in its place; the monitor then
// also samples the processes the guard watches. --monitor-lead=SECONDS waits
// for that many seconds of samples before the first job, so the first job is
// covered from its start.
const jobGuard = argv.includes('--job-guard');
if (jobGuard && (process.platform !== 'win32' || argv.includes('--no-monitor'))) throw new Error('--job-guard needs the Windows load monitor');
const monitor = argv.includes('--no-monitor') ? null : startMonitor(monitored, Number(single('monitor-interval', '5')), { processes: jobGuard ? [...JOB_GUARD.processes] : [] });
const timedJobs = [];
const guardDiscarded = [];
if (monitor) await log(`monitoring load with typeperf every ${monitor.seconds} s (${monitor.counters.join(', ')})`);
const pause = (ms) => new Promise((resolve) => { setTimeout(resolve, ms); });
const monitorSamples = () => toSamples(monitor.rows(), monitor.counters);
const monitorLead = Number(single('monitor-lead', '0'));
if (monitor && monitorLead > 0) {
  const until = Date.now() + (monitorLead + 10) * 1000;
  while (Date.now() < until && monitorSamples().length < Math.ceil(monitorLead / monitor.seconds)) await pause(200);
  await log(`monitor lead: ${monitorSamples().length} sample(s) before the first job`);
}
if (jobGuard) {
  environment.jobGuard = { rule: { ...JOB_GUARD, processes: [...JOB_GUARD.processes] }, description: describeGuard(), monitorLeadSeconds: monitorLead, discarded: guardDiscarded };
  await log(`job guard: ${describeGuard()}`);
}
// The load of one attempt [fromMs, toMs] on these processors, once the
// monitor has a sample covering its end (at most 5 s later).
async function guardVerdict(fromMs, toMs, processors) {
  const until = Date.now() + 5000;
  let samples = monitorSamples();
  while (Date.now() < until && !(samples.length && samples.at(-1).at >= toMs)) {
    await pause(100);
    samples = monitorSamples();
  }
  const { samples: within, covered } = covering(samples, fromMs, toMs);
  return judge(within, { processors, logicalProcessors: environment.machine.logicalProcessors, covered });
}
async function settle() {
  const until = Date.now() + JOB_GUARD.settleSeconds * 1000;
  const from = Date.now();
  while (Date.now() < until && !settled(monitorSamples().filter((sample) => sample.since >= from))) await pause(250);
  return (Date.now() - from) / 1000;
}

for (let repeat = 0; repeat < repeats; repeat++) {
  for (const job of jobs) {
    const workload = workloads.get(job.workload);
    const rotation = prepared.map((_, index) => prepared[(index + repeat) % prepared.length]);
    for (const target of rotation) {
      if (!needs[job.engine](target)) continue;
      if (job.engine === 'wasm' && adapterModes.has(job.mode) && !workload.adapter) continue;
      if (job.engine === 'wasm' && kitModes.has(job.mode) && !target.kitLib) continue;
      counter += 1;
      const name = `${String(counter).padStart(4, '0')}-${target.label}-${job.workload}-${job.engine}-${job.mode}`;
      const jobWork = path.join(work, name);
      await mkdir(jobWork, { recursive: true });
      const planFile = path.join(jobWork, 'plan.json');
      const rawFile = path.join(jobWork, 'raw.json');
      await writeFile(planFile, JSON.stringify(planFor(target, workload, job.engine, job.mode, jobWork)));
      const command = job.engine === 'native' ? target.native.binary : process.execPath;
      const args = job.engine === 'native' ? [planFile, rawFile]
        : ['--expose-gc', path.join(harnessDirectory, 'wasm-bench.mjs'), `--plan=${planFile}`, `--out=${rawFile}`];
      let started;
      let seconds;
      let child;
      let guard = null;
      for (let attempt = 1; ; attempt++) {
        started = Date.now();
        child = runPinned(command, args, { affinity: target.affinity, priority });
        seconds = (Date.now() - started) / 1000;
        if (!jobGuard || !existsSync(rawFile)) break;
        const attemptRaw = JSON.parse(await readFile(rawFile, 'utf8'));
        if (!Object.keys(attemptRaw.ops ?? {}).length) break;
        guard = { attempt, ...(await guardVerdict(started, started + seconds * 1000, maskProcessors(target.affinity) ?? [])) };
        if (guard.kept) break;
        const discarded = { job: name, repeat, attempt, from: new Date(started).toISOString(), seconds, ...guard };
        guardDiscarded.push(discarded);
        await log(`GUARD discarded ${name} attempt ${attempt} (${seconds.toFixed(1)} s): ${guard.reasons.join('; ')}`);
        // A correctness note does not depend on load: keep it.
        for (const note of attemptRaw.notes ?? []) if (note.startsWith('CORRECTNESS')) correctness.push({ job: `${name}#${attempt}`, note });
        await rm(rawFile, { force: true });
        if (attempt >= JOB_GUARD.attempts) {
          failures.push({ job: name, detail: `job guard: no attempt of ${JOB_GUARD.attempts} was quiet` });
          await log(`FAILED ${name}: job guard, no attempt of ${JOB_GUARD.attempts} was quiet`);
          break;
        }
        const waited = await settle();
        discarded.settledAfterSeconds = Number(waited.toFixed(1));
        await log(`GUARD load settled after ${waited.toFixed(1)} s; running ${name} again`);
      }
      if (!existsSync(rawFile)) {
        if (jobGuard && guard && !guard.kept) continue;
        const detail = `${child.error?.message ?? ''}\n${child.stdout ?? ''}\n${child.stderr ?? ''}`.trim();
        failures.push({ job: name, detail });
        await log(`FAILED ${name} after ${seconds.toFixed(1)} s: ${detail.slice(0, 2000)}`);
        continue;
      }
      const raw = JSON.parse(await readFile(rawFile, 'utf8'));
      const summary = { repeat, seconds, affinityMask: maskLabel(target.affinity), startedAtMs: started, endedAtMs: started + seconds * 1000, process: raw.process, method: raw.method, notes: raw.notes, extra: raw.extra, ops: {} };
      if (guard) summary.guard = guard;
      timedJobs.push(summary);
      const segments = perEventModes(job.mode) ? workload.segments : [];
      for (const [op, { unit, rounds }] of Object.entries(raw.ops)) {
        summary.ops[op] = summarizeRounds(rounds.map((values) => toMicroseconds(values, unit)), segments);
      }
      if (raw.finals?.values?.length) {
        const finals = { kind: raw.finals.kind, sha256: sha256(JSON.stringify(raw.finals.values)), count: raw.finals.values.length };
        summary.finals = finals;
        finalsSeen.push({ target: target.label, workload: job.workload, engine: job.engine, mode: job.mode, repeat, ...finals });
      }
      for (const note of raw.notes ?? []) {
        await log(`${name}: ${note}`);
        if (note.startsWith('skipped:')) skipped.push({ job: name, reason: note.slice('skipped:'.length).trim() });
        if (note.startsWith('CORRECTNESS')) correctness.push({ job: name, note });
      }
      ((((results[target.label] ??= {})[job.workload] ??= {})[job.engine] ??= {})[job.mode] ??= []).push(summary);
      if (keepSamples) {
        await mkdir(path.join(out, 'samples'), { recursive: true });
        await writeFile(path.join(out, 'samples', `${name}.json.gz`), gzipSync(await readFile(rawFile)));
      }
      await rm(jobWork, { recursive: true, force: true });
      const headline = Object.entries(summary.ops).slice(0, 3).map(([op, s]) => `${op} ${s.pooled?.median ?? '-'}`).join(', ');
      const guardNote = guard ? ` [guard: attempt ${guard.attempt}, ${guard.samples} samples, total CPU mean ${guard.meanTotal}% max ${guard.maxTotal}%, outside the mask ${guard.maxOutsideCores} cores, ${Object.entries(guard.maxProcess).map(([p, v]) => `${p} ${v}%`).join(' ')}]` : '';
      await log(`${name} ${seconds.toFixed(1)} s (parallelism ${raw.process?.availableParallelism ?? raw.process?.affinity?.logicalProcessors ?? '?'}): ${headline}${guardNote}`);
    }
  }
}

if (monitor) {
  const stopped = monitor.stop();
  await writeFile(path.join(out, 'load.csv'), `${stopped.csv}\n`);
  environment.loadDuring = summarizeMonitor(monitor, stopped);
  // The processors each job kept busy, and their clock while busy.
  for (const job of timedJobs) job.cores = coresDuring(monitor, stopped, job.startedAtMs, job.endedAtMs);
  await log(`load during: ${JSON.stringify(environment.loadDuring)}`);
}

// ---- aggregation ------------------------------------------------------------

// Across repeats: the spread of each op's pooled median and p95, and of each
// segment's median.
for (const byWorkload of Object.values(results)) {
  for (const byEngine of Object.values(byWorkload)) {
    for (const byMode of Object.values(byEngine)) {
      for (const [mode, runs] of Object.entries(byMode)) {
        const ops = {};
        for (const op of new Set(runs.flatMap((run) => Object.keys(run.ops)))) {
          const per = runs.map((run) => run.ops[op]).filter(Boolean);
          const segmentNames = [...new Set(per.flatMap((summary) => Object.keys(summary.segments ?? {})))];
          ops[op] = {
            median: spread(per.map((summary) => summary.pooled?.median)),
            p95: spread(per.map((summary) => summary.pooled?.p95)),
            iqr: spread(per.map((summary) => summary.pooled?.iqr)),
            max: spread(per.map((summary) => summary.pooled?.max)),
            n: per.reduce((sum, summary) => sum + (summary.pooled?.n ?? 0), 0),
            segments: Object.fromEntries(segmentNames.map((segment) => [segment, spread(per.map((summary) => summary.segments?.[segment]?.median))])),
          };
        }
        byMode[mode] = { runs, acrossRuns: ops };
      }
    }
  }
}

// Within one target, every mode, engine and repeat must end each workload in
// the same saved state: a difference is a correctness finding, not a timing
// one, and fails the run. Between targets the states are compared and
// reported; different runtime versions may legitimately differ.
const consistency = {};
for (const seen of finalsSeen) {
  const key = `${seen.workload} ${seen.kind}`;
  ((consistency[key] ??= {})[seen.sha256] ??= []).push(`${seen.target}/${seen.engine}/${seen.mode}#${seen.repeat}`);
}
const inconsistent = [];
const betweenTargets = {};
for (const [key, hashes] of Object.entries(consistency)) {
  const byTarget = {};
  for (const [hash, runs] of Object.entries(hashes)) {
    for (const run of runs) (byTarget[run.split('/')[0]] ??= new Set()).add(hash);
  }
  for (const [label, seen] of Object.entries(byTarget)) if (seen.size > 1) inconsistent.push(`${key} within ${label}`);
  betweenTargets[key] = Object.entries(hashes).map(([hash, runs]) => ({ sha256: hash, targets: [...new Set(runs.map((run) => run.split('/')[0]))] }));
}
for (const key of inconsistent) await log(`CORRECTNESS: final states differ for ${key}`);

environment.loadAfter = await load(work);
await log(`load after: ${JSON.stringify(environment.loadAfter)}`);

const report = {
  schema: 'caveat-performance-results/0.1',
  runId,
  startedAt,
  finishedAt: new Date().toISOString(),
  command: `node experiments/performance-0.1/run.mjs ${argv.join(' ')}`,
  suite: { name: suiteName, ...suite, jobs: undefined, repeats, engines },
  environment,
  published: manifest.published,
  targets: prepared.map((target) => ({
    label: target.label,
    baseLabel: target.baseLabel,
    affinityMask: maskLabel(target.affinity),
    kind: target.kind,
    directory: target.directory,
    git: target.git,
    packageInfo: target.packageInfo ?? null,
    runtimeDir: target.runtimeDir,
    runtimeFiles: target.runtimeFiles ?? null,
    kitLib: target.kitLib,
    kitFiles: target.kitFiles ?? null,
    adapterFile: target.adapterFile,
    build: target.build,
    native: target.native ? { binary: target.native.binary, profile: target.native.profile, features: target.native.features, rustc: target.native.rustc, lockedPackagesUnchanged: target.native.lockedPackagesUnchanged, reused: target.native.reused ?? false } : null,
  })),
  workloads: [...workloads.values()].map((workload) => ({
    id: workload.id,
    description: workload.description,
    program: { path: workload.program.path, fileSha256: workload.program.fileSha256, sha256: workload.program.sha256, addMushrooms: workload.program.addMushrooms },
    streamSha256: workload.streamSha256,
    episodes: workload.episodes.length,
    eventsPerEpisode: workload.episodes.map((episode) => episode.length),
    repeats: workload.repeats,
    eventsPerPass: workload.eventsPerPass,
    truncatedTo: workload.truncatedTo,
    segments: workload.segments,
    adapter: workload.adapter,
  })),
  consistency: { inconsistent, betweenTargets, byWorkload: consistency },
  failures,
  skipped,
  correctness,
  results,
};
await writeFile(path.join(out, 'results.json'), `${JSON.stringify(report, null, 1)}\n`);
await writeFile(path.join(out, 'summary.md'), renderSummary(report));
if (!argv.includes('--keep-work')) await rm(work, { recursive: true, force: true });
await log(`wrote ${path.join(out, 'results.json')} and summary.md; ${failures.length} failure(s), ${skipped.length} skipped, ${inconsistent.length + correctness.length} correctness finding(s)`);
if (failures.length || inconsistent.length || correctness.length) process.exitCode = 1;
