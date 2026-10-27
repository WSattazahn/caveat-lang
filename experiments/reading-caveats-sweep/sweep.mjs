// Corpus sweep for Reading Caveats 0.1 (spec/caveat-reading-caveats-0.1.md,
// "Corpus sweep"). The profile adds `with CAVEAT[, CAVEAT...] [when
// CONDITION]` at the end of a sample. This reads every tracked Caveat program
// and reports:
//   - every sample statement with the word `with` after its `=`: before this
//     profile none could load, so a program that has one either fails to load
//     or names something `with`;
//   - every declaration of a name `with` (a state, evidence, claim, caveat,
//     stream, series, event, parameter or define): such a program must load
//     and behave as before;
//   - every `qualify latest(STREAM)`: the way a program puts a caveat on one
//     reading today, as a late caveat.
// It reads source text only; it runs no program. Comments and quoted text are
// removed before reading.
// Run from any directory: node experiments/reading-caveats-sweep/sweep.mjs [--write]
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

const declaration = /\b(?:state|evidence|claim|caveat|readings|decisions|event|define|fn|proc|identifiers)\s+with\b|[(,]\s*with\s*(?:[,)]|\b(?:min|evidence|claim|caveat|readings|decisions|id)\b)/;
const samples = [];
const names = [];
const qualifyLatest = [];
for (const file of programs) {
  const text = code(readFileSync(resolve(root, file), 'utf8'));
  let line = 1;
  for (const statement of text.split(';')) {
    const at = line + (statement.match(/^\s*/)[0].split('\n').length - 1);
    const sample = statement.match(/\bsample\s+[\w$]+\s*=([\s\S]*)$/);
    if (sample && /\bwith\b/.test(sample[1])) samples.push({ file, line: at, text: statement.trim().replace(/\s+/g, ' ') });
    if (declaration.test(statement)) names.push({ file, line: at, text: statement.trim().replace(/\s+/g, ' ') });
    if (/\bqualify\s+latest\s*\(/.test(statement)) qualifyLatest.push({ file, line: at, text: statement.trim().replace(/\s+/g, ' ') });
    line += statement.split('\n').length - 1;
  }
}

const results = {
  programs: programs.length,
  samples_with_the_word_with: samples,
  names_with: names,
  qualify_latest: qualifyLatest.length,
  programs_with_qualify_latest: new Set(qualifyLatest.map(entry => entry.file)).size,
  qualify_latest_statements: qualifyLatest,
};
const json = `${JSON.stringify(results, null, 2)}\n`;
if (process.argv.includes('--write') && extra.length === 0) writeFileSync(join(here, 'RESULTS.json'), json);
process.stdout.write(json);
