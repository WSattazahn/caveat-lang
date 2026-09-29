// The tables of RESULTS.md, from a view-path run (run.mjs --suite=view-path
// --keep-samples, then analyze.mjs) and a size report (size.mjs).
//
//   node experiments/performance-opt-0.1/report.mjs --run=DIR --size=FILE [--target=opt] [--control=main]
//        [--replicate=NAME=DIR ...] [--load=no] [--out=tables.md]
//
// DIR holds results.json and classes.json, or both gzipped, and load.csv.
// Each --replicate adds a session to the table that sets every session's
// saving side by side (give DIR itself as one of them to include it).
//
// Every timing is DIRECT: timed around exactly that path, per event, in the
// baseline's protocol (one process per target, workload and mode; 3 repeats,
// interleaved; one warm-up pass and 3 timed passes). A saving is the
// difference of two DIRECT medians of the same event class in the same repeat
// of this session (paired by repeat), and its range is over the repeats: no
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
const signed = (value, places = 1) => {
  if (!Number.isFinite(value)) return '-';
  const text = value.toFixed(places);
  if (Number(text) === 0) return text.replace('-', '');
  return `${value > 0 ? '+' : ''}${text}`;
};

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

// Load (load.mjs): per timed job and per repeat, over the samples covering
// the kept timed jobs.
const { loadPerRepeat } = await import('./load.mjs');
const loadOf = async (directory) => {
  const load = await loadPerRepeat(directory);
  const seen = load.repeats.flatMap((repeat) => [repeat.total]);
  const samples = seen.reduce((sum, total) => sum + total.samples, 0);
  const mean = seen.reduce((sum, total) => sum + total.mean * total.samples, 0) / (samples || 1);
  return { ...load, keptMean: mean, keptMax: Math.max(...seen.map((total) => total.max)), keptP95: Math.max(...seen.map((total) => total.p95)) };
};
const loadText = (load) => `total CPU over the kept timed jobs mean ${f(load.keptMean)}%, highest repeat p95 ${f(load.keptP95)}%, max ${f(load.keptMax)}%; ${load.discarded.length} attempt(s) discarded by the job guard and run again`;
const load = option('load') === 'no' ? null : await loadOf(run);
lines.push(`Run ${results.runId}; targets ${results.targets.map((t) => `${t.label} (${t.git?.revision?.slice(0, 7) ?? '?'}, reactive WebAssembly ${t.runtimeFiles?.['caveat_runtime_bg.wasm']?.sha256?.slice(0, 12) ?? '?'})`).join(', ')}; mask ${results.environment.pinning.affinityMask}, priority ${results.environment.pinning.priority}${load ? `; ${loadText(load)}` : ''}.`, '');

// The headline: the paths side by side and the measured saving, per event
// class. A refusal throws on the raw dispatch_view path, so its cell there is
// the call alone.
lines.push('### Headline: the paths side by side (DIRECT, µs) and the saving (paired by repeat, µs)', '');
table(['Event', 'existing: kit `dispatch()` + `view()`', 'new: kit `dispatchView()`', 'existing: raw `dispatch_view` + `JSON.parse`', 'new: raw `dispatch_view_outcome` + `JSON.parse`',
  'saving: kit `dispatch()` + `view()` − `dispatchView()`', 'old ÷ new', 'gap: `dispatchView()` − raw `dispatch_view` + `JSON.parse`'],
  [...KINDS, ...REFUSED].map(([workload, group, name]) => {
    const refused = REFUSED.some(([w, g]) => w === workload && g === group);
    return [name, cell(target, workload, KIT, group), cell(target, workload, VIEW, group),
      refused ? `throws: the call ${cell(target, workload, ['raw.dispatch_view', 'raw.dispatch_view'], group)}` : cell(target, workload, RAW, group),
      cell(target, workload, RAW_OUTCOME, group),
      range(paired(target, workload, KIT, VIEW, group, (a, b) => a - b)),
      range(paired(target, workload, KIT, VIEW, group, (a, b) => a / b), 2, '×'),
      refused ? '-' : signedRange(paired(target, workload, VIEW, RAW, group, (a, b) => a - b))];
  }));

lines.push('### Per event, on each path (DIRECT, µs)', '');
table(['Event', 'samples per run', `existing: ${KIT[2]}`, `new: ${VIEW[2]}`, `existing: ${RAW[2]}`, `new: ${RAW_OUTCOME[2]}`],
  KINDS.map(([workload, group, name]) => [name, counts(workload, group),
    cell(target, workload, KIT, group), cell(target, workload, VIEW, group), cell(target, workload, RAW, group), cell(target, workload, RAW_OUTCOME, group)]));

lines.push('### The saving (paired by repeat, µs)', '');
table(['Event', 'kit `dispatch()` + `view()` − `dispatchView()`', 'share of the old path', 'old ÷ new', '`dispatchView()` − raw `dispatch_view` + `JSON.parse`', '`dispatchView()` − raw `dispatch_view_outcome` + `JSON.parse`'],
  KINDS.map(([workload, group, name]) => [name,
    range(paired(target, workload, KIT, VIEW, group, (a, b) => a - b)),
    range(paired(target, workload, KIT, VIEW, group, (a, b) => ((a - b) / a) * 100), 1, '%'),
    range(paired(target, workload, KIT, VIEW, group, (a, b) => a / b), 2, '×'),
    signedRange(paired(target, workload, VIEW, RAW, group, (a, b) => a - b)),
    signedRange(paired(target, workload, VIEW, RAW_OUTCOME, group, (a, b) => a - b))]));

// A refusal leaves the view as it was, so a host that already holds it need
// not call view() after one: both savings are shown.
lines.push('### Refused events (DIRECT, µs; savings paired by repeat)', '');
table(['Event', 'samples per run', `existing: ${KIT[2]}`, `existing: ${KIT_DISPATCH[2]}`, `new: ${VIEW[2]}`, 'existing: raw `dispatch_view` (throws)', `new: ${RAW_OUTCOME[2]}`,
  'kit `dispatch()` + `view()` − `dispatchView()`', 'kit `dispatch()` alone − `dispatchView()`'],
  REFUSED.map(([workload, group, name]) => [name, counts(workload, group),
    cell(target, workload, KIT, group), cell(target, workload, KIT_DISPATCH, group), cell(target, workload, VIEW, group),
    cell(target, workload, ['raw.dispatch_view', 'raw.dispatch_view'], group), cell(target, workload, RAW_OUTCOME, group),
    range(paired(target, workload, KIT, VIEW, group, (a, b) => a - b)),
    signedRange(paired(target, workload, KIT_DISPATCH, VIEW, group, (a, b) => a - b))]));

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

// dispatchView against the raw paths, piece by piece: every row DIRECT (its
// own timer around exactly that piece, per event) but the two paired
// differences at the end. Medians of pieces do not add up to the median of a
// whole exactly; the sum row is timed from the first piece's start to the
// last's end.
const PIECES = [
  ['raw `dispatch_view` path: `JSON.stringify(payload)`', ['raw.dispatch_view', 'js.stringify_payload']],
  ['raw `dispatch_view` path: the `dispatch_view` call', ['raw.dispatch_view', 'raw.dispatch_view']],
  ['raw `dispatch_view` path: `JSON.parse` of the view', ['raw.dispatch_view', 'js.parse_view']],
  ['**raw `dispatch_view` + `JSON.parse`**', RAW],
  ['raw `dispatch_view_outcome` path: `JSON.stringify(payload)`', ['raw.dispatch_view_outcome', 'js.stringify_payload']],
  ['raw `dispatch_view_outcome` path: the `dispatch_view_outcome` call', ['raw.dispatch_view_outcome', 'raw.dispatch_view_outcome']],
  ['raw `dispatch_view_outcome` path: `JSON.parse` of the envelope', ['raw.dispatch_view_outcome', 'js.parse_view_outcome']],
  ['**raw `dispatch_view_outcome` + `JSON.parse`**', RAW_OUTCOME],
  ['pieces: the kit\'s `payloadText(payload)`', ['kit.dispatchView.pieces', 'kit.payloadText']],
  ['pieces: the `dispatch_view_outcome` call', ['kit.dispatchView.pieces', 'raw.dispatch_view_outcome']],
  ['pieces: `JSON.parse` of the envelope', ['kit.dispatchView.pieces', 'js.parse_view_outcome']],
  ['pieces: the kit\'s `validOutcome(outcome, \'view\')`', ['kit.dispatchView.pieces', 'kit.validOutcome']],
  ['**pieces: the four, first timer to last**', ['kit.dispatchView.pieces', 'kit.dispatchView.pieces']],
  ['pieces: `JSON.parse` of the view text alone, after the envelope\'s', ['kit.dispatchView.pieces', 'js.parse_view_alone']],
  ['**kit `dispatchView()`**', VIEW],
];
const PIECES_MODE = 'kit.dispatchView.pieces';
const hasPieces = Boolean(classes[target]?.['glowcap-replay']?.wasm?.[PIECES_MODE]);
const median = (label, workload, path_, group) => {
  const stats = across(label, workload, path_, group);
  return stats?.median ? `${f(stats.median.median)} [${f(stats.median.min)}–${f(stats.median.max)}]` : '-';
};
if (hasPieces) {
  const kinds = [...KINDS, ...REFUSED];
  lines.push('### `dispatchView()` and the raw paths, piece by piece (DIRECT medians, µs, [lowest–highest run])', '');
  table(['Piece', ...kinds.map(([, , name]) => name)], [
    ...PIECES.map(([name, path_]) => [name, ...kinds.map(([workload, group]) => median(target, workload, path_, group))]),
    ['`dispatchView()` − the four pieces (paired by repeat; the wrapper\'s state and argument checks, and the timer reads between pieces)',
      ...kinds.map(([workload, group]) => signedRange(paired(target, workload, VIEW, ['kit.dispatchView.pieces', 'kit.dispatchView.pieces'], group, (a, b) => a - b)))],
    ['`dispatchView()` − raw `dispatch_view` + `JSON.parse` (paired by repeat)',
      ...kinds.map(([workload, group]) => signedRange(paired(target, workload, VIEW, RAW, group, (a, b) => a - b)))],
    ['the kit\'s payload check: `payloadText` − `JSON.stringify` of the raw `dispatch_view_outcome` path (paired by repeat)',
      ...kinds.map(([workload, group]) => signedRange(paired(target, workload, ['kit.dispatchView.pieces', 'kit.payloadText'], ['raw.dispatch_view_outcome', 'js.stringify_payload'], group, (a, b) => a - b)))],
    ['envelope against view: `JSON.parse` of the envelope − of the view, each the first parse on its own raw path (paired by repeat)',
      ...kinds.map(([workload, group]) => signedRange(paired(target, workload, ['raw.dispatch_view_outcome', 'js.parse_view_outcome'], ['raw.dispatch_view', 'js.parse_view'], group, (a, b) => a - b)))],
    ['the call: `dispatch_view_outcome` − `dispatch_view`, each on its own raw path (paired by repeat)',
      ...kinds.map(([workload, group]) => signedRange(paired(target, workload, ['raw.dispatch_view_outcome', 'raw.dispatch_view_outcome'], ['raw.dispatch_view', 'raw.dispatch_view'], group, (a, b) => a - b)))],
  ]);
}

// Load per repeat and per timed job (load.mjs): total CPU over all logical
// processors while the kept timed jobs ran (the benchmark itself is about 5%),
// and the attempts the job guard discarded.
if (load) {
  lines.push(`### Load per repeat (typeperf every second; ${load.rule})`, '');
  table(['Repeat', 'from (UTC)', 'seconds', 'timed jobs kept', 'attempts discarded', 'samples', 'total CPU mean', 'p95', 'max', 'samples above 20%', 'kept'],
    load.repeats.map((repeat) => [repeat.repeat, repeat.from.slice(11, 19), repeat.seconds, `${repeat.jobsKept} of ${repeat.jobs}`, repeat.discardedAttempts, repeat.total.samples, `${f(repeat.total.mean)}%`, `${f(repeat.total.p95)}%`, `${f(repeat.total.max)}%`, repeat.total.above20, repeat.kept ? 'yes' : 'NO']));
  const kept = load.jobs;
  const most = (pick) => Math.max(...kept.map(pick).filter(Number.isFinite));
  lines.push(`### Load per timed job (${load.jobRule ?? 'no job guard'})`, '');
  lines.push(`Over the ${kept.length} kept timed jobs, judged again afterwards from load.csv: ${load.jobsBrokeRuleAfterwards} broke the rule; the highest job mean total CPU ${f(most((job) => job.after.meanTotal))}%, the highest sample ${f(most((job) => job.after.maxTotal))}%, at most ${f(most((job) => job.after.maxOutsideCores), 2)} cores busy outside the pinned processors, System at most ${f(most((job) => job.after.maxProcess.System ?? NaN))}% and MsMpEng at most ${f(most((job) => job.after.maxProcess.MsMpEng ?? NaN))}% of one core. The guard's own verdict, as each job ended, ${load.guardAgreesAfterwards ? 'agrees' : 'DISAGREES'} with it for every kept job.`, '');
  if (load.discarded.length) {
    table(['Discarded attempt', 'repeat', 'at (UTC)', 'seconds', 'why'],
      load.discarded.map((attempt) => [`\`${attempt.job.replace(/^\d+-/, '').replace('-wasm-', ' ')}\` #${attempt.attempt}`, attempt.repeat + 1, attempt.from.slice(11, 19), f(attempt.seconds), attempt.reasons.join('; ')]));
  }
}

// Sessions side by side: the saving and dispatchView's median in each.
const replicates = process.argv.filter((value) => value.startsWith('--replicate=')).map((value) => value.slice(12).split('='));
if (replicates.length) {
  const sessions = [];
  for (const [name, directory] of replicates) {
    const read = async (file) => {
      const plain = path.join(path.resolve(directory), `${file}.json`);
      if (existsSync(plain)) return JSON.parse(await readFile(plain, 'utf8'));
      return JSON.parse(gunzipSync(await readFile(`${plain}.gz`)).toString('utf8'));
    };
    const res = await read('results');
    sessions.push({ name, results: res, classes: (await read('classes')).results });
  }
  const within = (session, label, workload, [mode, op], group) => (session.classes[label]?.[workload]?.wasm?.[mode]?.[op]?.runs ?? []).map((s) => s?.[group] ?? null);
  const pairedIn = (session, workload, a, b, group) => {
    const left = within(session, target, workload, a, group);
    const right = within(session, target, workload, b, group);
    return spread(left.map((s, i) => (s && right[i] ? s.median - right[i].median : NaN)));
  };
  const medianIn = (session, workload, path_, group) => {
    const stats = session.classes[target]?.[workload]?.wasm?.[path_[0]]?.[path_[1]]?.acrossRuns?.[group];
    return stats?.median ? `${f(stats.median.median)} [${f(stats.median.min)}–${f(stats.median.max)}]` : '-';
  };
  const signedIn = (session, workload, a, b, group) => signedRange(pairedIn(session, workload, a, b, group));
  lines.push('### Every session: the saving, kit `dispatch()` + `view()` − `dispatchView()`, and the gap to the raw path, `dispatchView()` − raw `dispatch_view` + `JSON.parse` (both paired by repeat, µs); `dispatchView()` (DIRECT median, µs)', '');
  table(['Event', ...sessions.flatMap((s) => [`${s.name}: saving`, `${s.name}: gap to raw`, `${s.name}: \`dispatchView()\``])],
    [...KINDS, ...REFUSED].map(([workload, group, name]) => [name, ...sessions.flatMap((s) => [range(pairedIn(s, workload, KIT, VIEW, group)),
      REFUSED.some(([w, g]) => w === workload && g === group) ? '-' : signedIn(s, workload, VIEW, RAW, group), medianIn(s, workload, VIEW, group)])]));
  for (const [index, [, directory]] of replicates.entries()) sessions[index].load = await loadOf(path.resolve(directory));
  lines.push(...sessions.map((s) => `- ${s.name}: run ${s.results.runId}, mask ${s.results.environment.pinning.affinityMask}, ${s.results.suite.repeats} repeats, ${loadText(s.load)}`), '');
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
    row('  files in it', (s) => s.tarball.entries),
  ]);
}

const text = `${lines.join('\n')}\n`;
if (option('out')) await writeFile(path.resolve(option('out')), text);
else process.stdout.write(text);
