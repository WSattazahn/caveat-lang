// Corpus sweep for Retention Reasons 0.1 (spec/caveat-retention-reasons-0.1.md,
// "Corpus sweep"). The profile lets each caveat in a commit's `retaining`
// clause name its reason, `retaining C because E`, a form that refuses to load
// today. This reads every tracked Caveat program and reports:
//   - every commit's `retaining` clause, and every one that holds `because`
//     (there must be none: such a program does not load today);
//   - the caveats retained, and how many commits retain more than one;
//   - the retaining commits whose `using` cites evidence directly with
//     `qualified(…, EVIDENCE)`: the workaround the profile replaces, where the
//     reason was put into the grounds by hand.
// It reads source text only; it runs no program. Comments and text literals
// are removed before reading.
// Run from any directory: node experiments/retention-reasons-sweep/sweep.mjs [--write]
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
const COMMIT = /(?:^|\s)commit ([A-Za-z_$][A-Za-z0-9_$]*) because \S+(?: (.*))?$/;

const withBecause = [];
const retainingCommits = [];
const workaround = [];
let commits = 0;
for (const path of programs) {
  const text = code(readFileSync(join(root, path), 'utf8'));
  for (const line of statements(text)) {
    const m = line.match(COMMIT);
    if (!m) continue;
    commits += 1;
    const rest = m[2] ?? '';
    const at = rest.search(/(?:^|\s)retaining\s/);
    if (at < 0) continue;
    const clause = rest.slice(at).replace(/^\s*retaining\s+/, '');
    const caveats = clause.split(',').map(item => item.trim());
    const row = { path, action: m[1], retained: caveats };
    retainingCommits.push(row);
    if (/\sbecause\s/.test(` ${clause} `)) withBecause.push(row);
    if (/(?:^|\s)using\s.*qualified\(/.test(rest.slice(0, at))) workaround.push(row);
  }
}

const report = {
  note: 'Retention Reasons 0.1 corpus sweep. Source text only; no program is run. See spec/caveat-retention-reasons-0.1.md, "Corpus sweep".',
  revision: spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).stdout.trim(),
  programs: programs.length,
  commits,
  retaining_commits: retainingCommits.length,
  programs_with_retaining: new Set(retainingCommits.map(row => row.path)).size,
  retaining_with_because: withBecause,
  retaining_more_than_one: retainingCommits.filter(row => row.retained.length > 1).length,
  retaining_commits_using_qualified_evidence: workaround,
  retaining: retainingCommits,
};
assert.deepEqual(withBecause, [], 'a tracked program writes because inside retaining');
const out = JSON.stringify(report, null, 2) + '\n';
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), out);
console.log(`${programs.length} programs; ${commits} commits; ${retainingCommits.length} retain, in ${report.programs_with_retaining} programs; ` +
  `${withBecause.length} with because; ${report.retaining_more_than_one} retain more than one caveat; ` +
  `${workaround.length} cite evidence with qualified(…) in using`);
