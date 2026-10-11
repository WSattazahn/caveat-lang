// Corpus sweep for Take-back 0.1 (spec/caveat-take-back-0.1.md, "Corpus
// sweep"). The profile adds one effect word and restrictions that apply only
// to a program that takes back. This reads every tracked Caveat program and
// reports:
//   - every use of the new word `unqualify`, as a token anywhere (it must be
//     none outside this profile's own programs in programs/: the word is
//     reserved, so a declaration of it would stop loading);
//   - every `qualify EVIDENCE with CAVEAT`, and for each caveat whether a
//     take-back of it would meet the profile's load rules, or which rule it
//     would break. This changes no program; it shows where the rules bite.
// It reads source text only; it runs no program. Comments and text literals
// are removed before reading.
// Run from any directory: node experiments/take-back-sweep/sweep.mjs [--write]
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

// Text literals and `#` comments carry no source words.
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
    if (c === '#') {
      while (i < text.length && text[i] !== '\n') i += 1;
      out += '\n';
      continue;
    }
    out += c;
  }
  return out;
}

const WORD = /[A-Za-z_$][A-Za-z0-9_$@]*/g;

// The top-level arguments of each call to `name(`.
function calls(text, name) {
  const found = [];
  const pattern = new RegExp(`(?<![A-Za-z0-9_$])${name.replace('$', '\\$')}\\(`, 'g');
  for (const match of text.matchAll(pattern)) {
    let depth = 1;
    let start = match.index + match[0].length;
    const args = [];
    let i = start;
    for (; i < text.length && depth > 0; i += 1) {
      const c = text[i];
      if (c === '(') depth += 1;
      else if (c === ')') { depth -= 1; if (depth === 0) break; }
      else if (c === ',' && depth === 1) { args.push(text.slice(start, i).trim()); start = i + 1; }
    }
    args.push(text.slice(start, i).trim());
    found.push(args);
  }
  return found;
}

const statements = text => text.split(/[;{}]/).map(s => s.replace(/\s+/g, ' ').trim()).filter(Boolean);

const rows = [];
const wordUses = [];
for (const path of programs) {
  const text = code(readFileSync(join(root, path), 'utf8'));
  const words = text.match(WORD) ?? [];
  if (words.includes('unqualify')) wordUses.push(path);
  const lines = statements(text);
  const qualifies = [];
  for (const line of lines) {
    for (const m of line.matchAll(/(?:^|\s)qualify ([A-Za-z_$][A-Za-z0-9_$@]*) with ([A-Za-z_$][A-Za-z0-9_$]*)/g)) {
      qualifies.push({ evidence: m[1], caveat: m[2] });
    }
  }
  if (qualifies.length === 0) continue;
  const parameters = new Set();
  for (const m of text.matchAll(/proc\s+[A-Za-z_$][A-Za-z0-9_$]*\s*\(([^)]*)\)/g)) {
    for (const p of m[1].split(',').map(s => s.trim()).filter(Boolean)) parameters.add(p);
  }
  const templates = new Set();
  for (const line of lines) {
    const m = line.match(/^readings [A-Za-z_$][A-Za-z0-9_$]* from ([A-Za-z_$][A-Za-z0-9_$]*)/);
    if (m) templates.add(m[1]);
  }
  const declaredQualifies = lines
    .map(line => line.match(/^([A-Za-z_$][A-Za-z0-9_$]*) qualifies ([A-Za-z_$][A-Za-z0-9_$@]*)$/))
    .filter(Boolean)
    .map(m => ({ caveat: m[1], target: m[2] }));
  const extras = calls(text, 'qualified').flatMap(args => args.slice(2));
  const retained = lines.flatMap(line => {
    const m = line.match(/retaining (.*)$/);
    return m ? m[1].split(',').map(s => s.trim().split(' ')[0]) : [];
  });
  const examinedReads = calls(text, 'examined').map(args => args[0]);
  const examineEffects = lines.flatMap(line =>
    [...line.matchAll(/(?:^on [^ ]+(?: when .*)? |^when .* |^)examine ([A-Za-z_$][A-Za-z0-9_$]*)$/g)].map(m => m[1]));
  for (const caveat of [...new Set(qualifies.map(q => q.caveat))].sort()) {
    const broken = [];
    if (caveat === 'withdrawn') broken.push('it is withdrawal\'s built-in caveat');
    if (declaredQualifies.some(d => d.caveat === caveat)) broken.push('a declaration applies it');
    if (declaredQualifies.some(d => d.target === caveat)) broken.push('a declared caveat qualifies it');
    if (extras.includes(caveat)) broken.push('a qualified(...) names it');
    if (retained.includes(caveat)) broken.push('a commit retains it');
    if (examinedReads.includes(caveat) || examineEffects.includes(caveat)) broken.push('it is examined');
    const targets = qualifies.filter(q => q.caveat === caveat).map(q => q.evidence);
    for (const evidence of [...new Set(targets)].sort()) {
      const note = parameters.has(evidence)
        ? 'a procedure parameter'
        : templates.has(evidence) ? 'a reading stream template' : 'declared evidence';
      rows.push({ path, caveat, evidence, target: note, takeBackAllowed: broken.length === 0 && note !== 'a reading stream template', broken });
    }
  }
}

const programsWithQualify = new Set(rows.map(row => row.path));
const pairs = rows.length;
const allowed = rows.filter(row => row.takeBackAllowed).length;
const report = {
  note: 'Take-back 0.1 corpus sweep. Source text only; no program is run. See spec/caveat-take-back-0.1.md, "Corpus sweep".',
  revision: spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).stdout.trim(),
  programs: programs.length,
  programs_using_unqualify: wordUses,
  programs_with_qualify: programsWithQualify.size,
  qualify_pairs: pairs,
  pairs_a_take_back_could_name: allowed,
  rows,
};
// P4 adds the profile's own corpus programs; no other program may use the word.
const own = 'experiments/take-back-sweep/programs/';
assert.deepEqual(wordUses.filter(path => !path.startsWith(own)), [], 'a tracked program uses the word unqualify');
const text = JSON.stringify(report, null, 2) + '\n';
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), text);
console.log(`${programs.length} programs; unqualify used in ${wordUses.length}; ` +
  `${programsWithQualify.size} programs with qualify; ${pairs} (program, caveat, evidence) pairs; ` +
  `${allowed} a take-back could name`);
for (const row of rows.filter(r => !r.takeBackAllowed)) {
  console.log(`  not eligible: ${row.path} ${row.caveat} on ${row.evidence} (${row.target}): ${row.broken.join('; ') || 'stream template'}`);
}
