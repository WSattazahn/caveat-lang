// Load while each repeat of a run.mjs session ran, from its load.csv (typeperf)
// and the start and end of every job in results.json: total CPU over all 24
// logical processors (the benchmark itself is about 4%), per repeat and per
// (repeat, workload), and the keep rule the report applies.
//
//   node experiments/performance-opt-0.1/load.mjs DIR [--json=FILE]
//
// A repeat is KEPT when, over the typeperf samples taken from its first job's
// start to its last job's end, the mean total CPU is at most 12% and no
// sample is above 20%. Otherwise it overlapped background load, and the
// whole session is discarded and run again, not kept as a replicate.
import { existsSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { gunzipSync } from 'node:zlib';

export const KEEP = { meanAtMost: 12, above: 20 };

async function readJson(directory, name) {
  const plain = path.join(directory, `${name}.json`);
  if (existsSync(plain)) return JSON.parse(await readFile(plain, 'utf8'));
  return JSON.parse(gunzipSync(await readFile(`${plain}.gz`)).toString('utf8'));
}

// typeperf stamps each row with local time, "MM/DD/YYYY HH:MM:SS.mmm".
function samplesOf(csv) {
  return csv.split(/\r?\n/).filter((line) => /^"\d/.test(line)).map((line) => {
    const cells = line.split('","').map((cell) => cell.replace(/"/g, ''));
    return { at: new Date(cells[0]).getTime(), total: Number(cells[1]) };
  }).filter((sample) => Number.isFinite(sample.at) && Number.isFinite(sample.total));
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
  const samples = samplesOf(await readFile(path.join(directory, 'load.csv'), 'utf8'));
  const jobs = [];
  for (const [label, byWorkload] of Object.entries(results.results)) {
    for (const [workload, byEngine] of Object.entries(byWorkload)) {
      for (const [engine, byMode] of Object.entries(byEngine)) {
        for (const [mode, { runs }] of Object.entries(byMode)) {
          for (const run of runs) jobs.push({ label, workload, engine, mode, repeat: run.repeat, from: run.startedAtMs, to: run.endedAtMs });
        }
      }
    }
  }
  const within = (from, to) => samples.filter((sample) => sample.at >= from && sample.at <= to).map((sample) => sample.total);
  const repeats = [...new Set(jobs.map((job) => job.repeat))].sort((a, b) => a - b).map((repeat) => {
    const mine = jobs.filter((job) => job.repeat === repeat);
    const from = Math.min(...mine.map((job) => job.from));
    const to = Math.max(...mine.map((job) => job.to));
    const total = stats(within(from, to));
    const workloads = Object.fromEntries([...new Set(mine.map((job) => job.workload))].map((workload) => {
      const theirs = mine.filter((job) => job.workload === workload);
      return [workload, stats(within(Math.min(...theirs.map((job) => job.from)), Math.max(...theirs.map((job) => job.to))))];
    }));
    const kept = total.samples > 0 && total.mean <= KEEP.meanAtMost && total.above20 === 0;
    return { repeat: repeat + 1, from: new Date(from).toISOString(), to: new Date(to).toISOString(), seconds: Number(((to - from) / 1000).toFixed(1)), jobs: mine.length, total, workloads, kept };
  });
  return {
    runId: results.runId,
    rule: `a repeat is kept when its mean total CPU is at most ${KEEP.meanAtMost}% and no 1-s sample is above ${KEEP.above}%; a session is used only when every repeat is kept`,
    monitor: results.environment.loadDuring?.what ?? null,
    wholeRun: stats(samples.map((sample) => sample.total)),
    loadBefore: results.environment.loadBefore,
    loadAfter: results.environment.loadAfter,
    repeats,
    kept: repeats.every((repeat) => repeat.kept),
  };
}

if (path.resolve(process.argv[1] ?? '') === fileURLToPath(import.meta.url)) {
  const directory = path.resolve(process.argv[2] ?? '');
  const report = await loadPerRepeat(directory);
  const json = process.argv.find((value) => value.startsWith('--json='))?.slice(7);
  if (json) await writeFile(json, `${JSON.stringify(report, null, 1)}\n`);
  for (const repeat of report.repeats) {
    const { total } = repeat;
    process.stdout.write(`repeat ${repeat.repeat} ${repeat.from} ${repeat.seconds}s samples ${total.samples} mean ${total.mean}% p95 ${total.p95}% max ${total.max}% >20%: ${total.above20} ${repeat.kept ? 'KEPT' : 'DISCARD'}\n`);
  }
  process.stdout.write(`whole run: ${JSON.stringify(report.wholeRun)}; all repeats kept: ${report.kept}\n`);
}
