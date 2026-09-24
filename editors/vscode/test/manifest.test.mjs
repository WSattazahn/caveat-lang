// The committed grammar is the one the build script writes, and the
// extension manifest points at it.
import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { test } from 'node:test';
import { grammarText } from '../scripts/build-grammar.mjs';

const here = new URL('../', import.meta.url);
const read = path => readFileSync(new URL(path, here), 'utf8');

test('the committed grammar is what scripts/build-grammar.mjs writes', () => {
  assert.equal(read('syntaxes/caveat.tmLanguage.json').replace(/\r\n/g, '\n'), grammarText,
    'run `npm run build:grammar` in editors/vscode and commit the result');
});

test('the manifest contributes the language and its grammar', () => {
  const manifest = JSON.parse(read('package.json'));
  const grammar = JSON.parse(read('syntaxes/caveat.tmLanguage.json'));
  const [language] = manifest.contributes.languages;
  const [contributed] = manifest.contributes.grammars;
  assert.equal(language.id, 'caveat');
  assert.deepEqual(language.extensions, ['.cav']);
  assert.equal(contributed.language, language.id);
  assert.equal(contributed.scopeName, grammar.scopeName);
  for (const path of [language.configuration, contributed.path]) {
    assert.ok(existsSync(new URL(path, here)), `${path} is missing`);
  }
  const configuration = JSON.parse(read(language.configuration));
  assert.equal(configuration.comments.lineComment, '//');
});
