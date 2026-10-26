// The source sweep for Grounds Of 0.1 (spec/caveat-grounds-of-0.1.md): every
// tracked Caveat program, read as text. `grounds_of(…)` did not load before
// the profile unless the program defined `fn grounds_of`, so a program that
// mentions the word is one whose meaning this profile could touch.
// Usage: node experiments/grounds-of-sweep/sweep.mjs [--write]
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../..');
const listed = spawnSync('git', ['ls-files', '-z', '--', '*.cav'], { cwd: root, encoding: 'utf8' });
assert.equal(listed.status, 0, listed.stderr);
const programs = listed.stdout.split('\0').filter(Boolean).sort();
const mentions = programs.filter(file => /\bgrounds_of\b/.test(readFileSync(join(root, file), 'utf8')));
assert.deepEqual(mentions, [], 'a tracked program mentions grounds_of');
// Programs that read a series' current value, where grounds_of would stand.
const readsSeries = programs.filter(file => {
  const text = readFileSync(join(root, file), 'utf8');
  const series = new Set([...text.matchAll(/^\s*decisions\s+(\$?\w+)\s+limit/gm)].map(match => match[1]));
  return [...text.matchAll(/\blatest\((\$?\w+)\)/g)].some(match => series.has(match[1]));
});
const results = { programs: programs.length, mentions_grounds_of: mentions, reads_latest_of_a_series: readsSeries };
const json = `${JSON.stringify(results, null, 2)}\n`;
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), json);
process.stdout.write(json);
