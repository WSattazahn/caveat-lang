// The checker behind break.html: after each event, tests the five rules of
// the withdrawal claim against the runtime's own snapshot. No DOM, so
// scripts/test-break.mjs runs the same code. See spec/caveat-withdrawal-0.1.md.

export const RULES = Object.freeze([
  { id: 'R1', title: 'Nothing recorded changes' },
  { id: 'R2', title: 'Everything computed from it is flagged' },
  { id: 'R3', title: '"Rests on" means grounds, exactly' },
  { id: 'R4', title: 'Nothing reopens by itself' },
  { id: 'R5', title: 'A refused event withdraws nothing' },
]);

// The display group the checker adds to read `rests_on_withdrawn` from the
// runtime itself.
export const PROBE = 'break_check';

const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const list = value => (Array.isArray(value) ? value : []);
const withdrawnSet = snapshot => new Set(list(snapshot.withdrawals).map(record => record.evidence));
const carriesWithdrawn = provenance => list(provenance?.caveats).includes('withdrawn');
const holdsWithdrawn = (provenance, withdrawn) => list(provenance?.evidence).some(name => withdrawn.has(name));
const hasDeparted = provenance => provenance?.departed && Object.keys(provenance.departed).length > 0;
const windowed = snapshot => list(snapshot.windows).length > 0;

/**
 * The program with one display line per decision series added, each asking
 * the runtime `rests_on_withdrawn(SERIES)`. The lines only display; they
 * change no state, evidence or decision. `snapshot` is a snapshot of the
 * unchanged program, which names its decision series.
 */
export function probeSource(source, snapshot) {
  const lines = Object.keys(snapshot.decision_series ?? {})
    .map(name => `bind ${PROBE}.${name} = if(rests_on_withdrawn(${name}), 1, 0);`);
  return lines.length ? `${source}\n\n# Added by the checker on break.html.\n${lines.join('\n')}\n` : source;
}

function finding(rule, kind, detail) {
  return { rule, kind, detail };
}

// R1: every record that existed before the event is still there, unchanged.
function recorded(before, after, findings, notes) {
  if (windowed(after)) {
    notes.push('R1 is not checked here: this program has a window, which retires old records by design (spec/caveat-windows-0.1.md).');
    return;
  }
  for (const key of ['commitment_grounds', 'commitment_bases']) {
    for (const [revision, record] of Object.entries(before[key] ?? {})) {
      if (!same(after[key]?.[revision], record)) {
        findings.push(finding('R1', 'changed the past', `${key}["${revision}"] changed from ${JSON.stringify(record)} to ${JSON.stringify(after[key]?.[revision] ?? null)}`));
      }
    }
  }
  const journal = list(before.decision_journal);
  if (!same(list(after.decision_journal).slice(0, journal.length), journal)) {
    findings.push(finding('R1', 'changed the past', 'an earlier decision journal entry changed or disappeared'));
  }
  const withdrawals = list(before.withdrawals);
  if (!same(list(after.withdrawals).slice(0, withdrawals.length), withdrawals)) {
    findings.push(finding('R1', 'changed the past', 'an earlier withdrawal record changed or disappeared'));
  }
  const claims = snapshot => new Set(list(snapshot.relations)
    .filter(edge => edge.relation === 'supports' || edge.relation === 'opposes')
    .map(edge => `${edge.from} ${edge.relation} ${edge.to}`));
  const kept = claims(after);
  for (const edge of claims(before)) {
    if (!kept.has(edge)) findings.push(finding('R1', 'changed the past', `the observation "${edge}" disappeared`));
  }
}

// R2: a value carries `withdrawn` exactly when its lineage, or its grounds,
// hold a withdrawn occurrence.
function flagged(after, findings, notes) {
  const withdrawn = withdrawnSet(after);
  let skipped = false;
  const test = (what, provenance) => {
    if (!provenance) return;
    if (hasDeparted(provenance)) { skipped = true; return; }
    const reached = holdsWithdrawn(provenance, withdrawn);
    const carries = carriesWithdrawn(provenance);
    if (reached && !carries) {
      findings.push(finding('R2', 'too little', `${what} rests on withdrawn ${list(provenance.evidence).filter(name => withdrawn.has(name)).join(', ')} but does not carry withdrawn`));
    } else if (carries && !reached) {
      findings.push(finding('R2', 'too much', `${what} carries withdrawn but nothing withdrawn is in it: ${JSON.stringify(provenance.evidence ?? [])}`));
    }
  };
  for (const [state, value] of Object.entries(after.qualified_values ?? {})) test(`state ${state}`, value.provenance);
  for (const [state, grounds] of Object.entries(after.value_grounds ?? {})) test(`the grounds of state ${state}`, grounds);
  for (const [group, fields] of Object.entries(after.binding_qualifications ?? {})) {
    if (group === PROBE) continue;
    for (const [field, provenance] of Object.entries(fields)) test(`displayed ${group}.${field}`, provenance);
  }
  if (skipped) notes.push('R2 skipped values whose lineage holds departed records, which no longer name their evidence (spec/caveat-lineage-compaction-0.1.md).');
}

// R3: `rests_on_withdrawn(D)` is true exactly when D's current revision has a
// withdrawn occurrence in its frozen grounds.
function restsOn(after, findings, notes) {
  const withdrawn = withdrawnSet(after);
  const shown = after.bindings?.[PROBE];
  if (!shown) {
    if (Object.keys(after.decision_series ?? {}).length) notes.push('R3 is not checked: the checker could not add its display lines to this program.');
    return;
  }
  for (const [series, record] of Object.entries(after.decision_series ?? {})) {
    if (!record.current) continue;
    const grounds = after.commitment_grounds?.[record.current];
    const expected = holdsWithdrawn(grounds, withdrawn);
    const actual = shown[series] === 1;
    if (actual && !expected) {
      findings.push(finding('R3', 'too much', `rests_on_withdrawn(${series}) is true, but the grounds of ${record.current} hold nothing withdrawn: ${JSON.stringify(grounds?.evidence ?? [])}`));
    } else if (expected && !actual) {
      findings.push(finding('R3', 'too little', `the grounds of ${record.current} hold withdrawn ${list(grounds?.evidence).filter(name => withdrawn.has(name)).join(', ')}, but rests_on_withdrawn(${series}) is false`));
    }
  }
}

// R4: a decision reopens only through an authored step, which the journal
// records in this event.
function reopenedByAuthor(before, after, findings) {
  const wasOpen = new Map(list(before.commitments).map(item => [item.action, item.open]));
  const journal = list(after.decision_journal).filter(entry => entry.sequence === after.sequence && entry.change === 'reopened');
  const effects = list(after.effects).filter(effect => effect.kind === 'reopen');
  for (const item of list(after.commitments)) {
    if (!item.open || wasOpen.get(item.action) !== false) continue;
    const journaled = journal.some(entry => entry.commitment === item.action);
    const effected = effects.some(effect => effect.action === item.action);
    if (!journaled && !effected) {
      findings.push(finding('R4', 'too much', `${item.action} reopened with no reopen step recorded in this event`));
    }
  }
}

/**
 * Checks one event. `before` and `after` are snapshots of the probed program
 * around it; `accepted` is whether the runtime accepted the event. Returns
 * the rules that failed, as findings, and notes on what was not checked.
 */
export function checkEvent(before, after, accepted) {
  const findings = [];
  const notes = [];
  if (!accepted) {
    if (!same(list(after.withdrawals), list(before.withdrawals))) {
      findings.push(finding('R5', 'leaked out of a refusal', 'a refused event changed the withdrawal records'));
    }
    if (!same(after.commitments, before.commitments) || !same(after.decision_journal, before.decision_journal)) {
      findings.push(finding('R5', 'leaked out of a refusal', 'a refused event changed a decision'));
    }
    return { findings, notes };
  }
  recorded(before, after, findings, notes);
  flagged(after, findings, notes);
  restsOn(after, findings, notes);
  reopenedByAuthor(before, after, findings);
  return { findings, notes };
}

/** Checks the snapshot a program starts with, before any event. */
export function checkStart(snapshot) {
  const findings = [];
  const notes = [];
  flagged(snapshot, findings, notes);
  restsOn(snapshot, findings, notes);
  return { findings, notes };
}

/**
 * Text for a GitHub issue body in the shape of
 * .github/ISSUE_TEMPLATE/break-report.md, filled from a run.
 */
export function reportBody({ version, source, events, findings }) {
  const rules = new Set(findings.map(item => item.rule));
  const kinds = new Set(findings.map(item => item.kind));
  const box = on => (on ? '[x]' : '[ ]');
  return [
    '## Which rule', '',
    ...RULES.map(rule => `- ${box(rules.has(rule.id))} ${rule.id} ${rule.title}`), '',
    '## What kind of report', '',
    `- ${box(kinds.has('too much'))} Reopens too much (flagged or reopened something that didn't rest on the evidence)`,
    `- ${box(kinds.has('too little'))} Reopens too little (missed something that did)`,
    `- ${box(kinds.has('changed the past'))} Changed the past (a recorded basis, observation or journal entry changed)`,
    `- ${box(kinds.has('leaked out of a refusal'))} A refused event left a withdrawal behind`,
    '- [ ] I disagree with a rule as written', '',
    '## Version', '',
    `- \`caveat-lang\` version: ${version}`,
    '- Where you ran it: the Try to break this page', '',
    '## The program', '', '```caveat', source.trimEnd(), '```', '',
    '## The events', '', '```jsonl', events.trimEnd(), '```', '',
    '## What you expected, and what happened', '',
    '- Expected:',
    `- Actual (the checker's red lines):`,
    ...findings.map(item => `  - ${item.rule} (${item.kind}) at ${item.at}: ${item.detail}`), '',
    '## Anything else', '', '',
  ].join('\n');
}
