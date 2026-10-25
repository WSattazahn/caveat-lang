// Corpus sweep for Foreclosure 0.2 (spec/caveat-foreclosure-0.2.md, "Corpus
// sweep"). The profile lets `forecloses ACTION, …` follow `limit N` without
// `irreversible`, a declaration that refuses to load today. This reads every
// tracked Caveat program and reports:
//   - every `decisions` declaration written with `forecloses` and without
//     `irreversible` (there must be none: such a program does not load today);
//   - declared names spelled `forecloses`, which stay legal names because the
//     word is read only after `limit N` or `irreversible`;
//   - the convention the profile replaces: a rule that rejects an event while
//     another decision is in force (`committed(D) and not reopened(D)`), on an
//     event whose rules commit a different decision.
// It reads source text only; it runs no program. Comments and text literals
// are removed before reading. Files named on the command line are read
// instead of the corpus, and `--write` is ignored then.
// Run from any directory: node experiments/reversible-foreclosure-sweep/sweep.mjs [--write]
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

const statements = text => text.split(/[;{}]/).map(s => s.replace(/\s+/g, ' ').trim()).filter(Boolean);
const NAME = '[A-Za-z_$][A-Za-z0-9_$]*';
const DECISIONS = new RegExp(`^decisions (${NAME}) limit (\\S+)(.*)$`);
const DECLARED = new RegExp(`^(?:claim|evidence|caveat|readings|decisions|event|state|fn|proc|grant) (${NAME})\\b`);
const ON = new RegExp(`^on (${NAME})\\b(.*)$`);
const COMMIT = new RegExp(`(?:^|\\s)commit (${NAME})\\b`);
const IN_FORCE = new RegExp(`(?:^|[\\s(])committed\\((${NAME})\\) and not reopened\\(\\1\\)`, 'g');

const declarations = [];
const newSyntax = [];
const spelled = [];
const rejectsWhileInForce = [];
for (const path of programs) {
  const text = code(readFileSync(resolve(root, path), 'utf8'));
  const commitsByEvent = new Map();
  const rejects = [];
  for (const line of statements(text)) {
    const named = line.match(DECLARED);
    if (named && named[1] === 'forecloses') spelled.push({ path, name: named[1] });
    const d = line.match(DECISIONS);
    if (d) {
      declarations.push({ path, series: d[1] });
      const words = d[3].trim().split(' ');
      if (words[0] === 'forecloses') newSyntax.push({ path, series: d[1] });
    }
    const on = line.match(ON);
    if (!on) continue;
    const commit = on[2].match(COMMIT);
    if (commit) {
      if (!commitsByEvent.has(on[1])) commitsByEvent.set(on[1], new Set());
      commitsByEvent.get(on[1]).add(commit[1]);
    }
    if (/(?:^|\s)reject(?:\s|$)/.test(on[2])) {
      for (const match of on[2].matchAll(IN_FORCE)) rejects.push({ event: on[1], while_in_force: match[1] });
    }
  }
  for (const r of rejects) {
    for (const target of commitsByEvent.get(r.event) ?? []) {
      if (target !== r.while_in_force) rejectsWhileInForce.push({ path, event: r.event, rejects_commit_of: target, while_in_force: r.while_in_force });
    }
  }
}

const results = {
  programs: programs.length,
  decisions_declarations: declarations.length,
  programs_with_decisions: new Set(declarations.map(row => row.path)).size,
  new_syntax: newSyntax,
  names_spelled_forecloses: spelled,
  rejects_while_another_is_in_force: rejectsWhileInForce,
};
const json = `${JSON.stringify(results, null, 2)}\n`;
if (process.argv.includes('--write') && extra.length === 0) writeFileSync(join(here, 'RESULTS.json'), json);
process.stdout.write(json);
