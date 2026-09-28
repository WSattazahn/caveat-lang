// Rewrites summary.md from a run's results.json, for example after the
// report format changes.
//
//   node experiments/performance-0.1/summarize.mjs RESULTS_DIR
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { renderSummary } from './lib/report.mjs';

const directory = process.argv[2];
if (!directory) throw new Error('usage: summarize.mjs RESULTS_DIR');
const report = JSON.parse(await readFile(path.join(directory, 'results.json'), 'utf8'));
await writeFile(path.join(directory, 'summary.md'), renderSummary(report));
console.log(`wrote ${path.join(directory, 'summary.md')}`);
