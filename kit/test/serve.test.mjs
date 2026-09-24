// caveat serve: one session behind a JSON-lines protocol. Requests that are
// malformed are answered and change nothing; a fatal outcome ends the server;
// save and restore round-trip; the process speaks the same protocol.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
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
    { response: { id: 'a', ok: true, outcome: 'accepted', sequence: 1 }, exit: null });
  const refused = send({ id: 2, op: 'dispatch', event: 'read', payload: { value: 9999 } }).response;
  assert.deepEqual([refused.ok, refused.outcome, refused.origin, refused.code, refused.sequence], [true, 'rejected', 'input', 'bound_exceeded', 1]);
  const withSnapshot = send({ id: 3, op: 'dispatch', event: 'read', payload: { value: 25 }, snapshot: true }).response;
  assert.equal(withSnapshot.snapshot.sequence, 2);
  assert.deepEqual(send({ id: 4, op: 'snapshot' }).response.snapshot, withSnapshot.snapshot);

  const explained = send({ id: 5, op: 'explain' }).response.report;
  assert.deepEqual(explained.events.map(entry => entry.outcome.outcome), ['accepted', 'rejected', 'accepted']);
  const report = send({ id: 6, op: 'dependents', of: 'temperature@1' }).response.report;
  assert.deepEqual(report.decisions.map(item => [item.id, item.basis]), [['heating@1', 'grounds'], ['heating@2', 'lineage']]);
  assert.deepEqual(send({ id: 7, op: 'close' }), { response: { id: 7, ok: true }, exit: 0 });
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
