// Load while a run.mjs session ran, from its load.csv (typeperf, every second)
// and the start and end of every timed job in results.json: total CPU over all
// 24 logical processors (the benchmark itself is about 5%), per timed job and
// per repeat, and the keep rules the report applies.
//
//   node experiments/performance-opt-0.1/load.mjs DIR [--json=FILE]
//
// Per timed job: the job guard's rule (experiments/performance-0.1/lib/guard.mjs),
// which run.mjs --job-guard applied as each job ended, discarding and running
// again every job that broke it. Here it is applied again, afterwards, to the
// kept jobs from load.csv, and the jobs the guard discarded are listed.
// Per repeat: over the samples covering its kept timed jobs, the mean total
// CPU is at most 12% and no sample is above 20%. A session is used only when
// every timed job and every repeat is kept; otherwise it overlapped background
// load, and it is discarded and run again, not kept as a replicate.
import { existsSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { gunzipSync } from 'node:zlib';
import { covering, describeGuard, judge, toSamples } from '../performance-0.1/lib/guard.mjs';
import { maskProcessors, parseMask } from '../performance-0.1/lib/pinning.mjs';

export const KEEP = { meanAtMost: 12, above: 20 };

async function readJson(directory, name) {
  const plain = path.join(directory, `${name}.json`);
  if (existsSync(plain)) return JSON.parse(await readFile(plain, 'utf8'));
  return JSON.parse(gunzipSync(await readFile(`${plain}.gz`)).toString('utf8'));
}

// load.csv to rows, checking that the header names as many columns as every
// row has.
function rowsOf(csv, counters) {
  const lines = csv.split(/\r?\n/).filter(Boolean);
  const header = lines[0].split('","');
  if (header.length !== counters.length + 1) throw new Error(`load.csv names ${header.length - 1} counters, the run monitored ${counters.length}`);
  const rows = lines.slice(1).filter((line) => /^"\d/.test(line)).map((line) => line.split('","').map((cell) => cell.replace(/"/g, '')));
  const bad = rows.filter((row) => row.length !== counters.length + 1);
  if (bad.length) throw new Error(`load.csv has ${bad.length} row(s) whose columns do not match its header`);
  return rows;
}

function stats(values) {
  if (!values.length) return { samples: 0 };
  const sorted = [...values].sort((a, b) => a - b);
  const mean = values.reduce((sum, value) => sum + value, 0) / values.length;
  return {
    samples: values.length,
    mean: Number(mean.toFixed(2)),
    median: sorted[Math.floor(0.5 * (sorted.length - 1))],
    p95: sorted[Math.floor(0.95 * (sorted.length - 1))],
    max: sorted.at(-1),
    above20: values.filter((value) => value > KEEP.above).length,
  };
}

export async function loadPerRepeat(directory) {
  const results = await readJson(directory, 'results');
  const counters = Object.keys(results.environment.loadDuring?.counters ?? {});
  const samples = toSamples(rowsOf(await readFile(path.join(directory, 'load.csv'), 'utf8'), counters), counters);
  const logicalProcessors = results.environment.machine.logicalProcessors;
  const jobs = [];
  for (const [label, byWorkload] of Object.entries(results.results)) {
    for (const [workload, byEngine] of Object.entries(byWorkload)) {
      for (const [engine, byMode] of Object.entries(byEngine)) {
        for (const [mode, { runs }] of Object.entries(byMode)) {
          for (const run of runs) {
            if (!Object.keys(run.ops ?? {}).length) continue; // skipped: nothing timed
            const { samples: within, covered } = covering(samples, run.startedAtMs, run.endedAtMs);
            const after = judge(within, { processors: maskProcessors(parseMask(run.affinityMask)) ?? [], logicalProcessors, covered });
            jobs.push({ label, workload, engine, mode, repeat: run.repeat, from: run.startedAtMs, to: run.endedAtMs, guard: run.guard ?? null, after });
          }
        }
      }
    }
  }
  jobs.sort((a, b) => a.from - b.from);
  const discarded = results.environment.jobGuard?.discarded ?? [];
  const repeats = [...new Set(jobs.map((job) => job.repeat))].sort((a, b) => a - b).map((repeat) => {
    const mine = jobs.filter((job) => job.repeat === repeat);
    const seen = new Map();
    for (const job of mine) for (const sample of covering(samples, job.from, job.to).samples) seen.set(sample.at, sample.total);
    const total = stats([...seen.values()]);
    const from = Math.min(...mine.map((job) => job.from));
    const to = Math.max(...mine.map((job) => job.to));
    const jobsKept = mine.filter((job) => job.after.kept).length;
    const kept = total.samples > 0 && total.mean <= KEEP.meanAtMost && total.above20 === 0 && jobsKept === mine.length;
    return {
      repeat: repeat + 1, from: new Date(from).toISOString(), to: new Date(to).toISOString(), seconds: Number(((to - from) / 1000).toFixed(1)),
      jobs: mine.length, jobsKept, discardedAttempts: discarded.filter((attempt) => attempt.repeat === repeat).length, total, kept,
    };
  });
  const failures = results.failures ?? [];
  return {
    runId: results.runId,
    rule: `a repeat is kept when, over the samples covering its timed jobs, the mean total CPU is at most ${KEEP.meanAtMost}% and no 1-s sample is above ${KEEP.above}%, and every one of its timed jobs keeps the job guard's rule; a session is used only when every repeat is kept`,
    jobRule: results.environment.jobGuard ? describeGuard(results.environment.jobGuard.rule) : null,
    guarded: Boolean(results.environment.jobGuard),
    monitor: results.environment.loadDuring?.what ?? null,
    wholeRun: stats(samples.map((sample) => sample.total)),
    loadBefore: results.environment.loadBefore,
    loadAfter: results.environment.loadAfter,
    jobs: jobs.map((job) => ({ label: job.label, workload: job.workload, mode: job.mode, repeat: job.repeat + 1, from: new Date(job.from).toISOString(), seconds: Number(((job.to - job.from) / 1000).toFixed(1)), guard: job.guard, after: job.after })),
    jobsBrokeRuleAfterwards: jobs.filter((job) => !job.after.kept).length,
    guardAgreesAfterwards: jobs.every((job) => !job.guard || job.guard.kept === job.after.kept),
    discarded,
    failures,
    repeats,
    kept: repeats.every((repeat) => repeat.kept) && !failures.length,
  };
}

if (path.resolve(process.argv[1] ?? '') === fileURLToPath(import.meta.url)) {
  const directory = path.resolve(process.argv[2] ?? '');
  const report = await loadPerRepeat(directory);
  const json = process.argv.find((value) => value.startsWith('--json='))?.slice(7);
  if (json) await writeFile(json, `${JSON.stringify(report, null, 1)}\n`);
  for (const attempt of report.discarded) process.stdout.write(`discarded ${attempt.job} attempt ${attempt.attempt} ${attempt.from}: ${attempt.reasons.join('; ')}\n`);
  for (const job of report.jobs.filter((entry) => !entry.after.kept)) process.stdout.write(`BROKE THE JOB RULE: ${job.label} ${job.workload} ${job.mode} repeat ${job.repeat} ${job.from}: ${job.after.reasons.join('; ')}\n`);
  for (const repeat of report.repeats) {
    const { total } = repeat;
    process.stdout.write(`repeat ${repeat.repeat} ${repeat.from} ${repeat.seconds}s jobs ${repeat.jobsKept}/${repeat.jobs} kept, ${repeat.discardedAttempts} attempt(s) discarded; samples ${total.samples} mean ${total.mean}% p95 ${total.p95}% max ${total.max}% >20%: ${total.above20} ${repeat.kept ? 'KEPT' : 'DISCARD'}\n`);
  }
  process.stdout.write(`whole run: ${JSON.stringify(report.wholeRun)}; guarded: ${report.guarded}; guard agrees afterwards: ${report.guardAgreesAfterwards}; failures: ${report.failures.length}; session usable: ${report.kept}\n`);
}
