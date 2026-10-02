export async function attempt(session, score) {
  const start = session.snapshot().sequence;
  try {
    const observation = session.dispatch('observe', {confidence: score});
    if (observation.outcome !== 'accepted') return {ok:false, observation};
    const assessment = session.dispatch('assess');
    if (assessment.outcome !== 'accepted') return {ok:false, observation, assessment};
    const snapshot = session.snapshot();
    const current = snapshot.decision_series.assessment.current;
    const fresh = snapshot.decision_journal.some(entry =>
      entry.commitment === current && entry.change === 'committed' &&
      entry.event === 'assess' && entry.sequence > start);
    const inForce = snapshot.commitments.some(c => c.action === current && !c.open);
    return {ok: snapshot.bindings.assessment.verdict === 'approved' && fresh && inForce,
      observation, assessment, current};
  } catch (error) {
    return {ok:false, error:{kind:error.kind, message:error.message}};
  }
}
