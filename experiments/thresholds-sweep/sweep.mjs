// Corpus sweep for Thresholds 0.1 (spec/caveat-thresholds-0.1.md, "Corpus
// sweep"). The profile adds the statement `threshold NAME = NUMBER`. This
// reads every tracked Caveat program and reports:
//   - every statement that starts with `threshold`: before this profile none
//     could load;
//   - every other use of the word `threshold`, such as a state of that name:
//     such a program must load and behave as before;
//   - every `define NAME = NUMBER`: a named number a program could declare as
//     a threshold to have its commits list it.
// It reads source text only; it runs no program. Comments and quoted text are
// removed before reading.
// Run from any directory: node experiments/thresholds-sweep/sweep.mjs [--write]
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

const statements = [];
const words = [];
const numberDefines = [];
for (const file of programs) {
  const text = code(readFileSync(resolve(root, file), 'utf8'));
  let line = 1;
  for (const statement of text.split(';')) {
    const at = line + (statement.match(/^\s*/)[0].split('\n').length - 1);
    const flat = statement.trim().replace(/\s+/g, ' ');
    if (/^threshold\b/.test(flat)) statements.push({ file, line: at, text: flat });
    else if (/\bthreshold\b/.test(flat)) words.push({ file, line: at, text: flat });
    if (/^define [\w$]+ = [+-]?(?:\d+\.?\d*|\.\d+)$/.test(flat)) numberDefines.push({ file, line: at, text: flat });
    line += statement.split('\n').length - 1;
  }
}

const results = {
  programs: programs.length,
  threshold_statements: statements,
  other_uses_of_threshold: words,
  number_defines: numberDefines.length,
  programs_with_number_defines: new Set(numberDefines.map(entry => entry.file)).size,
  number_define_statements: numberDefines,
};
const json = `${JSON.stringify(results, null, 2)}\n`;
if (process.argv.includes('--write') && extra.length === 0) writeFileSync(join(here, 'RESULTS.json'), json);
process.stdout.write(json);
