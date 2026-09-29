// The published figure's own command, run as published: `node
// experiments/glowcap/harness.mjs --bench` inside a tree whose dist/ was
// built by `npm run build`. Nothing about the benchmark changes: all six
// implementations run in one process, caveat5 last, 3 rounds of 10,000
// events with no warm-up, as when 51.6 µs was measured. This only repeats
// the command, optionally pinned, and records what it printed.
//
//   node experiments/performance-0.1/verbatim.mjs --tree=LABEL=DIR [--tree=...]
//        [--repeats=3] [--affinity=0x3C00|none] [--priority=high|normal] --out=FILE
//
// Trees run interleaved (every tree once per repeat, the first rotating).
// The harness's --bench does not write runs.jsonl, but its hash is checked
// before and after every run; in a git checkout a change is reverted with
// `git checkout -- experiments/glowcap/runs.jsonl` and reported.
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { sha256 } from './lib/workloads.mjs';
import { load, machine } from './lib/environment.mjs';

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

function runBench(tree) {
  const args = ['experiments/glowcap/harness.mjs', '--bench'];
  const options = { cwd: tree.directory, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024, timeout: 30 * 60 * 1000 };
  if (process.platform === 'win32' && (affinityOption !== 'none' || priority !== 'normal')) {
    const flags = [priority === 'normal' ? '' : `/${priority}`, affinityOption === 'none' ? '' : `/affinity ${BigInt(affinityOption).toString(16).toUpperCase()}`].filter(Boolean).join(' ');
    const line = `start "" /b /wait ${flags} "${process.execPath}" ${args.map((arg) => `"${arg}"`).join(' ')}`;
    return spawnSync('cmd.exe', ['/d', '/s', '/c', line], { ...options, windowsVerbatimArguments: true });
  }
  return spawnSync(process.execPath, args, options);
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
  pinning: { affinity: affinityOption, priority, how: affinityOption === 'none' && priority === 'normal' ? 'none: as published' : 'cmd /c start /b /wait' },
  loadBefore: await load(scratch),
  trees: trees.map(({ label, directory, files, revision, buildInfo }) => ({ label, directory, revision, files, buildInfo })),
  runs: [],
  runsJsonl: [],
};
for (let repeat = 0; repeat < repeats; repeat++) {
  const rotation = trees.map((_, index) => trees[(index + repeat) % trees.length]);
  for (const tree of rotation) {
    const before = await hashOf(runsFile(tree));
    const started = Date.now();
    const child = runBench(tree);
    const seconds = (Date.now() - started) / 1000;
    const after = await hashOf(runsFile(tree));
    if (before !== after) {
      const restored = git(tree, ['checkout', '--', 'experiments/glowcap/runs.jsonl']);
      record.runsJsonl.push({ tree: tree.label, repeat, before, after, reverted: restored.status === 0, afterRevert: await hashOf(runsFile(tree)) });
    }
    const lines = parseBench(child.stdout ?? '');
    record.runs.push({ tree: tree.label, repeat, seconds, status: child.status, lines, stdout: child.stdout, stderr: child.stderr?.slice(0, 4000) });
    console.log(`${tree.label} #${repeat}: ${seconds.toFixed(1)} s, caveat5 ${JSON.stringify(lines.caveat5 ?? null)}${child.status ? `, exit ${child.status}` : ''}`);
  }
}
record.loadAfter = await load(scratch);
record.finishedAt = new Date().toISOString();
await writeFile(out, `${JSON.stringify(record, null, 1)}\n`);
console.log(`wrote ${out}; runs.jsonl changed in ${record.runsJsonl.length} run(s)`);
