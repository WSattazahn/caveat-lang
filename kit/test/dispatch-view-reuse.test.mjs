// A differential check of dispatchView(event, payload, { reuse: true })
// against dispatchView(event, payload). Over every tracked program that opens
// as a reactive session, with a seeded random stream of valid and invalid
// events, and over the Glowcap, agent ledger and Trail Rescue streams, one
// session goes through dispatchView() and a twin through the reused view.
// After every event the two outcomes must be the same JSON text, byte for
// byte, the reused view must be read-only, and every member the event left
// alone must be the object the last reused view held. Now and then the twin
// goes through dispatch(), dispatchViewDelta() or a refused event instead, so
// the view it holds goes stale and the next patch must start from nothing.
//
// DISPATCH_VIEW_EVENTS sets the random events per program (default 120) and
// DISPATCH_VIEW_SEED the seed (default 20260929).
import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { real, repo, thermostat } from './helpers.mjs';
import { loadStreams } from './workload-streams.mjs';

const EVENTS = Number(process.env.DISPATCH_VIEW_EVENTS ?? 120);
const SEED = Number(process.env.DISPATCH_VIEW_SEED ?? 20260929);

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
    if (random.chance(0.2)) return random.below(members.length + 1) + 1;
    return members[random.below(members.length)];
  }
  if (random.chance(0.05)) return max + 1;
  const span = Math.min(max - min, 1e6);
  const number = min + random.unit() * span;
  if (random.chance(0.02)) return -0;
  return random.chance(0.5) ? Math.round(number) : number;
}

function randomStep(random, events) {
  if (random.chance(0.03) || !events.length) return ['no_such_event', {}];
  const { name, parameters } = events[random.below(events.length)];
  const payload = Object.fromEntries(parameters.map(parameter => [parameter.name, value(random, parameter)]));
  if (parameters.length && random.chance(0.03)) delete payload[parameters[random.below(parameters.length)].name];
  return [name, payload];
}

const fatalOf = run => { try { run(); } catch (error) { return error; } return null; };

function frozen(value) {
  if (value === null || typeof value !== 'object') return true;
  return Object.isFrozen(value) && Object.values(value).every(frozen);
}

// Plays steps on twins; `other(index)` says when the twin takes the event
// another way instead.
function play(label, source, steps, count, other = () => null) {
  const tally = { events: 0, accepted: 0, rejected: 0, fatal: 0, reused: 0, other: 0 };
  let plain = real.open(source);
  let reused = real.open(source);
  let held = null;
  try {
    for (let index = 0; index < count; index += 1) {
      const [event, payload] = Array.isArray(steps) ? steps[index] : steps(plain);
      const at = `${label}, event ${index + 1}: ${event} ${JSON.stringify(payload)}`;
      const way = other(index);
      let expected;
      const plainFatal = fatalOf(() => { expected = plain.dispatchView(event, payload); });
      let outcome;
      const reusedFatal = fatalOf(() => {
        if (way === 'dispatch') outcome = reused.dispatch(event, payload);
        else if (way === 'delta') outcome = reused.dispatchViewDelta(event, payload);
        else outcome = reused.dispatchView(event, payload, { reuse: true });
      });
      tally.events += 1;
      if (plainFatal || reusedFatal) {
        assert.ok(plainFatal && reusedFatal, `${at}: fatal on one path only: ${plainFatal?.message ?? reusedFatal?.message}`);
        assert.equal(reusedFatal.message, plainFatal.message, at);
        tally.fatal += 1;
        plain.close(); reused.close();
        plain = real.open(source);
        reused = real.open(source);
        held = null;
        continue;
      }
      if (way !== null) {
        tally.other += 1;
        // The runtime forgets the view it kept, so the next one is new.
        held = null;
        assert.equal(outcome.outcome, expected.outcome, at);
        continue;
      }
      assert.equal(JSON.stringify(outcome), JSON.stringify(expected), `${at}: not the same text`);
      if (outcome.outcome === 'accepted') {
        tally.accepted += 1;
        assert.ok(frozen(outcome.view), `${at}: the view is not read-only`);
        assert.equal(JSON.stringify(JSON.parse(reused.viewText())), JSON.stringify(outcome.view), `${at}: not the session's view`);
        if (held) {
          // A member the event left alone is the held one; one it changed is new.
          for (const [name, member] of Object.entries(outcome.view)) {
            if (member === null || typeof member !== 'object') continue;
            if (JSON.stringify(member) === JSON.stringify(held[name])) {
              assert.equal(member, held[name], `${at}: ${name} is unchanged but not reused`);
              tally.reused += 1;
            } else {
              assert.notEqual(member, held[name], `${at}: ${name} changed but is the held object`);
            }
          }
        }
        held = outcome.view;
      } else {
        tally.rejected += 1;
      }
    }
  } finally { plain.close(); reused.close(); }
  return tally;
}

const add = (total, tally) => { for (const key of Object.keys(tally)) total[key] = (total[key] ?? 0) + tally[key]; };

test('the reused view is dispatchView\'s view, byte for byte, on every tracked program', async () => {
  const listed = spawnSync('git', ['ls-files', '-z', '--', '*.cav'], { cwd: repo, encoding: 'utf8' });
  assert.equal(listed.status, 0, listed.stderr);
  const files = listed.stdout.split('\0').filter(Boolean).sort();
  const total = { programs: 0 };
  for (const [index, file] of files.entries()) {
    const source = await readFile(path.join(repo, file), 'utf8');
    let probe;
    try { probe = real.open(source); } catch (error) {
      assert.equal(error.kind, 'load', file);
      continue;
    }
    const { events } = probe.snapshot();
    probe.close();
    const random = mulberry(SEED + index);
    const ways = mulberry(SEED - index);
    const other = () => (ways.chance(0.05) ? (ways.chance(0.5) ? 'dispatch' : 'delta') : null);
    add(total, play(`${file} (seed ${SEED + index})`, source, () => randomStep(random, events), EVENTS, other));
    total.programs += 1;
  }
  console.log(`dispatch-view reuse: ${JSON.stringify(total)}`);
  assert.ok(total.programs >= 50, `only ${total.programs} programs open`);
  assert.ok(total.accepted > 0 && total.rejected > 0 && total.other > 0);
  assert.ok(total.reused > 0, 'no member was ever reused');
});

test('the reused view is dispatchView\'s view on the Glowcap, ledger and Trail Rescue streams', async () => {
  const totals = {};
  for (const { id, source, episodes, skipped } of await loadStreams()) {
    if (skipped) { console.log(`dispatch-view reuse workloads: skipped ${id}: ${skipped}`); continue; }
    const total = { episodes: 0 };
    for (const [number, steps] of episodes.entries()) {
      add(total, play(`${id} episode ${number + 1}`, source, steps, steps.length));
      total.episodes += 1;
    }
    assert.equal(total.fatal, 0, id);
    totals[id] = total;
  }
  console.log(`dispatch-view reuse workloads: ${JSON.stringify(totals)}`);
});

test('reuse is checked before the runtime is called', () => {
  const session = real.open(thermostat);
  try {
    assert.throws(() => session.dispatchView('read', { value: 20 }, { reuse: 'yes' }), TypeError);
    assert.throws(() => session.dispatchView('read', { value: 20 }, { reuse: true, fields: ['bindings'] }), TypeError);
    assert.equal(session.snapshot().sequence, 0);
    const { view } = session.dispatchView('read', { value: 20 }, { reuse: true });
    assert.equal(view.sequence, 1);
    assert.throws(() => { 'use strict'; view.bindings.extra = 1; }, TypeError);
  } finally { session.close(); }
});
