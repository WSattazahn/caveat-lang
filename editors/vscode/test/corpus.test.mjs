// Every tracked .cav file, tokenized as VS Code tokenizes it, checked against
// what the runtime makes of the same text (see reference.mjs).
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { root, trackedCavFiles } from './corpus.mjs';
import {
  classify, forBlocks, identifiers, numbers, statementHeads, substitutions,
} from './reference.mjs';
import { has, scopesOf } from './tokenize.mjs';
import { expectedHeadScope, highlightedWords, isBuiltin, profileHeads } from './expectations.mjs';

const files = trackedCavFiles();

const totals = { files: 0, strings: 0, comments: 0, blocks: 0, substitutions: 0, heads: 0, builtins: 0, properties: 0, numbers: 0 };

function position(text, index) {
  const before = text.slice(0, index);
  const line = before.split('\n').length;
  return `${line}:${index - before.lastIndexOf('\n')}`;
}

// Collects failures for one file so a report shows several at once.
function checker(file, text, scopes) {
  const failures = [];
  const check = (ok, index, message) => {
    if (!ok && failures.length < 12) {
      const shown = JSON.stringify(text.slice(index, index + 24));
      failures.push(`${file}:${position(text, index)} ${message} at ${shown}\n    scopes: ${scopes[index].join(' ')}`);
    }
  };
  const done = () => assert.deepEqual(failures, [], `\n${failures.join('\n')}`);
  return { check, done };
}

test('the corpus is discovered from git', () => {
  assert.ok(files.length >= 50, `expected the tracked .cav files, found ${files.length}`);
});

for (const file of files) {
  test(file, async () => {
    const text = readFileSync(join(root, file), 'utf8').replace(/\r\n/g, '\n');
    const scopes = await scopesOf(text);
    const classes = classify(text);
    const { check, done } = checker(file, text, scopes);
    totals.files += 1;

    // Strings and comments cover exactly the characters the runtime reads as
    // quoted text and comments, whatever they contain.
    for (let index = 0; index < text.length; index++) {
      if (text[index] === '\n') continue;
      const cls = classes[index];
      if (cls === 'string' && text[index] === '"' && classes[index - 1] !== 'string') totals.strings += 1;
      if (cls === 'comment' && classes[index - 1] !== 'comment') totals.comments += 1;
      check((cls === 'string') === has(scopes[index], 'string.quoted.double.caveat'), index,
        cls === 'string' ? 'quoted text is not a string' : 'a string scope outside quoted text');
      check((cls === 'comment') === has(scopes[index], 'comment.line'), index,
        cls === 'comment' ? 'a comment is not a comment' : 'a comment scope outside a comment');
    }

    // A `for` block is one region from `for` to its closing brace. Inside it,
    // each `$` plus the bound name it substitutes is a template, the rest of
    // the name is not, and the header's binding is a parameter.
    const blocks = forBlocks(text, classes);
    const inBlock = new Array(text.length).fill(false);
    const templated = new Array(text.length).fill(false);
    for (const block of blocks) {
      totals.blocks += 1;
      inBlock.fill(true, block.start, block.end);
      for (let index = block.bindingAt; index <= block.bindingAt + block.binding.length; index++) {
        check(has(scopes[index], 'variable.parameter.template'), index, 'the binding is not a template parameter');
      }
      for (const { index, length, word } of substitutions(text, block)) {
        totals.substitutions += 1;
        check(length > 0, index, `$${word} is not bound in this block`);
        templated.fill(true, index, index + length);
        const after = index + length;
        if (/\w/.test(text[after] ?? '')) {
          check(!has(scopes[after], 'variable.other.template'), after, 'the template runs past its bound name');
        }
      }
    }
    // Outside a block, `$NAME` in code is refused at load; in text it is text.
    const stray = new Array(text.length).fill(false);
    for (let index = 0; index < text.length; index++) {
      if (text[index] !== '$' || inBlock[index] || classes[index] !== 'code') continue;
      stray.fill(true, index, index + text.slice(index).match(/^\$(?:[A-Za-z_]\w*)?/)[0].length);
    }
    for (let index = 0; index < text.length; index++) {
      if (text[index] === '\n') continue;
      check(inBlock[index] === has(scopes[index], 'meta.block.repeat'), index,
        inBlock[index] ? 'a for block ends early' : 'a for block region outside a for block');
      check(templated[index] === has(scopes[index], 'variable.other.template'), index,
        templated[index] ? 'a substituted name is not a template' : 'a template scope where nothing is substituted');
      check(stray[index] === has(scopes[index], 'invalid.illegal.template'), index,
        stray[index] ? 'a $ outside a for block is not flagged' : 'flagged as a stray $ but is not one');
    }

    // Statement heads: keywords get their category's scope, and a head that
    // is not a keyword (`secondhand qualifies witness`) is left a name.
    for (const head of statementHeads(text, classes)) {
      totals.heads += 1;
      const expected = expectedHeadScope(head.word);
      const got = scopes[head.index];
      if (profileHeads.includes(head.word)) {
        check(head.lineStart, head.index, `${head.word} begins a statement mid-line, where the grammar cannot see it`);
      }
      if (expected) check(has(got, expected), head.index, `statement head ${head.word} should be ${expected}`);
      else check(!got.some(scope => /^(keyword|storage|support|constant)\./.test(scope)), head.index, `statement head ${head.word} is a name, not a keyword`);
    }

    for (const { index, word, before } of identifiers(text, classes)) {
      const got = scopes[index];
      const keywordLike = got.some(scope => /^(keyword|storage|constant\.language)\./.test(scope));
      // After a dot, every name is a property, whatever word it is.
      if (before === '.') {
        totals.properties += 1;
        check(has(got, 'variable.other.property') && !keywordLike, index, `.${word} is not a property`);
        continue;
      }
      // Nothing gets a keyword scope unless it is one of the runtime's words.
      if (keywordLike) check(highlightedWords.has(word), index, `${word} is highlighted as a keyword but is not one`);
      // A declared function is a function name (the prelude declares `abs`);
      // calls to builtins are support functions, other calls are calls.
      const call = text.slice(index + word.length).match(/^\s*\(/);
      if (call && !keywordLike) {
        const declared = /\b(fn|proc)\s+$/.test(text.slice(Math.max(0, index - 12), index));
        if (declared) {
          check(has(got, 'entity.name.function') && !has(got, 'entity.name.function.call'), index, `fn ${word} is not a function name`);
        } else if (isBuiltin(word)) {
          totals.builtins += 1;
          check(has(got, 'support.function.builtin'), index, `${word}( is not a builtin call`);
        } else {
          check(has(got, 'entity.name.function.call'), index, `${word}( is not a call`);
        }
      }
      if (['supports', 'opposes', 'qualifies'].includes(word)) {
        check(has(got, 'keyword.operator.relation'), index, `${word} is not a relation`);
      }
    }

    // Numbers are exactly the numeric literals.
    const numeric = new Array(text.length).fill(false);
    for (const { index, length } of numbers(text, classes)) {
      totals.numbers += 1;
      numeric.fill(true, index, index + length);
    }
    for (let index = 0; index < text.length; index++) {
      if (text[index] === '\n') continue;
      check(numeric[index] === has(scopes[index], 'constant.numeric'), index,
        numeric[index] ? 'a number is not numeric' : 'a numeric scope outside a number');
    }
    done();
  });
}

// The per-file checks prove nothing if the corpus stopped exercising them.
test('every check had something to check', () => {
  for (const [what, count] of Object.entries(totals)) assert.ok(count > 0, `no ${what} were checked`);
  assert.equal(totals.files, files.length);
});
