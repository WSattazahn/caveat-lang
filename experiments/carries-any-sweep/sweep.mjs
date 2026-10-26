// Corpus sweep for Carries Any 0.1 (spec/caveat-carries-any-0.1.md, "Corpus
// sweep"). The profile adds the predicate `carries_any(EVIDENCE [,
// CONSEQUENCE])`. This reads every tracked Caveat program and reports:
//   - every word `carries_any` outside quoted text, and every `fn
//     carries_any`: a program that uses the word as a name must load and
//     behave as before;
//   - every condition that writes "any caveat" out by hand: two or more
//     `carries(TARGET, CAVEAT)` or `withdrawn(TARGET)` joined by `or` on one
//     target, with the caveats it lists and the caveats the program declares
//     that the list leaves out.
// It reads source text only; it runs no program. Comments and quoted text are
// removed before reading.
// Run from any directory: node experiments/carries-any-sweep/sweep.mjs [--write]
// Files named after the flags are read instead of the corpus, to check the
// sweep on programs outside it; `--write` is ignored then.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../..');
const listed = spawnSync('git', ['ls-files', '-z', '--', '*.cav'], { cwd: root, encoding: 'utf8' });
assert.equal(listed.status, 0, listed.stderr);
const extra = process.argv.slice(2).filter(arg => !arg.startsWith('--')).map(file => resolve(file));
const programs = extra.length > 0 ? extra : listed.stdout.split('\0').filter(Boolean).sort();
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

// A target is a name, `latest(NAME)`, `occurrence(E, I)` or `reading(S, I)`.
const target = String.raw`(?:(?:latest|occurrence|reading)\s*\([^()]*\)|[\w$]+)`;
const term = new RegExp(String.raw`\b(?:carries\s*\(\s*(${target})\s*,\s*([\w$]+)\s*\)|withdrawn\s*\(\s*(${target})\s*\))`, 'g');

const words = [];
const definitions = [];
const lists = [];
for (const file of programs) {
  const text = code(readFileSync(resolve(root, file), 'utf8'));
  const declared = [...text.matchAll(/\bcaveat\s+([\w$]+)\s+consequence\b/g)].map(match => match[1]);
  text.split('\n').forEach((line, index) => {
    for (const match of line.matchAll(/\bcarries_any\b/g)) words.push({ file, line: index + 1, word: match[0] });
    if (/\bfn\s+carries_any\b/.test(line)) definitions.push({ file, line: index + 1 });
  });
  // A statement ends at `;`. Within one, terms joined only by `or` on the
  // same target are one hand-written list.
  let line = 1;
  for (const statement of text.split(';')) {
    const terms = [...statement.matchAll(term)];
    for (let start = 0; start < terms.length;) {
      const run = [terms[start]];
      const of = match => (match[1] ?? match[3]).replace(/\s+/g, '');
      while (start + run.length < terms.length) {
        const previous = run[run.length - 1];
        const next = terms[start + run.length];
        const between = statement.slice(previous.index + previous[0].length, next.index);
        if (!/^\s*or\s*$/.test(between) || of(next) !== of(run[0])) break;
        run.push(next);
      }
      if (run.length >= 2) {
        const listedCaveats = run.map(match => match[2] ?? 'withdrawn');
        const offset = statement.slice(0, run[0].index).split('\n').length - 1;
        lists.push({
          file,
          line: line + offset,
          target: of(run[0]),
          caveats: listedCaveats,
          declared_but_not_listed: declared.filter(name => !listedCaveats.includes(name)),
        });
      }
      start += run.length;
    }
    line += statement.split('\n').length - 1;
  }
}

const results = {
  programs: programs.length,
  words,
  definitions,
  hand_written_lists: lists.length,
  programs_with_lists: new Set(lists.map(entry => entry.file)).size,
  lists_missing_a_declared_caveat: lists.filter(entry => entry.declared_but_not_listed.length > 0).length,
  lists,
};
const json = `${JSON.stringify(results, null, 2)}\n`;
if (process.argv.includes('--write') && extra.length === 0) writeFileSync(join(here, 'RESULTS.json'), json);
process.stdout.write(json);
