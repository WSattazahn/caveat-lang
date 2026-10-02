export async function attempt(session, score) {
  const operations = [];
  try {
    const start = session.snapshot().sequence;
    const observation = await session.dispatch('observe', {confidence: score});
    operations.push({event:'observe', ...observation});
    if (observation.outcome !== 'accepted') {
      return {ok:false, operations, snapshot:session.snapshot()};
    }
    const assessment = await session.dispatch('assess');
    operations.push({event:'assess', ...assessment});
    const snapshot = session.snapshot();
    const current = snapshot.decision_series.assessment.current;
    const revision = snapshot.decision_series.assessment.revisions.find(r=>r.id===current);
    const commitment = snapshot.commitments.find(c=>c.action===current);
    const freshCommit = snapshot.decision_journal.some(e=>
      e.commitment===current && e.change==='committed' &&
      e.event==='assess' && e.sequence===assessment.snapshot?.sequence &&
      e.sequence>start);
    const ok = assessment.outcome==='accepted' &&
      snapshot.bindings.assessment.verdict==='approved' &&
      commitment?.open===false && revision?.sequence>start && freshCommit;
    return {ok:Boolean(ok), operations, current, snapshot};
  } catch (error) {
    return {ok:false, operations, error:{name:error.name,message:error.message}};
  }
}
