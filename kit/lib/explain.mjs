// Why a session is the way it is: every decision with what it rests on and its
// history, every piece of observed evidence with its caveats, and every
// displayed value with the evidence it cites. Pure: it reads a snapshot, and
// the events that produced it, and never runs a program.

export const EXPLAIN_SCHEMA = 'caveat-explain/0.1';

const none = () => ({ evidence: [], caveats: [] });

// The caveats that qualify each piece of observed evidence.
function caveatsByEvidence(snapshot) {
  const caveats = {};
  for (const relation of snapshot.relations ?? []) {
    if (relation.relation === 'qualifies') (caveats[relation.to] ??= []).push(relation.from);
  }
  for (const names of Object.values(caveats)) names.sort();
  return caveats;
}

/**
 * A structured explanation of `snapshot`. `events` lists what led to it, in
 * order, as `{event, payload, outcome}` where `outcome` is a dispatch outcome
 * without its snapshot.
 */
export function explain(snapshot, events = []) {
  const qualifiedBy = caveatsByEvidence(snapshot);
  const readings = new Map();
  for (const stream of Object.values(snapshot.reading_streams ?? {})) {
    for (const occurrence of stream.occurrences ?? []) readings.set(occurrence.id, occurrence);
  }
  const evidence = (snapshot.relations ?? [])
    .filter(relation => relation.relation === 'supports' || relation.relation === 'opposes')
    .map(relation => {
      const reading = readings.get(relation.from);
      return {
        id: relation.from, relation: relation.relation, claim: relation.to,
        value: reading?.value ?? null, sequence: reading?.sequence ?? null, event: reading?.event ?? null,
        caveats: qualifiedBy[relation.from] ?? [],
      };
    });

  const journal = snapshot.decision_journal ?? [];
  const decisions = Object.entries(snapshot.decision_series ?? {}).map(([name, series]) => ({
    name,
    limit: series.limit,
    current: series.current,
    revisions: (series.revisions ?? []).map(revision => {
      const history = journal.filter(entry => entry.commitment === revision.id).map(entry => ({
        change: entry.change, sequence: entry.sequence, event: entry.event, because: entry.because, caveats: entry.caveats,
      }));
      const reopened = history.some(entry => entry.change === 'reopened');
      return {
        id: revision.id,
        value: snapshot.commitment_bases?.[revision.id]?.value ?? null,
        status: revision.id !== series.current ? 'superseded' : reopened ? 'reopened' : 'in force',
        grounds: snapshot.commitment_grounds?.[revision.id] ?? none(),
        lineage: snapshot.commitment_bases?.[revision.id]?.provenance ?? none(),
        history,
      };
    }),
  }));

  const displayed = Object.entries(snapshot.bindings ?? {}).flatMap(([target, properties]) =>
    Object.entries(properties).map(([property, value]) => ({
      name: `${target}.${property}`, value, cites: snapshot.binding_explanations?.[target]?.[property] ?? none(),
    })));

  return { schema: EXPLAIN_SCHEMA, sequence: snapshot.sequence, elapsed: snapshot.elapsed, events, decisions, evidence, displayed };
}

const list = values => (values.length ? values.join(', ') : 'nothing');
const withCaveats = ({ evidence, caveats }) => `${list(evidence)}${caveats.length ? ` (caveats: ${caveats.join(', ')})` : ''}`;
const show = value => (typeof value === 'string' ? JSON.stringify(value) : String(value));

function outcomeText(outcome) {
  if (outcome.outcome === 'accepted') return 'accepted';
  if (outcome.outcome === 'fatal') return `failed: ${outcome.message}`;
  const code = outcome.code && outcome.code !== 'reject' ? `/${outcome.code}` : '';
  return `refused (${outcome.origin}${code}): ${outcome.message}`;
}

/** The explanation as text for a person. `title` names the program. */
export function formatExplanation(report, title = 'the program') {
  const count = report.events.length;
  const lines = [`${title} after ${count} event${count === 1 ? '' : 's'} (sequence ${report.sequence}${report.elapsed ? `, elapsed ${report.elapsed}` : ''})`];
  if (count) {
    lines.push('', 'Events');
    report.events.forEach((entry, index) => {
      const payload = entry.payload && Object.keys(entry.payload).length ? ` ${JSON.stringify(entry.payload)}` : '';
      lines.push(`  ${String(index + 1).padStart(3)}  ${entry.event}${payload}  ${outcomeText(entry.outcome)}`);
    });
  }
  lines.push('', 'Decisions');
  if (!report.decisions.length) lines.push('  none declared');
  for (const series of report.decisions) {
    lines.push(`  ${series.name}: ${series.revisions.length} of at most ${series.limit}`);
    for (const revision of series.revisions) {
      lines.push(`    ${revision.id} = ${show(revision.value)}  ${revision.status}`);
      lines.push(`      based on ${withCaveats(revision.grounds)}`);
      const also = revision.lineage.evidence.filter(name => !revision.grounds.evidence.includes(name));
      if (also.length) lines.push(`      could also have been influenced by ${list(also)}`);
      for (const entry of revision.history) {
        lines.push(`      #${entry.sequence} ${entry.event}: ${entry.change} because ${withCaveats({ evidence: entry.because, caveats: entry.caveats })}`);
      }
    }
  }
  lines.push('', 'Evidence');
  if (!report.evidence.length) lines.push('  none observed');
  for (const item of report.evidence) {
    const value = item.value === null ? '' : ` = ${show(item.value)}`;
    const when = item.sequence === null ? '' : `  (#${item.sequence} ${item.event})`;
    lines.push(`  ${item.id}${value} ${item.relation} ${item.claim}${when}${item.caveats.length ? `  caveats: ${item.caveats.join(', ')}` : ''}`);
  }
  lines.push('', 'Displayed');
  if (!report.displayed.length) lines.push('  nothing bound');
  for (const item of report.displayed) {
    const cited = item.cites.evidence.length || item.cites.caveats.length;
    lines.push(`  ${item.name} = ${show(item.value)}${cited ? `  because ${withCaveats(item.cites)}` : ''}`);
  }
  return lines.join('\n');
}

/**
 * Parses an events file: one JSON object per line, `{"event": NAME}` with an
 * optional `"payload"` object. Blank lines are skipped. Throws an Error that
 * names the line.
 */
export function parseEvents(text) {
  const events = [];
  text.split(/\r?\n/).forEach((line, index) => {
    if (!line.trim()) return;
    let record;
    try { record = JSON.parse(line); } catch (error) { throw new Error(`line ${index + 1}: not JSON: ${error.message}`); }
    if (!record || typeof record !== 'object' || Array.isArray(record)) throw new Error(`line ${index + 1}: expected an object`);
    const extra = Object.keys(record).filter(key => key !== 'event' && key !== 'payload');
    if (extra.length) throw new Error(`line ${index + 1}: unknown field ${extra[0]}`);
    if (typeof record.event !== 'string' || !record.event) throw new Error(`line ${index + 1}: "event" must name an event`);
    events.push({ event: record.event, payload: record.payload ?? {} });
  });
  return events;
}
