// Corpus sweep for Outcomes 0.1 (spec/caveat-outcomes-0.1.md, "Corpus
// sweep"). The profile adds the effect `resolve SERIES = VALUE because …
// [materialized CAVEAT, …]`, the predicate `resolved(…)` and the reading
// `outcome(…)`. This reads every tracked Caveat program and reports:
//   - every word `resolve`, `resolved`, `outcome` or `materialized` outside
//     quoted text, and every `fn resolved` or `fn outcome`: a program that
//     uses one as a name must load and behave as before;
//   - every `resolve` written as an effect would be, followed by a name and
//     `=` or by `revision(`: before this profile such a rule did not load;
//   - every decision series, and every series whose program samples a
//     reading the host would otherwise pair with it by hand.
// It reads source text only; it runs no program. Comments and quoted text are
// removed before reading.
// Run from any directory: node experiments/outcomes-sweep/sweep.mjs [--write]
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
const effects = [];
const series = [];
for (const file of programs) {
  const text = code(readFileSync(join(root, file), 'utf8'));
  const lines = text.split('\n');
  lines.forEach((line, index) => {
    for (const match of line.matchAll(/\b(resolve|resolved|outcome|materialized)\b/g)) {
      words.push({ file, line: index + 1, word: match[1] });
    }
    for (const match of line.matchAll(/\bfn\s+(resolved|outcome)\b/g)) {
      definitions.push({ file, line: index + 1, name: match[1] });
    }
    if (/\bresolve\s+(\w+\s*=[^=]|revision\s*\()/.test(line)) effects.push({ file, line: index + 1 });
  });
  for (const match of text.matchAll(/\bdecisions\s+(\w+)\s+limit\b/g)) series.push({ file, series: match[1] });
}

const results = {
  programs: programs.length,
  words,
  definitions,
  resolve_effects: effects,
  decision_series: series.length,
  programs_with_series: new Set(series.map(entry => entry.file)).size,
};
const json = `${JSON.stringify(results, null, 2)}\n`;
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), json);
process.stdout.write(json);
