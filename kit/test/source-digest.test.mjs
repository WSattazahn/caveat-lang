// Owner decision D6 (spec/caveat-save-0.1.md, "Source digest"): explain and
// dependents show the SHA-256 of the source each decision change was made
// under, and "not recorded" for changes a save made before rc.17 holds.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { real, thermostat } from './helpers.mjs';
import { dependents, explain, formatDependents, formatExplanation } from '../lib/explain.mjs';

const digest = createHash('sha256').update(thermostat, 'utf8').digest('hex');
const changes = report => report.decisions.flatMap(series => series.revisions.flatMap(revision => revision.history));

test('explain shows the source digest beside each decision change', () => {
  const session = real.open(thermostat);
  try {
    session.dispatch('read', { value: 17 });
    session.dispatch('read', { value: 25 });
    const snapshot = session.snapshot();
    assert.equal(snapshot.source_sha256, digest);
    assert.equal(snapshot.source_unrecorded_through, undefined);
    const report = explain(snapshot);
    assert.deepEqual(report.source, { sha256: digest, unrecorded_through: 0 });
    assert.ok(changes(report).length >= 2);
    assert.ok(changes(report).every(change => change.source_sha256 === digest));
    const text = formatExplanation(report);
    assert.match(text, new RegExp(`made under source sha256:${digest}`));
    assert.doesNotMatch(text, /not recorded/);
  } finally { session.close(); }
});

test('a save made before the digest shows its changes as not recorded, and later ones with it', () => {
  const first = real.open(thermostat);
  let older;
  try {
    first.dispatch('read', { value: 17 });
    const save = JSON.parse(first.save());
    assert.equal(save.source_sha256, digest);
    delete save.source_sha256;
    older = JSON.stringify(save);
  } finally { first.close(); }
  const session = real.restore(thermostat, older);
  try {
    session.dispatch('read', { value: 25 });
    const snapshot = session.snapshot();
    assert.equal(snapshot.source_unrecorded_through, 1);
    const report = explain(snapshot);
    const sources = changes(report).map(change => [change.sequence, change.source_sha256]);
    assert.ok(sources.some(([sequence, source]) => sequence === 1 && source === null));
    assert.ok(sources.some(([sequence, source]) => sequence === 2 && source === digest));
    const text = formatExplanation(report);
    assert.match(text, /source not recorded for events through #1/);
    assert.match(text, /#1 read: .*\(source not recorded\)/);
    const evidence = dependents(snapshot, 'temperature');
    assert.ok(evidence.changes.length);
    for (const change of evidence.changes) assert.equal(change.source_sha256, change.sequence > 1 ? digest : null);
    assert.match(formatDependents(evidence), /\(source not recorded\)/);
  } finally { session.close(); }
});

test('a save is still refused under edited source', () => {
  const session = real.open(thermostat);
  try {
    session.dispatch('read', { value: 17 });
    assert.throws(() => real.restore(`${thermostat}\n// edited\n`, session.save()),
      error => error.kind === 'restore' && error.message === 'cannot restore save: it belongs to a different program');
  } finally { session.close(); }
});
