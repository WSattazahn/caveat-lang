// The tables of RESULTS.md, from a view-path run (run.mjs --suite=view-path
// --keep-samples, then analyze.mjs) and a size report (size.mjs).
//
//   node experiments/performance-opt-0.1/report.mjs --run=DIR --size=FILE [--target=opt] [--control=main] [--out=tables.md]
//
// DIR holds results.json and classes.json, or both gzipped.
//
// Every timing is DIRECT: timed around exactly that path, per event, in the
// baseline's protocol (one process per target, workload and mode; 3 repeats,
// interleaved; one warm-up pass and 3 timed passes). A saving is the
// difference of two DIRECT medians of the same event class in the same repeat
// of this session (paired by repeat), and its range is over the 3 repeats: no
// share, model or earlier session enters it.
import { existsSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { gunzipSync } from 'node:zlib';
import { spread } from '../performance-0.1/lib/stats.mjs';

const option = (name, fallback = null) => process.argv.find((value) => value.startsWith(`--${name}=`))?.slice(name.length + 3) ?? fallback;
const run = path.resolve(option('run'));
const target = option('target', 'opt');
const control = option('control', 'main');
// NAME.json as run.mjs and analyze.mjs write it, or NAME.json.gz as committed.
async function readJson(name) {
  const plain = path.join(run, `${name}.json`);
  if (existsSync(plain)) return JSON.parse(await readFile(plain, 'utf8'));
  return JSON.parse(gunzipSync(await readFile(`${plain}.gz`)).toString('utf8'));
}
const results = await readJson('results');
const classes = (await readJson('classes')).results;
const size = option('size') ? JSON.parse(await readFile(path.resolve(option('size')), 'utf8')) : null;

const f = (value, places = 1) => (Number.isFinite(value) ? value.toFixed(places) : '-');
const bytes = (value) => (Number.isFinite(value) ? value.toLocaleString('en-US') : '-');
const signed = (value, places = 1) => (Number.isFinite(value) ? `${value > 0 ? '+' : ''}${value.toFixed(places)}` : '-');

// The owner's event kinds: [workload, group, label].
const KINDS = [
  ['glowcap-replay', 'category: idle tick', 'Glowcap idle tick'],
  ['glowcap-replay', 'category: state-changing tick', 'Glowcap state-changing tick'],
  ['trail-rescue-scenarios', 'class: evidence', 'Trail Rescue evidence'],
  ['trail-rescue-scenarios', 'class: commit', 'Trail Rescue commit'],
  ['trail-rescue-scenarios', 'class: reopen', 'Trail Rescue reopen'],
  ['trail-rescue-scenarios', 'class: qualify', 'Trail Rescue qualify'],
  ['ledger-session', 'class: evidence', 'Ledger evidence'],
];
const REFUSED = [
  ['trail-rescue-scenarios', 'class: refused', 'Trail Rescue refused'],
  ['ledger-session', 'class: refused', 'Ledger refused'],
];
// [mode, op, label]
const RAW = ['raw.dispatch_view', 'raw.dispatch_view+parse', 'raw `dispatch_view` + `JSON.parse`'];
const RAW_OUTCOME = ['raw.dispatch_view_outcome', 'raw.dispatch_view_outcome+parse', 'raw `dispatch_view_outcome` + `JSON.parse`'];
const KIT = ['kit', 'kit.dispatch+view', 'kit `dispatch()` + `view()`'];
const KIT_DISPATCH = ['kit', 'kit.dispatch', 'kit `dispatch()` alone'];
const VIEW = ['kit.dispatchView', 'kit.dispatchView', 'kit `dispatchView()`'];

// Per-repeat summaries of one (target, workload, mode, op, group).
const perRun = (label, workload, [mode, op], group) => (classes[label]?.[workload]?.wasm?.[mode]?.[op]?.runs ?? []).map((runSummary) => runSummary?.[group] ?? null);
const across = (label, workload, path_, group) => classes[label]?.[workload]?.wasm?.[path_[0]]?.[path_[1]]?.acrossRuns?.[group] ?? null;

function cell(label, workload, path_, group) {
  const stats = across(label, workload, path_, group);
  if (!stats?.median) return '-';
  return `${f(stats.median.median)} [${f(stats.median.min)}–${f(stats.median.max)}] · p95 ${f(stats.p95.median)}`;
}

// A paired quantity per repeat, then its spread over the repeats.
function paired(label, workload, a, b, group, combine) {
  const left = perRun(label, workload, a, group);
  const right = perRun(label, workload, b, group);
  const values = left.map((summary, index) => (summary && right[index] ? combine(summary.median, right[index].median) : NaN));
  return spread(values);
}
const range = (s, places = 1, unit = '') => (s ? `${f(s.median, places)}${unit} [${f(s.min, places)}–${f(s.max, places)}]` : '-');
const signedRange = (s, places = 1, unit = '') => (s ? `${signed(s.median, places)}${unit} [${signed(s.min, places)} to ${signed(s.max, places)}]` : '-');

const lines = [];
const table = (header, rows) => {
  lines.push(`| ${header.join(' | ')} |`, `| ${header.map(() => '---').join(' | ')} |`);
  for (const row of rows) lines.push(`| ${row.join(' | ')} |`);
  lines.push('');
};
const counts = (workload, group) => {
  const stats = across(target, workload, VIEW, group);
  return stats ? stats.nPerRun.join('/') : '-';
};

lines.push(`Run ${results.runId}; targets ${results.targets.map((t) => `${t.label} (${t.git?.revision?.slice(0, 7) ?? '?'}, reactive WebAssembly ${t.runtimeFiles?.['caveat_runtime_bg.wasm']?.sha256?.slice(0, 12) ?? '?'})`).join(', ')}; mask ${results.environment.pinning.affinityMask}, priority ${results.environment.pinning.priority}; total CPU during the run mean ${results.environment.loadDuring?.counters.total.mean}%, p95 ${results.environment.loadDuring?.counters.total.p95}%, max ${results.environment.loadDuring?.counters.total.max}%.`, '');

lines.push('### Per event, on each path (DIRECT, µs)', '');
table(['Event', 'samples per run', RAW[2], RAW_OUTCOME[2], KIT[2], VIEW[2]],
  KINDS.map(([workload, group, name]) => [name, counts(workload, group),
    cell(target, workload, RAW, group), cell(target, workload, RAW_OUTCOME, group), cell(target, workload, KIT, group), cell(target, workload, VIEW, group)]));

lines.push('### The saving (paired by repeat, µs)', '');
table(['Event', 'kit `dispatch()` + `view()` − `dispatchView()`', 'share of the old path', 'old ÷ new', '`dispatchView()` − raw `dispatch_view` + `JSON.parse`', '`dispatchView()` − raw `dispatch_view_outcome` + `JSON.parse`'],
  KINDS.map(([workload, group, name]) => [name,
    range(paired(target, workload, KIT, VIEW, group, (a, b) => a - b)),
    range(paired(target, workload, KIT, VIEW, group, (a, b) => ((a - b) / a) * 100), 1, '%'),
    range(paired(target, workload, KIT, VIEW, group, (a, b) => a / b), 2, '×'),
    signedRange(paired(target, workload, VIEW, RAW, group, (a, b) => a - b)),
    signedRange(paired(target, workload, VIEW, RAW_OUTCOME, group, (a, b) => a - b))]));

lines.push('### Refused events (DIRECT, µs)', '');
table(['Event', 'samples per run', 'raw `dispatch_view` (throws)', RAW_OUTCOME[2], KIT_DISPATCH[2], KIT[2], VIEW[2]],
  REFUSED.map(([workload, group, name]) => [name, counts(workload, group),
    cell(target, workload, ['raw.dispatch_view', 'raw.dispatch_view'], group), cell(target, workload, RAW_OUTCOME, group),
    cell(target, workload, KIT_DISPATCH, group), cell(target, workload, KIT, group), cell(target, workload, VIEW, group)]));

if (results.targets.some((t) => t.label === control)) {
  lines.push(`### The existing paths, ${control} against ${target} (DIRECT medians, µs; change paired by repeat)`, '');
  table(['Event', `kit \`dispatch()\` + \`view()\`, ${control}`, target, 'change', `raw \`dispatch_view\` + \`JSON.parse\`, ${control}`, target, 'change'],
    KINDS.map(([workload, group, name]) => {
      const pair = (path_) => {
        const a = perRun(control, workload, path_, group);
        const b = perRun(target, workload, path_, group);
        return spread(a.map((s, i) => (s && b[i] ? ((b[i].median - s.median) / s.median) * 100 : NaN)));
      };
      return [name, cell(control, workload, KIT, group), cell(target, workload, KIT, group), signedRange(pair(KIT), 1, '%'),
        cell(control, workload, RAW, group), cell(target, workload, RAW, group), signedRange(pair(RAW), 1, '%')];
    }));
}

if (size) {
  lines.push('### Size (bytes)', '');
  const row = (name, pick) => [name, bytes(pick(size.base)), bytes(pick(size.branch)), signed(pick(size.branch) - pick(size.base), 0)];
  table(['', `base ${size.base.checkout.revision.slice(0, 7)}`, 'branch', 'change'], [
    row('reactive WebAssembly (`pkg-reactive`, bundled in the kit)', (s) => s.builds['pkg-reactive'].wasm.bytes),
    row('  gzip -9', (s) => s.builds['pkg-reactive'].wasm.gzipBytes),
    row('  its JavaScript glue', (s) => s.builds['pkg-reactive'].glue.bytes),
    row('full WebAssembly (`pkg`, the web pages)', (s) => s.builds.pkg.wasm.bytes),
    row('  gzip -9', (s) => s.builds.pkg.wasm.gzipBytes),
    row('  its JavaScript glue', (s) => s.builds.pkg.glue.bytes),
    row('kit `lib/session.mjs`', (s) => s.sessionLibrary.bytes),
    row('kit tarball (`npm pack`)', (s) => s.tarball.bytes),
    row('  unpacked', (s) => s.tarball.unpackedBytes),
  ]);
}

const text = `${lines.join('\n')}\n`;
if (option('out')) await writeFile(path.resolve(option('out')), text);
else process.stdout.write(text);
