// Pin attribution is independent of the runtime's incremental index. Consume the
// saved own/inherited split, never count compacted marker ranges as exact pins.
export const categories = ['state grounds', 'commitment in force', 'journal entry',
  'scheduled qualification', 'permission grant', 'withdrawal reason'];

export function pinnedRecords(save) {
  const retired = new Set(Object.keys(save.retired ?? {}));
  const by = Object.fromEntries(categories.map((name) => [name, new Set()]));
  const pin = (name, category) => { if (retired.has(name)) by[category].add(name); };
  const own = (provenance = {}) => (provenance.evidence ?? [])
    .filter((name) => !(provenance.inherited ?? []).includes(name));
  for (const state of Object.values(save.states)) {
    for (const name of own(state.grounds ?? state.lineage)) pin(name, 'state grounds');
  }
  const superseded = new Set(Object.values(save.decision_series ?? {}).flatMap((series) =>
    series.revisions.filter((revision) => revision.id !== series.current).map((revision) => revision.id)));
  for (const [commitment, grounds] of Object.entries(save.commitment_grounds ?? {})) {
    if (!superseded.has(commitment)) for (const name of own(grounds)) pin(name, 'commitment in force');
  }
  for (const [index, entry] of (save.decision_journal ?? []).entries()) {
    if (!retired.has(`journal@${(save.journal_departed ?? 0) + index + 1}`)) {
      for (const name of [...entry.because, ...(entry.permitted_by ? [entry.permitted_by] : [])]) pin(name, 'journal entry');
    }
  }
  for (const item of save.scheduled_qualifications ?? []) pin(item.evidence, 'scheduled qualification');
  for (const item of Object.values(save.commitment_permissions ?? {})) pin(item.grant, 'permission grant');
  for (const item of save.withdrawals ?? []) pin(item.because, 'withdrawal reason');
  const pinned = new Set(Object.values(by).flatMap((names) => [...names]));
  const declared = [...retired].filter((name) => Object.hasOwn(save.renewals ?? {}, name));
  return {
    retired: retired.size,
    pinned: pinned.size,
    by_provenance: Object.fromEntries(categories.map((name) => [name, [...by[name]].sort()])),
    declared_first_occurrences: declared.sort(),
    unpinned_non_declarations: [...retired].filter((name) => !pinned.has(name) && !declared.includes(name)).sort(),
  };
}

export function maximumPins(checkpoints) {
  return {
    pinned: Math.max(0, ...checkpoints.map((checkpoint) => checkpoint.pinned)),
    by_provenance: Object.fromEntries(categories.map((name) => [name,
      Math.max(0, ...checkpoints.map((checkpoint) => checkpoint.by_provenance[name].length))])),
  };
}
