// Pure browser-compatible archive membership reconstruction. The archive is
// supplied by its host: these hashes check consistency, never authenticity.
const TAG = 'caveat-archive-provenance/0.1';
const HEX = /^[a-f0-9]{64}$/;
const integer = value => Number.isSafeInteger(value) && value > 0;
const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);

// SHA-256 over UTF-8, kept synchronous so explain/dependents remain pure and
// usable in browsers without Node APIs or an asynchronous Web Crypto boundary.
export function archiveSha256(text) {
  const bytes = new TextEncoder().encode(text);
  const padded = new Uint8Array(Math.ceil((bytes.length + 9) / 64) * 64);
  padded.set(bytes); padded[bytes.length] = 128;
  const view = new DataView(padded.buffer);
  view.setUint32(padded.length - 8, Math.floor(bytes.length / 0x20000000));
  view.setUint32(padded.length - 4, bytes.length * 8);
  const state = new Uint32Array([0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19]);
  const constants = [0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2];
  const rotate = (value, bits) => (value >>> bits) | (value << (32 - bits));
  const words = new Uint32Array(64);
  for (let offset = 0; offset < padded.length; offset += 64) {
    for (let index = 0; index < 16; index++) words[index] = view.getUint32(offset + index * 4);
    for (let index = 16; index < 64; index++) {
      const a = words[index - 15], b = words[index - 2];
      words[index] = words[index - 16] + (rotate(a,7) ^ rotate(a,18) ^ (a >>> 3)) + words[index - 7] + (rotate(b,17) ^ rotate(b,19) ^ (b >>> 10));
    }
    let [a,b,c,d,e,f,g,h] = state;
    for (let index = 0; index < 64; index++) {
      const first = (h + (rotate(e,6) ^ rotate(e,11) ^ rotate(e,25)) + ((e & f) ^ (~e & g)) + constants[index] + words[index]) >>> 0;
      const second = ((rotate(a,2) ^ rotate(a,13) ^ rotate(a,22)) + ((a & b) ^ (a & c) ^ (b & c))) >>> 0;
      [a,b,c,d,e,f,g,h] = [(first + second) >>> 0,a,b,c,(d + first) >>> 0,e,f,g];
    }
    for (const [index, value] of [a,b,c,d,e,f,g,h].entries()) state[index] += value;
  }
  return [...state].map(value => value.toString(16).padStart(8, '0')).join('');
}

export function archiveNodeId(node) {
  const tuple = node.operation === 'record'
    ? [TAG, 'record', node.source_id, node.history, node.record, node.departed_at]
    : [TAG, 'union', node.history, node.parents];
  return archiveSha256(JSON.stringify(tuple));
}

// Sorting object keys makes identical duplicate JSON records order-independent.
// Conflicting duplicates poison only that record/root, and never pick a winner.
function canonical(value) {
  if (Array.isArray(value)) return '[' + value.map(canonical).join(',') + ']';
  if (object(value)) return '{' + Object.keys(value).sort().map(key => JSON.stringify(key) + ':' + canonical(value[key])).join(',') + '}';
  return JSON.stringify(value);
}

export function archiveResolver(snapshot, archive = []) {
  const nodes = new Map(), records = new Map(), invalid = new Set(), seenRecords = new Map();
  const entries = Array.isArray(archive) ? archive : [];
  for (const entry of entries) {
    if (!object(entry)) continue;
    if (entry.kind === 'provenance') {
      if (typeof entry.id !== 'string') continue;
      const previous = nodes.get(entry.id);
      try { if (previous && canonical(previous) !== canonical(entry)) invalid.add(entry.id); }
      catch { invalid.add(entry.id); }
      nodes.set(entry.id, entry);
    } else if (typeof entry.record === 'string') {
      const previous = seenRecords.get(entry.record);
      try { if (previous && canonical(previous) !== canonical(entry)) invalid.add('record:' + entry.record); }
      catch { invalid.add('record:' + entry.record); }
      seenRecords.set(entry.record, entry);
      if (typeof entry.history === 'string' && integer(entry.number)
        && entry.record === `${entry.history}@${entry.number}` && integer(entry.departed_at)
        && integer(entry.retired_at) && entry.retired_at <= entry.departed_at) records.set(entry.record, entry);
    }
  }
  const cache = new Map();
  function expand(root, history) {
    const key = history + '\0' + root;
    if (cache.has(key)) return cache.get(key);
    // Iterative traversal is safe for histories deeper than the JS call stack.
    const pending = [[root, false]], active = new Set(), visited = new Set(), found = new Map();
    let complete = true;
    while (pending.length && complete) {
      const [id, leaving] = pending.pop();
      if (leaving) { active.delete(id); visited.add(id); continue; }
      if (visited.has(id)) continue;
      if (active.has(id) || invalid.has(id) || !HEX.test(id)) { complete = false; break; }
      const node = nodes.get(id);
      if (!node || node.history !== history || !['record', 'union'].includes(node.operation)) { complete = false; break; }
      const allowed = node.operation === 'record'
        ? ['kind', 'id', 'source_id', 'history', 'operation', 'record', 'departed_at']
        : ['kind', 'id', 'history', 'operation', 'parents'];
      if (Object.keys(node).some(key => !allowed.includes(key))) { complete = false; break; }
      if (node.operation === 'record') {
        const record = records.get(node.record);
        if (typeof node.source_id !== 'string' || node.source_id !== snapshot.source_id || !record || invalid.has('record:' + node.record)
          || record.history !== history || record.departed_at !== node.departed_at
          || record.departed_at > snapshot.sequence || archiveNodeId(node) !== id) { complete = false; break; }
        if (Object.hasOwn(snapshot.retired ?? {}, record.record)
          || (snapshot.reading_streams?.[history]?.occurrences ?? []).some(item => item.id === record.record)
          || (snapshot.renewals?.[history]?.occurrences ?? []).includes(record.record)) { complete = false; break; }
        found.set(record.number, record); visited.add(id);
      } else {
        if (!Array.isArray(node.parents) || node.parents.length < 2 || node.parents.some((parent, index) =>
          typeof parent !== 'string' || !HEX.test(parent) || (index > 0 && node.parents[index - 1] >= parent))) { complete = false; break; }
        if (archiveNodeId(node) !== id) { complete = false; break; }
        active.add(id); pending.push([id, true]);
        for (const parent of node.parents) pending.push([parent, false]);
      }
    }
    const result = complete ? [...found.values()].sort((a, b) => a.number - b.number) : null;
    cache.set(key, result);
    return result;
  }
  return marker => {
    if (!object(marker) || typeof marker.history !== 'string' || typeof marker.archive_ref !== 'string' || !HEX.test(marker.archive_ref)
      || !integer(marker.from) || !integer(marker.through) || !integer(marker.read)
      || !integer(marker.departed_at) || marker.departed_at > snapshot.sequence) return null;
    const found = expand(marker.archive_ref, marker.history);
    if (!found?.length || found.length < marker.read || found[0].number !== marker.from
      || found.at(-1).number !== marker.through || found.reduce((latest, entry) => Math.max(latest, entry.departed_at), 0) !== marker.departed_at) return null;
    return found.map(entry => entry.record);
  };
}

// Draft historical reporting. This is deliberately separate from membership:
// leaves bind a source/name/departure, not payload bytes or archive inventory.
// "complete" below means the referenced closure of the supplied entries only.
const relationKinds = new Set(['supports', 'opposes', 'qualifies', 'in_context', 'retains', 'reopens', 'relies_on']);
const strings = value => Array.isArray(value) && value.every(item => typeof item === 'string');
const recordOrder = (a, b) => (typeof a.history === 'string' ? a.history : '').localeCompare(typeof b.history === 'string' ? b.history : '')
  || (integer(a.number) && integer(b.number) ? a.number - b.number : 0) || a.record.localeCompare(b.record);

/** Internal report builder; never substitutes archived data into a live snapshot. */
export function archiveHistory(snapshot, archive = [], required = [], query = null) {
  const entries = new Map(), conflicts = new Set();
  for (const item of Array.isArray(archive) ? archive : []) {
    if (!object(item) || item.kind === 'provenance' || typeof item.record !== 'string') continue;
    try {
      if (entries.has(item.record) && canonical(entries.get(item.record)) !== canonical(item)) conflicts.add(item.record);
    } catch { conflicts.add(item.record); }
    entries.set(item.record, item);
  }
  const resolve = archiveResolver(snapshot, archive);
  const held = new Set((snapshot.symbols ?? []).map(item => item.name));
  for (const name of snapshot.observations ?? []) held.add(name);
  for (const name of Object.keys(snapshot.retired ?? {})) held.add(name);
  for (const stream of Object.values(snapshot.reading_streams ?? {})) for (const item of stream.occurrences ?? []) held.add(item.id);
  for (const renewal of Object.values(snapshot.renewals ?? {})) for (const name of renewal.occurrences ?? []) held.add(name);
  for (const item of snapshot.commitments ?? []) held.add(item.action);
  // Journal decisions name the series declaration, separate from its revision symbols.
  for (const name of Object.keys(snapshot.decision_series ?? {})) held.add(name);
  const windows = new Set(snapshot.windows ?? []);
  const newest = new Map();
  const allocated = (history, name) => {
    const ordinal = Number(name.slice(history.length + 1));
    if (name.startsWith(history + '@') && integer(ordinal)) newest.set(history, Math.max(newest.get(history) ?? 0, ordinal));
  };
  for (const [history, stream] of Object.entries(snapshot.reading_streams ?? {})) for (const item of stream.occurrences ?? []) allocated(history, item.id);
  for (const [history, renewal] of Object.entries(snapshot.renewals ?? {})) for (const name of renewal.occurrences ?? []) allocated(history, name);
  // Only immutable, explicitly recorded creation times constrain references.
  // Declared evidence/renewals have no such timestamp here; equal sequences do
  // not establish order within an event, and marker departure times are later.
  const created = new Map(), revisions = new Map();
  const creation = (name, sequence) => {
    if (typeof name === 'string' && integer(sequence)) created.set(name, Math.max(created.get(name) ?? 0, sequence));
  };
  for (const stream of Object.values(snapshot.reading_streams ?? {})) {
    for (const reading of stream.occurrences ?? []) creation(reading.id, reading.sequence);
  }
  for (const [decision, series] of Object.entries(snapshot.decision_series ?? {})) {
    for (const revision of series.revisions ?? []) {
      creation(revision.id, revision.sequence);
      revisions.set(revision.id, {...revision, decision});
    }
  }
  for (const entry of entries.values()) if (entry.reading?.id === entry.record) creation(entry.record, entry.reading.sequence);
  const future = (name, sequence) => integer(sequence) && created.has(name) && created.get(name) > sequence;
  const rows = new Map(), relations = new Map();
  const markerMissing = marker => 'archive_ref:' + (typeof marker?.archive_ref === 'string' ? marker.archive_ref
    : `${typeof marker?.history === 'string' ? marker.history : '?'}@${integer(marker?.from) ? marker.from : '?'}`);
  const requiredNames = new Set(), requiredMissing = new Set();
  for (const item of required) {
    if (typeof item === 'string') requiredNames.add(item);
    else {
      const names = resolve(item);
      if (names) for (const name of names) requiredNames.add(name);
      else requiredMissing.add(markerMissing(item));
    }
  }
  for (const entry of entries.values()) {
    const row = {record: entry.record, entry, dependencies: new Set(), unresolved: new Set()};
    rows.set(entry.record, row);
    const need = (name, sequence = entry.departed_at) => {
      if (typeof name !== 'string' || !name) { row.unresolved.add(entry.record); return; }
      if (name !== entry.record) row.dependencies.add(name);
      if (future(name, sequence)) row.unresolved.add(name);
    };
    const names = (value, sequence = entry.departed_at) => {
      if (!strings(value)) { row.unresolved.add(entry.record); return; }
      for (const name of value) need(name, sequence);
    };
    const provenance = (value, sequence = entry.departed_at) => {
      if (!object(value) || Object.keys(value).some(key => !['evidence', 'caveats', 'inherited', 'departed'].includes(key))) {
        row.unresolved.add(entry.record); return;
      }
      names(value.evidence, sequence); names(value.caveats, sequence);
      if (value.inherited !== undefined) names(value.inherited, sequence);
      if (value.departed !== undefined) {
        if (!Array.isArray(value.departed)) { row.unresolved.add(entry.record); return; }
        for (const marker of value.departed) {
          const members = resolve(marker);
          if (members) for (const name of members) need(name, sequence);
          else row.unresolved.add(markerMissing(marker));
        }
      }
    };
    const leaf = {operation: 'record', source_id: snapshot.source_id, history: entry.history, record: entry.record, departed_at: entry.departed_at};
    const marker = typeof entry.history === 'string' && integer(entry.number) && integer(entry.departed_at)
      ? {history: entry.history, from: entry.number, through: entry.number, read: 1, departed_at: entry.departed_at, archive_ref: archiveNodeId(leaf)} : null;
    const allowed = ['record', 'history', 'number', 'retired_at', 'departed_at', 'relations', 'qualifications', 'reading', 'journal_entry', 'withdrawal', 'holders'];
    if (conflicts.has(entry.record) || held.has(entry.record) || !windows.has(entry.history)
      || ((entry.history !== 'journal' || Object.hasOwn(snapshot.reading_streams ?? {}, 'journal') || Object.hasOwn(snapshot.renewals ?? {}, 'journal'))
        && (!newest.has(entry.history) || entry.number >= newest.get(entry.history)
        || (entry.number === 1 && Object.hasOwn(snapshot.renewals ?? {}, entry.history))))
      || Object.keys(entry).some(key => !allowed.includes(key)) || !resolve(marker)) row.unresolved.add(entry.record);
    if (!Array.isArray(entry.holders) || entry.holders.some(holder => !object(holder)
      || !['state','commitment','series','stream','scheduled','cue','qualification'].includes(holder.kind)
      || typeof holder.name !== 'string' || typeof holder.in !== 'string')) row.unresolved.add(entry.record);
    if (entry.relations !== undefined) {
      if (!Array.isArray(entry.relations)) row.unresolved.add(entry.record);
      else for (const relation of entry.relations) {
        if (!strings(relation) || relation.length !== 3 || !relationKinds.has(relation[1])
          || (relation[0] !== entry.record && relation[2] !== entry.record)) { row.unresolved.add(entry.record); continue; }
        need(relation[0]); need(relation[2]);
        const key = JSON.stringify(relation);
        if (!relations.has(key)) relations.set(key, {from: relation[0], relation: relation[1], to: relation[2], owners: new Set()});
        relations.get(key).owners.add(entry.record);
      }
    }
    if (entry.qualifications !== undefined) {
      if (!object(entry.qualifications)) row.unresolved.add(entry.record);
      else for (const value of Object.values(entry.qualifications)) provenance(value);
    }
    if (entry.reading !== undefined) {
      const reading = entry.reading;
      if (!object(reading) || reading.id !== entry.record || reading.ordinal !== entry.number
        || !integer(reading.sequence) || reading.sequence > entry.retired_at || typeof reading.event !== 'string'
        || !Number.isFinite(reading.value) || typeof reading.relation !== 'string' || typeof reading.claim !== 'string') row.unresolved.add(entry.record);
      else { provenance(reading.provenance, reading.sequence); if (reading.claim) need(reading.claim, reading.sequence); }
    }
    if (entry.withdrawal !== undefined) {
      const withdrawal = entry.withdrawal;
      if (!object(withdrawal) || withdrawal.evidence !== entry.record || !integer(withdrawal.sequence)
        || withdrawal.sequence > entry.departed_at || typeof withdrawal.event !== 'string') row.unresolved.add(entry.record);
      else { need(withdrawal.evidence, withdrawal.sequence); need(withdrawal.because, withdrawal.sequence); }
    }
    if (entry.journal_entry !== undefined) {
      const journal = entry.journal_entry;
      if (!object(journal) || !integer(journal.sequence) || journal.sequence > entry.retired_at
        || !['committed','reopened'].includes(journal.change) || typeof journal.event !== 'string') row.unresolved.add(entry.record);
      else {
        need(journal.decision, journal.sequence); need(journal.commitment, journal.sequence);
        names(journal.because, journal.sequence); names(journal.caveats, journal.sequence);
        if (journal.permitted_by !== undefined) need(journal.permitted_by, journal.sequence);
        const revision = revisions.get(journal.commitment);
        if (revision && (revision.decision !== journal.decision || (journal.change === 'committed'
          && (revision.sequence !== journal.sequence || revision.event !== journal.event)))) row.unresolved.add(journal.commitment);
      }
    }
  }
  // An edge is archived once, under either departing endpoint. Join it from
  // both records without inventing an inverse semantic relationship.
  for (const relation of relations.values()) {
    for (const [name, other] of [[relation.from, relation.to], [relation.to, relation.from]]) {
      const row = rows.get(name);
      if (!row) continue;
      if (other !== name) row.dependencies.add(other);
      for (const owner of relation.owners) if (owner !== name) row.dependencies.add(owner);
    }
  }
  // Propagate invalid/missing dependencies backwards once. Valid self/mutual
  // cycles terminate without recursive traversal or quadratic closure copies.
  const reverse = new Map(), pending = [];
  for (const row of rows.values()) {
    for (const name of row.dependencies) {
      if (future(name, row.entry.departed_at)) row.unresolved.add(name);
      if (rows.has(name)) (reverse.get(name) ?? reverse.set(name, new Set()).get(name)).add(row.record);
      else if (!held.has(name)) row.unresolved.add(name);
    }
    if (row.unresolved.size) pending.push(row.record);
  }
  const bad = new Set(pending);
  for (let index = 0; index < pending.length; index++) {
    for (const name of reverse.get(pending[index]) ?? []) {
      rows.get(name).unresolved.add(pending[index]);
      if (!bad.has(name)) { bad.add(name); pending.push(name); }
    }
  }
  const selected = new Set(requiredNames);
  const selects = name => query?.names.has(name) || (query?.histories?.has(entries.get(name)?.history));
  const selectedEvidence = new Set();
  if (query?.kind === 'caveat') {
    for (const relation of [...(snapshot.relations ?? []), ...relations.values()]) {
      if (relation.relation === 'qualifies' && query.names.has(relation.from)) selectedEvidence.add(relation.to);
    }
  }
  const matches = name => query?.kind === 'caveat' ? selectedEvidence.has(name) : selects(name);
  for (const row of rows.values()) {
    if (!query || matches(row.record) || matches(row.entry.withdrawal?.because)) selected.add(row.record);
  }
  // Include cross-record payloads needed to review each selected record.
  const queue = [...selected];
  for (let index = 0; index < queue.length; index++) {
    const row = rows.get(queue[index]);
    if (!row) { if (!held.has(queue[index])) requiredMissing.add(queue[index]); continue; }
    for (const name of row.dependencies) if (rows.has(name) && !selected.has(name)) { selected.add(name); queue.push(name); }
  }
  const result = [...selected].filter(name => rows.has(name)).map(name => rows.get(name)).sort((a, b) => recordOrder(a.entry, b.entry));
  if (!result.length && !requiredMissing.size) return null;
  const complete = new Set(result.filter(row => !bad.has(row.record)).map(row => row.record));
  const withdrawals = result.filter(row => complete.has(row.record) && row.entry.withdrawal && (!query || matches(row.record)))
    .map(row => row.entry.withdrawal).sort((a, b) => a.sequence - b.sequence || a.evidence.localeCompare(b.evidence));
  const reasonForWithdrawals = query ? result.filter(row => complete.has(row.record) && row.entry.withdrawal && matches(row.entry.withdrawal.because))
    .map(row => row.entry.withdrawal).sort((a, b) => a.sequence - b.sequence || a.evidence.localeCompare(b.evidence)) : undefined;
  return {
    scope: 'provided records and their referenced closure', authenticated: false,
    records: result.map(row => ({record: row.record, status: bad.has(row.record) ? 'unavailable' : 'complete',
      dependencies: [...row.dependencies].sort(), unresolved: [...row.unresolved].sort(),
      ...(!bad.has(row.record) ? {entry: row.entry} : {})})),
    unresolved: [...requiredMissing].sort(), withdrawals,
    ...(reasonForWithdrawals ? {reasonForWithdrawals} : {}),
    relations: [...relations.values()].filter(item => [...item.owners].some(name => complete.has(name))
      && (!query || (query.kind === 'caveat' ? query.names.has(item.from) : matches(item.from) || matches(item.to))))
      .map(({from, relation, to}) => ({from, relation, to}))
      .sort((a, b) => a.from.localeCompare(b.from) || a.relation.localeCompare(b.relation) || a.to.localeCompare(b.to)),
  };
}
