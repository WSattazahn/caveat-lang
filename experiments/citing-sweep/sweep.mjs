// Corpus sweep for Citing 0.1 (spec/caveat-citing-0.1.md, "Corpus sweep").
// The profile adds one commit clause, `citing CITATION, …`, in a position that
// refuses to load today. This reads every tracked Caveat program and reports:
//   - every use of the word `citing`, and every commit that writes `citing`
//     right after its reason (there must be none: such a program does not load
//     today);
//   - every commit, split by whether it has `using`;
//   - the `using` commits whose decision value no `latest(…)` or
//     `history_at(…)` in the same program reads: the commits whose value only
//     a reader of `explain` or a view ever sees, which `citing` could replace
//     if the value means nothing.
// It reads source text only; it runs no program. Comments and text literals
// are removed before reading.
// Run from any directory: node experiments/citing-sweep/sweep.mjs [--write]
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
const statements = text => text.split(/[;{}]/).map(s => s.replace(/\s+/g, ' ').trim()).filter(Boolean);
const COMMIT = /(?:^|\s)commit ([A-Za-z_$][A-Za-z0-9_$]*) because (\S+)(?: (.*))?$/;

const wordUses = [];
const citingCommits = [];
const unread = [];
let commits = 0;
let usingCommits = 0;
let bareCommits = 0;
const programsWithCommits = new Set();
for (const path of programs) {
  const text = code(readFileSync(join(root, path), 'utf8'));
  if ((text.match(WORD) ?? []).includes('citing')) wordUses.push(path);
  const lines = statements(text);
  for (const line of lines) {
    const m = line.match(COMMIT);
    if (!m) continue;
    const [, action, , rest = ''] = m;
    commits += 1;
    programsWithCommits.add(path);
    if (/^citing\b/.test(rest)) citingCommits.push({ path, line });
    if (/^using\b/.test(rest)) {
      usingCommits += 1;
      const read = new RegExp(`(?<![A-Za-z0-9_$])(?:latest|history_at)\\(\\s*${action.replace(/\$/g, '\\$')}\\s*[,)]`);
      if (!read.test(text)) unread.push({ path, action });
    } else {
      bareCommits += 1;
    }
  }
}

const unreadKeys = [...new Set(unread.map(row => `${row.path}\t${row.action}`))].sort();
const report = {
  note: 'Citing 0.1 corpus sweep. Source text only; no program is run. See spec/caveat-citing-0.1.md, "Corpus sweep".',
  revision: spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).stdout.trim(),
  programs: programs.length,
  programs_using_the_word_citing: wordUses,
  commits_writing_citing: citingCommits,
  programs_with_commits: programsWithCommits.size,
  commits,
  commits_with_using: usingCommits,
  commits_without_using: bareCommits,
  using_commits_whose_value_no_expression_reads: unread.length,
  decisions_with_unread_using_values: unreadKeys.map(key => { const [path, action] = key.split('\t'); return { path, action }; }),
};
assert.deepEqual(citingCommits, [], 'a tracked program writes citing after a commit reason');
const out = JSON.stringify(report, null, 2) + '\n';
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), out);
console.log(`${programs.length} programs; the word citing in ${wordUses.length}; ${citingCommits.length} commits write citing; ` +
  `${commits} commits in ${programsWithCommits.size} programs: ${usingCommits} with using, ${bareCommits} without; ` +
  `${unread.length} using commits (${unreadKeys.length} decisions) whose value no latest or history_at reads`);
