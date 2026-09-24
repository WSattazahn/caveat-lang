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

export const DEPENDENTS_SCHEMA = 'caveat-dependents/0.1';

// What a subject stands for. A caveat stands for itself. Evidence stands for
// itself and its occurrences (NAME@1, NAME@2, ...); a reading stream, and the
// evidence a stream reads from, stand for the stream's readings. Null when the
// program declares no such evidence, stream or caveat.
function resolveSubject(snapshot, subject) {
  const kinds = new Map((snapshot.symbols ?? []).map(symbol => [symbol.name, symbol.kind]));
  if (kinds.get(subject) === 'caveat') return { kind: 'caveat', ids: new Set([subject]) };
  const streams = snapshot.reading_streams ?? {};
  const known = kinds.get(subject) === 'evidence' || Object.hasOwn(streams, subject)
    || Object.values(streams).some(stream => stream.template === subject);
  if (!known) return null;
  const ids = new Set();
  for (const [name, kind] of kinds) {
    const suffix = name.startsWith(`${subject}@`) ? name.slice(subject.length + 1) : null;
    if (kind === 'evidence' && (name === subject || (suffix && /^\d+$/.test(suffix)))) ids.add(name);
  }
  for (const [name, stream] of Object.entries(streams)) {
    if (name === subject || stream.template === subject) for (const reading of stream.occurrences ?? []) ids.add(reading.id);
  }
  return { kind: 'evidence', ids };
}

/**
 * Everything in `snapshot` that rests on `subject`, which names evidence, a
 * reading stream or a caveat: the decisions and values based on it or that it
 * could have influenced, the decision changes it caused, and the displayed
 * values that cite it or could have been influenced by it. The reverse of
 * `explain`. Throws an Error when the program declares no such name.
 */
export function dependents(snapshot, subject) {
  const resolved = resolveSubject(snapshot, subject);
  if (!resolved) throw new Error(`${subject} is not evidence, a reading stream or a caveat in this program`);
  const via = provenance => {
    const names = resolved.kind === 'caveat' ? provenance?.caveats : provenance?.evidence;
    return (names ?? []).filter(name => resolved.ids.has(name));
  };
  const basis = (primary, primaryLabel, lineage) => {
    const direct = via(primary);
    if (direct.length) return { basis: primaryLabel, via: direct };
    const possible = via(lineage);
    return possible.length ? { basis: 'lineage', via: possible } : null;
  };

  const decisions = explain(snapshot).decisions.flatMap(series => series.revisions.flatMap(revision => {
    const found = basis(revision.grounds, 'grounds', revision.lineage);
    return found ? [{ id: revision.id, value: revision.value, status: revision.status, ...found }] : [];
  }));
  const changes = (snapshot.decision_journal ?? []).flatMap(entry => {
    const names = resolved.kind === 'caveat' ? entry.caveats : entry.because;
    const found = (names ?? []).filter(name => resolved.ids.has(name));
    return found.length ? [{ sequence: entry.sequence, event: entry.event, commitment: entry.commitment, change: entry.change, via: found }] : [];
  });
  const values = Object.entries(snapshot.qualified_values ?? {}).flatMap(([name, value]) => {
    const found = basis(snapshot.value_grounds?.[name], 'grounds', value.provenance);
    return found ? [{ name, value: value.value, ...found }] : [];
  });
  const displayed = Object.entries(snapshot.bindings ?? {}).flatMap(([target, properties]) =>
    Object.entries(properties).flatMap(([property, value]) => {
      const found = basis(snapshot.binding_explanations?.[target]?.[property], 'cites',
        snapshot.binding_qualifications?.[target]?.[property]);
      return found ? [{ name: `${target}.${property}`, value, ...found }] : [];
    }));

  return {
    schema: DEPENDENTS_SCHEMA, subject, kind: resolved.kind, sequence: snapshot.sequence,
    decisions, changes, values, displayed,
  };
}

/** The dependents report as text for a person. `title` names the program. */
export function formatDependents(report, title = 'the program', events = 0) {
  const through = names => (report.kind === 'caveat' ? `evidence with ${list(names)}` : list(names));
  const label = { grounds: 'based on', cites: 'cites', lineage: 'could have been influenced by' };
  const lines = [`What rests on ${report.subject} in ${title} after ${events} event${events === 1 ? '' : 's'} (sequence ${report.sequence})`];
  const section = (heading, items, line) => {
    lines.push('', heading);
    if (!items.length) lines.push('  nothing');
    for (const item of items) lines.push(`  ${line(item)}`);
  };
  section('Decisions', report.decisions, item => `${item.id} = ${show(item.value)}  ${item.status}  ${label[item.basis]} ${through(item.via)}`);
  section('Decision changes', report.changes, item => `#${item.sequence} ${item.event}: ${item.commitment} ${item.change} because ${through(item.via)}`);
  section('Values', report.values, item => `${item.name} = ${show(item.value)}  ${label[item.basis]} ${through(item.via)}`);
  section('Displayed', report.displayed, item => `${item.name} = ${show(item.value)}  ${label[item.basis]} ${through(item.via)}`);
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
