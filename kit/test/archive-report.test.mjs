import assert from 'node:assert/strict';
import test from 'node:test';
import { explain, formatExplanation } from '../lib/explain.mjs';

function fixture(read) {
  const marker = { history: 's', from: 1, through: 4, read, departed_at: 5 };
  const provenance = { evidence: [], caveats: [], departed: [marker] };
  const snapshot = { sequence: 6, elapsed: 0, reading_streams: { s: { occurrences: [{ id: 's@5' }] } },
    commitment_bases: { chosen: { value: 1, provenance } }, commitment_grounds: { chosen: provenance },
    commitments: [{ action: 'chosen', open: false, retained: [] }] };
  const archive = [1, 2, 3, 4].map(number => ({ history: 's', number, record: `s@${number}`,
    holders: number === 3 ? [] : [{ kind: 'commitment', name: 'chosen', in: 'basis' },
      { kind: 'commitment', name: 'chosen', in: 'grounds' }] }));
  return { snapshot, archive };
}

test('holder-count equality cannot turn a merged lower bound into exact record names', () => {
  // Real information-loss counterexample: runtime/tests/departure.rs,
  // copying_markers_after_departure_does_not_reconstruct_exact_archive_membership.
  const { snapshot, archive } = fixture(3);
  const report = explain(snapshot, [], { archive });
  assert.equal(report.decisions[0].revisions[0].lineage.departed[0].records, undefined);
  assert.match(formatExplanation(report), /at least 3 of departed s@1 to @4/);
});

test('legacy markers without provenance roots do not claim archive reconstruction', () => {
  const { snapshot, archive } = fixture(4);
  const report = explain(snapshot, [], { archive });
  assert.equal(report.decisions[0].revisions[0].lineage.departed[0].records, undefined);
  assert.match(formatExplanation(report), /all 4 of departed s@1 to @4/);
  const partial = explain(snapshot, [], { archive: [archive[0], archive[0], archive[2], archive[3]] });
  assert.equal(partial.decisions[0].revisions[0].lineage.departed[0].records, undefined,
    'duplicates cannot stand in for a missing record');
});
