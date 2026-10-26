#!/usr/bin/env node
// Carmack check 3: nothing hidden. Package size, start-up time and memory per
// session, compared with bench/carmack/baseline.json so none of them creeps
// up between releases unnoticed. Sizes and WebAssembly memory are exact for a
// given build; start-up is a median of fresh processes and noisy.
//
//   node bench/carmack/footprint.mjs [--runtime DIR] [--out DIR] [--strict] [--write-baseline]
//
// Every measure is printed with its baseline and the allowed growth. --strict
// exits 1 when one is over; --write-baseline records this run as the baseline.
import { execFileSync } from 'node:child_process';
import { readFileSync, readdirSync, statSync, writeFileSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { gzipSync } from 'node:zlib';
import { defaultRuntimeDirectory } from '../../kit/lib/node.mjs';
import { WORKLOADS, ROOT } from './workloads.mjs';

const arg = (name) => { const i = process.argv.indexOf(`--${name}`); return i < 0 ? undefined : process.argv[i + 1]; };
const flag = (name) => process.argv.includes(`--${name}`);
const out = path.resolve(arg('out') ?? path.join(ROOT, 'test-results', 'carmack'));
mkdirSync(out, { recursive: true });
const runtimeDir = path.resolve(arg('runtime') ?? defaultRuntimeDirectory());
const baselineFile = path.join(ROOT, 'bench', 'carmack', 'baseline.json');

const bytes = (file) => statSync(file).size;
const gzipped = (file) => gzipSync(readFileSync(file), { level: 9 }).length;
const median = (values) => [...values].sort((a, b) => a - b)[Math.floor((values.length - 1) / 2)];
const wasmFile = path.join(runtimeDir, 'caveat_runtime_bg.wasm');
const glueFile = path.join(runtimeDir, 'caveat_runtime.js');
const kitLib = path.join(ROOT, 'kit', 'lib');
const kitFiles = readdirSync(kitLib).filter((f) => f.endsWith('.mjs')).map((f) => path.join(kitLib, f));

const measures = {};
const put = (name, value, unit, allowedGrowth, exact) => { measures[name] = { value, unit, allowed_growth: allowedGrowth, exact }; };

// Size: what a host downloads, and what the kit's code adds beside it.
put('wasm_bytes', bytes(wasmFile), 'bytes', 0.02, true);
put('wasm_gzip_bytes', gzipped(wasmFile), 'bytes', 0.02, true);
put('host_ships_gzip_bytes', gzipped(wasmFile) + gzipped(glueFile) + gzipped(path.join(kitLib, 'session.mjs')), 'bytes', 0.02, true);
put('kit_lib_bytes', kitFiles.reduce((n, f) => n + bytes(f), 0), 'bytes', 0.05, true);

// Start-up: a fresh Node process loads the runtime and opens each workload's
// program, timed from inside the process. Median of nine processes.
const probe = `
const { loadRuntimeFromDirectory } = await import(${JSON.stringify(pathToFileURL(path.join(ROOT, 'kit', 'lib', 'node.mjs')).href)});
const { readFileSync } = await import('node:fs');
const t0 = performance.now();
const runtime = await loadRuntimeFromDirectory(${JSON.stringify(runtimeDir)});
const loaded = performance.now() - t0;
const opened = {};
for (const [name, file] of ${JSON.stringify(WORKLOADS.map((w) => [w.name, w.program]))}) {
  const source = readFileSync(file, 'utf8');
  const t = performance.now(); runtime.open(source).close(); opened[name] = performance.now() - t;
}
console.log(JSON.stringify({ loaded, opened }));`;
const runs = Array.from({ length: 9 }, () => JSON.parse(execFileSync(process.execPath, ['--input-type=module', '-e', probe], { encoding: 'utf8' })));
const ms = (n) => Number(n.toFixed(2));
put('runtime_load_ms', ms(median(runs.map((r) => r.loaded))), 'ms', 0.5, false);
for (const w of WORKLOADS) put(`open_${w.name}_ms`, ms(median(runs.map((r) => r.opened[w.name]))), 'ms', 0.5, false);

// Memory: WebAssembly memory grows in 64 KiB pages and never shrinks, so
// open many sessions, run each workload's events in every one, and divide the
// growth. The instance is loaded directly to read its memory.
const namespace = await import(`${pathToFileURL(glueFile).href}?footprint`);
const exports = await namespace.default({ module_or_path: readFileSync(wasmFile) });
const memory = () => exports.memory.buffer.byteLength;
put('wasm_memory_initial_bytes', memory(), 'bytes', 0.05, true);
for (const w of WORKLOADS) {
  const source = readFileSync(w.program, 'utf8');
  const sessions = 20;
  // One session first, so the growth counted is per session, not the
  // allocator's first arena.
  const warm = new namespace.WebReactiveSession(source);
  for (const [event, payload] of w.events) warm.dispatch_view_outcome(event, JSON.stringify(payload));
  const before = memory();
  const open = [];
  for (let i = 0; i < sessions; i += 1) {
    const session = new namespace.WebReactiveSession(source);
    for (const [event, payload] of w.events) session.dispatch_view_outcome(event, JSON.stringify(payload));
    open.push(session);
  }
  put(`memory_per_session_${w.name}_bytes`, Math.round((memory() - before) / sessions), 'bytes', 0.1, true);
  put(`state_save_${w.name}_bytes`, Buffer.byteLength(open[0].save()), 'bytes', 0.05, true);
  for (const session of open) session.free();
  warm.free();
}

let baseline = {};
try { baseline = JSON.parse(readFileSync(baselineFile, 'utf8')).measures ?? {}; } catch { /* no baseline yet */ }
const over = [];
const rows = [];
for (const [name, m] of Object.entries(measures)) {
  const base = baseline[name]?.value;
  const limit = base === undefined ? undefined : base * (1 + m.allowed_growth);
  const status = base === undefined ? 'new' : m.value > limit ? 'OVER' : 'ok';
  if (status === 'OVER') over.push(name);
  rows.push(`| ${name} | ${m.value} ${m.unit} | ${base ?? '-'} | +${Math.round(m.allowed_growth * 100)}% | ${status} |`);
}
const table = ['| Measure | This build | Baseline | Allowed growth | |', '| --- | ---: | ---: | ---: | --- |', ...rows].join('\n');
writeFileSync(path.join(out, 'footprint.json'), `${JSON.stringify({ schema: 'caveat-carmack-footprint/1', node: process.version, platform: process.platform, measures, over }, null, 2)}\n`);
writeFileSync(path.join(out, 'footprint.md'), `# Footprint\n\n${table}\n`);
console.log(table);
if (flag('write-baseline')) {
  writeFileSync(baselineFile, `${JSON.stringify({ schema: 'caveat-carmack-baseline/1', recorded: new Date().toISOString().slice(0, 10), node: process.version, platform: process.platform, measures }, null, 2)}\n`);
  console.log(`baseline written to ${path.relative(ROOT, baselineFile)}`);
}
if (over.length) console.log(`over the baseline: ${over.join(', ')}`);
process.exit(over.length && flag('strict') ? 1 : 0);
