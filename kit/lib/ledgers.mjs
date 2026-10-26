// ledgers: reviewing a set of past ledgers, and the past reviews a program
// says it consulted (spec/caveat-ledger-sets-0.1.md). A ledger is a program
// and its record; the command finds them on disk and this module reviews
// every revision that has an outcome. It runs fresh sessions only.
import { explain } from './explain.mjs';
import { review } from './review.mjs';

export const LEDGER_REVIEW_SCHEMA = 'caveat-ledger-review/0.1';

// An evidence source naming a past review: the reviewed program's source
// SHA-256, as its saves and snapshots bind it, and the revision.
export const CONSULTED_SOURCE = /^caveat-review:([0-9a-f]{64})#([A-Za-z_][A-Za-z0-9_]*@[1-9][0-9]*)$/;

// Sends every event to a fresh session, as review does. A fatal outcome ends
// the run.
function run(open, events) {
  const session = open();
  const sent = [];
  let snapshot = session.snapshot();
  try {
    for (const [index, { event, payload = {} }] of events.entries()) {
      try {
        const { snapshot: after, ...outcome } = session.dispatch(event, payload);
        sent.push({ index, event, payload, outcome,
          ...(outcome.outcome === 'accepted' ? { sequence: after.sequence, effects: after.effects } : {}) });
        if (outcome.outcome === 'accepted') snapshot = after;
      } catch (error) {
        sent.push({ index, event, payload, outcome: { outcome: 'fatal', kind: error.kind ?? null, message: error.message } });
        break;
      }
    }
  } finally {
    session.close();
  }
  return { snapshot, sent };
}

/**
 * The evidence in `snapshot`'s program whose source names a past review, in
 * declaration order: `{ evidence, program_sha256, revision }`.
 */
export function consultedSources(snapshot) {
  const found = [];
  for (const symbol of snapshot.symbols ?? []) {
    if (symbol.kind !== 'evidence' || typeof symbol.source !== 'string') continue;
    const match = CONSULTED_SOURCE.exec(symbol.source);
    if (match) found.push({ evidence: symbol.name, program_sha256: match[1], revision: match[2] });
  }
  return found;
}

/**
 * The past reviews `report` (a review of the session that `snapshot` and
 * `sent` describe) consulted: consulted-review evidence observed before its
 * deciding event, each saying whether it is in the revision's lineage.
 */
function consultedBy(report, snapshot, sent) {
  // The sequence of the event whose reveal first observed each evidence,
  // neutral reveals included.
  const observedAt = new Map();
  for (const entry of sent) {
    if (entry.outcome.outcome !== 'accepted') continue;
    for (const effect of entry.effects ?? []) {
      if (effect.kind === 'reveal' && !observedAt.has(effect.evidence)) observedAt.set(effect.evidence, entry.sequence);
    }
  }
  const explained = explain(snapshot, sent);
  const decision = explained.decisions.flatMap(item => item.revisions ?? []).find(item => item.id === report.revision);
  const lineage = new Set(decision?.lineage?.evidence ?? []);
  return consultedSources(snapshot)
    .filter(item => (observedAt.get(item.evidence) ?? Infinity) < report.decided_by.sequence)
    .map(item => ({ ...item, in_lineage: lineage.has(item.evidence) }));
}

/** Reviews `revision` as `review` does, adding what it consulted. */
export function reviewWithConsulted(open, events, revision, options) {
  const report = review(open, events, revision, options);
  const { snapshot, sent } = run(open, events);
  return { ...report, consulted: consultedBy(report, snapshot, sent) };
}

/**
 * Reviews every revision in each ledger that has an outcome or consulted a
 * past review. `ledgers` are
 * `{ program, events_file, source, events }`; `open(source)` returns a fresh
 * session. Returns one entry per revision, or one `{ error }` entry for a
 * ledger that cannot be used, in the ledgers' order. Each consulted review is
 * resolved against the set: `found` names the ledger with that source and
 * `came_true` lists what its review says came true.
 */
export function reviewLedgers(open, ledgers, options) {
  const entries = [];
  const bySha = new Map();
  for (const ledger of ledgers) {
    const head = { schema: LEDGER_REVIEW_SCHEMA, program: ledger.program, events: ledger.events_file };
    let full;
    try { full = run(() => open(ledger.source), ledger.events); } catch (error) {
      entries.push({ ...head, error: error.message });
      continue;
    }
    const sha = full.snapshot.source_sha256;
    if (!bySha.has(sha)) bySha.set(sha, { ledger, reviews: new Map() });
    const explained = explain(full.snapshot, full.sent);
    const consults = consultedSources(full.snapshot).length > 0;
    for (const decision of explained.decisions) {
      for (const revision of decision.revisions ?? []) {
        if (!revision.outcome && !consults) continue;
        const report = review(() => open(ledger.source), ledger.events, revision.id, options);
        const consulted = consultedBy(report, full.snapshot, full.sent);
        if (!revision.outcome && !consulted.length) continue;
        const { schema: _schema, ...rest } = report;
        const entry = { ...head, program_sha256: sha, ...rest, consulted };
        bySha.get(sha).reviews.set(revision.id, entry);
        entries.push(entry);
      }
    }
  }
  for (const entry of entries) {
    for (const item of entry.consulted ?? []) {
      const target = bySha.get(item.program_sha256);
      const reviewed = target?.reviews.get(item.revision);
      item.found = target ? { program: target.ledger.program, events: target.ledger.events_file } : null;
      item.came_true = reviewed ? reviewed.came_true.map(({ caveat, decisive }) => ({ caveat, decisive })) : null;
    }
  }
  return entries;
}

/** The lines `formatReview` lacks: what a review consulted. */
export function formatConsulted(consulted) {
  return consulted.map(item => {
    const where = item.found === undefined ? '' : item.found === null
      ? '; no ledger in the set has that program'
      : item.came_true === null
        ? `; ${item.found.program} has no such revision with an outcome`
        : `; ${item.found.program}: ${item.came_true.length ? item.came_true.map(c => `${c.caveat} came true${c.decisive ? ', decisive' : ''}`).join('; ') : 'nothing came true'}`;
    return `Consulted: ${item.evidence}, the review of ${item.revision} of program ${item.program_sha256.slice(0, 12)}${item.in_lineage ? ' (in its lineage)' : ''}${where}`;
  });
}
