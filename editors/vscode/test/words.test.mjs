// The grammar's words against the runtime's own lists of them.
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { test } from 'node:test';
import { expectedHeadScope, groups, notSourceWords } from './expectations.mjs';
import { statementHeads } from './reference.mjs';
import { has, scopesOf } from './tokenize.mjs';

const runtime = new URL('../../../runtime/', import.meta.url);
const read = path => readFileSync(new URL(path, runtime), 'utf8');

// `const NAME: &[&str] = &[ "a", "b", ... ];`
function rustList(source, name) {
  const start = source.indexOf(`const ${name}: &[&str] = &[`);
  assert.ok(start >= 0, `${name} not found`);
  const body = source.slice(start, source.indexOf('];', start));
  return [...body.replace(/\/\/[^\n]*/g, '').matchAll(/"([a-z0-9_]+)"/g)].map(match => match[1]);
}

const link = read('src/link.rs');
const modules = read('tests/modules.rs');
const lists = {
  'link.rs RESERVED': rustList(link, 'RESERVED'),
  'link.rs CALLABLE': rustList(link, 'CALLABLE'),
  'tests/modules.rs POSITIONAL': rustList(modules, 'POSITIONAL'),
  'tests/modules.rs OUTSIDE_MODULES': rustList(modules, 'OUTSIDE_MODULES'),
};

test('the runtime lists were read', () => {
  // A reformatted list would read as empty and account for nothing.
  assert.ok(lists['link.rs RESERVED'].length >= 90);
  assert.ok(lists['link.rs CALLABLE'].length >= 15);
  assert.deepEqual([...lists['tests/modules.rs POSITIONAL']].sort(), ['every', 'integer', 'journal', 'max', 'min', 'reset', 'window']);
  assert.ok(lists['tests/modules.rs OUTSIDE_MODULES'].length >= 15);
});

test('every word the runtime lists is accounted for', () => {
  const known = new Set([...groups.flatMap(group => group.words), ...notSourceWords.words]);
  for (const [list, words] of Object.entries(lists)) {
    const missing = words.filter(word => !known.has(word));
    assert.deepEqual(missing, [], `${list} has words the grammar does not account for`);
  }
});

test('every highlighted word is one the runtime reads', () => {
  const sources = readdirSync(new URL('src/', runtime))
    .filter(file => file.endsWith('.rs'))
    .map(file => read(`src/${file}`))
    .join('\n') + read('prelude.cav');
  for (const group of groups) {
    for (const word of group.words) {
      assert.ok(sources.includes(`"${word}"`) || new RegExp(`\\bfn ${word}\\b`).test(sources),
        `${word} (${group.name}) does not appear in the runtime`);
    }
  }
});

const keywordLike = scopes => scopes.some(scope => /^(keyword|storage|support|constant\.language)\./.test(scope));

async function scopesAt(probe, word) {
  const text = probe.replace('@', word);
  const scopes = await scopesOf(text);
  return scopes.slice(probe.indexOf('@'), probe.indexOf('@') + word.length);
}

test('journal window statement heads agree with the tokenizer without reserving the name', async () => {
  for (const text of ['journal window 2;', 'state x = 0; journal window 3;', 'journal\twindow 8;']) {
    const head = statementHeads(text).find(item => item.word === 'journal');
    assert.ok(head);
    const expected = expectedHeadScope(head.word, text.slice(head.index + head.word.length));
    assert.equal(expected, 'keyword.other');
    assert.ok(has((await scopesOf(text))[head.index], expected));
  }
  for (const text of ['journal supports seen;', 'journal(1);', 'journal_window;']) {
    const [head] = statementHeads(text);
    assert.equal(expectedHeadScope(head.word, text.slice(head.index + head.word.length)), null);
    assert.equal(keywordLike((await scopesOf(text))[head.index]), false);
  }
  for (const probe of ['state @ = 1;', 'on tick set x = @;', 'bind hud.@ = 1;']) {
    for (const scopes of await scopesAt(probe, 'journal')) assert.equal(keywordLike(scopes), false);
  }
});

for (const group of [...groups, { name: 'not source words', ...notSourceWords }]) {
  test(group.name, async () => {
    for (const word of group.words) {
      for (const probe of [group.probe ?? []].flat()) {
        for (const scopes of await scopesAt(probe, word)) {
          assert.ok(has(scopes, group.scope), `${JSON.stringify(probe.replace('@', word))}: ${word} is ${scopes.join(' ')}, not ${group.scope}`);
        }
      }
      if (group.alsoProbe) {
        const [probe, scope] = group.alsoProbe;
        for (const scopes of await scopesAt(probe, word)) {
          assert.ok(has(scopes, scope), `${JSON.stringify(probe.replace('@', word))}: ${word} is ${scopes.join(' ')}, not ${scope}`);
        }
      }
      for (const probe of group.notKeyword ?? []) {
        for (const scopes of await scopesAt(probe, word)) {
          assert.ok(!keywordLike(scopes), `${JSON.stringify(probe.replace('@', word))}: ${word} should be a name, is ${scopes.join(' ')}`);
        }
      }
    }
  });
}
