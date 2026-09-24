// The editor check's comparison, without VS Code: tokens are built the way
// VS Code reports them, from each line's vscode-textmate tokens.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import textmate from 'vscode-textmate';
import { difference } from './editor/compare.mjs';
import { loadGrammar, scopesOf } from './tokenize.mjs';

async function reported(text) {
  const grammar = await loadGrammar();
  const tokens = [];
  let stack = textmate.INITIAL;
  for (const line of text.split('\n')) {
    const result = grammar.tokenizeLine(line, stack);
    for (const token of result.tokens) {
      tokens.push([line.substring(token.startIndex, token.endIndex), token.scopes.join(' ')]);
    }
    stack = result.ruleStack;
  }
  return tokens;
}

const text = 'state z = 1; # 😀\ndisplay d "Glühwein 😀 🍄"; set z = 2;\n';

test('text beyond the BMP compares by code unit', async () => {
  assert.equal(difference(text, await scopesOf(text), await reported(text)), null);
});

test('a disagreement after an emoji is found where it is', async () => {
  const tokens = await reported(text);
  const last = tokens.findLastIndex(([content]) => content === '2');
  tokens[last] = ['2', 'source.caveat something.else'];
  const found = difference(text, await scopesOf(text), tokens);
  assert.equal(found?.at, text.replace(/\n/g, '').lastIndexOf('2'));
});

test('different text is reported', async () => {
  const tokens = await reported(text);
  tokens[0] = ['stat', tokens[0][1]];
  assert.equal(difference(text, await scopesOf(text), tokens)?.message, 'VS Code tokenized different text');
});
