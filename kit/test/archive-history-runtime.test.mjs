// Collector DRAFT consumer checks. These require the rebuilt collector WASM;
// hand-written payload tests live separately in archive-history.test.mjs.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile, mkdtemp, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {real} from './helpers.mjs';
import {explain, dependents, formatExplanation} from '../lib/explain.mjs';
import {archiveNodeId} from '../lib/archive.mjs';
import {createServer} from '../lib/serve.mjs';
import {runAuthoringOperation} from '../lib/authoring.mjs';

const mutual = await readFile(new URL('../../experiments/departure-gate/withdrawal-fixtures/mutual.cav', import.meta.url), 'utf8');
const self = await readFile(new URL('../../experiments/departure-gate/withdrawal-fixtures/self.cav', import.meta.url), 'utf8');
const nested = `claim seen; evidence a from "a"; renewable a window 1;
evidence template from "template"; readings r from template window 1;
event create; event advance;
on create renew a; on create reveal a supports seen; on create withdraw a because a;
on create when withdrawn(a) sample r = 1 supports seen;
on advance renew a; on advance sample r = 2 supports seen;
`;

function checkArchive(snapshot, archive, expected) {
  const report = explain(snapshot, [], {archive});
  assert.equal(report.archive.authenticated, false);
  assert.ok(report.archive.records.length);
  assert.deepEqual(report.archive.unresolved, []);
  for (const row of report.archive.records) {
    assert.equal(row.status, 'complete', `${row.record}: ${JSON.stringify(row.unresolved)}`);
    const entry = row.entry;
    const leaf = {operation: 'record', source_id: snapshot.source_id, history: entry.history, record: entry.record, departed_at: entry.departed_at};
    assert.ok(archive.some(item => item.kind === 'provenance' && item.id === archiveNodeId(leaf)), 'even disconnected entries have matching source leaves');
  }
  assert.deepEqual(report.archive.withdrawals.map(item => [item.evidence, item.because]), expected);
  return report;
}

test('actual collector self/mutual cycle archives preserve both true reasons and independent live statuses', () => {
  for (const [source, expected] of [[self, [['a@2','a@2']]], [mutual, [['a@2','b@2'], ['b@2','a@2']]]]) {
    const session = real.open(source);
    let restored;
    try {
      assert.equal(session.dispatch('cycle').outcome, 'accepted');
      const first = session.drainArchive();
      assert.equal(session.dispatch('cycle').outcome, 'accepted');
      const archive = [...first, ...session.drainArchive()];
      const snapshot = session.snapshot();
      const report = checkArchive(snapshot, archive, expected);
      assert.deepEqual(report.evidence, explain(snapshot).evidence, 'historical rendering does not rewrite live observations');
      assert.deepEqual(dependents(snapshot, 'a@2', {archive}).withdrawals, [], 'current fields keep current meaning');
      assert.deepEqual(dependents(snapshot, 'a@2', {archive}).archive.withdrawals.map(item => item.evidence), ['a@2']);
      const saved = session.save();
      restored = real.restore(source, saved);
      assert.equal(restored.undrained, 0);
      assert.deepEqual(explain(restored.snapshot(), [], {archive}).archive, report.archive);
      assert.ok(dependents(restored.snapshot(), 'a@2').archive.unresolved.includes('a@2'));
      assert.equal(session.save(), saved, 'archive report is read-only');
      assert.match(formatExplanation(report), /not authenticated or proven exhaustive/);
    } finally { session.close(); restored?.close(); }
  }
});

test('actual simultaneous reading/withdrawal departures join nested payload references across records', () => {
  const session = real.open(nested);
  try {
    assert.equal(session.dispatch('create').outcome, 'accepted');
    assert.equal(session.dispatch('advance').outcome, 'accepted');
    const archive = session.drainArchive(), snapshot = session.snapshot();
    const report = checkArchive(snapshot, archive, [['a@2','a@2']]);
    const reading = report.archive.records.find(row => row.record === 'r@1');
    assert.ok(reading.dependencies.includes('a@2'));
    assert.equal(reading.entry.reading.value, 1);
    const missing = archive.filter(item => item.record !== 'a@2');
    const unavailable = explain(snapshot, [], {archive: missing}).archive.records.find(row => row.record === 'r@1');
    assert.equal(unavailable.status, 'unavailable');
    assert.equal(unavailable.entry, undefined);
  } finally { session.close(); }
});

test('CLI, authoring and explicit serve archive queries carry collector history without server retention', async t => {
  const directory = await mkdtemp(join(tmpdir(), 'caveat-collector-report-'));
  t.after(() => rm(directory, {recursive: true, force: true}));
  const program = join(directory, 'mutual.cav'), eventFile = join(directory, 'events.jsonl');
  const events = [{event: 'cycle'}, {event: 'cycle'}];
  await writeFile(program, mutual); await writeFile(eventFile, events.map(item => JSON.stringify(item)).join('\n'));
  const cli = fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url));
  const result = spawnSync(process.execPath, [cli, 'explain', '--json', program, eventFile], {encoding: 'utf8'});
  assert.equal(result.status, 0, result.stderr);
  const authored = await runAuthoringOperation('caveat_explain', {source: mutual, events});
  for (const report of [JSON.parse(result.stdout), authored.report]) assert.deepEqual(report.archive.withdrawals.map(item => item.because), ['b@2','a@2']);
  const query = spawnSync(process.execPath, [cli, 'dependents', '--json', program, 'a@2', eventFile], {encoding: 'utf8'});
  assert.equal(query.status, 0, query.stderr);
  assert.deepEqual(JSON.parse(query.stdout).archive.reasonForWithdrawals.map(item => item.evidence), ['b@2']);
  const server = createServer({runtime: real, source: mutual});
  const send = request => {
    const result = server.handle(JSON.stringify(request));
    assert.equal(result.response.ok, true, JSON.stringify(result)); return result.response;
  };
  try {
    for (const item of events) send({op: 'dispatch', event: item.event});
    const saved = send({op: 'save'}).save;
    const archive = send({op: 'drainArchive'}).archive;
    const report = send({op: 'dependents', of: 'a@2', archive}).report;
    assert.deepEqual(report.archive.withdrawals.map(item => item.because), ['b@2']);
    assert.deepEqual(report.archive.reasonForWithdrawals.map(item => item.evidence), ['b@2']);
    assert.ok(send({op: 'dependents', of: 'a@2'}).report.archive.unresolved.includes('a@2'));
    assert.equal(send({op: 'save'}).save, saved);
    send({op: 'restore', save: saved});
    assert.equal(send({op: 'undrained'}).undrained, 0);
    assert.deepEqual(send({op: 'dependents', of: 'a@2', archive}).report.archive, report.archive);
    const malformed = send({op: 'explain', archive: [{record: 'bad-one'}, {record: 'bad-two'}]}).report;
    assert.ok(malformed.archive.records.every(row => row.status === 'unavailable'));
  } finally { server.close(); }
});

test('actual decision-series and plain journal archives resolve held decision identities conservatively', () => {
  for (const series of [true, false]) {
    const source = `evidence reason from "reason";
${series ? 'decisions choice limit 2;' : ''} journal window 1;
event init; event revise;
on init reveal reason;
on init commit choice because enough;
on revise reopen choice because reason;
${series ? 'on revise commit choice because enough;' : ''}
`;
    const session = real.open(source);
    let restored;
    try {
      for (const event of ['init', 'revise']) assert.equal(session.dispatch(event).outcome, 'accepted');
      const snapshot = session.snapshot(), archive = session.drainArchive();
      const history = explain(snapshot, [], {archive}).archive;
      assert.equal(history.records.length, series ? 2 : 1);
      assert.ok(history.records.every(row => row.status === 'complete'), JSON.stringify(history.records));
      assert.deepEqual(history.unresolved, []);
      for (const row of history.records) {
        assert.equal(row.entry.journal_entry.decision, 'choice');
        assert.equal(row.entry.journal_entry.commitment, series ? 'choice@1' : 'choice');
      }
      if (series) {
        assert.ok(Object.hasOwn(snapshot.decision_series, 'choice'));
        assert.ok(!snapshot.symbols.some(symbol => symbol.name === 'choice'), 'series declarations are distinct from revision symbols');
      }
      restored = real.restore(source, session.save());
      assert.deepEqual(explain(restored.snapshot(), [], {archive}).archive, history);

      const first = archive.find(item => item.record === 'journal@1' && item.kind !== 'provenance');
      const unavailable = supplied => {
        const row = explain(snapshot, [], {archive: supplied}).archive.records.find(item => item.record === first.record);
        assert.equal(row.status, 'unavailable');
        assert.equal(row.entry, undefined);
        return row;
      };
      unavailable(archive.filter(item => !(item.kind === 'provenance' && item.record === first.record)));
      unavailable([...archive, {...first, retired_at: first.retired_at + 1}]);
      for (const [field, name] of [['decision', 'missing_choice'], ['commitment', 'choice@99']]) {
        const supplied = archive.map(item => item === first ? {...first, journal_entry: {...first.journal_entry, [field]: name}} : item);
        assert.ok(unavailable(supplied).unresolved.includes(name), 'a declaration does not validate a missing name or revision');
      }
    } finally { session.close(); restored?.close(); }
  }
});


test('actual reading archives reject a withdrawal before sampling but accept same-event withdrawal', () => {
  const source = `claim c; evidence reason from "reason"; evidence sensor from "sensor";
readings r from sensor window 1;
event init; event sample; event withdraw; event together;
on init reveal reason;
on sample sample r = 1 supports c;
on withdraw withdraw latest(r) because reason;
on together sample r = 1 supports c;
on together withdraw latest(r) because reason;
`;
  for (const events of [['init', 'sample', 'withdraw', 'sample'], ['init', 'together', 'sample']]) {
    const session = real.open(source);
    try {
      for (const event of events) assert.equal(session.dispatch(event).outcome, 'accepted');
      const snapshot = session.snapshot(), archive = session.drainArchive();
      const authentic = explain(snapshot, [], {archive}).archive;
      assert.ok(authentic.records.every(row => row.status === 'complete'));
      const modified = archive.map(entry => entry.withdrawal
        ? {...entry, withdrawal: {...entry.withdrawal, sequence: 1, event: 'init'}} : entry);
      const report = explain(snapshot, [], {archive: modified}).archive;
      assert.equal(report.records.find(row => row.record === 'r@1').status, 'unavailable');
      assert.deepEqual(report.withdrawals, []);
      assert.deepEqual(report.relations, []);
    } finally { session.close(); }
  }
});
