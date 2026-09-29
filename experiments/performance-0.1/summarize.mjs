// Rewrites summary.md from a run's results.json (or results.json.gz, as
// committed under results/), for example after the report format changes.
//
//   node experiments/performance-0.1/summarize.mjs RESULTS_DIR
import { existsSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { gunzipSync } from 'node:zlib';
import { renderSummary } from './lib/report.mjs';

const directory = process.argv[2];
if (!directory) throw new Error('usage: summarize.mjs RESULTS_DIR');
const file = path.join(directory, 'results.json');
const text = existsSync(file) ? await readFile(file, 'utf8') : gunzipSync(await readFile(`${file}.gz`)).toString('utf8');
await writeFile(path.join(directory, 'summary.md'), renderSummary(JSON.parse(text)));
console.log(`wrote ${path.join(directory, 'summary.md')}`);
