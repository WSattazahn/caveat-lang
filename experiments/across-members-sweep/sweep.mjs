// Corpus sweep for Across Members 0.1 (spec/caveat-across-members-0.1.md,
// "Corpus sweep"). The profile adds `sum(for KIND as $NAME: E)`, `any(…)` and
// `all(…)`, which refuse to load today: `for` is no expression. This reads
// every tracked Caveat program and reports:
//   - every `sum(for`, `any(for` and `all(for` outside comments and quoted
//     text: none but in the intel example, which the profile's change
//     rewrites with them, so every other program expands to itself;
//   - every declaration of a name `sum`, `any` or `all`, and every call of one
//     that is not an across form: their meaning must not change;
//   - every hand-written sum of defines that differ only by a member of one
//     kind, such as `north_ocelot_ok + north_eva_ok`: the form it replaces.
// It reads source text only; it runs no program.
// Run from any directory: node experiments/across-members-sweep/sweep.mjs [--write]
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

const forms = [];
const names = [];
const calls = [];
const handSums = [];
for (const file of programs) {
  const source = readFileSync(join(root, file), 'utf8');
  const text = code(source);
  for (const match of text.matchAll(/(?<![\w$.:])(sum|any|all)\s*\(\s*for\s/g)) forms.push({ file, form: match[1] });
  for (const match of text.matchAll(/\b(?:evidence|readings|claim|caveat|state|decisions|entity|place|event|renewable|define|proc|kind)\s+(sum|any|all)\b/g)) {
    names.push({ file, name: match[1] });
  }
  for (const match of text.matchAll(/(?<![\w$.:])(sum|any|all)\s*\((?!\s*for\s)/g)) calls.push({ file, name: match[1] });
  // Members of each kind, as top-level entity statements declare them.
  const kinds = new Map();
  for (const match of text.matchAll(/(?:^|;)\s*entity\s+(\S+)\s+kind\s+(\S+)\s+at\s+\S+\s*(?=;|$)/gm)) {
    if (!kinds.has(match[2])) kinds.set(match[2], []);
    kinds.get(match[2]).push(match[1]);
  }
  for (const [kind, members] of kinds) {
    if (members.length < 2) continue;
    for (const match of text.matchAll(/[A-Za-z_][\w]*(?:\s*\+\s*[A-Za-z_][\w]*)+/g)) {
      const terms = match[0].split(/\s*\+\s*/);
      if (terms.length < 2) continue;
      const shapes = terms.map(term => {
        const member = members.find(name => term.includes(name));
        return member ? term.replace(member, '\0') : null;
      });
      if (shapes.every(shape => shape !== null && shape === shapes[0])) handSums.push({ file, kind, sum: match[0].replace(/\s+/g, ' ') });
    }
  }
}

const results = {
  schema: 'caveat.across-members-sweep/1',
  programs: programs.length,
  across_forms: forms,
  names_sum_any_all: names,
  other_calls_sum_any_all: calls,
  hand_written_member_sums: handSums,
};
// The intel example is rewritten with the form by the change that adds it;
// no other program may use one.
const adopters = ['kit/examples/intel-betrayal/intel.cav'];
assert.deepEqual(forms.filter(({ file }) => !adopters.includes(file)), [], 'no other tracked program may use an across form');
const out = JSON.stringify(results, null, 2) + '\n';
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), out);
console.log(`${programs.length} programs; across forms: ${forms.length}; names sum/any/all: ${names.length}; other calls: ${calls.length}; hand-written member sums: ${handSums.length}`);
for (const { file, kind, sum } of handSums) console.log(`  ${file} (${kind}): ${sum}`);
