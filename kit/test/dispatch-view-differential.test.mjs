// A differential check of session.dispatchView() against dispatch() + view().
// Over every tracked program that opens as a reactive session, with a seeded
// random stream of valid and invalid events, and over the recorded streams of
// the Glowcap, agent ledger and Trail Rescue workloads, one session goes
// through dispatch() and view() and a twin through dispatchView(). After every
// event the two must give the same outcome and the same view, save and
// snapshot. A fatal outcome must be the same on both, and both start again.
//
// DISPATCH_VIEW_EVENTS sets the random events per program (default 120) and
// DISPATCH_VIEW_SEED the seed (default 20260929).
import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { real, repo } from './helpers.mjs';
import { STREAMS, loadStreams } from './workload-streams.mjs';

const EVENTS = Number(process.env.DISPATCH_VIEW_EVENTS ?? 120);
const SEED = Number(process.env.DISPATCH_VIEW_SEED ?? 20260929);

// The runtime tests' generator (runtime/tests/save_restore.rs), so a failure
// names its seed.
function mulberry(seed) {
  let state = seed >>> 0;
  const next = () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let t = state;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return (t ^ (t >>> 14)) >>> 0;
  };
  return { below: n => next() % Math.max(1, n), chance: p => next() / 2 ** 32 < p, unit: () => next() / 2 ** 32 };
}

const TEXTS = ['a', 'b', 'c', '602bdbec0a047a5319f53e83f336b9f7aec0e5ed', 'pr-21', 'x'.repeat(40)];

function value(random, parameter) {
  const { min, max, domain } = parameter;
  if (domain?.identifier !== undefined || domain?.id !== undefined) {
    return random.chance(0.05) ? random.below(3) + 1 : TEXTS[random.below(TEXTS.length)];
  }
  const members = domain?.entity?.members ?? domain?.member?.members;
  if (members) {
    if (random.chance(0.05)) return 'nobody';
    if (random.chance(0.05)) return 1.5;
    if (random.chance(0.2)) return random.below(members.length + 1) + 1;
    return members[random.below(members.length)];
  }
  if (random.chance(0.05)) return max + 1;
  const span = Math.min(max - min, 1e6);
  const number = min + random.unit() * span;
  return random.chance(0.5) ? Math.round(number) : number;
}

// One event from a program's signatures, now and then malformed.
function randomStep(random, events) {
  if (random.chance(0.03) || !events.length) return ['no_such_event', {}];
  const { name, parameters } = events[random.below(events.length)];
  const payload = Object.fromEntries(parameters.map(parameter => [parameter.name, value(random, parameter)]));
  if (parameters.length && random.chance(0.03)) delete payload[parameters[random.below(parameters.length)].name];
  if (random.chance(0.03)) payload.extra = 1;
  return [name, payload];
}

const state = session => ({ save: session.save(), snapshot: session.snapshotText(), view: session.viewText() });
const fatalOf = run => { try { run(); } catch (error) { return error; } return null; };

// Plays steps on twins. steps is an array or a function giving the next step.
// Each event compares the outcomes, then both sessions' save, snapshot and
// view text; a refusal must also leave the twin's text as it was.
function play(label, source, steps, count) {
  const tally = { events: 0, accepted: 0, rejected: 0, fatal: 0, codes: {} };
  let old = real.open(source);
  let viewed = real.open(source);
  let previous = state(viewed);
  try {
    for (let index = 0; index < count; index += 1) {
      const [event, payload] = Array.isArray(steps) ? steps[index] : steps(old);
      const at = `${label}, event ${index + 1}: ${event} ${JSON.stringify(payload)}`;
      let expected;
      const oldFatal = fatalOf(() => { expected = old.dispatch(event, payload); });
      let outcome;
      const newFatal = fatalOf(() => { outcome = viewed.dispatchView(event, payload); });
      tally.events += 1;
      if (oldFatal || newFatal) {
        assert.ok(oldFatal && newFatal, `${at}: fatal on one path only: ${oldFatal?.message ?? newFatal?.message}`);
        assert.equal(newFatal.kind, 'fatal', at);
        assert.equal(newFatal.message, oldFatal.message, at);
        assert.deepEqual(newFatal.report, oldFatal.report, at);
        assert.equal(viewed.state, 'fatal', at);
        tally.fatal += 1;
        old.close(); viewed.close();
        old = real.open(source);
        viewed = real.open(source);
        previous = state(viewed);
        continue;
      }
      const now = state(viewed);
      assert.deepEqual(now, state(old), `${at}: the sessions differ`);
      if (expected.outcome === 'accepted') {
        tally.accepted += 1;
        assert.deepEqual(Object.keys(outcome), ['schema', 'outcome', 'view'], at);
        // The view the session then shows, which is also dispatch()'s view().
        assert.deepEqual(outcome, { schema: expected.schema, outcome: 'accepted', view: JSON.parse(now.view) }, at);
      } else {
        tally.rejected += 1;
        const key = `${expected.origin}/${expected.code}`;
        tally.codes[key] = (tally.codes[key] ?? 0) + 1;
        assert.deepEqual(outcome, expected, at);
        assert.deepEqual(now, previous, `${at}: the refusal changed the session`);
      }
      previous = now;
    }
  } finally { old.close(); viewed.close(); }
  return tally;
}

const add = (total, tally) => {
  for (const key of ['events', 'accepted', 'rejected', 'fatal']) total[key] += tally[key];
  for (const [code, n] of Object.entries(tally.codes)) total.codes[code] = (total.codes[code] ?? 0) + n;
};

test('dispatchView agrees with dispatch() and view() on every tracked program, event by event', async () => {
  const listed = spawnSync('git', ['ls-files', '-z', '--', '*.cav'], { cwd: repo, encoding: 'utf8' });
  assert.equal(listed.status, 0, listed.stderr);
  const files = listed.stdout.split('\0').filter(Boolean).sort();
  assert.ok(files.length > 50, `only ${files.length} tracked programs`);
  const total = { programs: 0, skipped: [], events: 0, accepted: 0, rejected: 0, fatal: 0, codes: {} };
  for (const [index, file] of files.entries()) {
    const source = await readFile(path.join(repo, file), 'utf8');
    let probe;
    try { probe = real.open(source); } catch (error) {
      // Modules, programs that import them, and sequential or 3D games.
      assert.equal(error.kind, 'load', file);
      total.skipped.push(file);
      continue;
    }
    const { events } = probe.snapshot();
    probe.close();
    const random = mulberry(SEED + index);
    add(total, play(`${file} (seed ${SEED + index})`, source, () => randomStep(random, events), EVENTS));
    total.programs += 1;
  }
  console.log(`dispatch-view differential: ${JSON.stringify(total)}`);
  assert.ok(total.programs >= 50, `only ${total.programs} programs open`);
  assert.ok(total.accepted > 0 && total.rejected > 0);
});

test('dispatchView agrees with dispatch() and view() on the Glowcap, ledger and Trail Rescue streams', async () => {
  const totals = {};
  for (const { id, source, episodes, skipped } of await loadStreams()) {
    if (skipped) { console.log(`dispatch-view workloads: skipped ${id}: ${skipped}`); continue; }
    const total = { episodes: 0, events: 0, accepted: 0, rejected: 0, fatal: 0, codes: {} };
    for (const [number, steps] of episodes.entries()) {
      add(total, play(`${id} episode ${number + 1}`, source, steps, steps.length));
      total.episodes += 1;
    }
    assert.equal(total.fatal, 0, id);
    totals[id] = total;
  }
  console.log(`dispatch-view workloads: ${JSON.stringify(totals)}`);
  assert.ok(totals['ledger-session'].rejected > 0, 'the ledger refuses by policy');
  assert.ok(totals['trail-rescue-scenarios'].rejected > 0, 'Trail Rescue refuses');
});

const MANIFEST = 'experiments/performance-0.1/inputs/workloads.json';
const sha256 = data => createHash('sha256').update(data).digest('hex');

// While a stream's files are the ones the baseline recorded, the stream is the
// baseline's own: its program has the sha256 the baseline generated, and its
// events, encoded as the baseline encodes them, the stream's recorded sha256.
// A stream whose files changed since is not compared (see the next test).
test('the recorded streams are the baseline\'s while their files are unchanged', async () => {
  const manifest = JSON.parse(await readFile(path.join(repo, MANIFEST), 'utf8'));
  const compared = [];
  for (const stream of await loadStreams()) {
    const spec = manifest.workloads.find(workload => workload.id === stream.id);
    const files = [spec.program, spec.episodes.jsonl, spec.episodes.json].filter(Boolean);
    const hashes = await Promise.all(files.map(async file => sha256(await readFile(path.join(repo, file.path)))));
    if (files.some((file, index) => hashes[index] !== file.sha256)) {
      console.log(`dispatch-view workloads: ${stream.id} changed since the baseline; not compared with it`);
      continue;
    }
    assert.equal(stream.skipped, null, stream.id);
    assert.equal(sha256(stream.source), spec.program.generatedSha256 ?? spec.program.sha256, `${stream.id}: the program`);
    const encoded = stream.episodes.map(episode => episode.map(([event, payload]) => [event, JSON.stringify(payload)]));
    assert.equal(sha256(JSON.stringify(encoded)), spec.streamSha256, `${stream.id}: the events`);
    compared.push(stream.id);
  }
  console.log(`dispatch-view workloads: the same as the baseline's: ${compared.join(', ') || 'none'}`);
});

// The streams come from the frozen baseline's inputs, but the programs and the
// ledger's log are read as they are now: an edit to any of them changes its
// sha256 from the one the baseline recorded, and must not fail this test.
test('the recorded streams load after their programs and logs are edited', async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), 'caveat-streams-'));
  try {
    const manifest = JSON.parse(await readFile(path.join(repo, MANIFEST), 'utf8'));
    const workloads = manifest.workloads.filter(workload => STREAMS.includes(workload.id));
    const copy = async (relative, edit = text => text) => {
      await mkdir(path.dirname(path.join(root, relative)), { recursive: true });
      await writeFile(path.join(root, relative), edit(await readFile(path.join(repo, relative), 'utf8')));
    };
    const edited = new Set();
    await copy(MANIFEST);
    for (const { program, episodes } of workloads) {
      if (!edited.has(program.path)) await copy(program.path, text => `${text}\n// edited after the baseline\n`);
      edited.add(program.path);
      if (episodes.jsonl) await copy(episodes.jsonl.path, text => `${text}\n`);
      if (episodes.json) await copy(episodes.json.path);
    }
    const now = await loadStreams();
    const later = await loadStreams(root);
    assert.deepEqual(later.map(stream => stream.id), STREAMS);
    for (const [index, stream] of later.entries()) {
      assert.equal(stream.skipped, null, stream.id);
      assert.match(stream.source, /\/\/ edited after the baseline\n$/, stream.id);
      assert.deepEqual(stream.episodes, now[index].episodes, `${stream.id}: the same events`);
    }
    // The first events of each still open and play on both paths.
    for (const { id, source, episodes } of later) play(`${id} (edited)`, source, episodes[0], Math.min(20, episodes[0].length));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
