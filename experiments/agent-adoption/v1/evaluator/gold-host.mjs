// Private evaluator fixture, never an agent packet example.
export async function attempt(session, score) {
  const before = session.snapshot().sequence;
  const observed = session.dispatch('observe', { confidence: score });
  if (observed.outcome !== 'accepted') return { ok: false };
  const assessed = session.dispatch('assess');
  if (assessed.outcome !== 'accepted') return { ok: false };
  const snapshot = session.snapshot();
  const current = snapshot.decision_series.assessment.current;
  const committed = snapshot.decision_journal.find(entry => entry.commitment === current && entry.change === 'committed');
  const reopened = snapshot.decision_journal.some(entry => entry.commitment === current && entry.change === 'reopened');
  return { ok: snapshot.bindings.assessment.verdict === 'approved' && !reopened && committed?.sequence > before };
}
