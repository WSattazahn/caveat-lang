// The reference's reading of `$` words in for blocks, checked on programs
// whose reading runtime/tests/routed_repetition.rs checks against the runtime
// (fixtures/substitutions.json). The corpus holds only programs that load, so
// these cover what it cannot: words the runtime refuses, and readings no
// tracked program needs, such as a part whose last statement has no `;`.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { forBlocks, substitutions } from './reference.mjs';

const { cases } = JSON.parse(readFileSync(new URL('./fixtures/substitutions.json', import.meta.url), 'utf8'));

test('the cases are read', () => {
  assert.ok(cases.length >= 5, `expected the cases, found ${cases.length}`);
});

for (const { name, source, substitutions: expected } of cases) {
  test(name, () => {
    const text = `${source.join('\n')}\n`;
    const found = forBlocks(text).flatMap(block => substitutions(text, block).map(({ word, length }) => [word, length]));
    assert.deepEqual(found, expected);
  });
}
