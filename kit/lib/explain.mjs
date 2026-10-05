// Why a session is the way it is: every decision with what it rests on and its
// history, every piece of observed evidence with its caveats, and every
// displayed value with the evidence it cites. Pure: it reads a snapshot, and
// the events that produced it, and never runs a program.

export const EXPLAIN_SCHEMA = 'caveat-explain/0.1';

const none = () => ({ evidence: [], caveats: [] });

// Departed records (spec/caveat-departure-0.1.md). A record of a windowed
// history is departed when the session no longer holds it: it is below the
// history's oldest live record and not retired. The session holds a reading
// stream's and a renewal's records in their occurrence lists.
function heldRecords(snapshot) {
  const held = new Map();
  const add = (history, id) => {
    const number = Number(id.slice(history.length + 1));
    if (id.startsWith(`${history}@`) && Number.isInteger(number)) (held.get(history) ?? held.set(history, new Set()).get(history)).add(number);
  };
  for (const [name, stream] of Object.entries(snapshot.reading_streams ?? {})) {
    held.set(name, held.get(name) ?? new Set());
    for (const occurrence of stream.occurrences ?? []) add(name, occurrence.id);
  }
  for (const [name, renewal] of Object.entries(snapshot.renewals ?? {})) {
    held.set(name, held.get(name) ?? new Set());
    for (const occurrence of renewal.occurrences ?? []) add(name, occurrence);
  }
  return held;
}

// How many of a history's records from `from` to `through` have departed, or
// null when the snapshot does not hold the history's records.
function departedBetween(held, history, from, through) {
  const records = held.get(history);
  if (!records) return null;
  let count = 0;
  for (let number = from; number <= through; number += 1) if (!records.has(number)) count += 1;
  return count;
}

// "read all 59 of departed witness_cave@1 to @59, the last at #412", or "read
// at least 41 of ...", or the one name when the range is one record
// (spec/caveat-lineage-compaction-0.1.md, "Explain, dependents and the view").
function markerText(marker) {
  const { history, read, from, through, departed_at: at, departed_between: total, records } = marker;
  if (records?.length) return `departed ${records.join(', ')} (${records.length === 1 ? 'at' : 'the last at'} #${at})`;
  if (from === through) return `departed ${history}@${from} (at #${at})`;
  const amount = total === read ? `all ${read}` : `at least ${read}`;
  return `read ${amount} of departed ${history}@${from} to @${through}, the last at #${at}`;
}

// A provenance with each marker given how many records its range holds that
// departed and, from a drained archive, which records the holder named.
function annotate(provenance, held, named = () => []) {
  if (!provenance?.departed?.length) return provenance;
  return {
    ...provenance,
    departed: provenance.departed.map(marker => {
      // The archive names a record under the holders that held it when it
      // departed. A marker merged into a later provenance has no such holder,
      // so its records are named only when the archive accounts for all of
      // them.
      const found = named(marker);
      const records = found.length === marker.read ? found : [];
      return {
        ...marker,
        departed_between: departedBetween(held, marker.history, marker.from, marker.through),
        ...(records.length ? { records } : {}),
      };
    }),
  };
}

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
export function explain(snapshot, events = [], { archive = [] } = {}) {
  const qualifiedBy = caveatsByEvidence(snapshot);
  const held = heldRecords(snapshot);
  // A drained archive names exactly which departed records a decision's
  // grounds or basis held (spec/caveat-departure-0.1.md, "The archive").
  const heldBy = new Map();
  for (const entry of archive) {
    for (const holder of entry.holders ?? []) {
      const key = `${holder.kind}\u0000${holder.name}\u0000${holder.in}`;
      (heldBy.get(key) ?? heldBy.set(key, []).get(key)).push(entry);
    }
  }
  const namedBy = (kind, name, field) => marker => (heldBy.get(`${kind}\u0000${name}\u0000${field}`) ?? [])
    .filter(entry => entry.history === marker.history && entry.number >= marker.from && entry.number <= marker.through)
    .sort((a, b) => a.number - b.number)
    .map(entry => entry.record);
  // Withdrawn observations (spec/caveat-withdrawal-0.1.md), by evidence.
  const withdrawals = new Map((snapshot.withdrawals ?? []).map(({ evidence, because, sequence, event }) =>
    [evidence, { because, sequence, event }]));
  // Retired records of windowed histories (spec/caveat-windows-0.1.md): shown
  // as they were, with the sequence at which they retired.
  const retired = snapshot.retired ?? {};
  const retiredAt = id => (Object.hasOwn(retired, id) ? { retired_at: retired[id] } : {});
  const readings = new Map();
  for (const stream of Object.values(snapshot.reading_streams ?? {})) {
    for (const occurrence of stream.occurrences ?? []) readings.set(occurrence.id, occurrence);
  }
  const stances = (snapshot.relations ?? [])
    .filter(relation => relation.relation === 'supports' || relation.relation === 'opposes');
  // Older and stance-only snapshots use edge order. Once neutral observation
  // exists, acquisition order comes from the explicit occurrence ledger. A
  // later stance must not move an earlier neutral observation in that order.
  const relations = snapshot.observations?.length
    ? [...new Set([...snapshot.observations, ...stances.map(item => item.from)])].flatMap(id => {
      const found = stances.filter(item => item.from === id);
      return found.length ? found : [{ from: id, relation: null, to: null }];
    })
    : stances;
  const evidence = relations.map(relation => {
    const reading = readings.get(relation.from);
    return {
      id: relation.from, relation: relation.relation, claim: relation.to,
      value: reading?.value ?? null, sequence: reading?.sequence ?? null, event: reading?.event ?? null,
      caveats: qualifiedBy[relation.from] ?? [],
      withdrawn: withdrawals.get(relation.from) ?? null,
      ...retiredAt(relation.from),
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
      const grounds = annotate(snapshot.commitment_grounds?.[revision.id] ?? none(), held,
        namedBy('commitment', revision.id, 'grounds'));
      return {
        id: revision.id,
        value: snapshot.commitment_bases?.[revision.id]?.value ?? null,
        status: revision.id !== series.current ? 'superseded' : reopened ? 'reopened' : 'in force',
        grounds,
        lineage: annotate(snapshot.commitment_bases?.[revision.id]?.provenance ?? none(), held,
          namedBy('commitment', revision.id, 'basis')),
        // What permitted it, frozen (spec/caveat-permission-0.1.md): not its
        // grounds, though its grant is in its lineage.
        permission: snapshot.commitment_permissions?.[revision.id] ?? null,
        // Grounds withdrawn since the decision was made. The grounds themselves
        // stay as they were.
        withdrawn: grounds.evidence.filter(name => withdrawals.has(name))
          .map(name => ({ evidence: name, ...withdrawals.get(name) })),
        history,
      };
    }),
  }));

  const displayed = Object.entries(snapshot.bindings ?? {}).flatMap(([target, properties]) =>
    Object.entries(properties).map(([property, value]) => ({
      name: `${target}.${property}`, value, cites: annotate(snapshot.binding_explanations?.[target]?.[property] ?? none(), held),
    })));

  return { schema: EXPLAIN_SCHEMA, sequence: snapshot.sequence, elapsed: snapshot.elapsed, events, decisions, evidence, displayed };
}

const list = values => (values.length ? values.join(', ') : 'nothing');
const withdrawnNote = ({ sequence, because }) => `withdrawn at #${sequence} because ${because}`;
const retiredNote = item => (item.retired_at === undefined ? '' : `  retired at #${item.retired_at}`);
const withCaveats = ({ evidence, caveats, departed = [] }) => {
  const names = [...evidence, ...departed.map(markerText)];
  return `${list(names)}${caveats.length ? ` (caveats: ${caveats.join(', ')})` : ''}`;
};
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
      for (const item of revision.withdrawn ?? []) lines.push(`      ${item.evidence} has since been ${withdrawnNote(item)}`);
      if (revision.permission) {
        const { grant, scope, caveats } = revision.permission;
        const checked = scope ? ` for ${show(scope.required)}` : '';
        lines.push(`      permitted by ${grant}${checked}${caveats?.length ? ` (caveats: ${caveats.join(', ')})` : ''}`);
        const revoked = report.evidence.find(item => item.id === grant)?.withdrawn;
        if (revoked) lines.push(`      ${grant} has since been ${withdrawnNote(revoked)}`);
      }
      const also = revision.lineage.evidence.filter(name => !revision.grounds.evidence.includes(name));
      const inGrounds = new Set((revision.grounds.departed ?? []).map(marker => JSON.stringify([marker.history, marker.read, marker.from, marker.through, marker.departed_at])));
      const alsoDeparted = (revision.lineage.departed ?? [])
        .filter(marker => !inGrounds.has(JSON.stringify([marker.history, marker.read, marker.from, marker.through, marker.departed_at])));
      if (also.length || alsoDeparted.length) {
        lines.push(`      could also have been influenced by ${list([...also, ...alsoDeparted.map(markerText)])}`);
      }
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
    const withdrawn = item.withdrawn ? `  ${withdrawnNote(item.withdrawn)}` : '';
    const stance = item.relation === null ? 'observed (no stance)' : `${item.relation} ${item.claim}`;
    lines.push(`  ${item.id}${value} ${stance}${when}${item.caveats.length ? `  caveats: ${item.caveats.join(', ')}` : ''}${withdrawn}${retiredNote(item)}`);
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

// The history and number of `subject` when it names a departed record of a
// windowed history: a reading stream's or renewal's record the session no
// longer holds, below its oldest held one. Null otherwise.
function departedSubject(snapshot, subject) {
  const at = subject.lastIndexOf('@');
  if (at < 1) return null;
  const history = subject.slice(0, at);
  const number = Number(subject.slice(at + 1));
  if (!Number.isInteger(number) || number < 1) return null;
  const windows = new Set(snapshot.windows ?? []);
  if (!windows.has(history)) return null;
  const records = heldRecords(snapshot).get(history);
  if (!records || records.has(number)) return null;
  const oldest = Math.min(...records);
  return number < oldest ? { history, number } : null;
}

/**
 * Everything in `snapshot` that rests on `subject`, which names evidence, a
 * reading stream or a caveat: the decisions and values based on it or that it
 * could have influenced, the decision changes it caused, and the displayed
 * values that cite it or could have been influenced by it. The reverse of
 * `explain`. Throws an Error when the program declares no such name.
 */
export function dependents(snapshot, subject) {
  const departedRecord = departedSubject(snapshot, subject);
  const resolved = departedRecord ? { kind: 'evidence', ids: new Set() } : resolveSubject(snapshot, subject);
  if (!resolved) throw new Error(`${subject} is not evidence, a reading stream or a caveat in this program`);
  // A departed record is named in no provenance; a marker whose range covers
  // its number says only that the value may rest on it
  // (spec/caveat-lineage-compaction-0.1.md).
  const covering = provenance => (departedRecord ? (provenance?.departed ?? [])
    .filter(marker => marker.history === departedRecord.history
      && marker.from <= departedRecord.number && departedRecord.number <= marker.through) : []);
  const via = provenance => {
    const names = resolved.kind === 'caveat' ? provenance?.caveats : provenance?.evidence;
    return (names ?? []).filter(name => resolved.ids.has(name));
  };
  const basis = (primary, primaryLabel, lineage) => {
    const direct = via(primary);
    if (direct.length) return { basis: primaryLabel, via: direct };
    const possible = via(lineage);
    if (possible.length) return { basis: 'lineage', via: possible };
    return covering(primary).length || covering(lineage).length ? { basis: 'may rest on', via: [subject] } : null;
  };

  // Report only withdrawals associated with this query. A caveat selects the
  // evidence it actually qualifies, not every withdrawn input in a value.
  const withdrawalIds = resolved.kind === 'caveat'
    ? new Set((snapshot.relations ?? [])
      .filter(item => item.relation === 'qualifies' && item.from === subject).map(item => item.to))
    : resolved.ids;
  const withdrawals = (snapshot.withdrawals ?? [])
    .filter(item => withdrawalIds.has(item.evidence))
    .map(({ evidence, because, sequence, event }) => ({ evidence, because, sequence, event }));
  const withdrawnVia = (found, provenance) => {
    const ids = new Set(resolved.kind === 'caveat' ? provenance?.evidence ?? [] : found.via);
    return withdrawals.filter(item => ids.has(item.evidence));
  };

  const decisions = explain(snapshot).decisions.flatMap(series => series.revisions.flatMap(revision => {
    // A grant is in the lineage, but its role is permission.
    const grant = revision.permission?.grant;
    const permitted = resolved.kind === 'evidence' && grant && resolved.ids.has(grant) && !via(revision.grounds).length;
    const found = permitted ? { basis: 'permission', via: [grant] } : basis(revision.grounds, 'grounds', revision.lineage);
    return found ? [{ id: revision.id, value: revision.value, status: revision.status, ...found,
      withdrawn: withdrawnVia(found, found.basis === 'grounds' ? revision.grounds : revision.lineage) }] : [];
  }));
  const changes = (snapshot.decision_journal ?? []).flatMap(entry => {
    const names = resolved.kind === 'caveat' ? entry.caveats : entry.because;
    const found = (names ?? []).filter(name => resolved.ids.has(name));
    return found.length ? [{ sequence: entry.sequence, event: entry.event, commitment: entry.commitment, change: entry.change, via: found,
      withdrawn: withdrawnVia({ via: found }, { evidence: entry.because }) }] : [];
  });
  const values = Object.entries(snapshot.qualified_values ?? {}).flatMap(([name, value]) => {
    const found = basis(snapshot.value_grounds?.[name], 'grounds', value.provenance);
    return found ? [{ name, value: value.value, ...found,
      withdrawn: withdrawnVia(found, found.basis === 'grounds' ? snapshot.value_grounds?.[name] : value.provenance) }] : [];
  });
  const displayed = Object.entries(snapshot.bindings ?? {}).flatMap(([target, properties]) =>
    Object.entries(properties).flatMap(([property, value]) => {
      const found = basis(snapshot.binding_explanations?.[target]?.[property], 'cites',
        snapshot.binding_qualifications?.[target]?.[property]);
      return found ? [{ name: `${target}.${property}`, value, ...found,
        withdrawn: withdrawnVia(found, found.basis === 'cites' ? snapshot.binding_explanations?.[target]?.[property]
          : snapshot.binding_qualifications?.[target]?.[property]) }] : [];
    }));

  // Retired records the subject stands for: what still cites them is why
  // they are still here (spec/caveat-windows-0.1.md).
  const retired = Object.entries(snapshot.retired ?? {})
    .filter(([record]) => resolved.ids.has(record))
    .map(([record, sequence]) => ({ record, sequence }));
  return {
    schema: DEPENDENTS_SCHEMA, subject, kind: resolved.kind, sequence: snapshot.sequence,
    ...(departedRecord ? { departed: true } : {}),
    ...(retired.length ? { retired } : {}),
    withdrawals, decisions, changes, values, displayed,
  };
}

/** The dependents report as text for a person. `title` names the program. */
export function formatDependents(report, title = 'the program', events = 0) {
  const through = names => (report.kind === 'caveat' ? `evidence with ${list(names)}` : list(names));
  const label = { grounds: 'based on', cites: 'cites', permission: 'permitted by', lineage: 'could have been influenced by', 'may rest on': 'may rest on' };
  const lines = [`What rests on ${report.subject} in ${title} after ${events} event${events === 1 ? '' : 's'} (sequence ${report.sequence})`];
  const withdrawalText = item => `${item.evidence} withdrawn at #${item.sequence} during ${item.event} because ${item.because}`;
  const section = (heading, items, line) => {
    lines.push('', heading);
    if (!items.length) lines.push('  nothing');
    for (const item of items) {
      lines.push(`  ${line(item)}`);
      for (const withdrawal of item.withdrawn ?? []) lines.push(`    ${withdrawalText(withdrawal)}`);
    }
  };
  if (report.departed) lines.push('', `${report.subject} has departed: the session no longer holds it, so a value whose departure marker covers it may rest on it. A drained archive names the provenances that held it when it departed.`);
  if (report.retired?.length) section('Retired', report.retired, item => `${item.record} retired at #${item.sequence}`);
  if (report.withdrawals?.length) section('Withdrawals', report.withdrawals, withdrawalText);
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
