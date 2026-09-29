// The tables and derived numbers of RESULTS.md, computed from the committed
// raw results so every figure there can be traced to a measurement.
//
//   node experiments/performance-0.1/attribute.mjs --baseline=DIR --instrumented=DIR
//        --verbatim=FILE [--verbatim=FILE] [--replicate=DIR ...] [--method=DIR]
//        [--cores=DIR --cores-verbatim=FILE ...] [--copies=DIR] [--allocbench=FILE] --out=DIR
//
// DIRs are results directories holding results.json and classes.json
// (analyze.mjs). Writes attribution.json (every derived number with its
// formula and paired inputs) and tables.md.
//
// Labels: DIRECT is a statistic of samples timed around exactly that
// operation. INFERRED is a difference of DIRECT medians. A paired difference
// is taken run by run (the same repeat, whose processes ran back to back in
// one interleaved block), then summarized as the median of the per-run
// differences with their lowest and highest; ± is a within-session bound,
// half the sum of the operands' run-to-run ranges. It does not cover
// session-to-session variation or core placement (RESULTS.md, sections 5
// and 7). A ratio is INFERRED the same way: per-run quotients of DIRECT
// medians, median [lowest–highest]. An i2 region converted to µs is
// INFERRED as its share of the instrumented call times the ordinary DIRECT
// median of the same path and class. Medians do not add, so an INFERRED
// remainder is not a measurement of that region.
//
// The i1 build. A row timed in the throwaway i1 build (its benchmark-only
// entry points, or its own copy of an ordinary path), or computed from such
// rows, is DIRECT or INFERRED for that instrumented build only. Adding the
// entry points changes ordinary-path timing by up to about 7% (the safeguard
// section), so these rows are labelled "i1: attribution only" and carry a
// `use` field in attribution.json: attribution evidence (proportions and
// ordering), not production-path costs. The production attribution is the
// i2 shares converted against the ordinary build's DIRECT paths (INFERRED).
import { existsSync } from 'node:fs';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { gunzipSync } from 'node:zlib';

const argv = process.argv.slice(2);
const option = (name) => argv.filter((arg) => arg.startsWith(`--${name}=`)).map((arg) => arg.slice(name.length + 3));
// FILE, or FILE.gz as committed under results/.
const readJson = async (file) => (existsSync(file)
  ? JSON.parse(await readFile(file, 'utf8'))
  : JSON.parse(gunzipSync(await readFile(`${file}.gz`)).toString('utf8')));
const baseDir = path.resolve(option('baseline')[0]);
const instDir = path.resolve(option('instrumented')[0]);
const outDir = path.resolve(option('out')[0]);
const base = await readJson(path.join(baseDir, 'results.json'));
const baseClasses = await readJson(path.join(baseDir, 'classes.json'));
const inst = await readJson(path.join(instDir, 'results.json'));
const instClasses = await readJson(path.join(instDir, 'classes.json'));
const verbatim = await Promise.all(option('verbatim').map(readJson));
// Optional replicates: other quiet sessions that timed the same operations. A
// session overlapped by background load is discarded, never passed here.
const replicates = await Promise.all(option('replicate').map(async (dir) => ({ dir: path.resolve(dir), report: await readJson(path.join(path.resolve(dir), 'results.json')) })));
// An optional short quiet session that re-timed the adapter, published
// method, raw and kit paths (run.mjs --modes=...).
const kitDir = option('kit')[0] ? path.resolve(option('kit')[0]) : null;
const kitReport = kitDir ? await readJson(path.join(kitDir, 'results.json')) : null;
const kitClasses = kitDir ? await readJson(path.join(kitDir, 'classes.json')) : null;

// ---- access ------------------------------------------------------------------

const modeOf = (report, t, w, e, m) => report.results?.[t]?.[w]?.[e]?.[m];
// Per run: the pooled summary of one op, indexed by repeat.
function runs(report, t, w, e, m, op) {
  const mode = modeOf(report, t, w, e, m);
  if (!mode) return null;
  const byRepeat = [];
  for (const run of mode.runs) if (run.ops[op]) byRepeat[run.repeat] = run.ops[op].pooled;
  return byRepeat.length ? byRepeat : null;
}
// Per run: one group's summary from classes.json, indexed by repeat.
function groupRuns(classes, t, w, e, m, op, group) {
  const entry = classes.results?.[t]?.[w]?.[e]?.[m]?.[op];
  if (!entry) return null;
  const list = entry.runs.map((run) => run?.[group] ?? null);
  return list.some(Boolean) ? list : null;
}

const q = (sorted, p) => sorted[Math.floor(p * (sorted.length - 1))];
function spreadOf(values) {
  const present = values.filter((value) => Number.isFinite(value)).sort((a, b) => a - b);
  if (!present.length) return null;
  return { runs: present.length, median: q(present, 0.5), min: present[0], max: present.at(-1) };
}

// A DIRECT statistic across runs: the median over runs of each run's median
// (and p95, IQR, n), with the lowest and highest run.
function direct(perRun) {
  if (!perRun) return null;
  const list = perRun.filter(Boolean);
  if (!list.length) return null;
  return {
    label: 'DIRECT',
    median: spreadOf(list.map((s) => s.median)),
    p95: spreadOf(list.map((s) => s.p95)),
    iqr: spreadOf(list.map((s) => s.iqr)),
    mean: spreadOf(list.map((s) => s.mean)),
    nPerRun: list.map((s) => s.n),
  };
}

// An INFERRED combination sum(sign * operand) of per-run medians (or means),
// paired by repeat.
function inferred(terms, { stat = 'median', formula }) {
  if (terms.some((term) => !term.runs)) return null;
  const repeats = Math.max(...terms.map((term) => term.runs.length));
  const per = [];
  for (let r = 0; r < repeats; r++) {
    if (terms.some((term) => !term.runs[r])) continue;
    per.push(terms.reduce((sum, term) => sum + term.sign * term.runs[r][stat], 0));
  }
  if (!per.length) return null;
  const ranges = terms.map((term) => {
    const values = term.runs.filter(Boolean).map((s) => s[stat]);
    return Math.max(...values) - Math.min(...values);
  });
  return {
    label: 'INFERRED',
    formula,
    stat,
    perRun: per.map((value) => Number(value.toFixed(3))),
    value: spreadOf(per),
    plusMinus: Number((ranges.reduce((a, b) => a + b, 0) / 2).toFixed(3)),
    operands: terms.map((term) => ({ name: term.name, sign: term.sign, perRun: term.runs.map((s) => (s ? s[stat] : null)) })),
  };
}

// An INFERRED ratio of two DIRECT statistics, paired by repeat.
function ratioInferred(numerator, denominator, { stat = 'median', formula }) {
  if (!numerator || !denominator) return null;
  const per = [];
  for (let r = 0; r < Math.max(numerator.length, denominator.length); r++) {
    if (numerator[r] && denominator[r] && denominator[r][stat]) per.push(numerator[r][stat] / denominator[r][stat]);
  }
  if (!per.length) return null;
  return { label: 'INFERRED', kind: 'ratio', formula, stat, perRun: per.map((value) => Number(value.toFixed(4))), value: spreadOf(per) };
}

// ---- formatting --------------------------------------------------------------

const f = (value) => {
  if (value === null || value === undefined || !Number.isFinite(value)) return '-';
  const abs = Math.abs(value);
  if (abs >= 1000) return value.toFixed(0);
  if (abs >= 100) return value.toFixed(1);
  if (abs >= 10) return value.toFixed(1);
  return value.toFixed(2);
};
function directCell(d, { p95 = true, n = false } = {}) {
  if (!d?.median) return '-';
  const range = d.median.runs > 1 ? ` [${f(d.median.min)}–${f(d.median.max)}]` : '';
  const tail = p95 ? ` · p95 ${f(d.p95?.median)}` : '';
  const count = n ? ` · n ${d.nPerRun.join('/')}` : '';
  return `${f(d.median.median)}${range}${tail}${count}`;
}
function inferredCell(i) {
  if (!i?.value) return '-';
  if (i.kind === 'ratio') return `${i.value.median.toFixed(2)}× [${i.value.min.toFixed(2)}–${i.value.max.toFixed(2)}]`;
  return `${f(i.value.median)} [${f(i.value.min)}–${f(i.value.max)}] ±${f(i.plusMinus)}`;
}
const table = (header, rows) => [
  `| ${header.join(' | ')} |`,
  `| ${header.map(() => '---').join(' | ')} |`,
  ...rows.map((row) => `| ${row.join(' | ')} |`),
  '',
].join('\n');

const out = { schema: 'caveat-performance-attribution/0.1', baseline: base.runId, instrumented: inst.runId, sections: {} };
const md = [];
const section = (key, title, intro) => {
  out.sections[key] = { title, rows: [] };
  md.push(`## ${title}`, '', ...(intro ? [intro, ''] : []));
  return out.sections[key];
};
const keep = (sec, row) => { sec.rows.push(row); return row; };

const LOCAL = 'main-local';
const CONTROL = 'rc4-local';
const PUBLISHED = 'rc4-published';
const HIST = 'hist-e6ace96';

// i1 rows: attribution evidence, not production-path costs (see the header).
const I1_TAG = 'i1: attribution only';
const I1_USE = 'attribution evidence, not a production-path cost: timed in (or computed from) the instrumented i1 build, whose added benchmark entry points change ordinary-path timing by up to about 7% (safeguard)';
const I1_WHY = 'i1 directly times the isolated operation in the instrumented build, but because adding the benchmark entry points changes ordinary-path timing by up to about 7% (safeguard), its absolute timings are attribution evidence (proportions and ordering), not production-path costs.';
const I1_INTRO = `Rows labelled "${I1_TAG}" are timed in, or computed from, the throwaway i1 build. ${I1_WHY}`;
const i1Label = (stats) => ({ ...stats, label: `${stats.label} (${I1_TAG})` });

// ---- 1. Control: rc4-local against main-local ------------------------------

{
  const sec = section('control', 'Control: the rc.4 and main builds', 'Byte-identical WebAssembly and identical runtime sources, so this is a control, not a comparison: each row gives both targets\' median over runs, and the difference against the run-to-run range.');
  const rows = [];
  const check = (w, e, m, op) => {
    const a = direct(runs(base, LOCAL, w, e, m, op));
    const b = direct(runs(base, CONTROL, w, e, m, op));
    if (!a || !b) return;
    const diff = b.median.median - a.median.median;
    const noise = Math.max(a.median.max - a.median.min, b.median.max - b.median.min);
    keep(sec, { workload: w, engine: e, mode: m, op, main: a, rc4: b, difference: diff, largestRunRange: noise, withinNoise: Math.abs(diff) <= noise });
    rows.push([w, e, `${m} / ${op}`, directCell(a, { p95: false }), directCell(b, { p95: false }), `${f(diff)} (${f((diff / a.median.median) * 100)}%)`, f(noise), Math.abs(diff) <= noise ? 'yes' : '**no**']);
  };
  for (const w of ['glowcap-replay', 'ledger-session', 'trail-rescue-scenarios']) {
    for (const [e, m, op] of [['native', 'apply', 'apply'], ['native', 'web.dispatch_view', 'web.dispatch_view'], ['native', 'web.dispatch_outcome', 'web.dispatch_outcome'],
      ['native', 'read', 'view'], ['native', 'read', 'snapshot'], ['wasm', 'abi.dispatch_view', 'abi.exec'], ['wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse'],
      ['wasm', 'kit', 'kit.dispatch+view'], ['wasm', 'published-method', 'adapter.dispatch+view']]) check(w, e, m, op);
  }
  md.push(table(['Workload', 'Engine', 'Mode / operation', 'main-local µs [runs]', 'rc4-local µs [runs]', 'rc4 − main', 'largest run range', 'within noise'], rows));
}

// ---- 0. Background load, per session, repeat and workload ------------------------
// Each job's concurrent load: the mean of the typeperf total-CPU samples
// taken while it ran (run.log gives each job's end and duration; load.csv
// the samples, in local time). The benchmark itself keeps about one of the
// 24 logical processors busy (about 4%).

async function jobLoads(dir) {
  const readText = async (name) => { try { return await readFile(path.join(dir, name), 'utf8'); } catch { return null; } };
  // Committed results keep the log as run-log.txt (the repository ignores *.log).
  const log = (await readText('run.log')) ?? (await readText('run-log.txt'));
  const csv = await readText('load.csv');
  if (!log || !csv) return null;
  const samples = csv.split(/\r?\n/).filter((line) => /^"\d/.test(line)).map((line) => {
    const cells = line.split('","').map((cell) => cell.replace(/"/g, ''));
    return { at: new Date(cells[0]).getTime(), total: Number(cells[1]) };
  }).filter((sample) => Number.isFinite(sample.at) && Number.isFinite(sample.total));
  const jobs = [];
  for (const line of log.split(/\r?\n/)) {
    const match = /^\[([^\]]+)\] (\d{4})-(\S+) ([\d.]+) s \(parallelism/.exec(line);
    if (!match) continue;
    const end = new Date(match[1]).getTime();
    const seconds = Number(match[4]);
    const within = samples.filter((sample) => sample.at >= end - seconds * 1000 - 2500 && sample.at <= end + 2500).map((sample) => sample.total);
    jobs.push({ counter: Number(match[2]), name: match[3], seconds, load: within.length ? within.reduce((a, b) => a + b, 0) / within.length : null });
  }
  return jobs;
}

{
  const sessions = [['baseline (primary)', baseDir, base], ...replicates.map(({ dir, report }) => [`baseline ${report.startedAt.slice(11, 16)} UTC`, dir, report]), ['instrumented', instDir, inst], ...(kitReport ? [['kit and adapter paths', kitDir, kitReport]] : [])];
  const sec = section('load', 'Background load during each timed session', 'Mean total CPU use (24 logical processors; the benchmark itself is about 4%) while each job ran, averaged over the jobs of each workload in each repeat. From each run\'s `run.log` (`run-log.txt` under `results/`) and `load.csv` (typeperf every 5 s).');
  const rows = [];
  for (const [name, dir, report] of sessions) {
    const jobs = await jobLoads(dir);
    if (!jobs) continue;
    const perRepeat = Math.ceil(jobs.at(-1).counter / report.suite.repeats);
    const groups = {};
    for (const job of jobs) {
      const repeat = Math.floor((job.counter - 1) / perRepeat);
      const workload = report.workloads.map((w) => w.id).sort((a, b) => b.length - a.length).find((id) => job.name.includes(`-${id}-`));
      if (job.load === null || !workload) continue;
      ((groups[workload] ??= [[], [], []])[repeat] ??= []).push(job.load);
    }
    const during = report.environment.loadDuring?.counters?.total;
    for (const [workload, byRepeat] of Object.entries(groups)) {
      const means = byRepeat.map((list) => (list?.length ? list.reduce((a, b) => a + b, 0) / list.length : null));
      keep(sec, { session: name, runId: report.runId, workload, meanTotalCpuPercentByRepeat: means });
      rows.push([name, workload, ...means.map((value) => (value === null ? '-' : `${f(value)}%`))]);
    }
    rows.push([name, '**whole run**', `mean ${during?.mean ?? '-'}%, p95 ${during?.p95 ?? '-'}%, max ${during?.max ?? '-'}%`, '', '']);
  }
  md.push(table(['Session', 'Workload', 'repeat 1', 'repeat 2', 'repeat 3'], rows));
}

// ---- 1a. Replication: a second complete baseline run -----------------------------

if (replicates.length) {
  const loadOf = (report) => `mean ${report.environment.loadDuring?.counters.total.mean}%, p95 ${report.environment.loadDuring?.counters.total.p95}%`;
  const sec = section('replicate', 'Replication: the same suite in other sessions',
    `The primary baseline (${base.runId}; total CPU during the run ${loadOf(base)}) against the other sessions given as replicates, which timed the same operations (${replicates.map(({ report }) => `${report.runId}: ${loadOf(report)}`).join('; ')}). Each cell: median over 3 runs [lowest–highest run], and the change from the primary.`);
  const rows = [];
  const specs = [
    ['glowcap-replay', 'native', 'apply', 'apply'], ['glowcap-replay', 'native', 'web.dispatch_view', 'web.dispatch_view'], ['glowcap-replay', 'native', 'read', 'view'], ['glowcap-replay', 'native', 'read', 'snapshot'],
    ['glowcap-replay', 'wasm', 'published-method', 'adapter.dispatch+view'], ['glowcap-replay', 'wasm', 'abi.dispatch_view', 'abi.exec'], ['glowcap-replay', 'wasm', 'raw.dispatch_view', 'js.parse_view'], ['glowcap-replay', 'wasm', 'kit', 'kit.dispatch+view'],
    ['trail-rescue-scenarios', 'native', 'apply', 'apply'], ['trail-rescue-scenarios', 'wasm', 'abi.dispatch_view', 'abi.exec'], ['trail-rescue-scenarios', 'wasm', 'kit', 'kit.dispatch+view'],
    ['ledger-session', 'wasm', 'abi.dispatch_view', 'abi.exec'], ['ledger-session', 'wasm', 'kit', 'kit.dispatch+view'],
  ];
  for (const t of [LOCAL, PUBLISHED]) {
    for (const [w, e, m, op] of specs) {
      const a = direct(runs(base, t, w, e, m, op));
      if (!a) continue;
      const others = replicates.map(({ report }) => direct(runs(report, t, w, e, m, op)));
      if (others.every((b) => !b)) continue;
      keep(sec, { target: t, workload: w, engine: e, mode: m, op, primary: a, others: others.map((b, index) => ({ runId: replicates[index].report.runId, stats: b, changePercent: b ? ((b.median.median - a.median.median) / a.median.median) * 100 : null })) });
      rows.push([t, w, `${e} ${m} / ${op}`, directCell(a, { p95: false }), ...others.map((b) => (b ? `${directCell(b, { p95: false })} (${b.median.median >= a.median.median ? '+' : ''}${f(((b.median.median - a.median.median) / a.median.median) * 100)}%)` : '-'))]);
    }
  }
  md.push(table(['Target', 'Workload', 'Operation', 'primary µs', ...replicates.map(({ report }) => `${report.startedAt.slice(11, 16)} UTC session µs`)], rows));
}

// ---- 1b. Every build, the questions (a)-(h) ------------------------------------

{
  const sec = section('builds', 'Results per build and operation (questions a–h)', 'All events of each workload, pooled per run; median over the 3 runs [lowest–highest run] · median p95. DIRECT. Native rows exist only for the trees; the published package and the historical runtime are WebAssembly only. The bench row is timed in the throwaway i1 build: DIRECT there, but adding its benchmark entry points changes ordinary-path timing by up to about 7% (safeguard), so it is attribution evidence, not a production-path cost.');
  const TARGETS = [LOCAL, CONTROL, PUBLISHED, HIST];
  const specs = [
    ['(a) transaction only, numeric parameters', 'native', 'apply', 'apply'],
    [`(a/b) payload + transaction, no reporting (${I1_TAG})`, 'native', 'bench.dispatch_only', 'bench.dispatch_only', 'inst'],
    ['(b) + view built, not serialized', 'native', 'dispatch_view_json', 'dispatch_view_json'],
    ['(b) + full snapshot built (legacy dispatch_json)', 'native', 'dispatch_json', 'dispatch_json'],
    ['(b) + outcome with snapshot built (dispatch_outcome_json)', 'native', 'dispatch_outcome_json', 'dispatch_outcome_json'],
    ['(c) view() built, not dropped (native)', 'native', 'read', 'view'],
    ['(c) view JSON', 'native', 'read', 'view.serialize'],
    ['(d) snapshot() built, not dropped (native)', 'native', 'read', 'snapshot'],
    ['(d) dropping that snapshot (native)', 'native', 'read', 'snapshot.drop'],
    ['(d) snapshot JSON', 'native', 'read', 'snapshot.serialize'],
    ['session clone (upper bound on the transaction copy)', 'native', 'read', 'clone'],
    ['(e) exported dispatch_view', 'native', 'web.dispatch_view', 'web.dispatch_view'],
    ['(8) exported dispatch_outcome', 'native', 'web.dispatch_outcome', 'web.dispatch_outcome'],
    ['(f) save', 'native', 'web.read', 'web.save'],
    ['(e) published method, dispatch + view', 'wasm', 'published-method', 'adapter.dispatch+view'],
    ['(e) adapter dispatch + view, warmed', 'wasm', 'adapter', 'adapter.dispatch+view'],
    ['(e) dispatch_view + JSON.parse', 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse'],
    ['(e) dispatch_view, wasm execution only', 'wasm', 'abi.dispatch_view', 'abi.exec'],
    ['(8) dispatch_outcome, wasm execution only', 'wasm', 'abi.dispatch_outcome', 'abi.exec'],
    ['(8) dispatch_outcome + JSON.parse', 'wasm', 'raw.dispatch_outcome', 'raw.dispatch_outcome+parse'],
    ['(8) kit dispatch + view', 'wasm', 'kit', 'kit.dispatch+view'],
    ['(c) view(), wasm execution only', 'wasm', 'abi.read', 'abi.view.exec'],
    ['(d) snapshot() (pretty), wasm execution only', 'wasm', 'abi.read', 'abi.snapshot.exec'],
    ['(f) save, wasm execution only', 'wasm', 'abi.read', 'abi.save.exec'],
  ];
  for (const w of ['glowcap-replay', 'trail-rescue-scenarios', 'ledger-session']) {
    const rows = [];
    // Every event of the workload (the transaction runs on each); a per-event
    // operation with fewer samples timed only the accepted events.
    const everyEvent = direct(runs(base, LOCAL, w, 'native', 'apply', 'apply'))?.nPerRun?.[0];
    for (const [name, e, m, op, from] of specs) {
      const report = from === 'inst' ? inst : base;
      const targets = from === 'inst' ? ['i1'] : TARGETS;
      const cells = TARGETS.map((t) => {
        if (!targets.includes(t) && !(from === 'inst' && t === LOCAL)) return '-';
        const d = direct(runs(report, from === 'inst' ? 'i1' : t, w, e, m, op));
        if (d) keep(sec, { workload: w, name, target: from === 'inst' ? 'i1' : t, engine: e, mode: m, op, stats: from === 'inst' ? i1Label(d) : d, ...(from === 'inst' ? { use: I1_USE } : {}) });
        const accepted = d && everyEvent && d.nPerRun[0] > 100 && d.nPerRun[0] < everyEvent;
        return `${directCell(d)}${accepted ? ` · accepted events only, n ${d.nPerRun[0]}/run` : ''}`;
      });
      if (cells.every((cell) => cell === '-')) continue;
      rows.push([name, `${e} ${m} / ${op}`, ...cells]);
    }
    md.push(`### ${w}`, '', table(['Question', 'Engine, mode / operation', `${LOCAL} (i1 build for the bench row: attribution only)`, CONTROL, PUBLISHED, HIST], rows));
  }
}

// ---- 2. The published figure ----------------------------------------------------

{
  const sec = section('reproduction', 'Reproducing the published 51.6 µs');
  const rows = [];
  for (const record of verbatim) {
    for (const tree of record.trees) {
      const lines = record.runs.filter((run) => run.tree === tree.label).map((run) => run.lines.caveat5).filter(Boolean);
      const ts = record.runs.filter((run) => run.tree === tree.label).map((run) => run.lines.ts).filter(Boolean);
      const med = spreadOf(lines.map((line) => line.median));
      const p95 = spreadOf(lines.map((line) => line.p95));
      keep(sec, { kind: 'verbatim', pinning: record.pinning, tree: tree.label, reactiveWasm: tree.files['dist/pkg-reactive/caveat_runtime_bg.wasm'], runs: lines, ts });
      rows.push([`\`harness.mjs --bench\` verbatim, ${record.pinning.how === 'none: as published' ? 'unpinned (as published)' : `pinned ${record.pinning.affinity} ${record.pinning.priority}`}, session ${record.startedAt.slice(11, 16)} UTC`, tree.label, `\`${tree.files['dist/pkg-reactive/caveat_runtime_bg.wasm']?.slice(0, 8)}\``,
        `${f(med?.median)} [${f(med?.min)}–${f(med?.max)}]`, `${f(p95?.median)} [${f(p95?.min)}–${f(p95?.max)}]`, lines.map((line) => f(line.median)).join(', '), ts.map((line) => f(line.median)).join(', ')]);
    }
  }
  for (const t of [HIST, PUBLISHED, CONTROL, LOCAL]) {
    const d = direct(runs(base, t, 'glowcap-replay', 'wasm', 'published-method', 'adapter.dispatch+view'));
    if (!d) continue;
    keep(sec, { kind: 'isolated', target: t, stats: d });
    rows.push(['`published-method` (caveat5 alone, fresh process, pinned 0x3C00 high)', t, `\`${base.targets.find((x) => x.label === t)?.runtimeFiles?.['caveat_runtime_bg.wasm']?.sha256.slice(0, 8)}\``,
      `${f(d.median.median)} [${f(d.median.min)}–${f(d.median.max)}]`, `${f(d.p95.median)} [${f(d.p95.min)}–${f(d.p95.max)}]`, '', '']);
  }
  md.push(table(['Method', 'Runtime', 'Reactive wasm', 'caveat5 median µs [runs]', 'p95 µs [runs]', 'per-run medians', 'ts per-run medians'], rows),
    'Every pinned row above ran on logical processors 10–13 (`0x3C00`); the unpinned rows ran wherever Windows put them. The core-placement subsection below gives the same command on each performance-core pair.', '');

  // The published method's samples by event kind.
  const cats = ['category: idle tick', 'category: state-changing tick', 'category: observation (commit)', 'category: observation (reopen)', 'category: observation (evidence)', 'all'];
  const catRows = [];
  for (const t of [HIST, PUBLISHED, LOCAL]) {
    for (const group of cats) {
      const d = direct(groupRuns(baseClasses, t, 'glowcap-replay', 'wasm', 'published-method', 'adapter.dispatch+view', group));
      if (!d) continue;
      keep(sec, { kind: 'published-method by event kind', target: t, group, stats: d });
      catRows.push([t, group.replace('category: ', ''), directCell(d, { n: true })]);
    }
  }
  md.push('The published method\'s own samples (3 rounds, no warm-up), split by event kind:', '', table(['Runtime', 'Event kind', 'median µs [runs] · p95 · samples per run'], catRows));
}

// ---- 2b. Method: the published command in-process against the loop alone -------
// Pairs, run back to back in one quiet window: verbatim-N.json (the published
// command, pinned; caveat5 after five other implementations in one process)
// then alone-N/ (run.mjs published-method: the same loop alone in a fresh
// pinned process).
const methodDir = option('method')[0] ? path.resolve(option('method')[0]) : null;
if (methodDir) {
  const { readdir } = await import('node:fs/promises');
  const names = await readdir(methodDir);
  const pairs = [];
  const unpaired = names.filter((n) => /^alone-\d+$/.test(n) && !names.includes(`verbatim-${/\d+/.exec(n)[0]}.json`));
  for (const name of names.filter((n) => /^verbatim-\d+\.json$/.test(n)).sort()) {
    const index = /\d+/.exec(name)[0];
    if (!names.includes(`alone-${index}`)) continue;
    const verbatimRecord = await readJson(path.join(methodDir, name));
    const alone = await readJson(path.join(methodDir, `alone-${index}`, 'results.json'));
    const inProcess = verbatimRecord.runs[0]?.lines?.caveat5?.median;
    const aloneRun = alone.results['main-local']?.['glowcap-replay']?.wasm?.['published-method']?.runs?.[0]?.ops?.['adapter.dispatch+view']?.pooled;
    if (inProcess && aloneRun) pairs.push({ pair: Number(index), inProcess, alone: aloneRun.median, difference: Number((inProcess - aloneRun.median).toFixed(1)) });
  }
  const sec = section('method', 'The published method in-process against the same loop alone', 'Paired, back to back in one quiet window, main-local, both pinned to 0x3C00 at High priority: the published command (caveat5 after five other implementations in one process) and `published-method` (the same loop alone in a fresh process). Pooled medians of each run\'s 30,000 samples.');
  for (const row of pairs) keep(sec, row);
  const d = spreadOf(pairs.map((row) => row.difference));
  md.push(table(['Pair', 'in-process (verbatim) µs', 'alone (published-method) µs', 'in-process − alone µs'], pairs.map((row) => [row.pair, f(row.inProcess), f(row.alone), f(row.difference)])),
    d ? `Median difference ${f(d.median)} µs [${f(d.min)}–${f(d.max)}] (INFERRED, paired).` : '', '');
  for (const name of unpaired) {
    const alone = await readJson(path.join(methodDir, name, 'results.json'));
    const run = alone.results['main-local']?.['glowcap-replay']?.wasm?.['published-method']?.runs?.[0]?.ops?.['adapter.dispatch+view']?.pooled;
    keep(sec, { unpaired: name, startedAt: alone.startedAt, alone: run?.median ?? null });
    md.push(`\`${name}\` (${alone.startedAt.slice(11, 16)} UTC, published-method ${f(run?.median)} µs) has no published-command half: no \`verbatim-${/\d+/.exec(name)[0]}.json\` was written for it, so it is kept only as a record and is not used.`, '');
  }
}

// ---- 3. Event kinds on each path --------------------------------------------------

const GLOWCAP_GROUPS = ['category: idle tick', 'category: state-changing tick', 'class: renew', 'class: qualify', 'category: observation (commit)', 'category: observation (reopen)', 'category: observation (evidence)', 'all'];
const TRAIL_GROUPS = ['class: idle', 'class: state-changing', 'class: qualify', 'class: evidence', 'class: commit', 'class: reopen', 'class: refused', 'all'];
const LEDGER_GROUPS = ['class: evidence', 'signature: pushed evidence [renew,reveal]', 'class: refused', 'all'];

const PATHS = [
  ['native', 'apply', 'apply', 'native: apply (numeric parameters; the transaction only)'],
  ['native', 'web.dispatch_view', 'web.dispatch_view', 'native: WebReactiveSession::dispatch_view (the exported function, JSON in and out)'],
  ['native', 'web.dispatch_outcome', 'web.dispatch_outcome', 'native: WebReactiveSession::dispatch_outcome (the kit\'s call)'],
  ['wasm', 'abi.dispatch_view', 'abi.exec', 'wasm: dispatch_view, WebAssembly execution only'],
  ['wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse', 'wasm: dispatch_view through the glue + JSON.parse (the web pages\' path)'],
  ['wasm', 'published-method', 'adapter.dispatch+view', 'wasm: the published method verbatim (the adapter, no warm-up)'],
  ['wasm', 'adapter', 'adapter.dispatch+view', 'wasm: the Glowcap adapter, dispatch + view (the published path, warmed)'],
  ['wasm', 'abi.dispatch_outcome', 'abi.exec', 'wasm: dispatch_outcome, WebAssembly execution only'],
  ['wasm', 'raw.dispatch_outcome', 'raw.dispatch_outcome+parse', 'wasm: dispatch_outcome through the glue + JSON.parse'],
  ['wasm', 'kit', 'kit.dispatch', 'wasm: kit session.dispatch() alone'],
  ['wasm', 'kit', 'kit.dispatch+view', 'wasm: kit session.dispatch() + session.view()'],
];

function kindTable(key, title, intro, workload, groups, target = LOCAL, report = base, classes = baseClasses) {
  const sec = section(key, title, intro);
  const rows = [];
  const counts = groups.map((group) => {
    const d = direct(groupRuns(classes, target, workload, 'native', 'apply', 'apply', group)) ?? direct(groupRuns(classes, target, workload, 'wasm', 'kit', 'kit.dispatch', group))
      ?? direct(groupRuns(classes, target, workload, 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse', group));
    return d ? d.nPerRun[0] : null;
  });
  for (const [e, m, op, name] of PATHS) {
    const cells = groups.map((group, index) => {
      const d = direct(groupRuns(classes, target, workload, e, m, op, group));
      if (d) keep(sec, { path: name, engine: e, mode: m, op, group, stats: d });
      // A refused event throws on the dispatch_view path, so its "all" holds
      // the accepted events only.
      const fewer = d && counts[index] && d.nPerRun[0] < counts[index];
      return `${directCell(d)}${fewer ? ` · accepted only, n ${d.nPerRun[0]}/run` : ''}`;
    });
    if (cells.every((cell) => cell === '-')) continue;
    rows.push([name, ...cells]);
  }
  md.push(table(['Path (µs: median [lowest–highest run] · p95)', ...groups.map((group, index) => `${group.replace(/^(category|class|signature): /, '')} (${counts[index] ? `n ${counts[index]}/run` : ''})`)], rows));
}

kindTable('glowcap-kinds', 'Glowcap replay by event kind, every path (main-local)',
  'Categories (1) idle tick, (2) state-changing tick and (3) observation, on the published stream. Every cell is DIRECT. Each observation, renewal and qualification occurs once per pass (3 samples a run): those columns are unreliable and shown only for completeness.',
  'glowcap-replay', GLOWCAP_GROUPS);
kindTable('trail-kinds', 'Trail Rescue by event class, every path (main-local)',
  'Category (3) on a decision workload: 24 scenarios, 140 runtime events, played 20 times per pass. Every cell is DIRECT.',
  'trail-rescue-scenarios', TRAIL_GROUPS);
kindTable('ledger-kinds', 'Agent ledger by event class, every path (main-local)',
  'The ledger log (20 events, 5 refused by policy), 100 times per pass. Every cell is DIRECT.',
  'ledger-session', LEDGER_GROUPS);

// The same tables from the instrumented session, whose background load was
// the lowest; it ran the ordinary build on the paths below (not the adapter
// or the kit).
const QUIET = `The same main-local build measured in the instrumented session (${inst.runId}; total CPU mean ${inst.environment.loadDuring?.counters.total.mean}%), the session the attribution in sections 9–10 comes from; it ran these paths but not the adapter or the kit. Every cell is DIRECT.`;
kindTable('glowcap-kinds-quiet', 'Glowcap replay by event kind (instrumented session)', QUIET, 'glowcap-replay', GLOWCAP_GROUPS, LOCAL, inst, instClasses);
kindTable('trail-kinds-quiet', 'Trail Rescue by event class (instrumented session)', QUIET, 'trail-rescue-scenarios', TRAIL_GROUPS, LOCAL, inst, instClasses);
kindTable('ledger-kinds-quiet', 'Agent ledger by event class (instrumented session)', QUIET, 'ledger-session', LEDGER_GROUPS, LOCAL, inst, instClasses);

// The adapter, published-method, raw and kit paths from the short quiet
// session, for main-local and the published bytes.
if (kitReport) {
  const KIT = `The short quiet session (${kitReport.runId}; total CPU mean ${kitReport.environment.loadDuring?.counters.total.mean}%, p95 ${kitReport.environment.loadDuring?.counters.total.p95}%) that re-timed the adapter, the published method, the raw calls and the kit. Every cell is DIRECT.`;
  for (const t of [LOCAL, PUBLISHED]) {
    kindTable(`glowcap-kinds-kit-${t}`, `Glowcap replay by event kind, adapter and kit paths (quiet kit session, ${t})`, KIT, 'glowcap-replay', GLOWCAP_GROUPS, t, kitReport, kitClasses);
    kindTable(`trail-kinds-kit-${t}`, `Trail Rescue by event class, raw and kit paths (quiet kit session, ${t})`, KIT, 'trail-rescue-scenarios', TRAIL_GROUPS, t, kitReport, kitClasses);
    kindTable(`ledger-kinds-kit-${t}`, `Agent ledger by event class, raw and kit paths (quiet kit session, ${t})`, KIT, 'ledger-session', LEDGER_GROUPS, t, kitReport, kitClasses);
  }
}

// ---- 4-8. Components -------------------------------------------------------------

const WORKLOAD_GROUPS = [
  ['glowcap-replay', 'category: idle tick', 'Glowcap idle tick'],
  ['glowcap-replay', 'category: state-changing tick', 'Glowcap state-changing tick'],
  ['trail-rescue-scenarios', 'class: evidence', 'Trail Rescue evidence'],
  ['trail-rescue-scenarios', 'class: commit', 'Trail Rescue commit'],
  ['trail-rescue-scenarios', 'class: reopen', 'Trail Rescue reopen'],
  ['ledger-session', 'class: evidence', 'ledger evidence'],
];

function componentValue(spec, workload, group, report, classes) {
  const pick = (t, e, m, op) => (group === 'all' ? runs(report, t, workload, e, m, op) : groupRuns(classes, t, workload, e, m, op, group));
  if (spec.ratio) return ratioInferred(pick(...spec.ratio[0]), pick(...spec.ratio[1]), { formula: spec.formula });
  if (spec.direct) {
    const [t, e, m, op] = spec.direct;
    return direct(group === 'all' ? runs(report, t, workload, e, m, op) : groupRuns(classes, t, workload, e, m, op, group));
  }
  const terms = spec.terms.map(([sign, t, e, m, op]) => ({
    sign,
    name: `${t} ${e} ${m} / ${op}`,
    runs: group === 'all' ? runs(report, t, workload, e, m, op) : groupRuns(classes, t, workload, e, m, op, group),
  }));
  return inferred(terms, { formula: spec.formula });
}

// One table per component: a row per quantity, a column per event kind.
// DIRECT cells are "median [runs] · p95"; INFERRED cells are "difference
// [runs] ± bound", with each formula listed under the table. A row timed in
// or computed from the i1 build is labelled as attribution only.
const usesI1 = (spec) => (spec.direct ? spec.direct[0] === 'i1' : (spec.terms ?? []).some(([, t]) => t === 'i1'));
function componentSection(key, title, intro, specs, report, classes) {
  const sec = section(key, title, intro);
  const rows = [];
  const notes = [];
  specs.forEach((spec, index) => {
    const i1 = usesI1(spec);
    const label = `${spec.direct ? 'DIRECT' : `INFERRED [${index + 1}]`}${i1 ? `, ${I1_TAG}` : ''}`;
    const cells = WORKLOAD_GROUPS.map(([workload, group, name]) => {
      const value = componentValue(spec, workload, group, report, classes);
      const labelled = value && i1 ? i1Label(value) : value;
      if (value) keep(sec, { quantity: spec.name, eventKind: name, workload, group, source: spec.direct ?? null, ...(spec.direct ? { stats: labelled } : labelled), ...(i1 ? { use: I1_USE } : {}) });
      return spec.direct ? directCell(value) : inferredCell(value);
    });
    rows.push([spec.name, label, ...cells]);
    if (!spec.direct) notes.push(`[${index + 1}] ${spec.formula}`);
    else if (spec.note) notes.push(`${spec.name}: ${spec.note}`);
  });
  md.push(table(['Quantity (µs)', 'Label', ...WORKLOAD_GROUPS.map(([, , name]) => name)], rows));
  if (notes.length) md.push(...notes.map((note) => `- ${note}`), '');
}

// (4) core runtime work: the transaction with no reporting.
componentSection('core', '(4) Core runtime work: the event without view or snapshot', `The instrumented session: the ordinary main-local build and the i1 build side by side, interleaved. ${I1_INTRO}`, [
  { name: 'transaction, numeric parameters (native apply)', direct: [LOCAL, 'native', 'apply', 'apply'] },
  { name: 'payload parse + names + transaction (native, i1 bench_dispatch_only)', direct: ['i1', 'native', 'bench.dispatch_only', 'bench.dispatch_only'], note: 'throwaway i1 build' },
  { name: 'payload parse + names + transaction (wasm execution, i1 bench_dispatch_only)', direct: ['i1', 'wasm', 'abi.dispatch_only', 'abi.dispatch_only.exec'], note: 'throwaway i1 build' },
  { name: 'payload parse + name resolution (native)', formula: 'i1 bench.dispatch_only − i1 apply', terms: [[1, 'i1', 'native', 'bench.dispatch_only', 'bench.dispatch_only'], [-1, 'i1', 'native', 'apply', 'apply']] },
  { name: 'WebAssembly execution over native, the same work', formula: 'i1 abi.dispatch_only.exec − i1 bench.dispatch_only', terms: [[1, 'i1', 'wasm', 'abi.dispatch_only', 'abi.dispatch_only.exec'], [-1, 'i1', 'native', 'bench.dispatch_only', 'bench.dispatch_only']] },
], inst, instClasses);

// (5) snapshot and reporting work.
componentSection('snapshot', '(5) Snapshot and reporting work', `The instrumented session. ${I1_INTRO}`, [
  { name: 'snapshot() built (native, not serialized, not dropped)', direct: [LOCAL, 'native', 'read', 'snapshot'] },
  { name: 'snapshot compact JSON (native serde_json)', direct: [LOCAL, 'native', 'read', 'snapshot.serialize'] },
  { name: 'snapshot drop (native)', direct: [LOCAL, 'native', 'read', 'snapshot.drop'] },
  { name: 'snapshot built + dropped (wasm, i1)', direct: ['i1', 'wasm', 'abi.bench_read', 'abi.snapshot_build.exec'], note: 'throwaway i1 build' },
  { name: 'dispatch_outcome over dispatch-only (native): snapshot + outcome + JSON', formula: 'i1 web.dispatch_outcome − i1 bench.dispatch_only', terms: [[1, 'i1', 'native', 'web.dispatch_outcome', 'web.dispatch_outcome'], [-1, 'i1', 'native', 'bench.dispatch_only', 'bench.dispatch_only']] },
  { name: 'dispatch_outcome over dispatch-only (wasm execution)', formula: 'i1 abi.dispatch_outcome exec − i1 abi.dispatch_only exec', terms: [[1, 'i1', 'wasm', 'abi.dispatch_outcome', 'abi.exec'], [-1, 'i1', 'wasm', 'abi.dispatch_only', 'abi.dispatch_only.exec']] },
  { name: 'JS JSON.parse of the outcome', direct: [LOCAL, 'wasm', 'raw.dispatch_outcome', 'js.parse_outcome'] },
], inst, instClasses);

// (6) view construction.
componentSection('view', '(6) View construction', `The instrumented session. ${I1_INTRO}`, [
  { name: 'view() built (native, not serialized, not dropped)', direct: [LOCAL, 'native', 'read', 'view'] },
  { name: 'view compact JSON (native serde_json)', direct: [LOCAL, 'native', 'read', 'view.serialize'] },
  { name: 'view built + dropped (native, i1)', direct: ['i1', 'native', 'bench.read', 'bench.view_build'], note: 'throwaway i1 build' },
  { name: 'view built + dropped (wasm, i1)', direct: ['i1', 'wasm', 'abi.bench_read', 'abi.view_build.exec'], note: 'throwaway i1 build' },
  { name: 'view exported: built + JSON + dropped (wasm, i1)', direct: ['i1', 'wasm', 'abi.bench_read', 'abi.view.exec'] },
  { name: 'view JSON inside wasm', formula: 'i1 abi.view.exec − i1 abi.view_build.exec (same step, same state)', terms: [[1, 'i1', 'wasm', 'abi.bench_read', 'abi.view.exec'], [-1, 'i1', 'wasm', 'abi.bench_read', 'abi.view_build.exec']] },
  { name: 'JS JSON.parse of the view', direct: [LOCAL, 'wasm', 'raw.dispatch_view', 'js.parse_view'] },
], inst, instClasses);

// (7) WebAssembly, JSON and JS overhead.
componentSection('bridge', '(7) WebAssembly, JSON and JS overhead on dispatch_view', 'The ordinary main-local build in the instrumented session, every part DIRECT or a paired difference.', [
  { name: 'payload JSON.stringify (JS)', direct: [LOCAL, 'wasm', 'raw.dispatch_view', 'js.stringify_payload'] },
  { name: 'argument copy into wasm memory (passStringToWasm0 ×2)', direct: [LOCAL, 'wasm', 'abi.dispatch_view', 'abi.encode_args'] },
  { name: 'wasm execution of dispatch_view', direct: [LOCAL, 'wasm', 'abi.dispatch_view', 'abi.exec'] },
  { name: 'result decode (TextDecoder, fatal)', direct: [LOCAL, 'wasm', 'abi.dispatch_view', 'abi.decode'] },
  { name: 'result free (__wbindgen_free)', direct: [LOCAL, 'wasm', 'abi.dispatch_view', 'abi.free'] },
  { name: 'JS JSON.parse of the view', direct: [LOCAL, 'wasm', 'raw.dispatch_view', 'js.parse_view'] },
  { name: 'the same exported function natively', direct: [LOCAL, 'native', 'web.dispatch_view', 'web.dispatch_view'] },
  { name: 'wasm execution penalty over native, same function', formula: 'wasm abi.exec − native web.dispatch_view', terms: [[1, LOCAL, 'wasm', 'abi.dispatch_view', 'abi.exec'], [-1, LOCAL, 'native', 'web.dispatch_view', 'web.dispatch_view']] },
  { name: 'glue overhead beyond the ABI pieces', formula: 'raw.dispatch_view − abi.dispatch_view (whole)', terms: [[1, LOCAL, 'wasm', 'raw.dispatch_view', 'raw.dispatch_view'], [-1, LOCAL, 'wasm', 'abi.dispatch_view', 'abi.dispatch_view']] },
], inst, instClasses);

// (8) the kit path against dispatch_view.
componentSection('kit', '(8) The kit session path against dispatch_view', 'The primary baseline, main-local; the kit and raw calls ran in the same interleaved blocks, so the differences are paired.', [
  { name: 'kit session.dispatch() (dispatch_outcome + full snapshot + JSON.parse + checks)', direct: [LOCAL, 'wasm', 'kit', 'kit.dispatch'] },
  { name: 'kit session.view() after it', direct: [LOCAL, 'wasm', 'kit', 'kit.view'] },
  { name: 'kit dispatch + view', direct: [LOCAL, 'wasm', 'kit', 'kit.dispatch+view'] },
  { name: 'raw dispatch_view + JSON.parse', direct: [LOCAL, 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse'] },
  { name: 'kit path over dispatch_view', formula: 'kit.dispatch+view − raw.dispatch_view+parse', terms: [[1, LOCAL, 'wasm', 'kit', 'kit.dispatch+view'], [-1, LOCAL, 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse']] },
  { name: 'kit JS layer over the raw outcome call', formula: 'kit.dispatch − raw.dispatch_outcome+parse', terms: [[1, LOCAL, 'wasm', 'kit', 'kit.dispatch'], [-1, LOCAL, 'wasm', 'raw.dispatch_outcome', 'raw.dispatch_outcome+parse']] },
  { name: 'kit path ÷ dispatch_view path', formula: 'kit.dispatch+view ÷ raw.dispatch_view+parse, per run', ratio: [[LOCAL, 'wasm', 'kit', 'kit.dispatch+view'], [LOCAL, 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse']] },
], base, baseClasses);

// The kit's own JavaScript, run by run: the per-run differences behind the
// INFERRED row above, which are too spread to resolve it.
{
  const sec = section('kit-js', 'The kit\'s JavaScript layer, run by run', 'kit.dispatch − raw.dispatch_outcome+parse in each repeat of the primary baseline (paired: the two processes ran back to back). µs.');
  const rows = [];
  for (const [w, group, name] of WORKLOAD_GROUPS) {
    const value = inferred([{ sign: 1, name: 'kit.dispatch', runs: groupRuns(baseClasses, LOCAL, w, 'wasm', 'kit', 'kit.dispatch', group) }, { sign: -1, name: 'raw.dispatch_outcome+parse', runs: groupRuns(baseClasses, LOCAL, w, 'wasm', 'raw.dispatch_outcome', 'raw.dispatch_outcome+parse', group) }], { formula: 'kit.dispatch − raw.dispatch_outcome+parse' });
    if (!value) continue;
    keep(sec, { eventKind: name, ...value });
    rows.push([name, value.perRun.map((v) => f(v)).join(', '), `${f(value.value.min)} to ${f(value.value.max)}`]);
  }
  md.push(table(['Event kind', 'per-run differences', 'range'], rows));
}

// ---- the attribution the owner asked for -----------------------------------------
// Absolute parts from the instrumented run (i1 and the ordinary main-local
// build, interleaved), each timed on its own; they are separate
// measurements, so their sum is not a measurement of the whole. The i1 parts
// are attribution evidence only (see the header). The i2 shares below them
// are the same split made inside one call, where the parts are disjoint and
// add up exactly (for means); converted against the ordinary DIRECT paths,
// they are the production attribution (INFERRED).

componentSection('attribution-view', 'Attribution on the dispatch_view path (web pages, adapter): language execution, view building, serialization and bridge', `Each part timed on its own in the instrumented run: the first three in, or computed from, the throwaway i1 build (labelled "${I1_TAG}"), the rest in the ordinary build. They are disjoint pieces of one dispatch_view call, but timed in separate calls, so their medians are not added; the whole is shown as its own DIRECT measurement. ${I1_WHY} The production attribution of this call is the i2 shares converted against the ordinary DIRECT path (INFERRED).`, [
  { name: 'language execution: payload parse, names, transaction, rules, change detection, bindings, commit (wasm, i1 bench_dispatch_only)', direct: ['i1', 'wasm', 'abi.dispatch_only', 'abi.dispatch_only.exec'] },
  { name: 'view building: name map, sort, records, assembly, drop (wasm, i1 bench_view_build)', direct: ['i1', 'wasm', 'abi.bench_read', 'abi.view_build.exec'] },
  { name: 'view serialization inside wasm (serde_json)', formula: 'i1 abi.view.exec − i1 abi.view_build.exec, the same step', terms: [[1, 'i1', 'wasm', 'abi.bench_read', 'abi.view.exec'], [-1, 'i1', 'wasm', 'abi.bench_read', 'abi.view_build.exec']] },
  { name: 'bridge: argument copy', direct: [LOCAL, 'wasm', 'abi.dispatch_view', 'abi.encode_args'] },
  { name: 'bridge: result decode to a JS string', direct: [LOCAL, 'wasm', 'abi.dispatch_view', 'abi.decode'] },
  { name: 'bridge: result free', direct: [LOCAL, 'wasm', 'abi.dispatch_view', 'abi.free'] },
  { name: 'JS: JSON.parse of the view', direct: [LOCAL, 'wasm', 'raw.dispatch_view', 'js.parse_view'] },
  { name: 'whole: dispatch_view through the glue + JSON.parse', direct: [LOCAL, 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse'] },
  { name: 'whole, wasm execution only', direct: [LOCAL, 'wasm', 'abi.dispatch_view', 'abi.exec'] },
], inst, instClasses);

componentSection('attribution-kit', 'Attribution on the kit path: language execution, snapshot reporting, serialization and bridge', 'As above, for the `dispatch_outcome` call the kit\'s `session.dispatch()` makes, all from the instrumented run; the kit\'s own JavaScript and its extra `view()` call are in section (8). The first two rows are from the i1 build: attribution evidence, not production-path costs; the production attribution is the i2 shares converted against the ordinary DIRECT path (INFERRED).', [
  { name: 'language execution (wasm, i1 bench_dispatch_only)', direct: ['i1', 'wasm', 'abi.dispatch_only', 'abi.dispatch_only.exec'] },
  { name: 'snapshot + outcome built, serialized and dropped inside wasm', formula: 'i1 abi.dispatch_outcome exec − i1 abi.dispatch_only exec', terms: [[1, 'i1', 'wasm', 'abi.dispatch_outcome', 'abi.exec'], [-1, 'i1', 'wasm', 'abi.dispatch_only', 'abi.dispatch_only.exec']] },
  { name: 'bridge: outcome decode to a JS string', direct: [LOCAL, 'wasm', 'abi.dispatch_outcome', 'abi.decode'] },
  { name: 'JS: JSON.parse of the outcome', direct: [LOCAL, 'wasm', 'raw.dispatch_outcome', 'js.parse_outcome'] },
  { name: 'whole: raw dispatch_outcome + JSON.parse', direct: [LOCAL, 'wasm', 'raw.dispatch_outcome', 'raw.dispatch_outcome+parse'] },
], inst, instClasses);

// ---- binding evaluation and size scaling ------------------------------------------

{
  const sec = section('bindings', 'Binding evaluation, by difference (glowcap-unbound)', 'The unbound program is the replay program without its 39 one-line bind statements; the same stream. Native apply, paired by repeat.');
  const rows = [];
  for (const [session, classes] of [['primary baseline', baseClasses], ['instrumented session', instClasses]]) {
    for (const group of ['category: idle tick', 'category: state-changing tick', 'all']) {
      const a = groupRuns(classes, LOCAL, 'glowcap-replay', 'native', 'apply', 'apply', group);
      const b = groupRuns(classes, LOCAL, 'glowcap-unbound', 'native', 'apply', 'apply', group);
      const value = inferred([{ sign: 1, name: 'apply glowcap-replay', runs: a }, { sign: -1, name: 'apply glowcap-unbound', runs: b }], { formula: 'apply(replay) − apply(unbound)' });
      keep(sec, { session, group, replay: direct(a), unbound: direct(b), bindings: value });
      rows.push([session, group.replace('category: ', ''), directCell(direct(a)), directCell(direct(b)), inferredCell(value)]);
    }
  }
  md.push(table(['Session', 'Event kind', 'apply, replay (DIRECT)', 'apply, unbound (DIRECT)', 'binding evaluation (INFERRED)'], rows));
}

{
  const sec = section('scaling', 'Size scaling: Glowcap with 4, 16 and 64 mushrooms (main-local)', 'The first 2,000 events of the replay stream on each program; idle ticks only. DIRECT.');
  const rows = [];
  const specs = [['native', 'apply', 'apply'], ['native', 'read', 'view'], ['native', 'read', 'view.serialize'], ['native', 'read', 'snapshot'], ['native', 'read', 'snapshot.serialize'], ['native', 'read', 'clone'],
    ['wasm', 'abi.dispatch_view', 'abi.exec'], ['wasm', 'raw.dispatch_view', 'js.parse_view'], ['wasm', 'kit', 'kit.dispatch+view']];
  for (const [e, m, op] of specs) {
    const cells = ['glowcap-replay', 'glowcap-scaled-16', 'glowcap-scaled-64'].map((w) => {
      const d = direct(groupRuns(baseClasses, LOCAL, w, e, m, op, 'category: idle tick'));
      if (d) keep(sec, { workload: w, engine: e, mode: m, op, stats: d });
      return directCell(d);
    });
    const growth = ratioInferred(groupRuns(baseClasses, LOCAL, 'glowcap-scaled-64', e, m, op, 'category: idle tick'), groupRuns(baseClasses, LOCAL, 'glowcap-replay', e, m, op, 'category: idle tick'), { formula: 'median(64 mushrooms) ÷ median(4), per run' });
    if (growth) keep(sec, { growth: `${e} ${m} / ${op}`, ...growth });
    rows.push([`${e} ${m} / ${op}`, ...cells, inferredCell(growth)]);
  }
  md.push(table(['Operation', '4 mushrooms (replay)', '16', '64', '64 ÷ 4 (INFERRED, paired by run)'], rows));
}

// ---- growth within a session: the ledger by position -------------------------------

{
  const sec = section('growth', 'Growth within a session: the ledger\'s 20 events by position', 'Each ledger episode starts from a fresh session and its graph, journal and renewals grow over its 20 events, so the same kind of event can be compared early and late. DIRECT; each position has 300 samples a run (100 plays × 3 passes). Native and wasm execution from the instrumented session; the kit path from the primary baseline.');
  const positions = Object.keys(instClasses.results?.[LOCAL]?.['ledger-session']?.native?.apply?.apply?.acrossRuns ?? {}).filter((group) => group.startsWith('position: ')).sort();
  const specs = [
    ['native apply', instClasses, LOCAL, 'native', 'apply', 'apply'],
    ['wasm dispatch_view exec', instClasses, LOCAL, 'wasm', 'abi.dispatch_view', 'abi.exec'],
    ['wasm dispatch_outcome exec', instClasses, LOCAL, 'wasm', 'abi.dispatch_outcome', 'abi.exec'],
    ['kit dispatch + view', baseClasses, LOCAL, 'wasm', 'kit', 'kit.dispatch+view'],
  ];
  const rows = positions.map((group) => {
    const cells = specs.map(([name, classes, t, e, m, op]) => {
      const d = direct(groupRuns(classes, t, 'ledger-session', e, m, op, group));
      if (d) keep(sec, { position: group.slice(10), path: name, stats: d });
      return d ? `${f(d.median.median)} [${f(d.median.min)}–${f(d.median.max)}]` : '-';
    });
    return [group.slice(10), ...cells];
  });
  md.push(table(['Position and event', ...specs.map(([name]) => `${name} µs`)], rows));
}

// ---- native against WebAssembly --------------------------------------------------

{
  const sec = section('native-vs-wasm', '(h) Native against WebAssembly, the same exported function', 'Native `web::WebReactiveSession` against the WebAssembly export\'s execution alone (`abi.*.exec`, no glue), main-local, all events of each workload. Ratio = median over runs of (wasm median / native median).');
  const rows = [];
  const pairs = [
    ['dispatch_view', ['native', 'web.dispatch_view', 'web.dispatch_view'], ['wasm', 'abi.dispatch_view', 'abi.exec']],
    ['dispatch_outcome', ['native', 'web.dispatch_outcome', 'web.dispatch_outcome'], ['wasm', 'abi.dispatch_outcome', 'abi.exec']],
    ['view', ['native', 'web.read', 'web.view'], ['wasm', 'abi.read', 'abi.view.exec']],
    ['snapshot (pretty)', ['native', 'web.read', 'web.snapshot'], ['wasm', 'abi.read', 'abi.snapshot.exec']],
    ['save', ['native', 'web.read', 'web.save'], ['wasm', 'abi.read', 'abi.save.exec']],
    ['new (load)', ['native', 'lifecycle', 'web.new'], ['wasm', 'lifecycle', 'raw.new']],
    ['restore', ['native', 'lifecycle', 'web.restore'], ['wasm', 'lifecycle', 'raw.restore']],
  ];
  for (const [session, report] of [['primary baseline', base], ['instrumented session', inst]]) {
    for (const w of ['glowcap-replay', 'trail-rescue-scenarios', 'ledger-session']) {
      for (const [name, [e1, m1, op1], [e2, m2, op2]] of pairs) {
        const a = runs(report, LOCAL, w, e1, m1, op1);
        const b = runs(report, LOCAL, w, e2, m2, op2);
        const da = direct(a);
        const db = direct(b);
        if (!da || !db) continue;
        const ratios = spreadOf(a.map((s, r) => (s && b[r] ? b[r].median / s.median : NaN)));
        const diff = inferred([{ sign: 1, name: `${m2} ${op2}`, runs: b }, { sign: -1, name: `${m1} ${op1}`, runs: a }], { formula: 'wasm − native' });
        keep(sec, { session, workload: w, operation: name, native: da, wasm: db, ratio: ratios, difference: diff });
        rows.push([session, w, name, directCell(da), directCell(db), `${f(ratios.median)}× [${f(ratios.min)}–${f(ratios.max)}]`, inferredCell(diff)]);
      }
    }
  }
  md.push(table(['Session', 'Workload', 'Operation', 'native µs (DIRECT)', 'wasm µs (DIRECT)', 'ratio', 'difference (INFERRED)'], rows), 'Load and restore rows use the WebAssembly glue calls (`raw.new`, `raw.restore`), which add the source and save copies into memory. The runtime sets no `#[global_allocator]`, so "native" here is the Windows system heap and "wasm" is Rust\'s bundled dlmalloc: these ratios compare that pairing, not code generation alone.', '');
}

// ---- save and restore ---------------------------------------------------------------

{
  const sec = section('lifecycle', '(f) save and (g) restore', 'At each workload\'s first episode\'s final state (the resume stream: ten minutes of play). DIRECT, 30 timed calls per run after 3 untimed.');
  const rows = [];
  const specs = [['native', 'save_json'], ['native', 'save'], ['native', 'restore.parse'], ['native', 'restore'], ['native', 'restore_json'], ['native', 'from_source'],
    ['wasm', 'raw.save'], ['wasm', 'raw.restore'], ['wasm', 'raw.new'], ['wasm', 'kit.save'], ['wasm', 'kit.restore']];
  for (const w of ['glowcap-resume', 'glowcap-replay', 'trail-rescue-scenarios', 'ledger-session']) {
    for (const t of [LOCAL, PUBLISHED]) {
      for (const [e, op] of specs) {
        const d = direct(runs(base, t, w, e, 'lifecycle', op));
        if (!d) continue;
        keep(sec, { workload: w, target: t, engine: e, op, stats: d });
        rows.push([w, t, e, op, directCell(d)]);
      }
    }
  }
  for (const t of [HIST, PUBLISHED, LOCAL]) {
    for (const op of ['adapter.resume.parse', 'adapter.resume.createPolicy', 'adapter.resume.view', 'adapter.resume']) {
      const d = direct(runs(base, t, 'glowcap-resume', 'wasm', 'adapter-resume', op));
      if (!d) continue;
      keep(sec, { workload: 'glowcap-resume', target: t, engine: 'wasm', op, stats: d });
      rows.push(['glowcap-resume (published resume method)', t, 'wasm', op, directCell(d)]);
    }
  }
  md.push(table(['Workload', 'Target', 'Engine', 'Operation', 'µs: median [runs] · p95'], rows));
}

// ---- instrumentation safeguard ----------------------------------------------------

{
  const sec = section('safeguard', 'Instrumentation safeguard', 'Each instrumented build against the ordinary build on the same events, in the same interleaved run, per event class: all events of the Glowcap replay (it has no refusals) and its idle and state-changing ticks, and each accepted and the refused class of the decision workloads. Every comparison is like for like: the WebAssembly dispatch_view probe total (the call and `JSON.parse`, no payload stringify) is compared by mean with the ordinary call\'s mean plus its `JSON.parse` mean (means add; medians do not), and by median with `raw.dispatch_view+parse`, which also includes the payload `JSON.stringify` (0.3–1.0 µs), so that median row understates the overhead by about that much; a refused event throws, so its probe total is the call alone and is compared with the ordinary call alone. i1 adds functions and changes none; i2 adds marks inside the runtime, so its totals include the marks\' own cost.');
  const rows = [];
  const GROUPS = { 'glowcap-replay': ['all', 'category: idle tick', 'category: state-changing tick'], 'trail-rescue-scenarios': ['class: evidence', 'class: commit', 'class: reopen', 'class: refused'], 'ledger-session': ['class: evidence', 'class: refused'] };
  // [instrumented target, engine, mode, op, reference terms, which classes, statistics]
  const pairs = [
    ['i1', 'native', 'web.dispatch_view', 'web.dispatch_view', [['native', 'web.dispatch_view', 'web.dispatch_view']], 'any', ['median', 'mean']],
    ['i1', 'native', 'web.dispatch_outcome', 'web.dispatch_outcome', [['native', 'web.dispatch_outcome', 'web.dispatch_outcome']], 'any', ['median', 'mean']],
    ['i1', 'native', 'apply', 'apply', [['native', 'apply', 'apply']], 'any', ['median', 'mean']],
    ['i1', 'native', 'read', 'view', [['native', 'read', 'view']], 'any', ['median', 'mean']],
    ['i1', 'wasm', 'abi.dispatch_view', 'abi.exec', [['wasm', 'abi.dispatch_view', 'abi.exec']], 'any', ['median', 'mean']],
    ['i1', 'wasm', 'abi.dispatch_outcome', 'abi.exec', [['wasm', 'abi.dispatch_outcome', 'abi.exec']], 'any', ['median', 'mean']],
    ['i1', 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse', [['wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse']], 'accepted', ['median', 'mean']],
    ['i1', 'wasm', 'raw.dispatch_view', 'raw.dispatch_view', [['wasm', 'raw.dispatch_view', 'raw.dispatch_view']], 'refused', ['median', 'mean']],
    ['i2', 'native', 'probe.dispatch_view', 'total', [['native', 'web.dispatch_view', 'web.dispatch_view']], 'any', ['median', 'mean']],
    ['i2', 'native', 'probe.dispatch_outcome', 'total', [['native', 'web.dispatch_outcome', 'web.dispatch_outcome']], 'any', ['median', 'mean']],
    ['i2', 'wasm', 'probe.dispatch_view', 'total', [['wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse']], 'accepted', ['median']],
    ['i2', 'wasm', 'probe.dispatch_view', 'total', [['wasm', 'raw.dispatch_view', 'raw.dispatch_view'], ['wasm', 'raw.dispatch_view', 'js.parse_view']], 'accepted', ['mean']],
    ['i2', 'wasm', 'probe.dispatch_view', 'total', [['wasm', 'raw.dispatch_view', 'raw.dispatch_view']], 'refused', ['median', 'mean']],
    ['i2', 'wasm', 'probe.dispatch_outcome', 'total', [['wasm', 'raw.dispatch_outcome', 'raw.dispatch_outcome+parse']], 'any', ['median', 'mean']],
  ];
  const worst = {};
  for (const [w, groups] of Object.entries(GROUPS)) {
    for (const group of groups) {
      const refused = group === 'class: refused';
      for (const [t, e, m, op, refs, which, stats] of pairs) {
        if ((which === 'accepted' && refused) || (which === 'refused' && !refused)) continue;
        for (const stat of stats) {
          const a = groupRuns(instClasses, t, w, e, m, op, group);
          const terms = refs.map(([e0, m0, op0]) => ({ sign: -1, name: `${LOCAL} ${m0} ${op0}`, runs: groupRuns(instClasses, LOCAL, w, e0, m0, op0, group) }));
          if (!a || terms.some((term) => !term.runs)) continue;
          const value = inferred([{ sign: 1, name: `${t} ${m} ${op}`, runs: a }, ...terms], { stat, formula: `${t} − ${LOCAL} (${stat})` });
          if (!value) continue;
          const perRunRef = [];
          for (let r = 0; r < a.length; r++) if (terms.every((term) => term.runs[r])) perRunRef.push(terms.reduce((sum, term) => sum + term.runs[r][stat], 0));
          const ref = spreadOf(perRunRef);
          const percent = (value.value.median / ref.median) * 100;
          const reference = refs.map(([, m0, op0]) => `${m0} / ${op0}`).join(' + ');
          keep(sec, { workload: w, group, target: t, engine: e, op: `${m} ${op}`, reference, stat, overhead: value, referenceValue: ref, overheadPercent: percent });
          const key = `${t} ${e}`;
          worst[key] = Math.max(worst[key] ?? 0, Math.abs(percent));
          rows.push([`${w}: ${group.replace(/^(class|category): /, '')}`, `${t} ${e} ${m} / ${op}`, reference, stat, f(spreadOf(a.filter(Boolean).map((s) => s[stat])).median), f(ref.median), inferredCell(value), `${f(percent)}%`]);
        }
      }
    }
  }
  md.push(table(['Workload: events', 'Instrumented', 'Ordinary (main-local)', 'Statistic', 'instrumented µs', 'ordinary µs', 'overhead (INFERRED)', 'overhead %'], rows));
  keep(sec, { largestAbsoluteOverheadPercent: worst });
  md.push(`Largest absolute overhead in the table above, by build and engine: ${Object.entries(worst).map(([key, value]) => `${key} ${f(value)}%`).join('; ')}. Because each instrumented build measurably changes ordinary-path timing, absolute timings from i1 are attribution evidence, not production-path costs, and i2 numbers are shares, converted to µs only as INFERRED against the ordinary build.`, '');
  const overhead = [];
  for (const e of ['native', 'wasm']) {
    for (const op of ['probe.mark', 'probe.now', 'js.performance_now']) {
      const d = direct(runs(inst, 'i2', 'glowcap-replay', e, 'probe.overhead', op));
      if (d) { overhead.push([e, op, directCell(d)]); keep(sec, { kind: 'mark cost', engine: e, op, stats: d }); }
    }
  }
  md.push('The cost of one mark (batches of 100):', '', table(['Engine', 'Operation', 'µs per call: median [runs] · p95'], overhead));
}

// ---- i2 shares as microseconds (INFERRED) ------------------------------------
// A region's share of the i2 call (per run, of its mean total) times the
// ordinary build's DIRECT median of the same path and class in the same
// session, paired by repeat. The uncertainty is the measured i2 overhead of
// that path and class (safeguard, means, like for like): ± spreads it over
// the regions in proportion; "worst" puts all of it in this region.

const I2_SAFEGUARD = out.sections.safeguard.rows.filter((row) => row.target === 'i2' && row.stat === 'mean');
function i2Overhead(e, m, w, group) {
  const row = I2_SAFEGUARD.find((x) => x.engine === e && x.op === `${m} total` && x.workload === w && x.group === group);
  return row ? row.overheadPercent / 100 : null;
}
const CONVERT = [
  ['language execution', ['payload.parse', 'payload.resolve', 'apply.validate', 'apply.clone', 'run.clock', 'run.prologue', 'run.rules', 'run.changes', 'bind.stale', 'bind.none', 'bind.eval', 'bind.sort', 'bind.explain', 'bind.write', 'apply.swap', 'apply.rollback']],
  ['  rules', ['run.prologue', 'run.rules']],
  ['  binding evaluation', ['bind.stale', 'bind.none', 'bind.eval', 'bind.sort', 'bind.explain', 'bind.write']],
  ['  session copy + old session dropped (or rolled back)', ['apply.clone', 'apply.swap', 'apply.rollback']],
  ['view building', ['view.enter', 'view.names', 'view.sort', 'view.commitments', 'view.relations', 'view.build']],
  ['  NodeId→name map', ['view.enter', 'view.names']],
  ['  symbol sort', ['view.sort']],
  ['  commitment records', ['view.commitments']],
  ['  relation records', ['view.relations']],
  ['snapshot + outcome building', ['snap.enter', 'snap.names', 'snap.sort', 'snap.symbols', 'snap.shown', 'snap.values', 'snap.records', 'snap.static', 'snap.relations', 'snap.build', 'outcome.built']],
  ['serialization (serde_json) + drop of what was serialized', ['web.serialize', 'web.drop']],
  ['bridge in and out', ['web.enter', 'exit']],
  ['JS JSON.parse', ['js.parse']],
  ['serialization + bridge + JSON.parse', ['web.serialize', 'web.drop', 'web.enter', 'exit', 'js.parse']],
];
const CONVERT_PATHS = [
  ['wasm', 'probe.dispatch_view', ['wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse'], 'wasm dispatch_view + JSON.parse (the web pages\' path; reference raw.dispatch_view+parse)'],
  ['native', 'probe.dispatch_view', ['native', 'web.dispatch_view', 'web.dispatch_view'], 'native dispatch_view (reference web.dispatch_view)'],
  ['wasm', 'probe.dispatch_outcome', ['wasm', 'raw.dispatch_outcome', 'raw.dispatch_outcome+parse'], 'wasm dispatch_outcome + JSON.parse (the kit\'s call; reference raw.dispatch_outcome+parse)'],
  ['native', 'probe.dispatch_outcome', ['native', 'web.dispatch_outcome', 'web.dispatch_outcome'], 'native dispatch_outcome (reference web.dispatch_outcome)'],
];
function i2Share(w, e, m, group, labels) {
  const total = groupRuns(instClasses, 'i2', w, e, m, 'total', group);
  if (!total) return null;
  return total.map((tot, r) => {
    if (!tot) return null;
    let sum = 0;
    for (const label of labels) {
      const region = groupRuns(instClasses, 'i2', w, e, m, label, group)?.[r];
      if (region) sum += (region.mean * region.n) / tot.n;
    }
    return sum / tot.mean;
  });
}
// The ordinary operation an i2 call is converted against: the same exported
// call; a refused dispatch_view throws, so its reference is the call alone.
function i2Reference(e, m, group) {
  if (e === 'wasm' && m === 'probe.dispatch_view') return group === 'class: refused' ? ['wasm', 'raw.dispatch_view', 'raw.dispatch_view'] : ['wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse'];
  if (e === 'wasm') return ['wasm', 'raw.dispatch_outcome', 'raw.dispatch_outcome+parse'];
  return m === 'probe.dispatch_view' ? ['native', 'web.dispatch_view', 'web.dispatch_view'] : ['native', 'web.dispatch_outcome', 'web.dispatch_outcome'];
}
const convertedCell = (x) => (x?.share ? `${f(x.share.median * 100)}% (≈${f(x.value.median)})` : '-');

function i2Inferred(w, e, m, group, labels, [e0, m0, op0]) {
  const shares = i2Share(w, e, m, group, labels);
  const reference = groupRuns(instClasses, LOCAL, w, e0, m0, op0, group);
  if (!shares || !reference) return null;
  const per = [];
  const shareRuns = [];
  for (let r = 0; r < shares.length; r++) {
    if (shares[r] === null || !reference[r]) continue;
    per.push(shares[r] * reference[r].median);
    shareRuns.push(shares[r]);
  }
  if (!per.length) return null;
  const value = spreadOf(per);
  const overhead = i2Overhead(e, m, w, group);
  const referenceMedian = spreadOf(reference.filter(Boolean).map((s) => s.median)).median;
  return {
    label: 'INFERRED',
    formula: `i2 share of the call × ${LOCAL} ${m0} / ${op0} DIRECT median (same session, class, repeat)`,
    share: spreadOf(shareRuns),
    value,
    perRun: per.map((v) => Number(v.toFixed(3))),
    overheadFraction: overhead,
    plusMinus: overhead === null ? null : Math.abs(overhead) * value.median,
    worstCase: overhead === null ? null : Math.abs(overhead) * referenceMedian,
  };
}
const i2Cell = (x) => (x ? `${f(x.value.median)} [${f(x.value.min)}–${f(x.value.max)}] ±${f(x.plusMinus)} (share ${f(x.share.median * 100)}%)` : '-');

// ---- i2 proportions ------------------------------------------------------------------

const REGION_GROUPS = [
  ['bridge in (JS call, argument copy)', ['web.enter']],
  ['payload parse + name resolution', ['payload.parse', 'payload.resolve']],
  ['transaction setup + session copy', ['apply.validate', 'apply.clone']],
  ['clock and due qualifications', ['run.clock']],
  ['rules', ['run.prologue', 'run.rules']],
  ['change detection', ['run.changes']],
  ['binding evaluation', ['bind.stale', 'bind.none', 'bind.eval', 'bind.sort', 'bind.explain', 'bind.write']],
  ['commit (old session dropped) or rollback', ['apply.swap', 'apply.rollback']],
  ['view: NodeId→name map', ['view.enter', 'view.names']],
  ['view: symbol sort', ['view.sort']],
  ['view: commitment records', ['view.commitments']],
  ['view: relation records', ['view.relations']],
  ['view: assemble + drop map', ['view.build']],
  ['snapshot: name map + symbol sort', ['snap.enter', 'snap.names', 'snap.sort']],
  ['snapshot: symbols + commitment records', ['snap.symbols']],
  ['snapshot: bindings, values, records, static world (clones)', ['snap.shown', 'snap.values', 'snap.records', 'snap.static']],
  ['snapshot: relation records + assemble', ['snap.relations', 'snap.build']],
  ['outcome wrapper', ['outcome.built']],
  ['serialize (serde_json)', ['web.serialize']],
  ['drop the built view/outcome', ['web.drop']],
  ['bridge out (return, decode, free)', ['exit']],
  ['JS JSON.parse', ['js.parse']],
];

{
  const sec = section('proportions', 'Where the time goes inside one call (i2 marks: proportions only)', 'Each cell: the mean time per event of one disjoint region between consecutive marks as a share of the instrumented call\'s mean total (median over runs), and in parentheses that share converted to µs (INFERRED: share × the ordinary build\'s DIRECT median of the same call and event class in the instrumented session, paired by repeat; the refused dispatch_view reference is the call alone, since it throws). The regions of a call add up exactly to its total, so the shares add to 100%; means, not medians, because only means add. The i2 marks cost about 0.04–0.05 µs each and read a 100 ns clock, so the instrumented totals are not production costs; the last row gives the measured overhead of each path and class (safeguard).');
  for (const [w, groups] of [['glowcap-replay', ['category: idle tick', 'category: state-changing tick']], ['trail-rescue-scenarios', ['class: evidence', 'class: commit', 'class: reopen', 'class: refused']], ['ledger-session', ['class: evidence', 'class: refused']]]) {
    for (const [e, m] of [['native', 'probe.dispatch_view'], ['wasm', 'probe.dispatch_view'], ['native', 'probe.dispatch_outcome'], ['wasm', 'probe.dispatch_outcome']]) {
      const rows = [];
      const header = ['Region: share (≈ µs, INFERRED)', ...groups.map((group) => group.replace(/^(category|class): /, ''))];
      const totals = groups.map((group) => groupRuns(instClasses, 'i2', w, e, m, 'total', group));
      if (totals.every((t) => !t)) continue;
      const share = (group, index, labels) => {
        const total = totals[index];
        if (!total) return null;
        const perRun = total.map((t, r) => {
          if (!t) return NaN;
          let sum = 0;
          for (const label of labels) {
            const region = groupRuns(instClasses, 'i2', w, e, m, label, group)?.[r];
            if (region) sum += (region.mean * region.n) / t.n;
          }
          return sum;
        });
        return { us: spreadOf(perRun), share: spreadOf(perRun.map((value, r) => (value / total[r].mean) * 100)) };
      };
      for (const [name, labels] of REGION_GROUPS) {
        const cells = groups.map((group, index) => share(group, index, labels));
        if (cells.every((cell) => !cell || !cell.us || cell.us.median < 0.005)) continue;
        const converted = groups.map((group) => i2Inferred(w, e, m, group, labels, i2Reference(e, m, group)));
        keep(sec, { workload: w, engine: e, mode: m, region: name, labels, byGroup: Object.fromEntries(groups.map((group, index) => [group, { ...cells[index], inferred: converted[index] }])) });
        rows.push([name, ...converted.map(convertedCell)]);
      }
      const totalCells = totals.map((t) => { const s = spreadOf((t ?? []).filter(Boolean).map((x) => x.mean)); return s ? `mean ${f(s.median)} µs` : '-'; });
      rows.push(['instrumented total (i2, DIRECT)', ...totalCells]);
      rows.push(['ordinary reference (DIRECT median)', ...groups.map((group) => {
        const reference = groupRuns(instClasses, LOCAL, w, ...i2Reference(e, m, group), group);
        const s = reference ? spreadOf(reference.filter(Boolean).map((x) => x.median)) : null;
        return s ? `${f(s.median)} µs` : '-';
      })]);
      rows.push(['i2 overhead (means, like for like)', ...groups.map((group) => {
        const o = i2Overhead(e, m, w, group);
        return o === null ? '-' : `${f(o * 100)}%`;
      })]);
      md.push(`### ${w}, ${e} ${m.replace('probe.', '')}`, '', table(header, rows));
    }
  }
}

// The same regions in the owner's three buckets, per event kind.
{
  const BUCKETS = [
    ['language execution', ['payload.parse', 'payload.resolve', 'apply.validate', 'apply.clone', 'run.clock', 'run.prologue', 'run.rules', 'run.changes', 'bind.stale', 'bind.none', 'bind.eval', 'bind.sort', 'bind.explain', 'bind.write', 'apply.swap', 'apply.rollback']],
    ['  of which rules', ['run.prologue', 'run.rules']],
    ['  of which binding evaluation', ['bind.stale', 'bind.none', 'bind.eval', 'bind.sort', 'bind.explain', 'bind.write']],
    ['  of which session copy + old session dropped', ['apply.clone', 'apply.swap', 'apply.rollback']],
    ['view building', ['view.enter', 'view.names', 'view.sort', 'view.commitments', 'view.relations', 'view.build']],
    ['snapshot building (outcome)', ['snap.enter', 'snap.names', 'snap.sort', 'snap.symbols', 'snap.shown', 'snap.values', 'snap.records', 'snap.static', 'snap.relations', 'snap.build', 'outcome.built']],
    ['serialization (serde_json) + drop of what was serialized', ['web.serialize', 'web.drop']],
    ['bridge in and out (JS call, argument copy, return, decode, free)', ['web.enter', 'exit']],
    ['JS JSON.parse', ['js.parse']],
  ];
  const sec = section('buckets', 'The three buckets inside one call (i2 marks: proportions only)', 'The region shares above, summed into language execution, view or snapshot building, and serialization + bridge + JS parse: shares of the instrumented call\'s mean total, which these rows (without the "of which" lines) add up to exactly, and in parentheses the share converted to µs as above (INFERRED). The i2 totals carry the marks\' cost; the converted values carry the overhead of each path and class as their uncertainty (section 12).');
  for (const [e, m] of [['wasm', 'probe.dispatch_view'], ['wasm', 'probe.dispatch_outcome'], ['native', 'probe.dispatch_view'], ['native', 'probe.dispatch_outcome']]) {
    const rows = [];
    for (const [name, labels] of BUCKETS) {
      const cells = WORKLOAD_GROUPS.map(([w, group]) => {
        const total = groupRuns(instClasses, 'i2', w, e, m, 'total', group);
        if (!total) return null;
        const perRun = total.map((t, r) => {
          if (!t) return NaN;
          let sum = 0;
          for (const label of labels) {
            const region = groupRuns(instClasses, 'i2', w, e, m, label, group)?.[r];
            if (region) sum += (region.mean * region.n) / t.n;
          }
          return sum;
        });
        return { us: spreadOf(perRun), share: spreadOf(perRun.map((value, r) => (value / total[r].mean) * 100)) };
      });
      if (cells.every((cell) => !cell?.us || cell.us.median < 0.005)) continue;
      const converted = WORKLOAD_GROUPS.map(([w, group]) => i2Inferred(w, e, m, group, labels, i2Reference(e, m, group)));
      keep(sec, { engine: e, mode: m, bucket: name, labels, byKind: Object.fromEntries(WORKLOAD_GROUPS.map(([, , kind], index) => [kind, { ...cells[index], inferred: converted[index] }])) });
      rows.push([name, ...converted.map(convertedCell)]);
    }
    rows.push(['instrumented mean total (i2, DIRECT), µs', ...WORKLOAD_GROUPS.map(([w, group]) => {
      const s = spreadOf((groupRuns(instClasses, 'i2', w, e, m, 'total', group) ?? []).filter(Boolean).map((x) => x.mean));
      return s ? f(s.median) : '-';
    })]);
    md.push(`### ${e} ${m.replace('probe.', '')}`, '', table(['Bucket: share (≈ µs, INFERRED)', ...WORKLOAD_GROUPS.map(([, , name]) => name)], rows));
  }
}

{
  const sec = section('i2-inferred', 'i2 shares converted to microseconds (INFERRED)', 'Each cell: the region\'s share of the i2 call (median over runs, in brackets its range) times the ordinary build\'s DIRECT median of the same path and event class in the instrumented session, paired by repeat: µs median [lowest–highest run] ± the i2 overhead of that path and class spread in proportion. The last row gives the overhead and the worst case, all of it in one region. The wasm dispatch_view reference includes the payload stringify (0.3–1.0 µs) that the i2 call does not.');
  for (const [e, m, ref, title] of CONVERT_PATHS) {
    const rows = [];
    for (const [name, labels] of CONVERT) {
      const cells = WORKLOAD_GROUPS.map(([w, group, kind]) => {
        const x = i2Inferred(w, e, m, group, labels, ref);
        if (x && x.value.median >= 0.005) keep(sec, { path: title, quantity: name.trim(), eventKind: kind, ...x });
        return x && x.value.median >= 0.005 ? i2Cell(x) : '-';
      });
      if (cells.every((cell) => cell === '-')) continue;
      rows.push([name, ...cells]);
    }
    rows.push(['i2 overhead (mean, like for like); worst case µs', ...WORKLOAD_GROUPS.map(([w, group]) => {
      const o = i2Overhead(e, m, w, group);
      const reference = groupRuns(instClasses, LOCAL, w, ...ref, group);
      const median = reference ? spreadOf(reference.filter(Boolean).map((s) => s.median)).median : null;
      return o === null ? '-' : `${f(o * 100)}%; ±${f(Math.abs(o) * median)}`;
    })]);
    md.push(`### ${title}`, '', table(['Quantity (µs, INFERRED)', ...WORKLOAD_GROUPS.map(([, , name]) => name)], rows));
  }
}

// ---- core placement ------------------------------------------------------------
// A run of run.mjs --affinity-set (targets LABEL@MASK) and verbatim.mjs
// --affinity-set, in one quiet session.
const coresDir = option('cores')[0] ? path.resolve(option('cores')[0]) : null;
if (coresDir) {
  const cores = await readJson(path.join(coresDir, 'results.json'));
  const coreClasses = await readJson(path.join(coresDir, 'classes.json'));
  const coreVerbatim = await Promise.all(option('cores-verbatim').map(readJson));
  const masks = cores.environment.pinning.affinitySet.map((entry) => entry.mask);
  const HOME = '0x3C00';
  const cpuSets = cores.environment.machine.cpuSetList ?? [];
  const sec = section('cores', 'Core placement: the same bytes on each performance-core pair', `One quiet session (${cores.runId}; total CPU mean ${cores.environment.loadDuring?.counters.total.mean}%, p95 ${cores.environment.loadDuring?.counters.total.p95}%): every job ran once under each mask per repeat, the masks rotating with the targets, 3 repeats, High priority. Each cell: DIRECT median over the 3 runs [lowest–highest run]; the ratio to the same job on ${HOME} (the mask of every other table) is INFERRED, per run, paired by repeat.`);
  // Which processors each mask held, their CPU-set classes, and the clock the
  // busy processor ran at during the jobs (typeperf Actual Frequency, 1 s).
  const identity = [];
  for (const mask of masks) {
    const processors = cores.environment.pinning.affinitySet.find((entry) => entry.mask === mask).logicalProcessors;
    const sets = processors.map((lp) => cpuSets.find((set) => set.logicalProcessor === lp));
    const jobs = [];
    for (const byWorkload of Object.values(cores.results)) for (const byEngine of Object.values(byWorkload)) for (const byMode of Object.values(byEngine)) for (const mode of Object.values(byMode)) for (const run of mode.runs) if (run.affinityMask === mask && run.cores?.cores) jobs.push(run);
    const busy = {};
    const clocks = [];
    for (const run of jobs) {
      for (const [lp, stats] of Object.entries(run.cores.cores)) {
        if (!processors.includes(Number(lp))) continue;
        (busy[lp] ??= []).push(stats.busyPercent ?? 0);
        if (stats.mhzWhileBusy) clocks.push(stats.mhzWhileBusy);
      }
    }
    const seen = {};
    for (const run of jobs) for (const [lp, count] of Object.entries(run.extra?.processorsSeen ?? {})) seen[lp] = (seen[lp] ?? 0) + count;
    const mean = (list) => (list.length ? list.reduce((a, b) => a + b, 0) / list.length : null);
    const row = { mask, processors, efficiencyClass: sets.map((s) => s?.efficiencyClass), schedulingClass: sets.map((s) => s?.schedulingClass), meanBusyPercent: Object.fromEntries(Object.entries(busy).map(([lp, list]) => [lp, Number(mean(list).toFixed(1))])), clockWhileBusyMHz: spreadOf(clocks), nativeEventsByProcessor: seen };
    keep(sec, { kind: 'identity', ...row });
    identity.push([mask, processors.join(', '), row.efficiencyClass.join(', '), row.schedulingClass.join(', '), Object.entries(row.meanBusyPercent).map(([lp, v]) => `${lp}: ${f(v)}%`).join(', '), row.clockWhileBusyMHz ? `${f(row.clockWhileBusyMHz.median)} [${f(row.clockWhileBusyMHz.min)}–${f(row.clockWhileBusyMHz.max)}]` : '-', Object.entries(seen).map(([lp, count]) => `${lp}: ${count}`).join(', ') || '-']);
  }
  md.push('Which processors each mask held (CPU-set classes read when the run started; Windows changes the scheduling class), how busy each of them was on average over the jobs that ran under the mask, the clock of the mask\'s processors while more than half busy (typeperf `Actual Frequency`, 1 s samples: the median and the range of those samples), and on which processor the native benchmark found itself at each timed event (`GetCurrentProcessorNumber`, outside the timed region; events summed over the native jobs):', '',
    table(['Mask', 'Logical processors', 'Efficiency class', 'Scheduling class', 'Mean busy, per processor', 'Clock while busy, MHz: median [range]', 'Native events by processor'], identity));

  const SPECS = [
    ['main-local', 'glowcap-replay', 'wasm', 'published-method', 'adapter.dispatch+view', 'all', 'Glowcap published method (adapter dispatch + view)'],
    ['hist-e6ace96', 'glowcap-replay', 'wasm', 'published-method', 'adapter.dispatch+view', 'all', 'the same, historical runtime e6ace96'],
    ['main-local', 'glowcap-replay', 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse', 'category: idle tick', 'Glowcap idle tick: dispatch_view + JSON.parse'],
    ['main-local', 'glowcap-replay', 'wasm', 'abi.dispatch_view', 'abi.exec', 'category: idle tick', 'Glowcap idle tick: wasm execution only'],
    ['main-local', 'glowcap-replay', 'native', 'web.dispatch_view', 'web.dispatch_view', 'category: idle tick', 'Glowcap idle tick: native dispatch_view'],
    ['main-local', 'glowcap-replay', 'wasm', 'kit', 'kit.dispatch+view', 'category: idle tick', 'Glowcap idle tick: kit dispatch + view'],
    ['main-local', 'glowcap-replay', 'wasm', 'raw.dispatch_view', 'js.parse_view', 'category: idle tick', 'Glowcap idle tick: JSON.parse of the view alone'],
    ['main-local', 'trail-rescue-scenarios', 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse', 'class: evidence', 'Trail Rescue evidence: dispatch_view + JSON.parse'],
    ['main-local', 'trail-rescue-scenarios', 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse', 'class: commit', 'Trail Rescue commit: dispatch_view + JSON.parse'],
    ['main-local', 'trail-rescue-scenarios', 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse', 'class: reopen', 'Trail Rescue reopen: dispatch_view + JSON.parse'],
    ['main-local', 'trail-rescue-scenarios', 'wasm', 'abi.dispatch_view', 'abi.exec', 'class: commit', 'Trail Rescue commit: wasm execution only'],
    ['main-local', 'trail-rescue-scenarios', 'native', 'web.dispatch_view', 'web.dispatch_view', 'class: commit', 'Trail Rescue commit: native dispatch_view'],
    ['main-local', 'trail-rescue-scenarios', 'wasm', 'kit', 'kit.dispatch+view', 'class: commit', 'Trail Rescue commit: kit dispatch + view'],
  ];
  const cell = (t, w, e, m, op, group, mask) => (group === 'all' ? runs(cores, `${t}@${mask}`, w, e, m, op) : groupRuns(coreClasses, `${t}@${mask}`, w, e, m, op, group));
  const rows = [];
  const ratioRows = [];
  for (const [t, w, e, m, op, group, name] of SPECS) {
    const home = cell(t, w, e, m, op, group, HOME);
    const cells = masks.map((mask) => {
      const d = direct(cell(t, w, e, m, op, group, mask));
      if (d) keep(sec, { kind: 'direct', path: name, target: t, workload: w, engine: e, mode: m, op, group, mask, stats: d });
      return directCell(d, { p95: false });
    });
    const ratios = masks.map((mask) => {
      const x = ratioInferred(cell(t, w, e, m, op, group, mask), home, { formula: `median on ${mask} ÷ median on ${HOME}, per run` });
      if (x) keep(sec, { kind: 'ratio to home', path: name, mask, ...x });
      return inferredCell(x);
    });
    if (cells.every((c) => c === '-')) continue;
    rows.push([name, ...cells]);
    ratioRows.push([name, ...ratios]);
  }
  md.push('DIRECT, µs: median over runs [lowest–highest run]:', '', table(['Path', ...masks.map((mask) => `${mask} (${cores.environment.pinning.affinitySet.find((x) => x.mask === mask).logicalProcessors.join(',')})`)], rows));
  md.push(`The same, as a ratio to ${HOME} (INFERRED, per run, paired by repeat):`, '', table(['Path', ...masks], ratioRows));
  // WebAssembly over native on each pair: does the ratio survive?
  const wn = [];
  for (const [w, group, name] of [['glowcap-replay', 'category: idle tick', 'Glowcap idle tick'], ['trail-rescue-scenarios', 'class: commit', 'Trail Rescue commit'], ['trail-rescue-scenarios', 'class: evidence', 'Trail Rescue evidence']]) {
    wn.push([name, ...masks.map((mask) => {
      const x = ratioInferred(groupRuns(coreClasses, `main-local@${mask}`, w, 'wasm', 'abi.dispatch_view', 'abi.exec', group), groupRuns(coreClasses, `main-local@${mask}`, w, 'native', 'web.dispatch_view', 'web.dispatch_view', group), { formula: 'wasm abi.exec ÷ native web.dispatch_view, per run' });
      if (x) keep(sec, { kind: 'wasm over native', eventKind: name, mask, ...x });
      return inferredCell(x);
    })]);
  }
  md.push('WebAssembly execution over native, the same exported dispatch_view, on each pair (INFERRED ratio, per run):', '', table(['Event kind', ...masks], wn));
  // The published command itself on each pair, and unpinned.
  const vrows = [];
  for (const record of coreVerbatim) {
    const groupsSeen = new Map();
    for (const run of record.runs) {
      const key = `${run.tree} ${run.affinity ?? 'none'}`;
      if (!groupsSeen.has(key)) groupsSeen.set(key, []);
      groupsSeen.get(key).push(run);
    }
    for (const [key, list] of groupsSeen) {
      const [tree, mask] = key.split(' ');
      const medians = list.map((run) => run.lines.caveat5?.median).filter(Number.isFinite);
      const med = spreadOf(medians);
      const clocks = list.flatMap((run) => Object.values(run.cores?.cores ?? {}).map((c) => c.mhzWhileBusy).filter(Boolean));
      const where = list.map((run) => Object.entries(run.cores?.cores ?? {}).filter(([, c]) => (c.busyPercent ?? 0) > 25).map(([lp, c]) => `${lp} (${f(c.busyPercent)}%)`).join(' ') || '-');
      keep(sec, { kind: 'published command', tree, mask, pinning: record.pinning, medians, clocks, where, runs: list.map((run) => ({ repeat: run.repeat, lines: run.lines, cores: run.cores })) });
      vrows.push([tree, mask === 'none' ? 'unpinned, normal priority (as published)' : `${mask}, High`, med ? `${f(med.median)} [${f(med.min)}–${f(med.max)}]` : '-', medians.map((v) => f(v)).join(', '), clocks.length ? f(spreadOf(clocks).median) : '-', where.join('; ')]);
    }
  }
  md.push('The published command itself (`node experiments/glowcap/harness.mjs --bench`, caveat5 after five other implementations in one process), on each pair and unpinned:', '', table(['Tree', 'Pinning', 'caveat5 median µs [runs]', 'per-run medians', 'clock while busy, MHz', 'processors more than 25% busy during each run'], vrows));
}

// ---- copies: state and provenance copying per event (i3) --------------------------
const copiesDir = option('copies')[0] ? path.resolve(option('copies')[0]) : null;
if (copiesDir) {
  const copies = await readJson(path.join(copiesDir, 'results.json'));
  const copyClasses = await readJson(path.join(copiesDir, 'classes.json'));
  // The cost of one clock read: natively the time-stamp counter (the
  // allocator check measured it, batches of 1,000), in WebAssembly the
  // host's performance.now() (the i2 session's js.performance_now).
  const allocbench = option('allocbench')[0] ? await readJson(path.resolve(option('allocbench')[0])) : null;
  const rdtsc = allocbench ? spreadOf(allocbench.runs.filter((run) => run.summaries.rdtsc_read).map((run) => run.summaries.rdtsc_read.median / 1000)) : null;
  const jsNow = direct(runs(inst, 'i2', 'glowcap-replay', 'wasm', 'probe.overhead', 'js.performance_now'));
  const clockRead = { native: rdtsc?.median ?? null, wasm: jsNow?.median?.median ?? null };
  const sec = section('copies', 'State and provenance copying per event (i3)', `The i3 session (${copies.runId}; total CPU mean ${copies.environment.loadDuring?.counters.total.mean}%, p95 ${copies.environment.loadDuring?.counters.total.p95}%): the i3 copy and the ordinary build, interleaved, 3 repeats, pinned to 0x3C00 at High priority. Per event, means (they add; a median of a mostly-zero quantity says little): the time inside the \`Arc::make_mut\` calls that copied a shared structure (DIRECT in i3, the timers around only those calls), how many there were, and the provenance copies (count and names copied; natively also their time, outside structure copies). Shares are of the i3 call's own mean total (the same call, so exact for means).`);
  const GROUPS = [...WORKLOAD_GROUPS, ['trail-rescue-scenarios', 'class: refused', 'Trail Rescue refused'], ['ledger-session', 'class: refused', 'ledger refused']];
  const COPY_GROUPS = ['states', 'graph', 'symbols', 'journal', 'commitments', 'qualifications', 'other'];
  const meanOf = (w, e, m, op, group) => spreadOf((groupRuns(copyClasses, 'i3', w, e, m, op, group) ?? []).filter(Boolean).map((s) => s.mean));
  const shareOf = (w, e, m, op, group, totalOp) => {
    const a = groupRuns(copyClasses, 'i3', w, e, m, op, group);
    const b = groupRuns(copyClasses, 'i3', w, e, m, totalOp, group);
    if (!a || !b) return null;
    return spreadOf(a.map((s, r) => (s && b[r] ? (s.mean / b[r].mean) * 100 : NaN)));
  };
  for (const [e, m, totalOp, title] of [['wasm', 'copy.dispatch_view', 'call', 'wasm dispatch_view (the call, every event)'], ['native', 'copy.dispatch_view', 'total', 'native dispatch_view'], ['wasm', 'copy.dispatch_outcome', 'call', 'wasm dispatch_outcome (the call, every event)'], ['native', 'copy.dispatch_outcome', 'total', 'native dispatch_outcome']]) {
    const rows = [];
    const add = (name, op, kind) => {
      const cells = GROUPS.map(([w, group, eventKind]) => {
        const v = meanOf(w, e, m, op, group);
        if (!v) return '-';
        keep(sec, { path: title, quantity: name, op, eventKind, mean: v, ...(kind === 'time' ? { sharePercent: shareOf(w, e, m, op, group, totalOp) } : {}) });
        if (kind === 'time') {
          const s = shareOf(w, e, m, op, group, totalOp);
          return `${f(v.median)} [${f(v.min)}–${f(v.max)}] (${f(s?.median)}%)`;
        }
        return `${f(v.median)}`;
      });
      if (cells.every((c) => c === '-' || c.startsWith('0.00 '))) return;
      rows.push([name, ...cells]);
    };
    add('the i3 call (mean µs)', totalOp, 'total');
    add('structure copies (make_mut that copied): time, µs (share of the call)', 'copy.cow', 'time');
    add('  how many per event', 'count.cow', 'count');
    // Less one clock read per timed copy: an estimate of the timers' own
    // share of the timed regions (INFERRED, means).
    const lessReads = (name, timeOp, countOp, read) => {
      if (!read) return;
      const cells = GROUPS.map(([w, group, eventKind]) => {
        const time = meanOf(w, e, m, timeOp, group);
        const count = meanOf(w, e, m, countOp, group);
        if (!time || !count) return '-';
        const value = time.median - count.median * read;
        keep(sec, { path: title, quantity: name, eventKind, label: 'INFERRED', formula: `mean(${timeOp}) − mean(${countOp}) × ${read} µs`, value });
        return f(Math.max(value, 0));
      });
      rows.push([name, ...cells]);
    };
    lessReads(`  the same less one clock read per copy (INFERRED: time − count × ${(clockRead[e] * 1000).toFixed(1)} ns)`, 'copy.cow', 'count.cow', clockRead[e]);
    for (const group of COPY_GROUPS) {
      add(`  ${group}: time, µs (share)`, `copy.cow.${group}`, 'time');
      add(`  ${group}: copies per event`, `count.cow.${group}`, 'count');
    }
    if (e === 'native') {
      add('provenance copies outside structure copies: time, µs (share)', 'copy.provenance', 'time');
      lessReads(`  the same less one clock read per copy (INFERRED: time − count × ${(clockRead.native * 1000).toFixed(1)} ns)`, 'copy.provenance', 'count.provenance', clockRead.native);
    }
    add('provenance copies per event', 'count.provenance', 'count');
    add('  names copied per event', 'count.provenance_names', 'count');
    add('provenance copied inside structure copies, per event', 'count.provenance_in_cow', 'count');
    md.push(`### ${title}`, '', table(['Quantity (means per event)', ...GROUPS.map(([, , name]) => name)], rows));
  }
  // Safeguard: i3 against the ordinary build, same session.
  const rows = [];
  for (const [w, group, eventKind] of GROUPS) {
    for (const [e, m, op, e0, m0, op0] of [['native', 'copy.dispatch_view', 'total', 'native', 'web.dispatch_view', 'web.dispatch_view'], ['native', 'copy.dispatch_outcome', 'total', 'native', 'web.dispatch_outcome', 'web.dispatch_outcome'], ['wasm', 'copy.dispatch_view', 'call', 'wasm', 'raw.dispatch_view', 'raw.dispatch_view'], ['wasm', 'copy.dispatch_outcome', 'call', 'wasm', 'raw.dispatch_outcome', 'raw.dispatch_outcome']]) {
      for (const stat of ['median', 'mean']) {
        const a = groupRuns(copyClasses, 'i3', w, e, m, op, group);
        const b = groupRuns(copyClasses, LOCAL, w, e0, m0, op0, group);
        const value = inferred([{ sign: 1, name: `i3 ${m} ${op}`, runs: a }, { sign: -1, name: `${LOCAL} ${m0} ${op0}`, runs: b }], { stat, formula: `i3 − ${LOCAL} (${stat})` });
        if (!value) continue;
        const ref = spreadOf(b.filter(Boolean).map((s) => s[stat]));
        const percent = (value.value.median / ref.median) * 100;
        keep(sec, { kind: 'safeguard', eventKind, engine: e, op: `${m} ${op}`, reference: `${m0} ${op0}`, stat, overhead: value, referenceValue: ref, overheadPercent: percent });
        rows.push([eventKind, `i3 ${e} ${m} / ${op}`, `${m0} / ${op0}`, stat, f(spreadOf(a.filter(Boolean).map((s) => s[stat])).median), f(ref.median), inferredCell(value), `${f(percent)}%`]);
      }
    }
  }
  md.push(`One clock read, as subtracted above: natively ${(clockRead.native * 1000).toFixed(1)} ns (\`rdtsc\`, the allocator check's batches of 1,000), in WebAssembly ${(clockRead.wasm * 1000).toFixed(0)} ns (\`performance.now()\` from JavaScript, the i2 session; a call from WebAssembly through the import costs at least that). Timing a copy puts about one read inside the timed region; the rest of the timers' cost falls outside it but inside the call (the safeguard below).`, '');
  md.push('i3 against the ordinary build (same session, same events):', '', table(['Event kind', 'Instrumented', 'Ordinary (main-local)', 'Statistic', 'i3 µs', 'ordinary µs', 'overhead (INFERRED)', 'overhead %'], rows));
}

// ---- the allocator, synthetically ------------------------------------------------
if (option('allocbench')[0]) {
  const bench = await readJson(path.resolve(option('allocbench')[0]));
  const sec = section('allocator', 'The allocator behind the native-against-WebAssembly ratios (synthetic)', 'The same Rust code (a throwaway crate with no dependencies, the runtime\'s release profile and toolchain) natively, with the Windows system heap, and as `wasm32-unknown-unknown` in Node, with Rust\'s bundled dlmalloc; pinned to 0x3C00 at High priority, 3 repeats alternating which engine goes first. Each call times a loop; the unit is ns per loop iteration. DIRECT (synthetic): median over the 3 runs of each run\'s median of 200 timed calls (after 20 untimed) [lowest–highest run]; the ratio is per run, paired by repeat.');
  const WHAT = {
    map_copy: 'copy a 64-entry map of names to (number, provenance names) and drop the copy',
    map_walk: 'walk the same map, reading every string (no allocation)',
    alloc_ring: 'allocate a 96–143 byte buffer, freeing the one 64 allocations older',
    touch_ring: 'the same writes into a preallocated ring (no allocation)',
  };
  const rows = [];
  for (const [name, what] of Object.entries(WHAT)) {
    const per = (engine) => { const list = []; for (const run of bench.runs) if (run.engine === engine) list[run.repeat] = run.summaries[name]; return list; };
    const native = per('native');
    const wasm = per('wasm');
    const ratio = ratioInferred(wasm, native, { formula: `wasm ÷ native, ${name}, per repeat` });
    keep(sec, { name, what, native: direct(native), wasm: direct(wasm), ratio });
    rows.push([`\`${name}\`: ${what}`, directCell(direct(native), { p95: false }), directCell(direct(wasm), { p95: false }), inferredCell(ratio)]);
  }
  md.push(table(['Loop', 'native ns (system heap)', 'wasm ns (dlmalloc)', 'wasm ÷ native (INFERRED)'], rows));
}

await mkdir(outDir, { recursive: true });
await writeFile(path.join(outDir, 'attribution.json'), `${JSON.stringify(out)}\n`);
await writeFile(path.join(outDir, 'tables.md'), `${md.join('\n')}\n`);
console.log(`wrote ${path.join(outDir, 'attribution.json')} and tables.md`);
