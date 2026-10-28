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

export const FIND_SCHEMA = 'caveat-find/0.1';

/** A query's words: lowercase, split on white space. */
export function queryWords(query) {
  return String(query).toLowerCase().split(/\s+/).filter(Boolean);
}

// What a word can match in one revision, as [kind, text] pairs in a fixed
// order: the decision, the program, the evidence its grounds, lineage and
// outcome name with their sources and claims, its caveats, and the reasons
// its caveats were retained for.
function termsOf(series, revision, program, snapshot) {
  const symbols = new Map((snapshot.symbols ?? []).map(symbol => [symbol.name, symbol]));
  const claims = new Map();
  for (const relation of snapshot.relations ?? []) {
    if (relation.relation !== 'supports' && relation.relation !== 'opposes') continue;
    if (!claims.has(relation.from)) claims.set(relation.from, new Set());
    claims.get(relation.from).add(relation.to);
  }
  const evidence = new Set([...revision.grounds.evidence, ...revision.lineage.evidence, ...(revision.outcome?.because ?? [])]);
  const caveats = new Set([...revision.grounds.caveats, ...revision.lineage.caveats, ...(revision.retained ?? []), ...(revision.outcome?.materialized ?? [])]);
  const terms = [['decision', series.name], ['revision', revision.id], ['program', program]];
  const add = (kind, text) => { if (typeof text === 'string' && text && !terms.some(([k, t]) => k === kind && t === text)) terms.push([kind, text]); };
  for (const name of evidence) {
    add('evidence', name);
    add('source', symbols.get(name)?.source);
    add('display', symbols.get(name)?.display);
    for (const claim of claims.get(name) ?? []) add('claim', claim);
  }
  for (const name of caveats) { add('caveat', name); add('display', symbols.get(name)?.display); }
  for (const reason of Object.values(revision.retention_reasons ?? {})) add('reason', reason);
  return terms;
}

// Where each word occurs among the terms, or null when a word occurs in none.
function matchWords(words, terms) {
  const matched = {};
  for (const word of words) {
    const where = terms.filter(([, text]) => text.toLowerCase().includes(word)).map(([kind, text]) => `${kind} ${text}`);
    if (!where.length) return null;
    matched[word] = where;
  }
  return matched;
}

/**
 * The decision revisions, in every ledger, whose decision, program, evidence,
 * evidence sources, claims, caveats or retention reasons contain every word
 * of `query` (spec/caveat-find-0.1.md). `ledgers` and `open` are as for
 * reviewLedgers; a ledger's optional `name` is the program text words match
 * (default: `program`). Each ledger's events are sent to one fresh session.
 */
export function findDecisions(open, ledgers, query) {
  const words = queryWords(query);
  if (!words.length) throw new Error('name at least one word to find');
  const searched = [];
  const found = [];
  for (const ledger of ledgers) {
    const head = { program: ledger.program, events: ledger.events_file };
    let full;
    try { full = run(() => open(ledger.source), ledger.events); } catch (error) {
      searched.push({ ...head, error: error.message });
      continue;
    }
    const fatal = full.sent.find(entry => entry.outcome.outcome === 'fatal');
    const sha = full.snapshot.source_sha256;
    const explained = explain(full.snapshot, full.sent);
    let revisions = 0;
    for (const series of explained.decisions) {
      for (const revision of series.revisions ?? []) {
        revisions += 1;
        const matched = matchWords(words, termsOf(series, revision, ledger.name ?? ledger.program, full.snapshot));
        if (!matched) continue;
        const decided = revision.history.find(entry => entry.change === 'committed') ?? null;
        found.push({
          ...head, program_sha256: sha, series: series.name, revision: revision.id,
          current: series.current === revision.id, value: revision.value, status: revision.status,
          grounds: revision.grounds, retained: revision.retained ?? revision.grounds.caveats,
          ...(revision.retention_reasons ? { retention_reasons: revision.retention_reasons } : {}),
          withdrawn: revision.withdrawn,
          decided: decided && { sequence: decided.sequence, event: decided.event },
          outcome: revision.outcome ?? null,
          sources: Object.fromEntries(revision.grounds.evidence.map(name => [name,
            (full.snapshot.symbols ?? []).find(symbol => symbol.name === name)?.source ?? null])),
          history: revision.history, matched,
        });
      }
    }
    searched.push({ ...head, program_sha256: sha, revisions,
      fatal: fatal ? { index: fatal.index, event: fatal.event, message: fatal.outcome.message } : null });
  }
  return { schema: FIND_SCHEMA, query: String(query), words, searched, found };
}

/** The found decisions as text for a person, ending with a count. */
export function formatFound(report) {
  const lines = [];
  const list = items => items.join(', ');
  for (const item of report.found) {
    lines.push(`${item.program}: ${item.revision} = ${typeof item.value === 'string' ? item.value : JSON.stringify(item.value)}  ${item.status}`);
    const grounds = item.grounds.evidence.map(name => (item.sources[name] ? `${name} ("${item.sources[name]}")` : name));
    lines.push(`  based on ${grounds.length ? list(grounds) : 'no evidence'}${item.retained.length ? `; caveats: ${list(item.retained)}` : ''}`);
    if (item.decided) lines.push(`  decided at #${item.decided.sequence} ${item.decided.event}`);
    for (const entry of item.history) {
      if (entry.change === 'committed' || entry.change === 'resolved') continue;
      lines.push(`  #${entry.sequence} ${entry.event}: ${entry.change}${entry.because?.length ? ` because ${list(entry.because)}` : ''}`);
    }
    if (item.withdrawn.length) lines.push(`  withdrawn since: ${list(item.withdrawn.map(entry => entry.evidence))}`);
    if (item.outcome) {
      const value = typeof item.outcome.value === 'string' ? item.outcome.value : JSON.stringify(item.outcome.value);
      lines.push(`  turned out ${value} at #${item.outcome.sequence} ${item.outcome.event}${item.outcome.because.length ? ` because ${list(item.outcome.because)}` : ''}${item.outcome.materialized.length ? `; came true: ${list(item.outcome.materialized)}` : ''}`);
    }
    for (const [word, where] of Object.entries(item.matched)) lines.push(`  matched ${word}: ${list(where)}`);
    lines.push('');
  }
  const searched = report.searched.filter(entry => !entry.error).length;
  lines.push(`${report.found.length} decision${report.found.length === 1 ? '' : 's'} found in ${searched} ledger${searched === 1 ? '' : 's'}.`);
  return lines.join('\n');
}
