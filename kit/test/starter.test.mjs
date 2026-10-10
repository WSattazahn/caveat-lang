// The integration starter over the real runtime: the separate answers a send
// gives, checkpoints and the archive through store failures, restarts and a
// killed process, and source identity across a replaced program.
import test from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtemp, readFile, rm, writeFile, appendFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { real } from './helpers.mjs';
import { explain } from '../lib/explain.mjs';
import { CHECKPOINT_SCHEMA, StarterError, memoryStore, openHost, sourceDigest } from '../lib/starter.mjs';
import { fileStore } from '../lib/starter-node.mjs';

const fixture = fileURLToPath(new URL('./fixtures/starter/assessment.cav', import.meta.url));
const source = await readFile(fixture, 'utf8');

// Observations alternate sides, so the window of 2 departs records on most events.
const observations = count => Array.from({ length: count }, (_, index) => ['observe', { confidence: index % 3 === 2 ? 40 : 75 + index }]);

// What a host that never failed would hold: every item drained after every
// accepted event, in order.
function reference(events) {
  const session = real.open(source);
  const archive = [];
  try {
    for (const [event, payload] of events) {
      if (session.dispatch(event, payload).outcome === 'accepted') archive.push(...session.drainArchive());
    }
    return { archive, view: session.view(), save: session.save() };
  } finally { session.close(); }
}

async function directory(t) {
  const made = await mkdtemp(path.join(tmpdir(), 'caveat-starter-'));
  t.after(() => rm(made, { recursive: true, force: true }));
  return made;
}

// A store whose next calls of one method fail, after doing what `partial` does.
function failing(store) {
  const plan = { writeCheckpoint: [], appendChunks: [] };
  const wrapped = { ...store };
  for (const method of Object.keys(plan)) {
    wrapped[method] = async (...args) => {
      const step = plan[method].shift();
      if (step === 'after') { await store[method](...args); throw new Error(`${method} reported failure after writing`); }
      if (step === 'before') throw new Error(`${method} failed`);
      return store[method](...args);
    };
  }
  return { store: wrapped, plan };
}

test('a send keeps handled, accepted, durable and the current assessment apart', async () => {
  const host = await openHost({ runtime: real, source, store: memoryStore() });
  try {
    assert.equal(host.assessment('assessment').status, 'none');
    assert.equal(host.permits('assessment'), false);

    const unhandled = await host.send('observe', { confidence: Number.NaN });
    assert.deepEqual([unhandled.handled, unhandled.accepted, unhandled.error.kind], [false, false, 'payload']);
    assert.equal(host.sequence, 0);

    const refused = await host.send('assess');
    assert.deepEqual([refused.handled, refused.accepted], [true, false]);
    assert.deepEqual(refused.rejection, { origin: 'policy', code: 'reject', message: 'No observation to assess.' });

    const observed = await host.send('observe', { confidence: 80 });
    assert.deepEqual([observed.handled, observed.accepted, observed.durable, observed.archived], [true, true, true, true]);
    const approved = await host.send('assess');
    assert.equal(approved.accepted, true);
    assert.equal(approved.view.bindings.assessment.verdict, 'approved');
    assert.equal(host.permits('assessment'), true);

    // An approval in force does not make a refused operation look successful.
    const again = await host.send('assess');
    assert.deepEqual([again.handled, again.accepted, again.rejection.message], [true, false, 'Already assessed.']);
    assert.equal(host.permits('assessment'), true, 'the refusal changed nothing');

    const withdrawn = await host.send('retract');
    assert.equal(withdrawn.accepted, true);
    const assessment = host.assessment('assessment');
    assert.equal(assessment.status, 'reopened');
    assert.equal(assessment.revision, 'assessment@1');
    assert.deepEqual(assessment.change, { change: 'reopened', sequence: 3, event: 'retract', because: ['recheck'], caveats: [], value: 80 });
    assert.equal(assessment.source_sha256, sourceDigest(source));
    assert.equal(host.permits('assessment'), false);

    assert.throws(() => host.assessment('nothing'), error => error instanceof StarterError && error.kind === 'unknown');
    assert.deepEqual(host.resync(host.sequence), { resync: false, sequence: 3 });
    assert.deepEqual(host.resync(1), { resync: true, sequence: 3, view: host.view() });
  } finally { await host.close(); }
  await assert.rejects(host.send('observe', { confidence: 80 }), error => error.kind === 'closed');
});

test('the file store keeps every archive item across a restart, and explain reads it', async t => {
  const where = await directory(t);
  const events = observations(9);
  const expected = reference(events);
  assert.ok(expected.archive.length > 0, 'the fixture departs records');

  let host = await openHost({ runtime: real, source, store: await fileStore(where) });
  for (const [event, payload] of events.slice(0, 5)) assert.equal((await host.send(event, payload)).archived, true);
  await host.close();
  host = await openHost({ runtime: real, source, store: await fileStore(where) });
  assert.equal(host.sequence, 5);
  for (const [event, payload] of events.slice(5)) await host.send(event, payload);
  try {
    assert.deepEqual(host.view(), expected.view);
    const archive = await host.archive();
    assert.deepEqual(archive, expected.archive);
    const checkpoint = JSON.parse(await readFile(path.join(where, 'checkpoint.json'), 'utf8'));
    assert.equal(checkpoint.schema, CHECKPOINT_SCHEMA);
    // The checkpoint is written before its chunks are appended, so it may
    // still list chunks the archive already holds.
    const stored = (await readFile(path.join(where, 'archive.jsonl'), 'utf8')).trim().split('\n').map(line => JSON.parse(line));
    for (const chunk of checkpoint.pending) assert.ok(stored.some(item => item.number === chunk.number && item.sha256 === chunk.sha256));
    assert.equal(checkpoint.source_sha256, sourceDigest(source));
    assert.equal(checkpoint.runtime.reactiveWasmSha256, real.identity.reactiveWasmSha256);
    const report = explain(host.snapshot(), [], { archive });
    assert.notEqual(report.displayed.length, 0);
  } finally { await host.close(); }
});

test('a failed checkpoint is reported, keeps its archive items and is retried', async t => {
  const where = await directory(t);
  const events = observations(8);
  const { store, plan } = failing(await fileStore(where));
  const host = await openHost({ runtime: real, source, store });
  try {
    for (const [event, payload] of events.slice(0, 4)) await host.send(event, payload);
    plan.writeCheckpoint.push('before');
    const failed = await host.send(...events[4]);
    assert.deepEqual([failed.accepted, failed.durable, failed.archived], [true, false, false]);
    assert.match(failed.storeError, /writeCheckpoint failed/);
    assert.equal(host.durable, false);
    assert.ok(host.pendingChunks > 0, 'drained items wait in memory');
    // A restart now would resume at the last durable checkpoint.
    const stale = JSON.parse(await readFile(path.join(where, 'checkpoint.json'), 'utf8'));
    assert.equal(stale.sequence, 4);
    const retried = await host.flush();
    assert.deepEqual(retried, { durable: true, archived: true });
    for (const [event, payload] of events.slice(5)) await host.send(event, payload);
    assert.deepEqual(await host.archive(), reference(events).archive);
  } finally { await host.close(); }
});

test('an append that fails before or after writing loses and repeats nothing across a crash', async t => {
  for (const step of ['before', 'after']) {
    const where = await directory(t);
    const events = observations(8);
    const { store, plan } = failing(await fileStore(where));
    const host = await openHost({ runtime: real, source, store });
    for (const [event, payload] of events.slice(0, 4)) await host.send(event, payload);
    plan.appendChunks.push(step);
    const failed = await host.send(...events[4]);
    assert.deepEqual([failed.accepted, failed.durable, failed.archived], [true, true, false], step);
    // The process stops here without closing; the checkpoint holds the pending chunk.
    const checkpoint = JSON.parse(await readFile(path.join(where, 'checkpoint.json'), 'utf8'));
    assert.equal(checkpoint.pending.length, 1, step);
    const resumed = await openHost({ runtime: real, source, store: await fileStore(where) });
    try {
      for (const [event, payload] of events.slice(5)) await resumed.send(event, payload);
      assert.deepEqual(await resumed.archive(), reference(events).archive, step);
      const numbers = (await readFile(path.join(where, 'archive.jsonl'), 'utf8')).trim().split('\n').map(line => JSON.parse(line).number);
      assert.deepEqual(numbers, [...new Set(numbers)].sort((a, b) => a - b), `${step}: each chunk written once, in order`);
    } finally { await resumed.close(); }
  }
});

test('an append retried in the same process after a reported failure is read once', async t => {
  const where = await directory(t);
  const events = observations(8);
  const { store, plan } = failing(await fileStore(where));
  const host = await openHost({ runtime: real, source, store });
  try {
    for (const [event, payload] of events.slice(0, 4)) await host.send(event, payload);
    plan.appendChunks.push('after');
    assert.equal((await host.send(...events[4])).archived, false);
    assert.deepEqual(await host.flush(), { durable: true, archived: true });
    for (const [event, payload] of events.slice(5)) await host.send(event, payload);
    const numbers = (await readFile(path.join(where, 'archive.jsonl'), 'utf8')).trim().split('\n').map(line => JSON.parse(line).number);
    assert.ok(numbers.length > new Set(numbers).size, 'the chunk is in the file twice');
    assert.deepEqual(await host.archive(), reference(events).archive, 'and in the archive once');
  } finally { await host.close(); }
});

test('a torn last archive line is cut off on open and its chunk appended again', async t => {
  const where = await directory(t);
  const events = observations(7);
  const { store, plan } = failing(await fileStore(where));
  const host = await openHost({ runtime: real, source, store });
  for (const [event, payload] of events.slice(0, 5)) await host.send(event, payload);
  plan.appendChunks.push('before');
  await host.send(...events[5]);
  const checkpoint = JSON.parse(await readFile(path.join(where, 'checkpoint.json'), 'utf8'));
  assert.equal(checkpoint.pending.length, 1);
  await appendFile(path.join(where, 'archive.jsonl'), JSON.stringify(checkpoint.pending[0]).slice(0, 40));
  const resumed = await openHost({ runtime: real, source, store: await fileStore(where) });
  try {
    await resumed.send(...events[6]);
    assert.deepEqual(await resumed.archive(), reference(events).archive);
  } finally { await resumed.close(); }
});

test('a store whose archive runs ahead of its checkpoint is refused', async t => {
  const where = await directory(t);
  const host = await openHost({ runtime: real, source, store: await fileStore(where) });
  for (const [event, payload] of observations(3)) await host.send(event, payload);
  const early = await readFile(path.join(where, 'checkpoint.json'), 'utf8');
  for (const [event, payload] of observations(6).slice(3)) await host.send(event, payload);
  await host.close();
  await writeFile(path.join(where, 'checkpoint.json'), early);
  await assert.rejects(openHost({ runtime: real, source, store: await fileStore(where) }),
    error => error instanceof StarterError && error.kind === 'store' && /mixes two histories/.test(error.message));
});

test('a checkpoint made under other source text is refused, and the approval names its source', async t => {
  const where = await directory(t);
  const host = await openHost({ runtime: real, source, store: await fileStore(where) });
  await host.send('observe', { confidence: 80 });
  await host.send('assess');
  assert.equal(host.assessment('assessment').source_sha256, sourceDigest(source));
  await host.close();

  const edited = `${source}\n# Edited policy text.\n`;
  await assert.rejects(openHost({ runtime: real, source: edited, store: await fileStore(where) }),
    error => error instanceof StarterError && error.kind === 'source' && error.message.includes(sourceDigest(source)));
  // The runtime refuses the same save under the edited text on its own.
  const checkpoint = JSON.parse(await readFile(path.join(where, 'checkpoint.json'), 'utf8'));
  assert.equal(checkpoint.source_sha256, sourceDigest(source));
  assert.throws(() => real.restore(edited, checkpoint.save), error => error.kind === 'restore');
});

// A child process sends events as fast as it can and is killed at a random
// moment. Whatever moment it died at, the store resumes at a checkpoint whose
// archive is exactly what a host that never stopped would hold by then.
test('a process killed mid-run resumes at a consistent checkpoint', async t => {
  const events = observations(60);
  const library = new URL('../lib/', import.meta.url).href;
  const runner = `
    import { readFile } from 'node:fs/promises';
    import { openHost } from '${library}starter.mjs';
    import { fileStore } from '${library}starter-node.mjs';
    import { loadRuntimeFromDirectory } from '${library}node.mjs';
    const runtime = await loadRuntimeFromDirectory();
    const source = await readFile(process.argv[1], 'utf8');
    const host = await openHost({ runtime, source, store: await fileStore(process.argv[2]) });
    const events = JSON.parse(process.argv[3]);
    process.stdout.write('ready\\n');
    for (const [event, payload] of events.slice(host.sequence)) await host.send(event, payload);
    process.stdout.write('done\\n');
  `;
  for (const delay of [5, 20, 60]) {
    const where = await directory(t);
    const child = spawn(process.execPath, ['--input-type=module', '-e', runner, fixture, where, JSON.stringify(events)], { stdio: ['ignore', 'pipe', 'inherit'] });
    await new Promise((resolve, reject) => {
      child.stdout.on('data', chunk => { if (String(chunk).includes('ready')) setTimeout(() => child.kill('SIGKILL'), delay); });
      child.on('exit', resolve);
      child.on('error', reject);
    });
    const host = await openHost({ runtime: real, source, store: await fileStore(where) });
    try {
      const reached = host.sequence;
      const expected = reference(events.slice(0, reached));
      assert.deepEqual(host.view(), expected.view, `killed after ${delay} ms at sequence ${reached}`);
      assert.deepEqual(await host.archive(), expected.archive, `killed after ${delay} ms at sequence ${reached}`);
      for (const [event, payload] of events.slice(reached)) await host.send(event, payload);
      assert.deepEqual(await host.archive(), reference(events).archive);
    } finally { await host.close(); }
  }
});
