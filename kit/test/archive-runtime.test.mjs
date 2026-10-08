// Actual Rust/WASM -> kit proof reconstruction across copy, replacement,
// overlapping reads, restore and rejected transactions. No hand-authored DAG.
import test from 'node:test';
import assert from 'node:assert/strict';
import {real} from './helpers.mjs';
import {explain, dependents} from '../lib/explain.mjs';
import {archiveNodeId} from '../lib/archive.mjs';
import {runAuthoringOperation} from '../lib/authoring.mjs';
import {createServer} from '../lib/serve.mjs';
import {spawnSync} from 'node:child_process';
import {mkdtemp, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';

// Same minimal information-loss example as runtime/tests/departure.rs.
const source = `
claim seen; evidence glimpse from "glimpse";
readings s from glimpse window 1;
state left = 0; state right = 0; state chosen = 0; state reused = 0;
event look v min 1 max 5; event pick side min 0 max 1; event refuse;
on look sample s = v supports seen;
on look when v < 5 and v != 3 and latest(s) > 0 set left = left;
on look when v < 5 and v != 2 and latest(s) > 0 set right = right;
on look when v < 5 and v != 3 and latest(s) > 0 set reused = reused;
on pick when side == 0 set chosen = left;
on pick when side == 0 set reused = left;
on pick when side == 1 set chosen = right;
on pick when side == 1 set reused = right;
on refuse set chosen = left + right;
on refuse reject "fixture refusal";
bind hud.chosen = chosen;
bind hud.union = left + right;
bind hud.overlap = left + reused;
`;

test('actual WASM archive roots explain copied, replaced and overlapping departed membership after restore', () => {
  const session = real.open(source);
  let restored;
  try {
    for (let v = 1; v <= 5; v++) assert.equal(session.dispatch('look', {v}).outcome, 'accepted');
    const archive = session.drainArchive();
    assert.ok(archive.some(item => item.kind === 'provenance'), 'rebuilt WASM must export real provenance nodes');
    for (const item of archive.filter(item => item.kind === 'provenance')) assert.equal(archiveNodeId(item), item.id, 'Rust and browser-compatible JS canonical hashes agree');
    const display = (current, name) => explain(current, [], {archive}).displayed.find(item => item.name === name).lineage.departed[0].records;
    const before = session.save();
    const readOnly = session.undrained;
    session.snapshot(); session.view();
    assert.equal(session.save(), before);
    assert.equal(session.undrained, readOnly);
    assert.equal(session.dispatch('pick', {side: 0}).outcome, 'accepted'); archive.push(...session.drainArchive());
    let snapshot = session.snapshot();
    assert.deepEqual(display(snapshot, 'hud.chosen'), ['s@1','s@2','s@4']);
    assert.deepEqual(display(snapshot, 'hud.union'), ['s@1','s@2','s@3','s@4']);
    assert.deepEqual(display(snapshot, 'hud.overlap'), ['s@1','s@2','s@4']);
    assert.equal(dependents(snapshot, 's@3', {archive}).values.some(item => item.name === 'chosen'), false);
    assert.equal(dependents(snapshot, 's@2', {archive}).values.find(item => item.name === 'chosen').basis, 'lineage');

    assert.equal(session.dispatch('pick', {side: 1}).outcome, 'accepted'); archive.push(...session.drainArchive());
    snapshot = session.snapshot();
    assert.deepEqual(display(snapshot, 'hud.chosen'), ['s@1','s@3','s@4']);
    assert.deepEqual(display(snapshot, 'hud.overlap'), ['s@1','s@2','s@3','s@4']);
    assert.equal(dependents(snapshot, 's@2', {archive}).values.some(item => item.name === 'reused'), false);
    restored = real.restore(source, session.save());
    assert.equal(restored.undrained, 0);
    assert.deepEqual(restored.snapshot(), snapshot);
    assert.deepEqual(display(restored.snapshot(), 'hud.chosen'), ['s@1','s@3','s@4']);
    assert.equal(explain(restored.snapshot()).displayed.find(item => item.name === 'hud.chosen').lineage.departed[0].records, undefined);
    const saved = restored.save();
    assert.equal(restored.dispatch('refuse').outcome, 'rejected');
    assert.equal(restored.save(), saved);
    assert.deepEqual(restored.drainArchive(), []);
    assert.equal(restored.dispatch('pick', {side: 0}).outcome, 'accepted'); archive.push(...restored.drainArchive());
    assert.deepEqual(display(restored.snapshot(), 'hud.chosen'), ['s@1','s@2','s@4']);
  } finally { session.close(); restored?.close(); }
});

test('public CLI and inline authoring replay preserve exact sparse archive explanations', async t => {
  const directory = await mkdtemp(join(tmpdir(), 'caveat-archive-replay-'));
  t.after(() => rm(directory, {recursive: true, force: true}));
  const program = join(directory, 'archive.cav'), eventFile = join(directory, 'events.jsonl');
  await writeFile(program, source);
  const events = [1,2,3,4,5].map(v => ({event: 'look', payload: {v}}));
  events.push({event: 'pick', payload: {side: 0}}, {event: 'refuse'}, {event: 'pick', payload: {side: 1}});
  await writeFile(eventFile, events.map(event => JSON.stringify(event)).join('\n'));
  const cli = fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url));
  const explained = spawnSync(process.execPath, [cli, 'explain', '--json', program, eventFile], {encoding: 'utf8'});
  assert.equal(explained.status, 0, explained.stderr);
  const authored = await runAuthoringOperation('caveat_explain', {source, events});
  for (const report of [JSON.parse(explained.stdout), authored.report]) {
    const display = report.displayed.find(item => item.name === 'hud.chosen');
    assert.deepEqual(display.lineage.departed[0].records, ['s@1','s@3','s@4']);
    assert.equal(display.lineage.departed[0].archive_status, 'complete');
  }
  const queried = spawnSync(process.execPath, [cli, 'dependents', '--json', program, 's@2', eventFile], {encoding: 'utf8'});
  assert.equal(queried.status, 0, queried.stderr);
  const authoredQuery = await runAuthoringOperation('caveat_dependents', {source, subject: 's@2', events});
  for (const report of [JSON.parse(queried.stdout), authoredQuery.report]) {
    assert.equal(report.values.some(item => item.name === 'chosen'), false);
    assert.equal(report.values.find(item => item.name === 'left').basis, 'lineage');
  }
});

test('actual renewable evidence archive roots reconstruct sparse membership after restore', () => {
  const renewable = source.replace('readings s from glimpse window 1;', 'renewable glimpse window 1;')
    .replace('on look sample s = v supports seen;', 'on look renew glimpse;\non look reveal glimpse supports seen;')
    .replaceAll('latest(s) > 0', 'observed(glimpse)');
  const session = real.open(renewable);
  let restored;
  try {
    const archive = [];
    for (let v = 1; v <= 5; v++) {
      assert.equal(session.dispatch('look', {v}).outcome, 'accepted');
      archive.push(...session.drainArchive());
    }
    assert.equal(session.dispatch('pick', {side: 1}).outcome, 'accepted');
    archive.push(...session.drainArchive());
    restored = real.restore(renewable, session.save());
    const snapshot = restored.snapshot();
    assert.deepEqual(explain(snapshot, [], {archive}).displayed.find(item => item.name === 'hud.chosen').lineage.departed[0].records,
      ['glimpse@2', 'glimpse@4', 'glimpse@5']);
    assert.equal(dependents(snapshot, 'glimpse@3', {archive}).values.some(item => item.name === 'chosen'), false);
    assert.equal(dependents(snapshot, 'glimpse@4', {archive}).values.find(item => item.name === 'chosen').basis, 'lineage');
    assert.equal(restored.undrained, 0);
  } finally { session.close(); restored?.close(); }
});

test('serve hands archive items to its host and reconstructs only explicitly supplied history', async t => {
  const server = createServer({runtime: real, source});
  t.after(() => server.close());
  const send = request => server.handle(JSON.stringify(request));
  const accepted = request => {
    const result = send(request);
    assert.equal(result.exit, null);
    assert.equal(result.response.ok, true, JSON.stringify(result.response));
    return result.response;
  };
  const events = [1,2,3,4,5].map(v => ({op: 'dispatch', event: 'look', payload: {v}}));
  events.push({op: 'dispatch', event: 'pick', payload: {side: 1}});
  for (const event of events) assert.equal(accepted(event).outcome, 'accepted');
  const saved = accepted({op: 'save'}).save;
  const snapshot = accepted({op: 'snapshot'}).snapshot;
  const pending = accepted({op: 'undrained'}).undrained;
  assert.ok(pending > 4);
  const marker = report => report.displayed.find(item => item.name === 'hud.chosen').lineage.departed[0];
  assert.equal(marker(accepted({op: 'explain'}).report).archive_status, 'unavailable');
  for (const archive of [null, {}, '', [null], [[]], [1]]) {
    for (const op of ['explain', 'dependents']) {
      const result = send({op, ...(op === 'dependents' ? {of: 's@2'} : {}), archive});
      assert.equal(result.exit, null);
      assert.equal(result.response.error.kind, 'request');
      assert.match(result.response.error.message, /archive.*array/);
    }
  }
  assert.equal(send({op: 'drainArchive', archive: []}).response.error.kind, 'request');
  assert.equal(accepted({op: 'undrained'}).undrained, pending);
  assert.equal(accepted({op: 'save'}).save, saved);
  const archive = accepted({op: 'drainArchive'}).archive;
  assert.equal(archive.length, pending);
  assert.ok(archive.some(item => item.kind === 'provenance'));
  assert.deepEqual(accepted({op: 'drainArchive'}).archive, []);
  assert.equal(accepted({op: 'undrained'}).undrained, 0);
  assert.equal(accepted({op: 'save'}).save, saved);
  assert.deepEqual(accepted({op: 'snapshot'}).snapshot, snapshot);
  assert.deepEqual(marker(accepted({op: 'explain', archive}).report).records, ['s@1', 's@3', 's@4']);
  assert.equal(accepted({op: 'dependents', of: 's@2', archive}).report.values.some(item => item.name === 'chosen'), false);
  const incomplete = archive.filter(item => item.record !== 's@3');
  assert.equal(marker(accepted({op: 'explain', archive: incomplete}).report).archive_status, 'unavailable');
  assert.equal(accepted({op: 'dependents', of: 's@2', archive: incomplete}).report.values.find(item => item.name === 'chosen').basis, 'may rest on');
  // Supplying an archive does not cache it for the next request.
  assert.equal(marker(accepted({op: 'explain'}).report).archive_status, 'unavailable');
  assert.equal(accepted({op: 'dispatch', event: 'look', payload: {v: 5}}).outcome, 'accepted');
  assert.ok(accepted({op: 'undrained'}).undrained > 0);
  accepted({op: 'restore', save: saved});
  assert.equal(accepted({op: 'undrained'}).undrained, 0);
  assert.deepEqual(accepted({op: 'drainArchive'}).archive, []);
  assert.deepEqual(marker(accepted({op: 'explain', archive}).report).records, ['s@1', 's@3', 's@4']);

  const directory = await mkdtemp(join(tmpdir(), 'caveat-archive-serve-'));
  t.after(() => rm(directory, {recursive: true, force: true}));
  const program = join(directory, 'archive.cav');
  await writeFile(program, source);
  const requests = [...events, {id: 'pending', op: 'undrained'}, {id: 'drained', op: 'drainArchive'},
    {id: 'empty', op: 'undrained'}, {id: 'save', op: 'save'}, {id: 'exact', op: 'explain', archive},
    {id: 'query', op: 'dependents', of: 's@2', archive}, {id: 'missing', op: 'explain'},
    {op: 'restore', save: saved}, {id: 'restored', op: 'undrained'}, {op: 'close'}];
  const cli = fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url));
  const result = spawnSync(process.execPath, [cli, 'serve', program], {
    encoding: 'utf8', input: requests.map(request => JSON.stringify(request)).join('\n') + '\n',
  });
  assert.equal(result.status, 0, result.stderr);
  const lines = result.stdout.trim().split('\n').map(line => JSON.parse(line));
  assert.equal(lines[0].ready, true);
  assert.ok(lines.slice(1).every(line => line.ok));
  const response = id => lines.find(line => line.id === id);
  assert.equal(response('pending').undrained, pending);
  assert.deepEqual(response('drained').archive, archive);
  assert.equal(response('empty').undrained, 0);
  assert.equal(response('save').save, saved);
  assert.deepEqual(marker(response('exact').report).records, ['s@1', 's@3', 's@4']);
  assert.equal(response('query').report.values.some(item => item.name === 'chosen'), false);
  assert.equal(marker(response('missing').report).archive_status, 'unavailable');
  assert.equal(response('restored').undrained, 0);
});

test('departed gaps after older pinned records remain queryable for readings and renewals', () => {
  const reading = `claim seen; evidence glimpse from "glimpse";
readings s from glimpse window 1; state held = 0; state log = 0;
event look v min 1 max 3;
on look sample s = v supports seen;
on look when v == 1 set held = latest(s);
on look when latest(s) > 0 set log = log;
bind hud.log = log;`;
  const renewal = reading.replace('readings s from glimpse window 1;', 'renewable glimpse window 1;')
    .replace('on look sample s = v supports seen;', 'on look renew glimpse;\non look reveal glimpse supports seen;')
    .replace('set held = latest(s)', 'set held = qualified(1, glimpse)')
    .replace('latest(s) > 0', 'observed(glimpse)');
  for (const [program, pinned, departed, history] of [[reading, 's@1', 's@2', 's'], [renewal, 'glimpse@2', 'glimpse@3', 'glimpse']]) {
    const session = real.open(program);
    try {
      const archive = [];
      for (let v = 1; v <= 3; v++) {
        assert.equal(session.dispatch('look', {v}).outcome, 'accepted');
        archive.push(...session.drainArchive());
      }
      const snapshot = session.snapshot();
      assert.ok(Object.hasOwn(snapshot.retired, pinned), 'older directly held record remains pinned');
      assert.equal(Object.hasOwn(snapshot.retired, departed), false);
      const exact = dependents(snapshot, departed, {archive});
      assert.equal(exact.departed, true);
      assert.equal(exact.values.find(item => item.name === 'log').basis, 'lineage');
      assert.deepEqual(exact.displayed.find(item => item.name === 'hud.log').via, [departed]);
      assert.equal(exact.values.some(item => item.name === 'held'), false);
      assert.equal(dependents(snapshot, departed).values.find(item => item.name === 'log').basis, 'may rest on');
      assert.equal(dependents(snapshot, pinned, {archive}).departed, undefined);
      for (const malformed of [`${history}@01`, `${history}@1e0`, `${history}@1.0`, `${history}@9007199254740992`, `${history}@99`]) {
        assert.throws(() => dependents(snapshot, malformed, {archive}), /not evidence/);
      }
      if (history === 'glimpse') assert.throws(() => dependents(snapshot, 'glimpse@1', {archive}), /not evidence/);
    } finally { session.close(); }
  }
});

test('stream, evidence-template and renewal queries include departed-only holders', () => {
  const reading = `claim seen; evidence glimpse from "glimpse";
readings s from glimpse window 1; state old = 0; event look v min 1 max 3;
on look sample s = v supports seen;
on look when v == 1 and latest(s) > 0 set old = old;
bind hud.old = old;`;
  const renewal = reading.replace('readings s from glimpse window 1;', 'renewable glimpse window 1;')
    .replace('on look sample s = v supports seen;', 'on look renew glimpse;\non look reveal glimpse supports seen;')
    .replace('latest(s) > 0', 'observed(glimpse)');
  for (const [program, subjects, record] of [[reading, ['s', 'glimpse'], 's@1'], [renewal, ['glimpse'], 'glimpse@2']]) {
    const session = real.open(program);
    try {
      const archive = [];
      for (let v = 1; v <= 3; v++) {
        assert.equal(session.dispatch('look', {v}).outcome, 'accepted');
        archive.push(...session.drainArchive());
      }
      const snapshot = session.snapshot();
      for (const subject of subjects) {
        const exact = dependents(snapshot, subject, {archive});
        assert.equal(exact.values.find(item => item.name === 'old').basis, 'lineage');
        assert.deepEqual(exact.values.find(item => item.name === 'old').via, [record]);
        assert.deepEqual(exact.displayed.find(item => item.name === 'hud.old').via, [record]);
        const missing = dependents(snapshot, subject);
        assert.equal(missing.values.find(item => item.name === 'old').basis, 'may rest on');
        assert.deepEqual(missing.values.find(item => item.name === 'old').via, [subject]);
      }
    } finally { session.close(); }
  }
});
