// Neutral evidence through the public WASM, report, CLI and server consumers.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { real, repo } from './helpers.mjs';
import { explain, formatExplanation, dependents } from '../lib/explain.mjs';
import { createServer } from '../lib/serve.mjs';

const SOURCE = `evidence memory from "memory lookup";
evidence correction from "user correction";
caveat stale consequence material;
state confidence = 0; decisions strategy limit 3;
event consult; event assess; event learn_stale; event correct; event failed;
on consult reveal memory;
on consult set confidence = qualified(80, memory);
on assess commit strategy because enough using confidence;
on learn_stale qualify memory with stale;
on correct reveal correction;
on correct withdraw memory because correction;
on correct reopen strategy because correction;
on failed reveal correction;
on failed reject "try again";
bind hud.seen = observed(memory);
bind hud.confidence = confidence because confidence;`;
const cli = fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url));

function accepted(session, event, payload = {}) {
  const result = session.dispatch(event, payload);
  assert.equal(result.outcome, 'accepted', JSON.stringify(result));
  return result.snapshot;
}

test('neutral consultation updates bindings, grounds, qualification and withdrawal without a claim', () => {
  const session = real.open(SOURCE);
  try {
    assert.equal(session.view().bindings.hud.seen, false);
    assert.equal(Object.hasOwn(session.snapshot(), 'observations'), false);
    const observed = accepted(session, 'consult');
    assert.equal(session.view().bindings.hud.seen, true);
    assert.deepEqual(observed.observations, ['memory']);
    assert.deepEqual(observed.effects, [{ kind: 'reveal', evidence: 'memory' }]);
    assert.deepEqual(observed.relations, []);
    assert.deepEqual(observed.value_grounds.confidence, { evidence: ['memory'], caveats: [] });
    assert.equal(explain(observed).evidence[0].claim, null);
    assert.equal(explain(observed).evidence[0].relation, null);
    assert.match(formatExplanation(explain(observed)), /memory observed \(no stance\)/);
    accepted(session, 'assess');
    const frozen = session.snapshot().commitment_grounds['strategy@1'];
    const stale = accepted(session, 'learn_stale');
    assert.deepEqual(stale.commitment_grounds['strategy@1'], frozen);
    assert.deepEqual(stale.value_grounds.confidence.caveats, ['stale']);
    const uses = dependents(stale, 'memory');
    assert.ok(uses.values.some(item => item.name === 'confidence' && item.basis === 'grounds'));
    assert.ok(uses.decisions.some(item => item.id === 'strategy@1' && item.basis === 'grounds'));
    const corrected = accepted(session, 'correct');
    assert.deepEqual(corrected.commitment_grounds['strategy@1'], frozen);
    const memory = explain(corrected).evidence.find(item => item.id === 'memory');
    assert.equal(memory.withdrawn.because, 'correction');
    assert.ok(memory.caveats.includes('withdrawn'));
    const resumed = real.restore(SOURCE, session.save());
    try { assert.deepEqual(resumed.snapshot(), corrected); } finally { resumed.close(); }
  } finally { session.close(); }
});

test('explain preserves acquisition order when a neutral observation later gains a stance', () => {
  const source = `claim safe; evidence a from "A"; evidence b from "B";
    event neutral; event first; event second;
    on neutral reveal b; on first reveal a supports safe; on second reveal b opposes safe;`;
  const session = real.open(source);
  try {
    for (const event of ['neutral', 'first', 'second', 'neutral']) accepted(session, event);
    assert.deepEqual(session.snapshot().observations, ['b', 'a']);
    assert.deepEqual(session.snapshot().effects, []);
    assert.deepEqual(explain(session.snapshot()).evidence.map(item => [item.id, item.relation, item.claim]),
      [['b', 'opposes', 'safe'], ['a', 'supports', 'safe']]);
    const resumed = real.restore(source, session.save());
    try { assert.deepEqual(explain(resumed.snapshot()), explain(session.snapshot())); } finally { resumed.close(); }
  } finally { session.close(); }
});

test('rejection removes a neutral observation and malformed save records are refused', () => {
  const session = real.open(SOURCE);
  try {
    const before = session.save();
    const result = session.dispatch('failed');
    assert.equal(result.outcome, 'rejected');
    assert.equal(result.code, 'reject');
    assert.equal(session.save(), before);
    accepted(session, 'consult');
    const savedText = session.save();
    const saved = JSON.parse(savedText);
    for (const mutate of [
      doc => { doc.observations = []; },
      doc => { doc.observations = null; },
      doc => { doc.observations.push('memory'); },
      doc => { delete doc.observations; },
      doc => { doc.effects[0].relation = 'supports'; },
      doc => { doc.effects[0].target = 'memory'; },
    ]) {
      const invalid = structuredClone(saved); mutate(invalid);
      assert.throws(() => real.restore(SOURCE, JSON.stringify(invalid)), error => error.kind === 'restore');
    }
    assert.equal(session.save(), savedText);
  } finally { session.close(); }
});

test('the documented neutral source loads cleanly and sampling alone still does not observe its template', async () => {
  const spec = await readFile(path.join(repo, 'spec/caveat-neutral-reveal-0.1.md'), 'utf8');
  const source = /```caveat\n([\s\S]*?)```/.exec(spec)[1];
  assert.deepEqual(real.check(source).diagnostics, []);
  const session = real.open(source);
  try {
    accepted(session, 'consult'); accepted(session, 'learn_stale');
    assert.deepEqual(explain(session.snapshot()).evidence.map(item => [item.id, item.relation, item.caveats]),
      [['memory', null, ['stale']]]);
  } finally { session.close(); }
  assert.throws(() => real.open(`claim safe; evidence memory from "lookup";
    caveat stale consequence material; readings freshness from memory limit 3;
    event sample; event learn; on sample sample freshness = 1 supports safe;
    on learn qualify memory with stale;`), error => error.kind === 'load' && /nothing observes/.test(error.message));
});

test('serve returns neutral snapshots, explanations and dependents across restore', () => {
  const server = createServer({ runtime: real, source: SOURCE, program: 'memory.cav' });
  const send = request => server.handle(JSON.stringify(request)).response;
  try {
    assert.equal(send({ op: 'dispatch', event: 'consult' }).outcome, 'accepted');
    assert.equal(send({ op: 'dispatch', event: 'assess' }).outcome, 'accepted');
    const saved = send({ op: 'save' }).save;
    assert.equal(send({ op: 'restore', save: saved }).ok, true);
    assert.deepEqual(send({ op: 'snapshot' }).snapshot.observations, ['memory']);
    assert.equal(send({ op: 'explain' }).report.evidence[0].relation, null);
    assert.equal(send({ op: 'dependents', of: 'memory' }).report.decisions[0].id, 'strategy@1');
  } finally { send({ op: 'close' }); }
});

test('CLI check, explain and dependents accept neutral observations and label them honestly', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-neutral-'));
  try {
    const program = path.join(directory, 'memory.cav');
    const events = path.join(directory, 'events.jsonl');
    await writeFile(program, SOURCE);
    await writeFile(events, ['consult', 'assess', 'learn_stale'].map(event => JSON.stringify({ event })).join('\n'));
    const run = (...args) => spawnSync(process.execPath, [cli, ...args], { encoding: 'utf8' });
    const checked = run('check', program);
    assert.equal(checked.status, 0, checked.stderr);
    const text = run('explain', program, events);
    assert.equal(text.status, 0, text.stderr);
    assert.match(text.stdout, /memory observed \(no stance\).*stale/);
    const output = run('explain', program, events, '--json');
    assert.equal(output.status, 0, output.stderr);
    assert.equal(JSON.parse(output.stdout).evidence[0].claim, null);
    const reverse = run('dependents', program, 'memory', events, '--json');
    assert.equal(reverse.status, 0, reverse.stderr);
    assert.equal(JSON.parse(reverse.stdout).decisions[0].id, 'strategy@1');
  } finally { await rm(directory, { recursive: true, force: true }); }
});
