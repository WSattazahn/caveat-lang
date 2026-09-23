// Integrity is distinct from candidate correctness and cannot establish read isolation.
import { writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { here, assertRegisteredInputs, assertFinishedRuns } from './integrity.mjs';

const inputs = await assertRegisteredInputs();
const { summaries } = await assertFinishedRuns(inputs.registration);
const report = { schema: 1, at: new Date().toISOString(), frozenFiles: inputs.files,
  references: Object.keys(inputs.manifest.references).length,
  runtimeFiles: Object.keys(inputs.manifest.runtime).length, runs: summaries,
  isolation: 'Fresh contexts, instructions and author disclosures; no OS sandbox or exhaustive read-access telemetry.' };
const args = process.argv.slice(2);
if (args.length && (args.length !== 2 || args[0] !== '--output')) throw new Error('Usage: node audit.mjs [--output path]');
const destination = args.length ? path.resolve(args[1]) : path.join(here, 'results/integrity.json');
await mkdir(path.dirname(destination), { recursive: true });
await writeFile(destination, `${JSON.stringify(report, null, 2)}\n`, { flag: args.length ? 'w' : 'wx' });
console.log('Integrity audit passed: four final candidates, budgets, archives, corpus and frozen inputs match.');
