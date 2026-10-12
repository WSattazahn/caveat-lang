// Saves written by published releases (docs/COMPATIBILITY.md, "Saves,
// archives and source"): with the same source, this runtime restores each
// one, and the events after it give the outcomes and views the release gave.
// scripts/save-fixture.mjs writes a fixture per release from its installed
// package; nothing here regenerates them.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { real } from './helpers.mjs';
import { dependents, explain } from '../lib/explain.mjs';

const fixtures = fileURLToPath(new URL('./fixtures/saves/', import.meta.url));
const releases = readdirSync(fixtures).filter(name => /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.]+)?$/.test(name)).sort();
const programs = readdirSync(path.join(fixtures, 'programs')).filter(name => name.endsWith('.cav')).sort();

// View 0.2 is not frozen until 1.0.0, and rc.18 changed it on purpose: binding
// explanations list evidence in first-observed order, not sorted (rc.18 plan,
// Q2; spec/caveat-view-0.2.md). For a fixture written before that
// change, View 0.2 is compared with each evidence list sorted on both sides,
// so every other byte must still match that release, and the order is held
// byte for byte to the same events played live, without a save, by this
// runtime: a restore that lost first-observed order fails. View 0.1 is
// compared exactly for every release.
const sortedEvidence = view => {
  const copy = structuredClone(view);
  for (const fields of Object.values(copy.binding_explanations ?? {})) {
    for (const explanation of Object.values(fields)) explanation.evidence.sort();
  }
  return JSON.stringify(copy);
};
const viewV2Before = { '0.1.0-rc.17': sortedEvidence };

// Expected objects match partially and arrays exactly, as in a scenario's
// `expect` (spec/caveat-scenarios-0.1.md): a later kit may add report fields.
function contains(actual, expected, at = '') {
  if (Array.isArray(expected)) {
    assert.ok(Array.isArray(actual), `${at} is not an array`);
    assert.equal(actual.length, expected.length, `${at} length`);
    expected.forEach((item, index) => contains(actual[index], item, `${at}/${index}`));
  } else if (expected && typeof expected === 'object') {
    assert.ok(actual && typeof actual === 'object' && !Array.isArray(actual), `${at} is not an object`);
    for (const [key, value] of Object.entries(expected)) contains(actual[key], value, `${at}/${key}`);
  } else {
    assert.equal(actual, expected, at);
  }
}

test('every published release since rc.17 has a save fixture for every program', () => {
  assert.ok(releases.includes('0.1.0-rc.17'), 'the first fixture release is 0.1.0-rc.17');
  for (const release of releases) {
    assert.deepEqual(readdirSync(path.join(fixtures, release)).sort(), programs.map(name => name.replace(/\.cav$/, '.json')), release);
  }
});

for (const release of releases) {
  for (const file of readdirSync(path.join(fixtures, release)).sort()) {
    test(`a save written by ${release} for ${file.replace(/\.json$/, '.cav')} restores and continues as it did`, () => {
      const fixture = JSON.parse(readFileSync(path.join(fixtures, release, file), 'utf8'));
      assert.equal(fixture.package, `caveat-lang@${release}`);
      const source = readFileSync(path.join(fixtures, fixture.program), 'utf8');
      const { before, after } = JSON.parse(readFileSync(path.join(fixtures, fixture.events), 'utf8'));
      const saved = JSON.parse(fixture.save);
      if (saved.source_sha256 !== undefined) {
        assert.equal(saved.source_sha256, createHash('sha256').update(source).digest('hex'), 'the fixture program is the source the save was made under');
      }
      const session = real.restore(source, fixture.save);
      try {
        const archive = [...fixture.archive];
        after.forEach(({ event, payload }, index) => {
          const result = session.dispatch(event, payload);
          const { view, ...expected } = fixture.after[index];
          const outcome = result.outcome === 'accepted' ? { outcome: 'accepted' } : { outcome: result.outcome, origin: result.origin, code: result.code };
          assert.deepEqual(outcome, expected, `${event} (after event ${index + 1})`);
          assert.equal(JSON.stringify(session.view()), view, `View 0.1 after ${event} (after event ${index + 1})`);
          archive.push(...session.drainArchive());
        });
        const asWritten = viewV2Before[release];
        const viewV2 = session.view({ schema: '0.2' });
        if (asWritten) {
          assert.equal(asWritten(viewV2), asWritten(JSON.parse(fixture.view_v2)), 'View 0.2 after the last event, evidence order aside');
          const live = real.open(source);
          try {
            for (const { event, payload } of [...before, ...after]) live.dispatch(event, payload);
            assert.equal(JSON.stringify(viewV2), JSON.stringify(live.view({ schema: '0.2' })), 'View 0.2 after the last event, as the same events give it without a save');
          } finally {
            live.close();
          }
        } else assert.equal(JSON.stringify(viewV2), fixture.view_v2, 'View 0.2 after the last event');
        contains(explain(session.snapshot(), [], { archive }), fixture.explain, 'explain');
        // The archive that release drained stays readable by `dependents`: with
        // it, every evidence name and departed record gives the report the same
        // events give when played live, without a save, by this runtime.
        const live = real.open(source);
        try {
          const liveArchive = [];
          for (const { event, payload } of [...before, ...after]) {
            live.dispatch(event, payload);
            liveArchive.push(...live.drainArchive());
          }
          const subjects = new Set([...fixture.explain.evidence.map(item => item.id), ...fixture.archive.flatMap(item => item.record ?? [])]);
          const report = (snapshot, subject, items) => {
            try { return dependents(snapshot, subject, { archive: items }); } catch (error) { return { error: error.message }; }
          };
          for (const subject of subjects) {
            assert.deepEqual(report(session.snapshot(), subject, archive), report(live.snapshot(), subject, liveArchive), `dependents of ${subject}`);
          }
        } finally {
          live.close();
        }
        // The continued session saves and restores in this runtime.
        const again = real.restore(source, session.save());
        assert.equal(again.save(), session.save());
        again.close();
      } finally {
        session.close();
      }
    });
  }
}
