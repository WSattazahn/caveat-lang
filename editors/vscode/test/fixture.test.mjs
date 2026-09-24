// Checks the caret assertions in fixtures/scopes.cav (format in its header).
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { has, scopesOf } from './tokenize.mjs';

const fixture = new URL('./fixtures/scopes.cav', import.meta.url);

// [{line, column, length, scopes, forbidden, source}] from the caret lines.
function assertionsIn(text) {
  const lines = text.split('\n');
  const assertions = [];
  let target = null;
  lines.forEach((line, number) => {
    const carets = /^#( *)(\^+)\s+(.+)$/.exec(line);
    if (!carets) {
      target = number;
      return;
    }
    assert.ok(target !== null && lines[target].trim(), `line ${number + 1}: carets under no code`);
    const words = carets[3].trim().split(/\s+/);
    assertions.push({
      line: target,
      column: 1 + carets[1].length,
      length: carets[2].length,
      scopes: words.filter(word => !word.startsWith('!')),
      forbidden: words.filter(word => word.startsWith('!')).map(word => word.slice(1)),
      source: number + 1,
    });
  });
  return { lines, assertions };
}

test('the fixture highlights as annotated', async () => {
  const text = readFileSync(fixture, 'utf8').replace(/\r\n/g, '\n');
  const { lines, assertions } = assertionsIn(text);
  assert.ok(assertions.length >= 80, `only ${assertions.length} assertions were read`);
  const scopes = await scopesOf(text);
  const offsets = [];
  lines.reduce((offset, line, number) => ((offsets[number] = offset), offset + line.length + 1), 0);

  const failures = [];
  for (const assertion of assertions) {
    const code = lines[assertion.line];
    assert.ok(assertion.column + assertion.length <= code.length, `line ${assertion.source}: carets run past the code`);
    for (let column = assertion.column; column < assertion.column + assertion.length; column++) {
      const got = scopes[offsets[assertion.line] + column];
      const missing = assertion.scopes.filter(scope => !has(got, scope));
      const present = assertion.forbidden.filter(scope => has(got, scope));
      if (missing.length || present.length) {
        failures.push(`line ${assertion.source}, column ${column} ${JSON.stringify(code[column])} in ${JSON.stringify(code.trim())}: `
          + `${missing.length ? `missing ${missing.join(' ')}; ` : ''}${present.length ? `has ${present.join(' ')}; ` : ''}got ${got.join(' ')}`);
        break;
      }
    }
  }
  assert.deepEqual(failures, [], `\n${failures.join('\n')}`);
});
