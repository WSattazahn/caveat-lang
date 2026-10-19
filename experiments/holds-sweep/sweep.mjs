// Corpus sweep for Holds 0.1 (spec/caveat-holds-0.1.md, "Corpus sweep"). The
// profile adds the effects `hold SERIES because …` and `release SERIES
// because …` and the predicate `on_hold(SERIES)`. This reads every tracked
// Caveat program and reports:
//   - every word `hold`, `release` or `on_hold` outside quoted text, and
//     every `fn on_hold`: a program that uses one as a name must load and
//     behave as before;
//   - every decision series with `reopened by` triggers, and every rule that
//     reopens a series: the reopenings a hold could defer.
// It reads source text only; it runs no program. Comments and quoted text are
// removed before reading.
// Run from any directory: node experiments/holds-sweep/sweep.mjs [--write]
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
assert.ok(programs.length > 0, 'the tracked Caveat corpus must not be empty');

// `#` or `//` comments and quoted text carry no source words.
function code(text) {
  let out = '';
  let inString = false;
  for (let i = 0; i < text.length; i += 1) {
    const c = text[i];
    if (inString) {
      if (c === '\\') { i += 1; continue; }
      if (c === '"') { inString = false; out += '"'; }
      continue;
    }
    if (c === '"') { inString = true; out += '"'; continue; }
    if (c === '#' || (c === '/' && text[i + 1] === '/')) {
      while (i < text.length && text[i] !== '\n') i += 1;
      out += '\n';
      continue;
    }
    out += c;
  }
  return out;
}

const words = [];
const definitions = [];
const triggered = [];
const reopenRules = [];
for (const file of programs) {
  const text = code(readFileSync(join(root, file), 'utf8'));
  const lines = text.split('\n');
  lines.forEach((line, index) => {
    for (const match of line.matchAll(/\b(hold|release|on_hold)\b/g)) {
      words.push({ file, line: index + 1, word: match[1] });
    }
    if (/\bfn\s+on_hold\b/.test(line)) definitions.push({ file, line: index + 1 });
  });
  for (const match of text.matchAll(/\bdecisions\s+(\w+)\s+limit\s+\S+\s+reopened\s+by\b/g)) {
    triggered.push({ file, series: match[1] });
  }
  for (const match of text.matchAll(/\breopen\s+(\w+)\s+because\b/g)) {
    reopenRules.push({ file, series: match[1] });
  }
}

const results = {
  programs: programs.length,
  words,
  on_hold_definitions: definitions,
  series_with_triggers: triggered.length,
  programs_with_triggers: new Set(triggered.map(entry => entry.file)).size,
  reopen_effects: reopenRules.length,
  programs_with_reopen_effects: new Set(reopenRules.map(entry => entry.file)).size,
};
const json = `${JSON.stringify(results, null, 2)}\n`;
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), json);
process.stdout.write(json);
