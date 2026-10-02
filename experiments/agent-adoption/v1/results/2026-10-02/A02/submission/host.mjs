/** One attempt at a time per supplied public session. The caller owns its lifecycle. */
export async function attempt(session, score) {
  const operations = [];
  try {
    const start = session.snapshot().sequence;
    const observation = session.dispatch('observe', {confidence: score});
    operations.push({event: 'observe', ...observation});
    if (observation.outcome !== 'accepted') return {ok: false, operations};
    const assessment = session.dispatch('assess');
    operations.push({event: 'assess', ...assessment});
    if (assessment.outcome !== 'accepted') return {ok: false, operations};
    const snapshot = assessment.snapshot;
    const current = snapshot.decision_series.assessment.current;
    const revision = snapshot.decision_series.assessment.revisions.find(r => r.id === current);
    const commitment = snapshot.commitments.find(c => c.action === current);
    const committedNow = snapshot.decision_journal.some(j => j.commitment === current &&
      j.change === 'committed' && j.event === 'assess' && j.sequence === snapshot.sequence && j.sequence > start);
    const ok = snapshot.bindings.assessment.verdict === 'approved' &&
      !!commitment && commitment.open === false && !!revision &&
      revision.event === 'assess' && revision.sequence === snapshot.sequence && committedNow;
    return {ok: Boolean(ok), current, operations};
  } catch (error) {
    return {ok: false, operations, error: {kind: error.kind ?? 'host', message: error.message}};
  }
}
