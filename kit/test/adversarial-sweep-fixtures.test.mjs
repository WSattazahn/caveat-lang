// Sequence-bound originals and the narrowly derived arbitrary-event programs.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFile} from 'node:fs/promises';
import test from 'node:test';
import {real} from './helpers.mjs';

const sha = value => createHash('sha256').update(value).digest('hex');
const provenance = JSON.parse(await readFile(new URL('fixtures/adversarial/provenance.json', import.meta.url), 'utf8'));
const names = [['marker-journal.cav', 'record'], ['post-collection.cav', 'start']];
for (const [name, first] of names) {
  const derivation = provenance.sweepDerivatives[name];
  const original = await readFile(new URL(`fixtures/adversarial/${derivation.original}`, import.meta.url));
  const guarded = await readFile(new URL(`fixtures/adversarial/${name}`, import.meta.url));
  test(`sweep fixture ${name}: original source bytes and duplicate plain-commit control remain`, () => {
    assert.equal(original.length, provenance.files[derivation.original].bytes);
    assert.equal(sha(original), provenance.files[derivation.original].sha256);
    assert.equal(sha(original), derivation.originalSha256);
    const session = real.open(original.toString('utf8'));
    try {
      assert.equal(session.dispatch(first).outcome, 'accepted');
      assert.equal(session.dispatch('decide').outcome, 'accepted');
      assert.throws(() => session.dispatch('decide'), error => error.kind === 'fatal'
        && /commitment trust already exists/.test(error.message));
    } finally { session.close(); }
  });

  test(`sweep fixture ${name}: only the explicit guard is added and repeated decide preserves the decision`, () => {
    const source = original.toString('utf8');
    assert.equal(source.split(derivation.replacement.from).length, 2);
    assert.equal(guarded.toString('utf8'), source.replace(derivation.replacement.from, derivation.replacement.to));
    assert.equal(guarded.length, provenance.files[name].bytes);
    assert.equal(sha(guarded), provenance.files[name].sha256);
    const session = real.open(guarded.toString('utf8'));
    let restored;
    try {
      assert.equal(session.dispatch(first).outcome, 'accepted');
      assert.equal(session.dispatch('decide').outcome, 'accepted');
      const before = session.snapshot();
      restored = real.restore(guarded.toString('utf8'), session.save());
      for (const active of [session, restored]) {
        for (let index = 0; index < 20; index++) assert.equal(active.dispatch('decide').outcome, 'accepted');
        const after = active.snapshot();
        assert.equal(after.sequence, before.sequence + 20);
        for (const field of ['commitments', 'commitment_bases', 'commitment_grounds', 'decision_journal', 'withdrawals']) {
          assert.deepEqual(after[field], before[field], field);
        }
        assert.deepEqual(active.drainArchive(), []);
      }
      assert.equal(restored.save(), session.save());
      assert.deepEqual(restored.snapshot(), session.snapshot());
    } finally { session.close(); restored?.close(); }
  });
}
