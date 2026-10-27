// review: which evidence a decision hinged on (spec/caveat-review-0.1.md).
// It replays the events without the events that observed parts of a
// revision's lineage (its grounds, and the control it read outside them,
// such as the deciding rule's `when` guard), and reports which absences mean the deciding event no
// longer commits, which only change the committed value, and which change
// nothing. It runs fresh sessions and never changes the one it reviews.
import { explain } from './explain.mjs';

export const REVIEW_SCHEMA = 'caveat-review/0.1';

// Every combination of units is replayed up to this many units; past it, each
// unit alone.
export const EXHAUSTIVE_LIMIT = 8;

// Sends the events at `indices`, in order, to a fresh session. A fatal
// outcome ends the run. Each sent entry keeps its index in `events`.
function run(open, events, indices) {
  const session = open();
  const sent = [];
  let snapshot = session.snapshot();
  try {
    for (const index of indices) {
      const { event, payload, agent } = events[index];
      // An event an agent sent is replayed as that agent (spec/caveat-agents-0.1.md).
      const from = agent === undefined ? {} : { agent };
      try {
        const { snapshot: after, ...outcome } = agent === undefined
          ? session.dispatch(event, payload)
          : session.dispatch(event, payload, from);
        sent.push({ index, event, payload, ...from, outcome,
          ...(outcome.outcome === 'accepted' ? { sequence: after.sequence, effects: after.effects } : {}) });
        if (outcome.outcome === 'accepted') snapshot = after;
      } catch (error) {
        sent.push({ index, event, payload, ...from, outcome: { outcome: 'fatal', kind: error.kind ?? null, message: error.message } });
        break;
      }
    }
  } finally {
    session.close();
  }
  return { snapshot, sent };
}

// The index of the event whose `reveal` or `sample` first observed each
// occurrence, as explain's `observed_by` finds it.
function observerIndices(sent) {
  const found = new Map();
  for (const entry of sent) {
    if (entry.outcome.outcome !== 'accepted' || !Array.isArray(entry.effects)) continue;
    for (const effect of entry.effects) {
      const id = effect.kind === 'reveal' ? effect.evidence : effect.kind === 'sample' ? effect.id : null;
      if (typeof id === 'string' && !found.has(id)) found.set(id, entry.index);
    }
  }
  return found;
}

// The revision of `series` that the event at `index` committed in this run,
// or null.
function committedBy(snapshot, sent, series, index) {
  const entry = sent.find(item => item.index === index);
  if (entry?.outcome.outcome !== 'accepted') return null;
  const revisions = new Set((snapshot.decision_series?.[series]?.revisions ?? []).map(revision => revision.id));
  const commit = (snapshot.decision_journal ?? []).find(item =>
    item.change === 'committed' && item.sequence === entry.sequence && revisions.has(item.commitment));
  if (!commit) return null;
  return { id: commit.commitment, value: snapshot.commitment_bases?.[commit.commitment]?.value ?? null };
}

// Why the deciding event committed nothing in a replay.
function failure(sent, index) {
  const fatal = sent.find(item => item.outcome.outcome === 'fatal');
  if (fatal && fatal.index !== index) return { event: fatal.event, outcome: 'fatal', message: fatal.outcome.message };
  const entry = sent.find(item => item.index === index);
  if (!entry) return { event: null, outcome: 'not_sent', message: 'an earlier event failed' };
  const { outcome } = entry;
  if (outcome.outcome === 'accepted') return { event: entry.event, outcome: 'accepted', message: 'accepted, and no rule committed' };
  if (outcome.outcome === 'fatal') return { event: entry.event, outcome: 'fatal', message: outcome.message };
  return { event: entry.event, outcome: outcome.outcome, origin: outcome.origin ?? null,
    code: outcome.code ?? null, message: outcome.message ?? null };
}

// Non-empty subsets of 0..n-1, smallest first, in a fixed order.
function subsets(n) {
  const all = [];
  for (let mask = 1; mask < 1 << n; mask++) {
    all.push([...Array(n).keys()].filter(bit => mask & (1 << bit)));
  }
  return all.sort((a, b) => a.length - b.length || a.join().localeCompare(b.join()));
}

/**
 * Reviews revision `revisionId` of a session made by sending `events` to
 * `open()`. `open` returns a fresh session of the program each time it is
 * called; `events` are `{event, payload}` in order.
 */
export function review(open, events, revisionId, { limit = EXHAUSTIVE_LIMIT } = {}) {
  const at = revisionId.lastIndexOf('@');
  const series = at > 0 ? revisionId.slice(0, at) : null;
  const all = events.map((_, index) => index);
  const full = run(open, events, all);
  const report = explain(full.snapshot, full.sent);
  const decision = series === null ? null : report.decisions.find(item => item.name === series && item.kind !== 'plain');
  const revision = decision?.revisions.find(item => item.id === revisionId);
  if (!revision) throw new Error(`no revision ${revisionId}: name a revision of a decision series, such as choose@1`);
  const commit = revision.history.find(item => item.change === 'committed');
  const deciding = commit ? full.sent.find(item => item.sequence === commit.sequence)?.index : undefined;
  if (deciding === undefined) throw new Error(`the journal no longer says which event committed ${revisionId}`);

  const evidenceById = new Map(report.evidence.map(item => [item.id, item]));
  // The caveats a piece of evidence carried for this revision: those its
  // frozen grounds hold that the evidence carries, or carried before a
  // take-back (spec/caveat-take-back-0.1.md; D029). A retained caveat no
  // grounds evidence carries reached the revision through its lineage.
  const everCarried = id => {
    const item = evidenceById.get(id);
    return [...(item?.caveats ?? []), ...(item?.taken_back ?? []).map(record => record.caveat)];
  };
  const caveatsOf = id => revision.grounds.caveats.filter(name => everCarried(id).includes(name));
  const observedAt = observerIndices(full.sent);
  const grounds = revision.grounds.evidence;
  // Lineage evidence outside the grounds: read by the deciding rule's guard,
  // a permission grant or a predecessor's basis. A decision can hinge on it.
  const control = revision.lineage.evidence.filter(id => !grounds.includes(id));
  const tested = [...grounds, ...control];

  // Evidence observed by one earlier event forms one unit: dropping the event
  // drops them together.
  const byEvent = new Map();
  const entangled = [];
  const unobserved = [];
  for (const id of tested) {
    const index = observedAt.get(id);
    if (index === undefined) unobserved.push(id);
    else if (index === deciding) entangled.push(id);
    else byEvent.set(index, [...(byEvent.get(index) ?? []), id]);
  }
  const units = [...byEvent.entries()].sort(([a], [b]) => a - b)
    .map(([index, evidence]) => ({ index, evidence }));
  const exhaustive = units.length <= limit;
  const tried = exhaustive ? subsets(units.length) : units.map((_, unit) => [unit]);

  const decisive = [];
  const valueOnly = [];
  let replays = 0;
  const covers = (found, set) => found.some(item => item.units.every(unit => set.includes(unit)));
  for (const set of tried) {
    if (covers(decisive, set)) continue;
    const dropped = new Set(set.map(unit => units[unit].index));
    const replay = run(open, events, all.filter(index => !dropped.has(index)));
    replays += 1;
    const made = committedBy(replay.snapshot, replay.sent, series, deciding);
    const evidence = set.flatMap(unit => units[unit].evidence);
    const without = set.map(unit => ({ sequence: full.sent[units[unit].index].sequence, event: events[units[unit].index].event,
      ...(full.sent[units[unit].index].agent === undefined ? {} : { agent: full.sent[units[unit].index].agent }) }));
    if (!made) decisive.push({ units: set, evidence, without, because: failure(replay.sent, deciding) });
    else if (made.value !== revision.value && !covers(valueOnly, set)) {
      valueOnly.push({ units: set, evidence, without, value: made.value });
    }
  }
  const counted = new Set([...decisive, ...valueOnly].flatMap(item => item.evidence));
  const outcome = revision.outcome ?? null;
  const materialized = outcome?.materialized ?? [];
  const decisiveEvidence = new Set(decisive.flatMap(item => item.evidence));
  const strip = ({ units: _units, ...item }) => ({ ...item, caveats: [...new Set(item.evidence.flatMap(caveatsOf))] });

  return {
    schema: REVIEW_SCHEMA,
    revision: revisionId,
    value: revision.value,
    status: revision.status,
    decided_by: { sequence: commit.sequence, event: commit.event, ...(commit.agent === undefined ? {} : { agent: commit.agent }) },
    grounds: grounds.map(id => ({ evidence: id, caveats: caveatsOf(id) })),
    control,
    retained: revision.retained,
    outcome,
    decisive: decisive.map(strip),
    value_only: valueOnly.map(strip),
    not_decisive: units.flatMap(unit => unit.evidence).filter(id => !counted.has(id)),
    entangled,
    unobserved,
    came_true: materialized.map(caveat => {
      const carriedBy = tested.filter(id => caveatsOf(id).includes(caveat));
      return { caveat, carried_by: carriedBy, decisive: carriedBy.some(id => decisiveEvidence.has(id)) };
    }),
    withdrawn: revision.withdrawn,
    outside_grounds: report.evidence.map(item => item.id)
      .filter(id => !tested.includes(id) && (observedAt.get(id) ?? Infinity) < deciding),
    // The events after a fatal one are not sent; the review is of the session before it.
    fatal: full.sent.find(item => item.outcome.outcome === 'fatal')?.outcome.message ?? null,
    search: { mode: exhaustive ? 'exhaustive' : 'single', units: units.length, replays, limit },
  };
}

const list = values => (values.length ? values.join(', ') : 'nothing');
const caveatNote = caveats => (caveats.length ? ` (caveats: ${caveats.join(', ')})` : '');

function failureText(because) {
  if (because.outcome === 'accepted') return `${because.event} is accepted, and no rule commits`;
  if (because.outcome === 'not_sent') return 'an earlier event fails first';
  if (because.outcome === 'fatal') return `${because.event} fails: ${because.message}`;
  const code = because.code && because.code !== 'reject' ? `/${because.code}` : '';
  return `${because.event} is refused (${because.origin}${code}): ${because.message}`;
}

/** The review as text for a person. `title` names the program. */
export function formatReview(report, title = 'the program') {
  const lines = [`${title}: review of ${report.revision} = ${report.value}  ${report.status}`,
    `  decided at #${report.decided_by.sequence} ${report.decided_by.event}${report.decided_by.agent === undefined ? '' : ` by ${report.decided_by.agent}`}`,
    `  based on ${list(report.grounds.map(item => item.evidence))}${caveatNote(report.grounds.flatMap(item => item.caveats))}`];
  // A retained caveat no grounds evidence carries now reached the revision
  // through its lineage, or was taken back since (D029).
  const kept = report.retained.filter(name => !report.grounds.some(item => item.caveats.includes(name)));
  if (kept.length) lines.push(`  also carrying, through its lineage: ${list(kept)}`);
  if (report.control.length) lines.push(`  control outside its grounds: ${list(report.control)}`);
  if (report.outcome) {
    const { value, because, materialized = [], sequence, event } = report.outcome;
    lines.push(`  resolved at #${sequence} ${event} = ${value} because ${list(because)}${materialized.length ? `; materialized: ${materialized.join(', ')}` : ''}`);
  }
  lines.push('', 'Decisive (without these, it is not decided)');
  if (!report.decisive.length) lines.push('  nothing');
  for (const item of report.decisive) {
    lines.push(`  ${item.evidence.join(' + ')}${caveatNote(item.caveats)}: ${failureText(item.because)}`);
  }
  lines.push('', 'Value only (without these, it is decided with another value)');
  if (!report.value_only.length) lines.push('  nothing');
  for (const item of report.value_only) lines.push(`  ${item.evidence.join(' + ')}: ${report.revision} would be ${item.value}`);
  lines.push('', `Not decisive: ${list(report.not_decisive)}`);
  if (report.entangled.length) lines.push(`Observed by the deciding event, not separable: ${list(report.entangled)}`);
  if (report.unobserved.length) lines.push(`Not observed by any event given: ${list(report.unobserved)}`);
  if (report.outside_grounds.length) lines.push(`Observed before the decision, outside its lineage: ${list(report.outside_grounds)}`);
  for (const item of report.came_true) {
    const carrier = item.carried_by.length ? `carried by ${list(item.carried_by)}${item.decisive ? ', decisive' : ''}` : 'retained, carried by no evidence in its lineage';
    lines.push(`Came true: ${item.caveat} (${carrier})${report.status === 'in force' ? '; the revision is still in force' : ''}`);
  }
  for (const item of report.withdrawn) lines.push(`Withdrawn since: ${item.evidence} at #${item.sequence} because ${item.because}`);
  const { mode, units, replays } = report.search;
  const count = (n, word) => `${n} ${word}${n === 1 ? '' : 's'}`;
  lines.push('', mode === 'exhaustive'
    ? `${count(replays, 'replay')} searched every combination of ${count(units, 'observing event')}.`
    : `${count(replays, 'replay')} tried each of ${count(units, 'observing event')} alone (more than ${report.search.limit}); combinations are not searched.`);
  return lines.join('\n');
}
