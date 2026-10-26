// Corpus sweep for Strict Irreversible 0.1
// (spec/caveat-strict-irreversible-0.1.md, "Corpus sweep"). The profile adds
// `strict` straight after `irreversible` in a `decisions` declaration, a form
// that refuses to load today. This reads every tracked Caveat program and
// reports:
//   - every irreversible series, and every one written `irreversible strict`
//     (there must be none: such a program does not load today);
//   - declared names spelled `strict`, which stay legal names because the
//     word is read only straight after `irreversible`;
//   - each rule that commits an irreversible series, and whether it has a
//     `when` guard or a `permitted by` grant: the control `strict` would read
//     beyond the grounds. Whether a guard reads caveated evidence depends on
//     the events, so these are places to look, not changes.
// It reads source text only; it runs no program. Comments and text literals
// are removed before reading.
// Run from any directory: node experiments/strict-irreversible-sweep/sweep.mjs [--write]
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
const DECLARED = new RegExp(`^(?:claim|evidence|caveat|readings|decisions|event|state|fn|proc|grant|renewable|identifiers) (${NAME})\\b`);
const ON = new RegExp(`^on (${NAME})\\b(.*)$`);
const COMMIT = new RegExp(`(?:^|\\s)commit (${NAME})\\b`);

const irreversible = [];
const strict = [];
const named = [];
const commits = [];
for (const path of programs) {
  const lines = statements(code(readFileSync(join(root, path), 'utf8')));
  const series = new Set();
  for (const line of lines) {
    const declared = line.match(DECLARED);
    if (declared && declared[1] === 'strict') named.push({ path, statement: line.split(' ')[0] });
    const d = line.match(DECISIONS);
    if (d && /^\s*irreversible\b/.test(d[3])) {
      series.add(d[1]);
      irreversible.push({ path, series: d[1] });
      if (/^\s*irreversible strict\b/.test(d[3])) strict.push({ path, series: d[1] });
    }
  }
  for (const line of lines) {
    const on = line.match(ON);
    const commit = on?.[2].match(COMMIT);
    if (!commit || !series.has(commit[1])) continue;
    // What `strict` would read beyond the grounds: a `when` guard before the
    // commit, and a `permitted by` grant.
    const before = on[2].slice(0, commit.index);
    commits.push({
      path, event: on[1], series: commit[1],
      guard: /(?:^|\s)when\s/.test(before),
      permitted: /(?:^|\s)permitted by\s/.test(on[2]),
      retaining: /(?:^|\s)retaining\s/.test(on[2]),
    });
  }
}

const report = {
  note: 'Strict Irreversible 0.1 corpus sweep. Source text only; no program is run. See spec/caveat-strict-irreversible-0.1.md, "Corpus sweep".',
  revision: spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).stdout.trim(),
  programs: programs.length,
  irreversible_series: irreversible,
  strict_series: strict,
  names_spelled_strict: named,
  irreversible_commit_rules: commits,
  rules_strict_would_read_more_of: commits.filter(row => row.guard || row.permitted).length,
};
assert.deepEqual(strict, [], 'a tracked program already writes irreversible strict');
const out = JSON.stringify(report, null, 2) + '\n';
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), out);
console.log(`${programs.length} programs; ${irreversible.length} irreversible series; ${strict.length} strict; ` +
  `${named.length} names spelled strict; ${commits.length} irreversible commit rules, ` +
  `${report.rules_strict_would_read_more_of} with a guard or a permission grant`);
