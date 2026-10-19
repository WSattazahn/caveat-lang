// Corpus sweep for Nested Repetition 0.1 (spec/caveat-nested-repetition-0.1.md,
// "Corpus sweep"). The profile lets a `for` block's body hold `for` blocks,
// which Repetition 0.1 refused to load. This reads every tracked Caveat
// program and reports:
//   - every `for` block, and every one nested in another (there must be none:
//     such a program does not load today);
//   - the hand-written form of a nested block: statements in a `for` block's
//     body that are the same apart from the name of a member of another kind,
//     written once per member of that kind; and statements anywhere that are
//     the same apart from the names of a member of each of two kinds, written
//     once per pair. Top-level statements written once per member of one
//     kind, which a single block covers, are counted too, for comparison.
// It reads source text only; it runs no program. Comments are removed before
// reading; quoted text is kept, since a block substitutes inside it.
// Run from any directory: node experiments/nested-repetition-sweep/sweep.mjs [--write]
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

// `#` or `//` comments carry no source words; quoted text is kept.
function code(text) {
  let out = '';
  let inString = false;
  for (let i = 0; i < text.length; i += 1) {
    const c = text[i];
    if (inString) {
      out += c;
      if (c === '\\') { out += text[i + 1] ?? ''; i += 1; continue; }
      if (c === '"') inString = false;
      continue;
    }
    if (c === '"') { inString = true; out += c; continue; }
    if (c === '#' || (c === '/' && text[i + 1] === '/')) {
      while (i < text.length && text[i] !== '\n') i += 1;
      out += '\n';
      continue;
    }
    out += c;
  }
  return out;
}

const squeeze = text => text.replace(/\s+/g, ' ').trim();

// Top-level statements, with each `for` block kept whole: a block's body is
// the text between its first `{` and the `}` that closes it.
function topLevel(text) {
  const out = [];
  let start = 0;
  let depth = 0;
  let inString = false;
  for (let i = 0; i < text.length; i += 1) {
    const c = text[i];
    if (inString) {
      if (c === '\\') i += 1;
      else if (c === '"') inString = false;
      continue;
    }
    if (c === '"') inString = true;
    else if (c === '{') depth += 1;
    else if (c === '}') depth -= 1;
    else if (c === ';' && depth === 0) {
      out.push(text.slice(start, i));
      start = i + 1;
    }
  }
  out.push(text.slice(start));
  return out.map(squeeze).filter(Boolean);
}

const BLOCK = /^for (\S+) as \$(\S+)(?: routed by \S+)? \{(.*)\}$/;
const ENTITY = /^entity (\S+) kind (\S+) at \S+$/;

const blocks = [];
const nested = [];
const candidates = [];
const pairs = [];
const escape = name => name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const wordOf = member => new RegExp(`(?<![A-Za-z0-9])${escape(member)}(?![A-Za-z0-9])`, 'g');
for (const path of programs) {
  const statements = topLevel(code(readFileSync(join(root, path), 'utf8')));
  const kinds = new Map();
  for (const statement of statements) {
    const m = statement.match(ENTITY);
    if (m) kinds.set(m[2], [...(kinds.get(m[2]) ?? []), m[1]]);
  }
  // A statement written once per member of a kind: the same text with that
  // member's name, as a whole word or a word part split by `_` or `.`, put
  // for `<KIND>`.
  const repeats = (scope, list, block) => {
    for (const [kind, members] of kinds) {
      if (members.length < 2 || kind === block?.kind) continue;
      const forms = new Map();
      for (const statement of list) {
        for (const member of members) {
          const word = wordOf(member);
          if (!word.test(statement)) continue;
          const form = statement.replace(word, `<${kind}>`);
          forms.set(form, new Set([...(forms.get(form) ?? []), member]));
        }
      }
      for (const [form, written] of forms) {
        if (written.size >= 2) {
          candidates.push({ path, scope, kind, members: [...written], form });
        }
      }
    }
  };
  // A statement written once per pair of members of two kinds: both names
  // put for `<KIND>`.
  const crossed = (scope, list) => {
    const named = [...kinds].filter(([, members]) => members.length >= 2);
    for (const [first, firstMembers] of named) {
      for (const [second, secondMembers] of named) {
        if (first >= second) continue;
        const forms = new Map();
        for (const statement of list) {
          for (const a of firstMembers) {
            const aWord = wordOf(a);
            if (!aWord.test(statement)) continue;
            const once = statement.replace(aWord, `<${first}>`);
            for (const b of secondMembers) {
              const bWord = wordOf(b);
              if (!bWord.test(once)) continue;
              const form = once.replace(bWord, `<${second}>`);
              const entry = forms.get(form) ?? { a: new Set(), b: new Set(), count: 0 };
              entry.a.add(a); entry.b.add(b); entry.count += 1;
              forms.set(form, entry);
            }
          }
        }
        for (const [form, entry] of forms) {
          if (entry.a.size >= 2 && entry.b.size >= 2) {
            pairs.push({ path, scope, kinds: [first, second], statements: entry.count, form: form.slice(0, 160) });
          }
        }
      }
    }
  };
  crossed('top level', statements.filter(statement => !BLOCK.test(statement)));
  for (const statement of statements) {
    const m = statement.match(BLOCK);
    if (!m) continue;
    const block = { path, kind: m[1], binding: `$${m[2]}` };
    blocks.push(block);
    const body = topLevel(m[3]);
    for (const inner of body) {
      if (/^for /.test(inner)) nested.push({ ...block, nested: inner.slice(0, 60) });
    }
    repeats(`for ${block.kind} as ${block.binding}`, body, block);
  }
  repeats('top level', statements.filter(statement => !BLOCK.test(statement)), null);
}

// One row per block or top level and kind: how many statement forms it
// writes once per member.
const summary = new Map();
for (const row of candidates) {
  const key = `${row.path}\t${row.scope}\t${row.kind}`;
  const entry = summary.get(key) ?? { path: row.path, scope: row.scope, kind: row.kind, forms: 0, members: new Set() };
  entry.forms += 1;
  row.members.forEach(member => entry.members.add(member));
  summary.set(key, entry);
}
const repeated = [...summary.values()]
  .map(entry => ({ ...entry, members: [...entry.members] }))
  .sort((a, b) => b.forms - a.forms || a.path.localeCompare(b.path));

const report = {
  note: 'Nested Repetition 0.1 corpus sweep. Source text only; no program is run. See spec/caveat-nested-repetition-0.1.md, "Corpus sweep".',
  revision: spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).stdout.trim(),
  programs: programs.length,
  for_blocks: blocks.length,
  programs_with_for_blocks: new Set(blocks.map(row => row.path)).size,
  nested_blocks: nested,
  written_once_per_pair_of_members: pairs,
  written_once_per_member: {
    in_a_for_block: repeated.filter(entry => entry.scope !== 'top level'),
    at_the_top_level: repeated.filter(entry => entry.scope === 'top level'),
  },
};
assert.deepEqual(nested, [], 'a tracked program nests a for block');
const out = JSON.stringify(report, null, 2) + '\n';
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), out);
const inBlocks = report.written_once_per_member.in_a_for_block;
const top = report.written_once_per_member.at_the_top_level;
console.log(`${programs.length} programs; ${blocks.length} for blocks in ${report.programs_with_for_blocks} programs; ` +
  `${nested.length} nested; ${pairs.length} statement forms written once per pair of members of two kinds; ${inBlocks.length} blocks write ${inBlocks.reduce((n, e) => n + e.forms, 0)} statement forms once per member of another kind; ` +
  `${top.length} top-level groups write ${top.reduce((n, e) => n + e.forms, 0)}`);
