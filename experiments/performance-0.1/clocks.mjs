// The clock each performance core reaches under a single-threaded busy loop.
//
//   node experiments/performance-0.1/clocks.mjs [--processors=0,1,10,...] [--seconds=5] --out=FILE
//
// For each logical processor in turn (by default every performance core), a
// child Node process spins for --seconds pinned to that processor alone at
// High priority, while typeperf samples that processor's use and actual clock
// (`\Processor Information(0,N)\Actual Frequency`) every second. The result
// is the mean clock over the samples where the processor was more than 80%
// busy. It explains part of why the same benchmark differs between cores
// (RESULTS.md, section 7); it times nothing of the runtime. Windows only.
import { spawn } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import os from 'node:os';
import { load, machine } from './lib/environment.mjs';
import { maskOf, runPinned } from './lib/pinning.mjs';

const argv = process.argv.slice(2);
const single = (name, fallback = null) => argv.filter((arg) => arg.startsWith(`--${name}=`)).map((arg) => arg.slice(name.length + 3)).at(-1) ?? fallback;
if (process.platform !== 'win32') throw new Error('clocks.mjs reads Windows performance counters');
const out = single('out');
if (!out) throw new Error('give --out=FILE');
const seconds = Number(single('seconds', '5'));
const info = await machine(os.tmpdir());
const processors = single('processors')?.split(',').map(Number) ?? info.performanceCores;
const spin = `const end = Date.now() + ${seconds * 1000}; let x = 0; while (Date.now() < end) { for (let i = 0; i < 1e5; i++) x += Math.sqrt(i); } if (x < 0) console.log(x);`;
const results = [];
for (const processor of processors) {
  const counters = [`\\Processor Information(0,${processor})\\Actual Frequency`, `\\Processor(${processor})\\% Processor Time`];
  const monitor = spawn('typeperf', [...counters, '-si', '1', '-sc', String(seconds + 1)], { stdio: ['ignore', 'pipe', 'ignore'] });
  let text = '';
  monitor.stdout.on('data', (chunk) => { text += chunk; });
  runPinned(process.execPath, ['-e', spin], { affinity: maskOf([processor]), priority: 'high' });
  await new Promise((resolve) => monitor.on('exit', resolve));
  const rows = text.split(/\r?\n/).filter((line) => /^"\d/.test(line)).map((line) => line.split('","').map((cell) => Number(cell.replace(/"/g, ''))));
  const busy = rows.filter((row) => row[2] > 80);
  const mhz = busy.map((row) => row[1]);
  const mean = mhz.length ? mhz.reduce((a, b) => a + b, 0) / mhz.length : null;
  const set = info.cpuSetList?.find((entry) => entry.logicalProcessor === processor) ?? null;
  results.push({ processor, cpuSet: set, busySamples: busy.length, meanMHz: mean === null ? null : Number(mean.toFixed(0)), samplesMHz: mhz.map((value) => Number(value.toFixed(0))) });
  console.log(`processor ${processor}: ${busy.length} busy samples, mean ${mean?.toFixed(0) ?? '-'} MHz`);
}
await writeFile(out, `${JSON.stringify({ schema: 'caveat-performance-clocks/0.1', at: new Date().toISOString(), seconds, machine: info, loadAfter: await load(os.tmpdir()), results }, null, 1)}\n`);
