// The published figure's own command, run as published: `node
// experiments/glowcap/harness.mjs --bench` inside a tree whose dist/ was
// built by `npm run build`. Nothing about the benchmark changes: all six
// implementations run in one process, caveat5 last, 3 rounds of 10,000
// events with no warm-up, as when 51.6 µs was measured. This only repeats
// the command, optionally pinned, and records what it printed.
//
//   node experiments/performance-0.1/verbatim.mjs --tree=LABEL=DIR [--tree=...]
//        [--repeats=3] [--affinity=0x3C00|none | --affinity-set=0x3,0xC00,...]
//        [--priority=high|normal] [--monitor-interval=1] [--monitor-processors=N,...] --out=FILE
//
// Trees run interleaved (every tree once per repeat, the first rotating).
// With --affinity-set every tree runs once under each mask per repeat, the
// (tree, mask) pairs rotating together; each run records its mask and, from
// typeperf, which of the monitored processors it kept busy and their clock.
// The harness's --bench does not write runs.jsonl, but its hash is checked
// before and after every run; in a git checkout a change is reverted with
// `git checkout -- experiments/glowcap/runs.jsonl` and reported.
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { sha256 } from './lib/workloads.mjs';
import { coresDuring, load, machine, startMonitor, summarizeMonitor } from './lib/environment.mjs';
import { maskLabel, maskProcessors, parseMask, parseMaskSet, runPinned } from './lib/pinning.mjs';

const argv = process.argv.slice(2);
const option = (name) => argv.filter((arg) => arg.startsWith(`--${name}=`)).map((arg) => arg.slice(name.length + 3));
const single = (name, fallback = null) => option(name).at(-1) ?? fallback;
const trees = option('tree').map((text) => {
  const at = text.indexOf('=');
  return { label: text.slice(0, at), directory: path.resolve(text.slice(at + 1)) };
});
if (!trees.length) throw new Error('give --tree=LABEL=DIR');
const repeats = Number(single('repeats', '3'));
const affinityOption = single('affinity', 'none');
const affinitySet = parseMaskSet(single('affinity-set'));
const masks = affinitySet ?? [parseMask(affinityOption)];
const priority = single('priority', 'normal');
const out = path.resolve(single('out') ?? 'verbatim.json');
const scratch = path.dirname(out);

const runsFile = (tree) => path.join(tree.directory, 'experiments', 'glowcap', 'runs.jsonl');
const hashOf = async (file) => (existsSync(file) ? sha256(await readFile(file)) : null);
const git = (tree, args) => spawnSync('git', args, { cwd: tree.directory, encoding: 'utf8' });

// What each tree runs: its harness, adapters and built runtimes, by hash.
for (const tree of trees) {
  const files = ['experiments/glowcap/harness.mjs', 'experiments/glowcap/caveat5/adapter.mjs', 'experiments/glowcap/caveat5/glowcap.cav',
    'dist/pkg-reactive/caveat_runtime_bg.wasm', 'dist/pkg-reactive/caveat_runtime.js', 'dist/pkg/caveat_runtime_bg.wasm', 'dist/pkg/caveat_runtime.js'];
  tree.files = {};
  for (const file of files) tree.files[file] = await hashOf(path.join(tree.directory, file));
  tree.revision = git(tree, ['rev-parse', 'HEAD']).stdout?.trim() || null;
  tree.buildInfo = existsSync(path.join(tree.directory, 'dist', 'build-info.json')) ? JSON.parse(await readFile(path.join(tree.directory, 'dist', 'build-info.json'), 'utf8')) : null;
}

function runBench(tree, mask) {
  const args = ['experiments/glowcap/harness.mjs', '--bench'];
  return runPinned(process.execPath, args, { affinity: mask, priority, cwd: tree.directory, maxBuffer: 16 * 1024 * 1024, timeout: 30 * 60 * 1000 });
}

// "caveat5 dispatch+view µs  median 51.6  p95 59.1  max 646.7"
function parseBench(text) {
  const lines = {};
  for (const match of text.matchAll(/^(\S+)\s+dispatch\+view µs\s+median ([\d.]+)\s+p95 ([\d.]+)\s+max ([\d.]+)/gmu)) {
    lines[match[1]] = { median: Number(match[2]), p95: Number(match[3]), max: Number(match[4]) };
  }
  return lines;
}

const record = {
  schema: 'caveat-performance-verbatim/0.1',
  command: 'node experiments/glowcap/harness.mjs --bench',
  startedAt: new Date().toISOString(),
  machine: await machine(scratch),
  pinning: {
    affinity: affinitySet ? null : affinityOption,
    affinitySet: affinitySet ? affinitySet.map(maskLabel) : null,
    priority,
    how: !affinitySet && affinityOption === 'none' && priority === 'normal' ? 'none: as published' : 'cmd /c start /b /wait',
  },
  loadBefore: await load(scratch),
  trees: trees.map(({ label, directory, files, revision, buildInfo }) => ({ label, directory, revision, files, buildInfo })),
  runs: [],
  runsJsonl: [],
};
// Unpinned, name the processors to watch (--monitor-processors=0,1,10,...) to
// see where the scheduler put the run.
const monitored = [...new Set([...masks.flatMap((mask) => maskProcessors(mask) ?? []), ...(single('monitor-processors')?.split(',').filter(Boolean).map(Number) ?? [])])].sort((a, b) => a - b);
const monitor = monitored.length && !argv.includes('--no-monitor') ? startMonitor(monitored, Number(single('monitor-interval', '1'))) : null;
const pairs = trees.flatMap((tree) => masks.map((mask) => ({ tree, mask })));
for (let repeat = 0; repeat < repeats; repeat++) {
  const rotation = pairs.map((_, index) => pairs[(index + repeat) % pairs.length]);
  for (const { tree, mask } of rotation) {
    const before = await hashOf(runsFile(tree));
    const started = Date.now();
    const child = runBench(tree, mask);
    const seconds = (Date.now() - started) / 1000;
    const after = await hashOf(runsFile(tree));
    if (before !== after) {
      const restored = git(tree, ['checkout', '--', 'experiments/glowcap/runs.jsonl']);
      record.runsJsonl.push({ tree: tree.label, repeat, before, after, reverted: restored.status === 0, afterRevert: await hashOf(runsFile(tree)) });
    }
    const lines = parseBench(child.stdout ?? '');
    record.runs.push({ tree: tree.label, affinity: maskLabel(mask), repeat, seconds, startedAtMs: started, endedAtMs: started + seconds * 1000, status: child.status, lines, stdout: child.stdout, stderr: child.stderr?.slice(0, 4000) });
    console.log(`${tree.label} ${maskLabel(mask)} #${repeat}: ${seconds.toFixed(1)} s, caveat5 ${JSON.stringify(lines.caveat5 ?? null)}${child.status ? `, exit ${child.status}` : ''}`);
  }
}
if (monitor) {
  const stopped = monitor.stop();
  record.loadDuring = summarizeMonitor(monitor, stopped);
  for (const run of record.runs) run.cores = coresDuring(monitor, stopped, run.startedAtMs, run.endedAtMs);
}
record.loadAfter = await load(scratch);
record.finishedAt = new Date().toISOString();
await writeFile(out, `${JSON.stringify(record, null, 1)}\n`);
console.log(`wrote ${out}; runs.jsonl changed in ${record.runsJsonl.length} run(s)`);
