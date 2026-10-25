#!/usr/bin/env node
// Carmack check 4: one speed number to beat, re-measured every release. Reads
// the profile check's profile.json and reports the release-gate workload's
// median time per event in WebAssembly, with the whole view a host reads,
// against bench/carmack/target.json. A report, not a gate: a miss is printed
// as a miss and the release record carries it.
//
//   node bench/carmack/target.mjs [--profile FILE] [--out DIR]
import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { ROOT } from './workloads.mjs';

const arg = (name) => { const i = process.argv.indexOf(`--${name}`); return i < 0 ? undefined : process.argv[i + 1]; };
const out = path.resolve(arg('out') ?? path.join(ROOT, 'test-results', 'carmack'));
const profile = JSON.parse(readFileSync(path.resolve(arg('profile') ?? path.join(out, 'profile.json')), 'utf8'));
const target = JSON.parse(readFileSync(path.join(ROOT, 'bench', 'carmack', 'target.json'), 'utf8'));

const measured = profile.workloads[target.workload]?.wasm?.event_median_us;
if (typeof measured !== 'number') throw new Error(`profile.json has no ${target.workload} WebAssembly median`);
if (target.target_us === null) {
  // No target is set yet: report the number alone.
  const line = `No target set yet: ${measured} µs per event.`;
  writeFileSync(path.join(out, 'target.json'), `${JSON.stringify({ schema: 'caveat-carmack-target/1', workload: target.workload, measure: target.measure, measured_us: measured, target_us: null, host: profile.host }, null, 2)}\n`);
  writeFileSync(path.join(out, 'target.md'), `# Speed target\n\n${target.measure}\n\n${line}\n`);
  console.log(line);
  process.exit(0);
}
const days = Math.ceil((Date.parse(target.deadline) - Date.now()) / 86_400_000);
const met = measured <= target.target_us;
const factor = measured / target.target_us;
const line = met
  ? `Met: ${measured} µs per event against the target of ${target.target_us} µs.`
  : `Not met: ${measured} µs per event, ${factor.toFixed(1)}x the target of ${target.target_us} µs, with ${days} day(s) to ${target.deadline}.`;
const result = { schema: 'caveat-carmack-target/1', workload: target.workload, measure: target.measure, measured_us: measured, target_us: target.target_us, deadline: target.deadline, met, factor: Number(factor.toFixed(2)), host: profile.host };
writeFileSync(path.join(out, 'target.json'), `${JSON.stringify(result, null, 2)}\n`);
writeFileSync(path.join(out, 'target.md'), `# Speed target\n\n${target.measure}\n\n${line}\n`);
console.log(line);
