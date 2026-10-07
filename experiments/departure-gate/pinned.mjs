// Seeded checkpoints for every named departure corpus program, including the
// forever-skipped commit fixture. Fatal dispatches/restore failures fail the run.
// node experiments/departure-gate/pinned.mjs --package=kit [--runs=8 --events=4000]
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import { pinnedRecords, maximumPins } from './pin-census.mjs';
const here = path.dirname(fileURLToPath(import.meta.url));
const option = (name, fallback) => process.argv.find((arg) => arg.startsWith(`--${name}=`))?.slice(name.length + 3) ?? fallback;
const packageDir = path.resolve(option('package', 'kit'));
const runs = Number(option('runs', '8'));
const events = Number(option('events', '4000'));
if (![runs, events].every((n) => Number.isInteger(n) && n > 0)) throw new Error('runs/events must be positive integers');
const { loadRuntimeFromDirectory } = await import(pathToFileURL(path.join(packageDir, 'lib/node.mjs')));
const runtime = await loadRuntimeFromDirectory();
const reports = [];
for (const file of readdirSync(here).filter((name) => name.endsWith('.cav')).sort()) {
  const source = readFileSync(path.join(here, file), 'utf8');
  const initial = runtime.open(source);
  const signatures = initial.snapshot().events;
  initial.close();
  const checkpoints = [];
  let accepted = 0, rejected = 0;
  for (let run = 0; run < runs; run++) {
    const session = runtime.open(source);
    let seed = (0x9e3779b9 ^ (run + 1) * 0x51ed270b) >>> 0;
    const random = () => { seed ^= seed << 13; seed ^= seed >>> 17; seed ^= seed << 5; return seed >>> 0; };
    try {
      for (let index = 1; index <= events; index++) {
        const signature = signatures[random() % signatures.length];
        const payload = Object.fromEntries(signature.parameters.map((parameter) => {
          const members = parameter.domain?.entity?.members ?? parameter.domain?.member?.members;
          const value = members ? members[random() % members.length]
            : parameter.domain?.identifier ? `id${random() % 12}`
            : parameter.min + (parameter.max - parameter.min) * (random() % 1001) / 1000;
          return [parameter.name, value];
        }));
        const outcome = session.dispatchView(signature.name, payload);
        if (outcome.outcome === 'accepted') accepted++; else rejected++;
        if (index % 100 === 0 || index === events) {
          const saved = session.save();
          const census = pinnedRecords(JSON.parse(saved));
          if (census.unpinned_non_declarations.length) throw new Error(`${file}: unpinned retired records remain: ${census.unpinned_non_declarations}`);
          const restored = runtime.restore(source, saved);
          restored.close();
          checkpoints.push({ run, event: index, ...census });
          if (typeof session.drainArchive === 'function') session.drainArchive();
        }
      }
    } finally { session.close(); }
  }
  reports.push({ program: file, source_sha256: createHash('sha256').update(source).digest('hex'),
    runs, events_per_run: events, accepted, rejected, checkpoints: checkpoints.length,
    maxima: maximumPins(checkpoints),
    checkpoint_maximum: checkpoints.find((entry) => entry.pinned === maximumPins(checkpoints).pinned),
    first_run_start: checkpoints.find((entry) => entry.run === 0),
    first_run_end: checkpoints.filter((entry) => entry.run === 0).at(-1) });
}
console.log(JSON.stringify({ package: JSON.parse(readFileSync(path.join(packageDir, 'package.json'), 'utf8')).version,
  runtime: runtime.identity, reports }, null, 2));
