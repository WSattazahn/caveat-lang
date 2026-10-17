// Corpus sweep for Earlier Occurrence 0.1 (spec/caveat-earlier-occurrence-0.1.md,
// "Corpus sweep"). The profile adds one selector, `occurrence(EVIDENCE, INDEX)`,
// in positions that refuse to load today. This reads every tracked Caveat
// program and reports:
//   - every use of the word `occurrence`, and every `occurrence(` (there must be
//     none: a selector position would mean a program that loads today, and a
//     `fn occurrence` would only be worth noting);
//   - every effect target or predicate argument, other than `latest(...)` and
//     `reopen ... because caveated(...)`, that
//     holds a parenthesis (there must be none: such a program does not load);
//   - the renewable evidence each program declares, and which of those names a
//     `withdraw`, `qualify`, `unqualify` or `reopen ... because` reaches: the
//     programs whose authors could now name an earlier occurrence.
// It reads source text only; it runs no program. Comments and text literals
// are removed before reading.
// Run from any directory: node experiments/earlier-occurrence-sweep/sweep.mjs [--write]
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

// Text literals and `#` or `//` comments carry no source words.
function code(text) {
  let out = '';
  let inString = false;
  for (let i = 0; i < text.length; i += 1) {
    const c = text[i];
    if (inString) {
      if (c === '\\') { i += 1; continue; }
      if (c === '"') { inString = false; out += '""'; }
      continue;
    }
    if (c === '"') { inString = true; continue; }
    if (c === '#' || (c === '/' && text[i + 1] === '/')) {
      while (i < text.length && text[i] !== '\n') i += 1;
      out += '\n';
      continue;
    }
    out += c;
  }
  return out;
}

const WORD = /[A-Za-z_$][A-Za-z0-9_$@]*/g;
const NAME = '[A-Za-z_$][A-Za-z0-9_$]*';
const statements = text => text.split(/[;{}]/).map(s => s.replace(/\s+/g, ' ').trim()).filter(Boolean);

// What each effect or predicate names as its one evidence occurrence.
const TARGETS = [
  [/(?:^|\s)withdraw (.+?) because /g, 'withdraw'],
  [/(?:^|\s)unqualify (.+?) with /g, 'unqualify'],
  [/(?:^|\s)(?<!un)qualify (.+?) with /g, 'qualify'],
  [/(?:^|\s)reopen \S+ because (.+?)$/g, 'reopen because'],
  [/(?<![A-Za-z0-9_$])(?:observed|withdrawn)\((.*?)\)(?:\s|$|,|\))/g, 'predicate'],
  [/(?<![A-Za-z0-9_$])carries\((.*?),/g, 'carries'],
];

const rows = [];
const wordUses = [];
const selectorUses = [];
const parenthesized = [];
let renewableNames = 0;
const programsWithRenewable = [];
for (const path of programs) {
  const text = code(readFileSync(join(root, path), 'utf8'));
  const words = text.match(WORD) ?? [];
  if (words.includes('occurrence')) wordUses.push(path);
  if (/(?<![A-Za-z0-9_$])occurrence\s*\(/.test(text)) selectorUses.push(path);
  const lines = statements(text);
  const renewable = new Set();
  for (const line of lines) {
    const m = line.match(new RegExp(`^renewable (${NAME}) (?:limit|window) `));
    if (m) renewable.add(m[1]);
  }
  const reached = new Map();
  for (const line of lines) {
    for (const [pattern, form] of TARGETS) {
      for (const m of line.matchAll(pattern)) {
        const target = m[1].trim();
        if (target.includes('(') && !/^(?:latest|caveated)\s*\(/.test(target)) parenthesized.push({ path, form, target });
        if (renewable.has(target)) {
          if (!reached.has(target)) reached.set(target, new Set());
          reached.get(target).add(form);
        }
      }
    }
  }
  if (renewable.size === 0) continue;
  programsWithRenewable.push(path);
  renewableNames += renewable.size;
  for (const evidence of [...renewable].sort()) {
    rows.push({ path, evidence, reachedBy: [...(reached.get(evidence) ?? [])].sort() });
  }
}

const reaching = new Set(rows.filter(row => row.reachedBy.length > 0).map(row => row.path));
const report = {
  note: 'Earlier Occurrence 0.1 corpus sweep. Source text only; no program is run. See spec/caveat-earlier-occurrence-0.1.md, "Corpus sweep".',
  revision: spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).stdout.trim(),
  programs: programs.length,
  programs_using_the_word_occurrence: wordUses,
  programs_writing_occurrence_call: selectorUses,
  targets_with_a_parenthesis_other_than_latest_or_caveated: parenthesized,
  programs_with_renewable_evidence: programsWithRenewable.length,
  renewable_evidence_names: renewableNames,
  programs_reaching_renewable_evidence: reaching.size,
  rows,
};
assert.deepEqual(selectorUses, [], 'a tracked program writes occurrence(');
assert.deepEqual(parenthesized, [], 'a tracked program names a parenthesized target that is not latest(...) or caveated(...)');
const out = JSON.stringify(report, null, 2) + '\n';
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), out);
console.log(`${programs.length} programs; the word occurrence in ${wordUses.length}; occurrence( in ${selectorUses.length}; ` +
  `${parenthesized.length} parenthesized targets; ${programsWithRenewable.length} programs declare ` +
  `${renewableNames} renewable evidence names; ${reaching.size} withdraw, qualify, take back or reopen on one`);
