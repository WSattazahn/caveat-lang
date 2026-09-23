import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { assertSessionMetadata } from './checks.mjs';
import { payloadJson } from './packet/transport.mjs';
import { transformReference } from './packet-transform.mjs';
import { describeCorpus } from './corpus.mjs';

test('Primary metadata check detects elapsed and sequence corruption independently', () => {
  const expected = { elapsed: 0, sequence: 0 };
  assertSessionMetadata({ ...expected }, expected);
  assert.throws(() => assertSessionMetadata({ elapsed: 123, sequence: 0 }, expected), /elapsed differs/);
  assert.throws(() => assertSessionMetadata({ elapsed: 0, sequence: 999 }, expected), /sequence differs/);
  assert.throws(() => assertSessionMetadata({ sequence: 0 }, expected), /elapsed differs/);
});

test('Public payload transport preserves explicit invalid null, arrays and scalar values', () => {
  assert.equal(payloadJson({ payload: null }), 'null');
  assert.equal(payloadJson({ payload: [] }), '[]');
  assert.equal(payloadJson({ payload: 7 }), '7');
  assert.equal(payloadJson({ payload: {} }), '{}');
  assert.equal(payloadJson({ event: 'empty' }), '{}');
});

test('Packet removes only the final historical-results section and rejects changed layout', () => {
  const prefix = '# Guide\n\nUse elapsed().\n\n';
  const original = Buffer.from(`${prefix}## Evidence from fresh authors\n\nEarlier results.\n`);
  assert.deepEqual(transformReference('docs/AI_AUTHORING.md', original), Buffer.from(prefix));
  assert.equal(transformReference('other.md', original), original);
  assert.throws(() => transformReference('docs/AI_AUTHORING.md', Buffer.from(prefix)), /heading is missing/);
  assert.throws(() => transformReference('docs/AI_AUTHORING.md', Buffer.concat([original, Buffer.from('\n## Later section\nMore text.')])), /final section/);
});

test('Repeated corpus keeps the exact v1 A/B contracts, oracle and sequences', async () => {
  const expected = {
    'tasks/cold-storage.md': 'b511f465695b91976d64a3dd9ab61effbd0f3ce837faf0383070d3de8ff87b9c',
    'tasks/access-review.md': '7bc7da394257a31d1d3f66d329d8de82bec626222afbec8367739b37616414de',
    'private/oracles.mjs': '47212b223d27d79867fffc2696402a39f0d5aa7eedff3d557f5a80251fe9b5c8',
  };
  for (const [file, hash] of Object.entries(expected)) {
    assert.equal(createHash('sha256').update(await readFile(new URL(file, import.meta.url))).digest('hex'), hash, file);
  }
  const corpus = describeCorpus();
  assert.equal(corpus.tasks.A.cases, 114);
  assert.equal(corpus.tasks.B.cases, 116);
  assert.equal(corpus.tasks.A.casesSha256, 'ae3a1be552d810c7231c087f14a6d9d714dd4ff6ff0e26a5e31a0786b81cce23');
  assert.equal(corpus.tasks.B.casesSha256, '9acbce2b085b7d35689bd1bba28ab1468926c802f4c675b16884c9818e71b10a');
});
