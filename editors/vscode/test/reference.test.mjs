// The reference's reading of `$` words in for blocks, checked on programs
// whose reading runtime/tests/routed_repetition.rs checks against the runtime
// (fixtures/substitutions.json). The corpus holds only programs that load, so
// these cover what it cannot: words the runtime refuses, and readings no
// tracked program needs, such as a part whose last statement has no `;`, or
// text with a character that only one of Rust and JavaScript reads as
// whitespace. The file lists Rust's whitespace too, which the runtime's tests
// check against char::is_whitespace, and the reference must read as
// whitespace exactly those characters.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { forBlocks, isWhitespace, substitutions } from './reference.mjs';

const { cases, whitespace } = JSON.parse(readFileSync(new URL('./fixtures/substitutions.json', import.meta.url), 'utf8'));

test('the cases are read', () => {
  assert.ok(cases.length >= 5, `expected the cases, found ${cases.length}`);
});

test('the reference reads as whitespace the characters the runtime does', () => {
  const listed = new Set(whitespace.map(point => Number.parseInt(point.replace(/^U\+/, ''), 16)));
  assert.equal(listed.size, whitespace.length);
  const differ = [];
  for (let point = 0; point <= 0x10ffff; point++) {
    if (point >= 0xd800 && point <= 0xdfff) continue;
    if (isWhitespace(String.fromCodePoint(point)) !== listed.has(point)) differ.push(`U+${point.toString(16).toUpperCase().padStart(4, '0')}`);
  }
  assert.deepEqual(differ, []);
});

for (const { name, source, substitutions: expected } of cases) {
  test(name, () => {
    const text = `${source.join('\n')}\n`;
    const found = forBlocks(text).flatMap(block => substitutions(text, block).map(({ word, length }) => [word, length]));
    assert.deepEqual(found, expected);
  });
}
