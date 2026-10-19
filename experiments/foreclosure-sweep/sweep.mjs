// Corpus sweep for Foreclosure 0.1 (spec/caveat-foreclosure-0.1.md, "Corpus
// sweep"). The profile adds `irreversible` and `forecloses SERIES, …` to a
// `decisions` declaration, a form that refuses to load today. This reads every
// tracked Caveat program and reports:
//   - every `decisions` declaration, and every one written with the new words
//     (there must be none: such a program does not load today);
//   - declared names spelled `irreversible` or `forecloses`, which stay legal
//     names because the words are read only after `limit N`;
//   - series declared `limit 1` and the reopening paths written for them
//     (`reopen NAME` or a `reopened by` trigger): a limit-1 series with none
//     is a candidate for `irreversible`, one with a path would not load if
//     declared irreversible;
//   - the convention the profile replaces: a rule that rejects an event when
//     another decision is committed, on an event whose rules commit a
//     different decision (foreclose.cav's `roll_back`), and a commit guarded by
//     `not committed(OTHER)`.
// Source text cannot say whether an action can be undone in the world, so the
// candidates are candidates, not mistakes. Reopenings inside a `proc` that
// takes a `decisions` parameter are counted only when they name the series.
// It reads source text only; it runs no program. Comments and text literals
// are removed before reading.
// Run from any directory: node experiments/foreclosure-sweep/sweep.mjs [--write]
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

const statements = text => text.split(/[;{}]/).map(s => s.replace(/\s+/g, ' ').trim()).filter(Boolean);
const NAME = '[A-Za-z_$][A-Za-z0-9_$]*';
const DECISIONS = new RegExp(`^decisions (${NAME}) limit (\\S+)(.*)$`);
const DECLARED = new RegExp(`^(?:claim|evidence|caveat|readings|decisions|event|state|fn|proc|grant) (${NAME})\\b`);
const ON = new RegExp(`^on (${NAME})\\b(.*)$`);
const COMMIT = new RegExp(`(?:^|\\s)commit (${NAME})\\b`);
const REOPEN = new RegExp(`(?:^|\\s)reopen (${NAME})\\b`, 'g');
const COMMITTED = new RegExp(`(not )?committed\\((${NAME})\\)`, 'g');
const NEW_WORDS = /(?:^|\s)(?:irreversible|forecloses)(?:\s|$)/;

const declarations = [];
const newSyntax = [];
const reservedLooking = [];
const reopened = new Map();
const rejectsOnCommitted = [];
const guardedCommits = [];
for (const path of programs) {
  const text = code(readFileSync(join(root, path), 'utf8'));
  const lines = statements(text);
  const series = new Map();
  const commitsByEvent = new Map();
  const rejects = [];
  for (const line of lines) {
    const named = line.match(DECLARED);
    if (named && (named[1] === 'irreversible' || named[1] === 'forecloses')) reservedLooking.push({ path, name: named[1] });
    const d = line.match(DECISIONS);
    if (d) {
      series.set(d[1], d[2]);
      declarations.push({ path, series: d[1], limit: d[2] });
      if (NEW_WORDS.test(d[3])) newSyntax.push({ path, series: d[1] });
      if (/(?:^|\s)reopened by\s/.test(d[3])) reopened.set(`${path}\0${d[1]}`, (reopened.get(`${path}\0${d[1]}`) ?? 0) + 1);
    }
    for (const r of line.matchAll(REOPEN)) reopened.set(`${path}\0${r[1]}`, (reopened.get(`${path}\0${r[1]}`) ?? 0) + 1);
    const on = line.match(ON);
    if (!on) continue;
    const committed = [...on[2].matchAll(COMMITTED)];
    const commit = on[2].match(COMMIT);
    if (commit) {
      if (!commitsByEvent.has(on[1])) commitsByEvent.set(on[1], new Set());
      commitsByEvent.get(on[1]).add(commit[1]);
      for (const c of committed) if (c[1] && c[2] !== commit[1]) guardedCommits.push({ path, event: on[1], commit: commit[1], unless: c[2] });
    }
    if (/(?:^|\s)reject(?:\s|$)/.test(on[2])) {
      for (const c of committed) if (!c[1]) rejects.push({ event: on[1], when: c[2] });
    }
  }
  for (const r of rejects) {
    for (const target of commitsByEvent.get(r.event) ?? []) {
      if (target !== r.when) rejectsOnCommitted.push({ path, event: r.event, rejects_commit_of: target, while_committed: r.when });
    }
  }
}

const limitOne = declarations.filter(row => row.limit === '1');
const withPath = limitOne.filter(row => reopened.has(`${row.path}\0${row.series}`));
const withoutPath = limitOne.filter(row => !reopened.has(`${row.path}\0${row.series}`));
const report = {
  note: 'Foreclosure 0.1 corpus sweep. Source text only; no program is run. See spec/caveat-foreclosure-0.1.md, "Corpus sweep".',
  revision: spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).stdout.trim(),
  programs: programs.length,
  decisions_declarations: declarations.length,
  programs_with_decisions: new Set(declarations.map(row => row.path)).size,
  new_syntax: newSyntax,
  names_spelled_like_the_new_words: reservedLooking,
  limit_one_series: limitOne.length,
  limit_one_without_reopening_path: withoutPath.map(({ path, series }) => ({ path, series })),
  limit_one_with_reopening_path: withPath.map(({ path, series }) => ({ path, series })),
  rejects_while_another_is_committed: rejectsOnCommitted,
  commits_guarded_by_not_committed_other: guardedCommits,
};
assert.deepEqual(newSyntax, [], 'a tracked program already writes irreversible or forecloses in a decisions declaration');
const out = JSON.stringify(report, null, 2) + '\n';
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), out);
console.log(`${programs.length} programs; ${declarations.length} decisions declarations in ${report.programs_with_decisions} programs; ` +
  `${newSyntax.length} with the new words; ${reservedLooking.length} names spelled like them; ` +
  `${limitOne.length} limit-1 series, ${withoutPath.length} with no reopening path; ` +
  `${rejectsOnCommitted.length} rejects while another decision is committed; ${guardedCommits.length} commits guarded by not committed(OTHER)`);
