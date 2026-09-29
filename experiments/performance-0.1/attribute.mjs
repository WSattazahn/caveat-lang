// The tables and derived numbers of RESULTS.md, computed from the committed
// raw results so every figure there can be traced to a measurement.
//
//   node experiments/performance-0.1/attribute.mjs --baseline=DIR --instrumented=DIR
//        --verbatim=FILE [--verbatim=FILE] --out=DIR
//
// DIRs are results directories holding results.json and classes.json
// (analyze.mjs). Writes attribution.json (every derived number with its
// formula and paired inputs) and tables.md.
//
// Labels: DIRECT is a statistic of samples timed around exactly that
// operation. INFERRED is a difference of DIRECT medians. A paired difference
// is taken run by run (the same repeat, whose processes ran back to back in
// one interleaved block), then summarized as the median of the per-run
// differences with their lowest and highest; ± is a conservative bound, half
// the sum of the operands' run-to-run ranges. Medians do not add, so an
// INFERRED remainder is not a measurement of that region.
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
// An optional second complete baseline run (another session, other load).
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
    `The primary baseline (${base.runId}; total CPU during the run ${loadOf(base)}) against the other complete runs of the same suite (${replicates.map(({ report }) => `${report.runId}: ${loadOf(report)}`).join('; ')}). Each cell: median over 3 runs [lowest–highest run], and the change from the primary.`);
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
      keep(sec, { target: t, workload: w, engine: e, mode: m, op, primary: a, others: others.map((b, index) => ({ runId: replicates[index].report.runId, stats: b, changePercent: b ? ((b.median.median - a.median.median) / a.median.median) * 100 : null })) });
      rows.push([t, w, `${e} ${m} / ${op}`, directCell(a, { p95: false }), ...others.map((b) => (b ? `${directCell(b, { p95: false })} (${b.median.median >= a.median.median ? '+' : ''}${f(((b.median.median - a.median.median) / a.median.median) * 100)}%)` : '-'))]);
    }
  }
  md.push(table(['Target', 'Workload', 'Operation', 'primary µs', ...replicates.map(({ report }) => `${report.startedAt.slice(11, 16)} UTC session µs`)], rows));
}

// ---- 1b. Every build, the questions (a)-(h) ------------------------------------

{
  const sec = section('builds', 'Results per build and operation (questions a–h)', 'All events of each workload, pooled per run; median over the 3 runs [lowest–highest run] · median p95. DIRECT. Native rows exist only for the trees; the published package and the historical runtime are WebAssembly only.');
  const TARGETS = [LOCAL, CONTROL, PUBLISHED, HIST];
  const specs = [
    ['(a) transaction only, numeric parameters', 'native', 'apply', 'apply'],
    ['(a/b) payload + transaction, no reporting (i1)', 'native', 'bench.dispatch_only', 'bench.dispatch_only', 'inst'],
    ['(b) + view built, not serialized', 'native', 'dispatch_view_json', 'dispatch_view_json'],
    ['(b) + full snapshot built (legacy dispatch_json)', 'native', 'dispatch_json', 'dispatch_json'],
    ['(b) + outcome with snapshot built (dispatch_outcome_json)', 'native', 'dispatch_outcome_json', 'dispatch_outcome_json'],
    ['(c) view() alone', 'native', 'read', 'view'],
    ['(c) view JSON', 'native', 'read', 'view.serialize'],
    ['(d) snapshot() alone', 'native', 'read', 'snapshot'],
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
    for (const [name, e, m, op, from] of specs) {
      const report = from === 'inst' ? inst : base;
      const targets = from === 'inst' ? ['i1'] : TARGETS;
      const cells = TARGETS.map((t) => {
        if (!targets.includes(t) && !(from === 'inst' && t === LOCAL)) return '-';
        const d = direct(runs(report, from === 'inst' ? 'i1' : t, w, e, m, op));
        if (d) keep(sec, { workload: w, name, target: from === 'inst' ? 'i1' : t, engine: e, mode: m, op, stats: d });
        return directCell(d);
      });
      if (cells.every((cell) => cell === '-')) continue;
      rows.push([name, `${e} ${m} / ${op}`, ...cells]);
    }
    md.push(`### ${w}`, '', table(['Question', 'Engine, mode / operation', `${LOCAL}${' (i1 for the bench row)'}`, CONTROL, PUBLISHED, HIST], rows));
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
  md.push(table(['Method', 'Runtime', 'Reactive wasm', 'caveat5 median µs [runs]', 'p95 µs [runs]', 'per-run medians', 'ts per-run medians'], rows));

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
  for (const [e, m, op, name] of PATHS) {
    const cells = groups.map((group) => {
      const d = direct(groupRuns(classes, target, workload, e, m, op, group));
      if (d) keep(sec, { path: name, engine: e, mode: m, op, group, stats: d });
      return directCell(d);
    });
    if (cells.every((cell) => cell === '-')) continue;
    rows.push([name, ...cells]);
  }
  const counts = groups.map((group) => {
    const d = direct(groupRuns(classes, target, workload, 'native', 'apply', 'apply', group)) ?? direct(groupRuns(classes, target, workload, 'wasm', 'kit', 'kit.dispatch', group))
      ?? direct(groupRuns(classes, target, workload, 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse', group));
    return d ? `n ${d.nPerRun[0]}/run` : '';
  });
  md.push(table(['Path (µs: median [lowest–highest run] · p95)', ...groups.map((group, index) => `${group.replace(/^(category|class|signature): /, '')} (${counts[index]})`)], rows));
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
// [runs] ± bound", with each formula listed under the table.
function componentSection(key, title, intro, specs, report, classes) {
  const sec = section(key, title, intro);
  const rows = [];
  const notes = [];
  specs.forEach((spec, index) => {
    const label = spec.direct ? 'DIRECT' : `INFERRED [${index + 1}]`;
    const cells = WORKLOAD_GROUPS.map(([workload, group, name]) => {
      const value = componentValue(spec, workload, group, report, classes);
      if (value) keep(sec, { quantity: spec.name, eventKind: name, workload, group, source: spec.direct ?? null, ...(spec.direct ? { stats: value } : value) });
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
componentSection('core', '(4) Core runtime work: the event without view or snapshot', 'The instrumented session: the ordinary main-local build and the i1 build side by side, interleaved.', [
  { name: 'transaction, numeric parameters (native apply)', direct: [LOCAL, 'native', 'apply', 'apply'] },
  { name: 'payload parse + names + transaction (native, i1 bench_dispatch_only)', direct: ['i1', 'native', 'bench.dispatch_only', 'bench.dispatch_only'], note: 'throwaway i1 build' },
  { name: 'payload parse + names + transaction (wasm execution, i1 bench_dispatch_only)', direct: ['i1', 'wasm', 'abi.dispatch_only', 'abi.dispatch_only.exec'], note: 'throwaway i1 build' },
  { name: 'payload parse + name resolution (native)', formula: 'i1 bench.dispatch_only − i1 apply', terms: [[1, 'i1', 'native', 'bench.dispatch_only', 'bench.dispatch_only'], [-1, 'i1', 'native', 'apply', 'apply']] },
  { name: 'WebAssembly execution over native, the same work', formula: 'i1 abi.dispatch_only.exec − i1 bench.dispatch_only', terms: [[1, 'i1', 'wasm', 'abi.dispatch_only', 'abi.dispatch_only.exec'], [-1, 'i1', 'native', 'bench.dispatch_only', 'bench.dispatch_only']] },
], inst, instClasses);

// (5) snapshot and reporting work.
componentSection('snapshot', '(5) Snapshot and reporting work', 'The instrumented session.', [
  { name: 'snapshot() built (native, not serialized, not dropped)', direct: [LOCAL, 'native', 'read', 'snapshot'] },
  { name: 'snapshot compact JSON (native serde_json)', direct: [LOCAL, 'native', 'read', 'snapshot.serialize'] },
  { name: 'snapshot drop (native)', direct: [LOCAL, 'native', 'read', 'snapshot.drop'] },
  { name: 'snapshot built + dropped (wasm, i1)', direct: ['i1', 'wasm', 'abi.bench_read', 'abi.snapshot_build.exec'], note: 'throwaway i1 build' },
  { name: 'dispatch_outcome over dispatch-only (native): snapshot + outcome + JSON', formula: 'i1 web.dispatch_outcome − i1 bench.dispatch_only', terms: [[1, 'i1', 'native', 'web.dispatch_outcome', 'web.dispatch_outcome'], [-1, 'i1', 'native', 'bench.dispatch_only', 'bench.dispatch_only']] },
  { name: 'dispatch_outcome over dispatch-only (wasm execution)', formula: 'i1 abi.dispatch_outcome exec − i1 abi.dispatch_only exec', terms: [[1, 'i1', 'wasm', 'abi.dispatch_outcome', 'abi.exec'], [-1, 'i1', 'wasm', 'abi.dispatch_only', 'abi.dispatch_only.exec']] },
  { name: 'JS JSON.parse of the outcome', direct: [LOCAL, 'wasm', 'raw.dispatch_outcome', 'js.parse_outcome'] },
], inst, instClasses);

// (6) view construction.
componentSection('view', '(6) View construction', 'The instrumented session.', [
  { name: 'view() built (native, not serialized, not dropped)', direct: [LOCAL, 'native', 'read', 'view'] },
  { name: 'view compact JSON (native serde_json)', direct: [LOCAL, 'native', 'read', 'view.serialize'] },
  { name: 'view built + dropped (native, i1)', direct: ['i1', 'native', 'bench.read', 'bench.view_build'], note: 'throwaway i1 build' },
  { name: 'view built + dropped (wasm, i1)', direct: ['i1', 'wasm', 'abi.bench_read', 'abi.view_build.exec'], note: 'throwaway i1 build' },
  { name: 'view exported: built + JSON + dropped (wasm)', direct: ['i1', 'wasm', 'abi.bench_read', 'abi.view.exec'] },
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
], base, baseClasses);

// ---- the attribution the owner asked for -----------------------------------------
// Absolute parts from the instrumented run (i1 and the ordinary main-local
// build, interleaved), each timed on its own; they are separate
// measurements, so their sum is not a measurement of the whole. The i2
// shares below them are the same split made inside one call, where the
// parts are disjoint and add up exactly (for means).

componentSection('attribution-view', 'Attribution on the dispatch_view path (web pages, adapter): language execution, view building, serialization and bridge', 'Each part timed on its own (i1 throwaway build for the first two; the ordinary build for the rest) in the instrumented run. They are disjoint pieces of one dispatch_view call, but timed in separate calls, so their medians are not added; the whole is shown as its own DIRECT measurement.', [
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

componentSection('attribution-kit', 'Attribution on the kit path: language execution, snapshot reporting, serialization and bridge', 'As above, for the `dispatch_outcome` call the kit\'s `session.dispatch()` makes, all from the instrumented run; the kit\'s own JavaScript and its extra `view()` call are in section (8).', [
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
    rows.push([`${e} ${m} / ${op}`, ...cells]);
  }
  md.push(table(['Operation', '4 mushrooms (replay)', '16', '64'], rows));
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
  md.push(table(['Session', 'Workload', 'Operation', 'native µs (DIRECT)', 'wasm µs (DIRECT)', 'ratio', 'difference (INFERRED)'], rows), 'Load and restore rows use the WebAssembly glue calls (`raw.new`, `raw.restore`), which add the source and save copies into memory.', '');
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
  const sec = section('safeguard', 'Instrumentation safeguard', 'Each instrumented build against the ordinary build on the same events, in the same interleaved run: all events of the Glowcap replay (it has no refusals), and the accepted classes of the decision workloads (the ordinary `raw.*+parse` operations have no sample for a refused event, while a probe total does, so their pooled "all" would not compare like with like). i1 adds functions and changes none; i2 adds marks inside the runtime, so its totals include the marks\' own cost.');
  const rows = [];
  const GROUPS = { 'glowcap-replay': ['all'], 'trail-rescue-scenarios': ['class: evidence', 'class: commit', 'class: reopen'], 'ledger-session': ['class: evidence'] };
  const pairs = [
    ['i1', 'native', 'web.dispatch_view', 'web.dispatch_view', 'native', 'web.dispatch_view', 'web.dispatch_view'],
    ['i1', 'native', 'web.dispatch_outcome', 'web.dispatch_outcome', 'native', 'web.dispatch_outcome', 'web.dispatch_outcome'],
    ['i1', 'native', 'apply', 'apply', 'native', 'apply', 'apply'],
    ['i1', 'native', 'read', 'view', 'native', 'read', 'view'],
    ['i1', 'wasm', 'abi.dispatch_view', 'abi.exec', 'wasm', 'abi.dispatch_view', 'abi.exec'],
    ['i1', 'wasm', 'abi.dispatch_outcome', 'abi.exec', 'wasm', 'abi.dispatch_outcome', 'abi.exec'],
    ['i1', 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse', 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse'],
    ['i2', 'native', 'probe.dispatch_view', 'total', 'native', 'web.dispatch_view', 'web.dispatch_view'],
    ['i2', 'native', 'probe.dispatch_outcome', 'total', 'native', 'web.dispatch_outcome', 'web.dispatch_outcome'],
    ['i2', 'wasm', 'probe.dispatch_view', 'total', 'wasm', 'raw.dispatch_view', 'raw.dispatch_view+parse'],
    ['i2', 'wasm', 'probe.dispatch_outcome', 'total', 'wasm', 'raw.dispatch_outcome', 'raw.dispatch_outcome+parse'],
  ];
  for (const [w, groups] of Object.entries(GROUPS)) {
    for (const group of groups) {
      for (const [t, e, m, op, e0, m0, op0] of pairs) {
        for (const stat of ['median', 'mean']) {
          const a = groupRuns(instClasses, t, w, e, m, op, group);
          const b = groupRuns(instClasses, LOCAL, w, e0, m0, op0, group);
          const value = inferred([{ sign: 1, name: `${t} ${m} ${op}`, runs: a }, { sign: -1, name: `${LOCAL} ${m0} ${op0}`, runs: b }], { stat, formula: `${t} − ${LOCAL} (${stat})` });
          if (!value) continue;
          const ref = spreadOf(b.filter(Boolean).map((s) => s[stat]));
          keep(sec, { workload: w, group, target: t, engine: e, op: `${m} ${op}`, reference: `${m0} ${op0}`, stat, overhead: value, referenceValue: ref });
          rows.push([`${w}: ${group.replace('class: ', '')}`, `${t} ${e} ${m} / ${op}`, `${m0} / ${op0}`, stat, f(spreadOf(a.filter(Boolean).map((s) => s[stat])).median), f(ref.median), inferredCell(value), `${f((value.value.median / ref.median) * 100)}%`]);
        }
      }
    }
  }
  md.push(table(['Workload: events', 'Instrumented', 'Ordinary (main-local)', 'Statistic', 'instrumented µs', 'ordinary µs', 'overhead (INFERRED)', 'overhead %'], rows));
  const overhead = [];
  for (const e of ['native', 'wasm']) {
    for (const op of ['probe.mark', 'probe.now', 'js.performance_now']) {
      const d = direct(runs(inst, 'i2', 'glowcap-replay', e, 'probe.overhead', op));
      if (d) { overhead.push([e, op, directCell(d)]); keep(sec, { kind: 'mark cost', engine: e, op, stats: d }); }
    }
  }
  md.push('The cost of one mark (batches of 100):', '', table(['Engine', 'Operation', 'µs per call: median [runs] · p95'], overhead));
}

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
  const sec = section('proportions', 'Where the time goes inside one call (i2 marks: proportions only)', 'Mean time per event of each disjoint region between consecutive marks, as a share of the instrumented call\'s mean total. The regions of a call add up exactly to its total, so these shares add to 100%. Means, not medians, because only means add; the marks read a 100 ns clock and cost about the mark cost above each, so the absolute values are not production costs.');
  for (const [w, groups] of [['glowcap-replay', ['category: idle tick', 'category: state-changing tick']], ['trail-rescue-scenarios', ['class: evidence', 'class: commit', 'class: reopen', 'class: refused']], ['ledger-session', ['class: evidence', 'class: refused']]]) {
    for (const [e, m] of [['native', 'probe.dispatch_view'], ['wasm', 'probe.dispatch_view'], ['native', 'probe.dispatch_outcome'], ['wasm', 'probe.dispatch_outcome']]) {
      const rows = [];
      const header = ['Region', ...groups.map((group) => group.replace(/^(category|class): /, ''))];
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
        keep(sec, { workload: w, engine: e, mode: m, region: name, labels, byGroup: Object.fromEntries(groups.map((group, index) => [group, cells[index]])) });
        rows.push([name, ...cells.map((cell) => (cell?.us ? `${f(cell.share.median)}% (${f(cell.us.median)} µs)` : '-'))]);
      }
      const totalCells = totals.map((t) => { const s = spreadOf((t ?? []).filter(Boolean).map((x) => x.mean)); return s ? `mean ${f(s.median)} µs` : '-'; });
      rows.push(['**instrumented total**', ...totalCells]);
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
  const sec = section('buckets', 'The three buckets inside one call (i2 marks: proportions only)', 'The region shares above, summed into language execution, view or snapshot building, and serialization + bridge + JS parse. Shares of the instrumented call\'s mean total, which these rows (without the "of which" lines) add up to exactly. Proportions only: the i2 totals carry the marks\' cost.');
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
      keep(sec, { engine: e, mode: m, bucket: name, labels, byKind: Object.fromEntries(WORKLOAD_GROUPS.map(([, , kind], index) => [kind, cells[index]])) });
      rows.push([name, ...cells.map((cell) => (cell?.us ? `${f(cell.share.median)}% (${f(cell.us.median)})` : '-'))]);
    }
    rows.push(['instrumented mean total, µs', ...WORKLOAD_GROUPS.map(([w, group]) => {
      const s = spreadOf((groupRuns(instClasses, 'i2', w, e, m, 'total', group) ?? []).filter(Boolean).map((x) => x.mean));
      return s ? f(s.median) : '-';
    })]);
    md.push(`### ${e} ${m.replace('probe.', '')}`, '', table(['Bucket: share (mean µs)', ...WORKLOAD_GROUPS.map(([, , name]) => name)], rows));
  }
}

await mkdir(outDir, { recursive: true });
await writeFile(path.join(outDir, 'attribution.json'), `${JSON.stringify(out)}\n`);
await writeFile(path.join(outDir, 'tables.md'), `${md.join('\n')}\n`);
console.log(`wrote ${path.join(outDir, 'attribution.json')} and tables.md`);
