// caveat serve: one session behind a JSON-lines protocol. Requests that are
// malformed are answered and change nothing; a fatal outcome ends the server;
// save and restore round-trip; the process speaks the same protocol; with
// --state it keeps the session in a file across runs.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdtemp, readdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { createServer, SERVE_SCHEMA } from '../lib/serve.mjs';
import { faulty, real, thermostat } from './helpers.mjs';

const cli = fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url));

function serve(runtime = real) {
  const server = createServer({ runtime, source: thermostat, program: 'thermostat.cav' });
  const send = request => server.handle(typeof request === 'string' ? request : JSON.stringify(request));
  return { server, send };
}

test('the server announces itself and answers dispatches with their outcomes', () => {
  const { server, send } = serve();
  assert.deepEqual({ ...server.ready, runtime: undefined }, { schema: SERVE_SCHEMA, ready: true, program: 'thermostat.cav', runtime: undefined });
  assert.equal(server.ready.runtime.reactiveWasmSha256, real.identity.reactiveWasmSha256);

  assert.deepEqual(send({ id: 'a', op: 'dispatch', event: 'read', payload: { value: 17 } }),
    { response: { id: 'a', ok: true, outcome: 'accepted', sequence: 1 }, exit: null, changed: true });
  const refused = send({ id: 2, op: 'dispatch', event: 'read', payload: { value: 9999 } }).response;
  assert.deepEqual([refused.ok, refused.outcome, refused.origin, refused.code, refused.sequence], [true, 'rejected', 'input', 'bound_exceeded', 1]);
  const withSnapshot = send({ id: 3, op: 'dispatch', event: 'read', payload: { value: 25 }, snapshot: true }).response;
  assert.equal(withSnapshot.snapshot.sequence, 2);
  assert.deepEqual(send({ id: 4, op: 'snapshot' }).response.snapshot, withSnapshot.snapshot);

  const explained = send({ id: 5, op: 'explain' }).response.report;
  assert.deepEqual(explained.events.map(entry => entry.outcome.outcome), ['accepted', 'rejected', 'accepted']);
  const report = send({ id: 6, op: 'dependents', of: 'temperature@1' }).response.report;
  assert.deepEqual(report.decisions.map(item => [item.id, item.basis]), [['heating@1', 'grounds'], ['heating@2', 'lineage']]);
  assert.deepEqual(send({ id: 7, op: 'close' }), { response: { id: 7, ok: true }, exit: 0, changed: false });
});

// Version Lab F309-F312, F338-F339: id_text of a number that is no handle
// refuses the event wherever it is read, and the server goes on. An id
// parameter sent as a number stays an input refusal (F339).
test('id_text of a non-handle is a refusal the server survives', () => {
  for (const [place, rule] of [
    ['guard', 'on run when id_text(value + 6) == "x" set value = 3;'],
    ['rule body', 'on run set shown = if(id_text(value + 6) == "x", 1, 2);'],
    ['binding', 'bind hud.label = id_text(value + 6) when value == 1;'],
  ]) {
    const source = `identifiers limit 4; state value = 0; state shown = 0; state head = 0;
      event run; event count; event pushed commit id;
      on run set value = 1; on count set value = value + 5; on pushed set head = commit;
      ${rule}`;
    const server = createServer({ runtime: real, source, program: 'f309.cav' });
    const send = request => server.handle(JSON.stringify(request));
    const before = send({ op: 'save' }).response.save;
    const { response, exit } = send({ id: 1, op: 'dispatch', event: 'run' });
    assert.equal(exit, null, place);
    assert.deepEqual([response.ok, response.outcome, response.origin, response.code, response.sequence],
      [true, 'rejected', 'evaluation', 'expression', 0], place);
    assert.match(response.message, /id_text: 7 is not the handle of an identifier$/, place);
    assert.equal(send({ op: 'save' }).response.save, before, place);
    const wrong = send({ id: 2, op: 'dispatch', event: 'pushed', payload: { commit: 1 } }).response;
    assert.deepEqual([wrong.ok, wrong.outcome, wrong.origin, wrong.code], [true, 'rejected', 'input', 'payload_invalid'], place);
    assert.deepEqual(send({ id: 3, op: 'dispatch', event: 'count' }).response,
      { id: 3, ok: true, outcome: 'accepted', sequence: 1 }, place);
  }
});

test('bad requests are answered as request errors and change nothing', () => {
  const { send } = serve();
  send({ op: 'dispatch', event: 'read', payload: { value: 17 } });
  const before = send({ op: 'save' }).response.save;
  const cases = [
    ['not json', null, /not JSON/],
    ['[1]', null, /must be a JSON object/],
    [{ id: 1, op: 'frobnicate' }, 1, /"op" must be one of/],
    [{ id: 2, op: 'dispatch', event: 'read', payload: { value: 17 }, extra: true }, 2, /dispatch has no field extra/],
    [{ id: 3, op: 'dispatch' }, 3, /needs an "event"/],
    [{ id: 4, op: 'dispatch', event: 'read', payload: [17] }, 4, /"payload" must be an object/],
    // JSON.parse turns 1e999 into Infinity, which the session library refuses.
    ['{"id":5,"op":"dispatch","event":"read","payload":{"value":1e999}}', 5, /not a finite number/],
    [{ id: 6, op: 'dependents', of: 'nowhere' }, 6, /not evidence, a reading stream or a caveat/],
    [{ id: 7, op: 'restore', save: 'garbage' }, 7, /./],
    [{ id: 8, op: 'dispatch', event: 'read', snapshot: 'yes' }, 8, /"snapshot" must be true or false/],
  ];
  for (const [request, id, message] of cases) {
    const { response, exit } = send(request);
    assert.equal(exit, null, JSON.stringify(request));
    assert.equal(response.id, id);
    assert.equal(response.ok, false);
    assert.equal(response.error.kind, 'request');
    assert.match(response.error.message, message);
  }
  assert.equal(send({ op: 'save' }).response.save, before);
});

test('restore replaces the session with the saved one and forgets the events since', () => {
  const { send } = serve();
  send({ op: 'dispatch', event: 'read', payload: { value: 17 } });
  const saved = send({ op: 'save' }).response.save;
  send({ op: 'dispatch', event: 'read', payload: { value: 25 } });
  assert.deepEqual(send({ id: 1, op: 'restore', save: saved }).response, { id: 1, ok: true, sequence: 1 });
  assert.equal(send({ op: 'save' }).response.save, saved);
  assert.deepEqual(send({ op: 'explain' }).response.report.events, []);
  assert.equal(send({ op: 'dispatch', event: 'read', payload: { value: 25 } }).response.sequence, 2);
});

test('a fatal outcome is reported and ends the server with status 1', () => {
  const { runtime } = faulty({
    dispatch() { throw JSON.stringify({ schema: 'caveat-dispatch/0.1', outcome: 'fatal', code: 'unclassified', message: 'boom' }); },
  });
  const { send } = serve(runtime);
  const { response, exit } = send({ id: 9, op: 'dispatch', event: 'read', payload: { value: 17 } });
  assert.equal(exit, 1);
  assert.deepEqual(response, { id: 9, ok: false, error: { kind: 'fatal', message: 'unclassified: boom' } });
});

test('the serve command speaks the protocol on standard input and output', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-serve-'));
  try {
    const program = path.join(directory, 'thermostat.cav');
    await writeFile(program, thermostat);
    const input = [
      { id: 1, op: 'dispatch', event: 'read', payload: { value: 17 } },
      'not json',
      '',
      { id: 2, op: 'close' },
      { id: 3, op: 'snapshot' },
    ].map(line => (typeof line === 'string' ? line : JSON.stringify(line))).join('\r\n');
    const result = spawnSync(process.execPath, [cli, 'serve', program], { input, encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr);
    const lines = result.stdout.trim().split('\n').map(line => JSON.parse(line));
    assert.equal(lines[0].ready, true);
    assert.deepEqual(lines.slice(1).map(line => [line.id, line.ok]), [[1, true], [null, false], [2, true]]);

    // Without close, the end of input ends the server normally.
    const ended = spawnSync(process.execPath, [cli, 'serve', program], { input: '{"op":"snapshot"}\n', encoding: 'utf8' });
    assert.equal(ended.status, 0, ended.stderr);
    assert.equal(ended.stdout.trim().split('\n').length, 2);

    await writeFile(program, 'this is not caveat;');
    const broken = spawnSync(process.execPath, [cli, 'serve', program], { input: '', encoding: 'utf8' });
    assert.equal(broken.status, 2);
    const announced = JSON.parse(broken.stdout);
    assert.deepEqual([announced.schema, announced.ready, announced.error.kind], [SERVE_SCHEMA, false, 'load']);
  } finally { await rm(directory, { recursive: true, force: true }); }
});

test('a compact save restores like any save, and a non-boolean compact is a request error', () => {
  const { send } = serve();
  send({ op: 'dispatch', event: 'read', payload: { value: 17 } });
  const state = send({ op: 'save' }).response.save;
  const compact = send({ op: 'save', compact: true }).response.save;
  assert.equal(JSON.parse(compact).schema, 'caveat-replay-save/0.1');
  assert.ok(compact.length < state.length);
  assert.equal(send({ op: 'save', compact: false }).response.save, state);
  assert.equal(send({ id: 3, op: 'save', compact: 'yes' }).response.ok, false);
  assert.deepEqual(send({ id: 4, op: 'restore', save: compact }).response, { id: 4, ok: true, sequence: 1 });
  assert.equal(send({ op: 'save' }).response.save, state);
});

test('view answers the View 0.1, alone or with a dispatch, and a non-boolean view is a request error', () => {
  const { send } = serve();
  const empty = send({ id: 1, op: 'view' }).response;
  assert.deepEqual([empty.ok, empty.view.schema, empty.view.sequence], [true, 'caveat-reactive-view/0.1', 0]);
  const dispatched = send({ id: 2, op: 'dispatch', event: 'read', payload: { value: 17 }, view: true }).response;
  assert.deepEqual([dispatched.ok, dispatched.outcome, dispatched.sequence, dispatched.view.sequence, dispatched.view.last_event],
    [true, 'accepted', 1, 1, 'read']);
  assert.equal(dispatched.snapshot, undefined);
  assert.deepEqual(send({ id: 3, op: 'view' }).response.view, dispatched.view);
  // A refused event's view is the unchanged session's.
  const refused = send({ id: 4, op: 'dispatch', event: 'read', payload: { value: 9999 }, view: true }).response;
  assert.deepEqual([refused.outcome, refused.view.sequence], ['rejected', 1]);
  const plain = send({ id: 5, op: 'dispatch', event: 'read', payload: { value: 18 } }).response;
  assert.equal(plain.view, undefined);
  const bad = send({ id: 6, op: 'dispatch', event: 'read', view: 'yes' }).response;
  assert.deepEqual([bad.ok, bad.error.kind], [false, 'request']);
  assert.match(bad.error.message, /"view" must be true or false/);
});

test('a server started from a save continues it, and handle says when the session changed', () => {
  const { server, send } = serve();
  assert.equal(send({ op: 'dispatch', event: 'read', payload: { value: 17 } }).changed, true);
  assert.equal(send({ op: 'dispatch', event: 'read', payload: { value: 9999 } }).changed, false);
  for (const request of [{ op: 'snapshot' }, { op: 'view' }, { op: 'save' }, { op: 'explain' }, 'not json', { op: 'dispatch' }]) {
    assert.equal(send(request).changed, false, JSON.stringify(request));
  }
  const saved = send({ op: 'save' }).response.save;
  assert.equal(send({ op: 'restore', save: saved }).changed, true);
  assert.equal(send({ op: 'close' }).changed, false);
  server.close();

  const resumed = createServer({ runtime: real, source: thermostat, program: 'thermostat.cav', restore: saved });
  const ask = request => resumed.handle(JSON.stringify(request));
  assert.deepEqual({ ...resumed.ready, runtime: undefined }, { schema: SERVE_SCHEMA, ready: true, program: 'thermostat.cav', runtime: undefined });
  assert.equal(ask({ op: 'snapshot' }).response.snapshot.sequence, 1);
  assert.equal(ask({ op: 'save' }).response.save, saved);
  // explain lists only this server's events, as after a restore request.
  assert.deepEqual(ask({ op: 'explain' }).response.report.events, []);
  assert.equal(ask({ op: 'dispatch', event: 'read', payload: { value: 25 } }).response.sequence, 2);
  resumed.close();

  assert.throws(() => createServer({ runtime: real, source: thermostat, restore: 'garbage' }), error => error.kind === 'restore');
});

test('save gives the state file text, compact where the runtime has it, and it restores', () => {
  const { server, send } = serve();
  send({ op: 'dispatch', event: 'read', payload: { value: 17 } });
  const { text, compact } = server.save();
  assert.equal(typeof compact, 'boolean');
  assert.equal(JSON.parse(text).schema, compact ? 'caveat-replay-save/0.1' : JSON.parse(send({ op: 'save' }).response.save).schema);
  // The same answer every time, and a session the runtime restores to the same sequence.
  assert.deepEqual(server.save(), { text, compact });
  const resumed = real.restore(thermostat, text);
  try { assert.equal(resumed.snapshot().sequence, 1); } finally { resumed.close(); }
  server.close();
});

test('the serve command keeps a state file across runs when asked', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-serve-state-'));
  try {
    const program = path.join(directory, 'thermostat.cav');
    await writeFile(program, thermostat);
    const state = path.join(directory, 'thermostat.save');
    const run = (lines, file = state) => {
      const input = lines.map(line => JSON.stringify(line)).join('\n');
      const result = spawnSync(process.execPath, [cli, 'serve', '--state', file, program], { input, encoding: 'utf8' });
      return { status: result.status, stderr: result.stderr, lines: result.stdout.trim().split('\n').filter(Boolean).map(line => JSON.parse(line)) };
    };

    // A fresh session writes the file on start, before any event.
    const fresh = run([{ id: 1, op: 'snapshot' }]);
    assert.equal(fresh.status, 0, fresh.stderr);
    assert.deepEqual(fresh.lines[0].state, { path: state, restored: false });
    assert.ok(existsSync(state));
    const sequence0 = await readFile(state, 'utf8');

    // An accepted event rewrites it; a refused one does not.
    const first = run([
      { id: 1, op: 'dispatch', event: 'read', payload: { value: 17 } },
      { id: 2, op: 'dispatch', event: 'read', payload: { value: 9999 } },
      { id: 3, op: 'close' },
    ]);
    assert.equal(first.status, 0, first.stderr);
    assert.deepEqual(first.lines.slice(1).map(line => [line.id, line.ok, line.outcome]), [[1, true, 'accepted'], [2, true, 'rejected'], [3, true, undefined]]);
    const sequence1 = await readFile(state, 'utf8');
    assert.notEqual(sequence1, sequence0);

    // The next run starts where the last one stopped and rewrites on its own change.
    const second = run([{ id: 1, op: 'snapshot' }, { id: 2, op: 'dispatch', event: 'read', payload: { value: 25 } }]);
    assert.equal(second.status, 0, second.stderr);
    assert.deepEqual(second.lines[0].state, { path: state, restored: true });
    assert.equal(second.lines[1].snapshot.sequence, 1);
    assert.equal(second.lines[2].sequence, 2);
    assert.notEqual(await readFile(state, 'utf8'), sequence1);
    assert.equal(run([{ id: 1, op: 'snapshot' }]).lines[1].snapshot.sequence, 2);
    assert.deepEqual((await readdir(directory)).filter(name => name.endsWith('.partial')), []);

    // A file that is not a save of the program is reported and nothing runs.
    const other = path.join(directory, 'other.save');
    await writeFile(other, 'garbage');
    const broken = run([{ id: 1, op: 'snapshot' }], other);
    assert.equal(broken.status, 2);
    assert.deepEqual([broken.lines.length, broken.lines[0].ready, broken.lines[0].error.kind], [1, false, 'state']);
    assert.match(broken.lines[0].error.message, /other\.save is not a save of thermostat\.cav/);
    assert.equal(await readFile(other, 'utf8'), 'garbage');

    // A program that does not load is a load error, whatever the state file holds.
    const unloadable = path.join(directory, 'unloadable.cav');
    await writeFile(unloadable, 'garbage here\n');
    const failed = spawnSync(process.execPath, [cli, 'serve', '--state', state, unloadable], { input: '', encoding: 'utf8' });
    const failedLines = failed.stdout.trim().split('\n').map(line => JSON.parse(line));
    assert.equal(failed.status, 2);
    assert.deepEqual([failedLines.length, failedLines[0].ready, failedLines[0].error.kind], [1, false, 'load']);
    assert.doesNotMatch(failedLines[0].error.message, /is not a save/);

    // --state needs a file, and belongs to serve only.
    const missing = spawnSync(process.execPath, [cli, 'serve', '--state'], { input: '', encoding: 'utf8' });
    assert.equal(missing.status, 2);
    assert.match(missing.stderr, /--state needs a file/);
    const elsewhere = spawnSync(process.execPath, [cli, 'validate', '--state', state, program], { input: '', encoding: 'utf8' });
    assert.equal(elsewhere.status, 2);
    assert.match(elsewhere.stderr, /unknown option --state for validate/);
  } finally { await rm(directory, { recursive: true, force: true }); }
});
