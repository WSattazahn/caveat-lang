// Corpus sweep for Untrusted Sources 0.1 (spec/caveat-untrusted-sources-0.1.md,
// "Corpus sweep"). The profile adds two forms that refuse to load today,
// `withdraw every NAME because EVIDENCE` and `retaining CAVEAT because once
// EVIDENCE`, and a kit report that adds no syntax. This reads every tracked
// Caveat program and reports:
//   - every `withdraw` effect, by form, and every `withdraw every NAME` (there
//     must be none: such a program does not load today);
//   - every retained caveat that names a reason, and every `because once E`
//     among them (there must be none);
//   - every declaration of a name `every` or `once`, whose meaning must not
//     change (`withdraw every because R` and `because once` with three words
//     keep today's reading);
//   - the programs that withdraw readings of one stream from more than one
//     rule, or by `occurrence(…)`: the hand-written form `every` replaces;
//   - every nested routed `for` block, and every event that names two members
//     of one kind: the pattern for one action on several sources;
//   - every `rests_on_withdrawn(revision(…))`: past revisions asked about in
//     the program, which the undo list reports for the host.
// It reads source text only; it runs no program. Comments and quoted text are
// removed before reading.
// Run from any directory: node experiments/untrusted-sources-sweep/sweep.mjs [--write]
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

const statements = text => text.split(/[;{}]/).map(s => s.replace(/\s+/g, ' ').trim()).filter(Boolean);

const withdraws = { named: 0, latest: 0, occurrence: 0, every: [] };
const byStream = new Map();
const reasons = [];
const once = [];
const names = [];
const nestedRouted = [];
const twoMembers = [];
const pastRevisions = [];
for (const file of programs) {
  const text = code(readFileSync(join(root, file), 'utf8'));
  for (const match of text.matchAll(/\b(?:evidence|readings|claim|caveat|state|decisions|entity|place|event|renewable)\s+(every|once)\b/g)) {
    names.push({ file, name: match[1] });
  }
  for (const statement of statements(text)) {
    const withdraw = statement.match(/(?:^|\s)withdraw\s+(.+?)\s+because\s+\S+$/);
    if (withdraw) {
      const target = withdraw[1];
      if (/^every\s+\S+$/.test(target)) withdraws.every.push({ file, statement });
      else if (/^latest\s*\(/.test(target)) {
        withdraws.latest += 1;
        const stream = target.replace(/^latest\s*\(\s*|\s*\)$/g, '');
        const key = `${file} ${stream}`;
        byStream.set(key, (byStream.get(key) ?? 0) + 1);
      } else if (/^occurrence\s*\(/.test(target)) withdraws.occurrence += 1;
      else withdraws.named += 1;
    }
    const retaining = statement.match(/(?:^|\s)commit\s+\S+\s+because\s+\S+.*?\sretaining\s+(.+)$/);
    if (retaining) {
      for (const item of retaining[1].split(',').map(part => part.trim().split(/\s+/))) {
        if (item[1] !== 'because') continue;
        reasons.push({ file, caveat: item[0], reason: item.slice(2).join(' ') });
        if (item.length === 4 && item[2] === 'once') once.push({ file, statement });
      }
    }
    if (/\brests_on_withdrawn\s*\(\s*revision\s*\(/.test(statement)) pastRevisions.push({ file });
  }
  // A routed `for` whose body holds another routed `for`.
  const headers = [...text.matchAll(/\bfor\s+\w+\s+as\s+\$\w+(\s+routed\s+by\s+\w+)?\s*\{/g)];
  const stack = [];
  let at = 0;
  for (let i = 0; i < text.length; i += 1) {
    const header = headers[at];
    if (header && header.index + header[0].length - 1 === i) {
      if (header[1] && stack.some(Boolean)) nestedRouted.push({ file, header: header[0].replace(/\s+/g, ' ').replace(/ ?\{$/, '') });
      stack.push(Boolean(header[1]));
      at += 1;
    } else if (text[i] === '{') stack.push(null);
    else if (text[i] === '}') stack.pop();
  }
  for (const match of text.matchAll(/\bevent\s+(\w+)\s+([^;]+);/g)) {
    const kinds = [...match[2].matchAll(/\b(\w+)\s+kind\s+(\w+)/g)].map(([, , kind]) => kind);
    if (kinds.length !== new Set(kinds).size) twoMembers.push({ file, event: match[1] });
  }
}

const repeated = [...byStream].filter(([, count]) => count > 1).map(([key, count]) => {
  const [file, stream] = key.split(' ');
  return { file, stream, rules: count };
});
assert.equal(withdraws.every.length, 0, 'a tracked program writes `withdraw every`, which does not load today');
assert.equal(once.length, 0, 'a tracked program writes `because once`, which does not load today');

const results = {
  programs: programs.length,
  withdraw_effects: {
    named: withdraws.named, latest: withdraws.latest, occurrence: withdraws.occurrence, every: withdraws.every,
  },
  programs_with_withdraw: null,
  streams_withdrawn_by_several_rules: repeated,
  retention_reasons: reasons.length,
  because_once: once,
  names_every_or_once: names,
  nested_routed_blocks: nestedRouted,
  events_naming_two_members_of_one_kind: twoMembers,
  past_revision_questions: pastRevisions.length,
  programs_asking_about_past_revisions: new Set(pastRevisions.map(entry => entry.file)).size,
};
results.programs_with_withdraw = programs.filter(file => /(?:^|[\s;{}])withdraw\s/.test(code(readFileSync(join(root, file), 'utf8')))).length;
const json = `${JSON.stringify(results, null, 2)}\n`;
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), json);
process.stdout.write(json);
