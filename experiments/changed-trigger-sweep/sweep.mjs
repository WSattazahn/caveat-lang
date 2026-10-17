// Corpus sweep for Changed Triggers 0.1 (spec/caveat-changed-triggers-0.1.md,
// "Corpus sweep"). The profile adds `reopened by changed STREAM`, a trigger
// that fires only on a reading whose value differs from the stream's previous
// reading, a form that refuses to load today. This reads every tracked Caveat
// program and reports:
//   - every `reopened by` trigger, and every one written `changed STREAM`
//     (there must be none: such a program does not load today);
//   - authored reopening rules that compare a stream's latest reading with an
//     earlier one, the hand-written form of the new trigger.
// It reads source text only; it runs no program. Comments and text literals
// are removed before reading.
// Run from any directory: node experiments/changed-trigger-sweep/sweep.mjs [--write]
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
const DECISIONS = new RegExp(`^decisions (${NAME}) limit \\S+ reopened by (.+)$`);

const triggers = [];
const changed = [];
const authored = [];
for (const path of programs) {
  const text = code(readFileSync(join(root, path), 'utf8'));
  const lines = statements(text);
  for (const line of lines) {
    const m = line.match(DECISIONS);
    if (m) {
      for (const item of m[2].split(',').map(part => part.trim())) {
        const words = item.split(' ');
        const row = { path, series: m[1], trigger: item };
        triggers.push(row);
        if (words[0] === 'changed' && words.length > 1) changed.push(row);
      }
    }
    if (/(?:^|\s)reopen\s/.test(line) && /latest\([^)]*\)\s*!=\s*history_at\(|history_at\([^)]*\)\s*!=\s*latest\(/.test(line)) {
      authored.push({ path, rule: line });
    }
  }
}

const report = {
  note: 'Changed Triggers 0.1 corpus sweep. Source text only; no program is run. See spec/caveat-changed-triggers-0.1.md, "Corpus sweep".',
  revision: spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).stdout.trim(),
  programs: programs.length,
  triggers: triggers.length,
  programs_with_triggers: new Set(triggers.map(row => row.path)).size,
  changed_triggers: changed,
  authored_reopenings_comparing_readings: authored,
  all_triggers: triggers,
};
assert.deepEqual(changed, [], 'a tracked program writes reopened by changed STREAM');
const out = JSON.stringify(report, null, 2) + '\n';
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), out);
console.log(`${programs.length} programs; ${triggers.length} triggers in ${report.programs_with_triggers} programs; ` +
  `${changed.length} changed; ` +
  `${authored.length} authored reopenings compare readings`);
