// The source sweep for Only If 0.1 (spec/caveat-only-if-0.1.md): every
// tracked Caveat program, read as text. `only_if(…)` did not load before the
// profile unless the program defined `fn only_if`, so a program that mentions
// the word is one whose meaning this profile could touch.
// Usage: node experiments/only-if-sweep/sweep.mjs [--write]
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
// Programs written for this profile; the sweep is of the corpus before it.
const writtenForProfile = new Set(['experiments/model-votes/votes.cav']);
const mentions = programs
  .filter(file => /\bonly_if\b/.test(readFileSync(join(root, file), 'utf8')))
  .map(file => ({ file, written_for_profile: writtenForProfile.has(file) }));
assert.deepEqual(mentions.filter(item => !item.written_for_profile), [], 'a tracked program mentions only_if');
const results = { programs: programs.length, mentions_only_if: mentions };
const json = `${JSON.stringify(results, null, 2)}\n`;
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), json);
process.stdout.write(json);
