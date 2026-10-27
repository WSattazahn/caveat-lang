// session.whatIf(event, payload, options): what dispatchView() would return,
// run on a copy that is dropped. Each check drives twins, one asked what-if
// before every event and the other not, and requires the answer to equal the
// twin's real dispatchView() outcome and the asked session to be unchanged
// by asking. The sweep runs every scenarios file in the repository.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readdir, readFile } from 'node:fs/promises';
import path from 'node:path';
import { CaveatError, createRuntime } from '../lib/session.mjs';
import { Real, real, repo, thermostat, trail } from './helpers.mjs';

const state = session => ({ save: session.save(), snapshot: session.snapshotText(), view: session.viewText() });

function step(asked, plain, event, payload = {}, options) {
  const before = state(asked);
  const answer = options ? asked.whatIf(event, payload, options) : asked.whatIf(event, payload);
  const at = `${event} ${JSON.stringify(payload)}`;
  assert.deepEqual(state(asked), before, `${at}: asking changed the session`);
  const outcome = options ? plain.dispatchView(event, payload, options) : plain.dispatchView(event, payload);
  assert.deepEqual(answer, outcome, `${at}: the answer is what the event does`);
  const applied = options ? asked.dispatchView(event, payload, options) : asked.dispatchView(event, payload);
  assert.deepEqual(applied, outcome, `${at}: the event after asking`);
  assert.deepEqual(state(asked), state(plain), `${at}: the same session`);
  return answer;
}

const DECISION = `
claim safe;
claim misreading;
evidence report from "the check report";
evidence recheck from "the agent re-read the log";
decisions ship limit 4;
state ready = 0;
event see;
event decide;
event misread;
on see reveal report supports safe;
on see set ready = qualified(1, report);
on decide when ready > 0 commit ship because enough using ready;
on misread reveal recheck supports misreading;
on misread withdraw report because recheck;
on misread when committed(ship) and not reopened(ship) and rests_on_withdrawn(ship)
    reopen ship because recheck;
`;

test('asking what a withdrawal would do shows the reopening, and changes nothing', () => {
  const asked = real.open(DECISION);
  const plain = real.open(DECISION);
  const ship = view => view.commitments.find(commitment => commitment.action === 'ship@1');
  try {
    step(asked, plain, 'see');
    step(asked, plain, 'decide');
    const again = asked.whatIf('decide');
    assert.equal(again.outcome, 'rejected', 'a decision in force must be reopened before it is revised');
    assert.equal(again.code, 'decision_in_force');
    const answer = asked.whatIf('misread');
    assert.equal(answer.outcome, 'accepted');
    assert.deepEqual(ship(answer.view), { ...ship(asked.view()), open: true, reopened_by: ['recheck'] }, 'the answer shows it reopened');
    assert.equal(ship(asked.view()).open, false, 'the real decision stays in force');
    step(asked, plain, 'misread');
    const revised = asked.whatIf('decide');
    assert.equal(revised.outcome, 'accepted');
    assert.deepEqual(revised.view.commitment_grounds['ship@2'], { evidence: ['report'], caveats: ['withdrawn'] });
    step(asked, plain, 'decide');
  } finally { asked.close(); plain.close(); }
});

test('the answer equals the event, with and without a field selection', () => {
  for (const [source, events] of [
    [thermostat, [17, 25, 5, 40, 0].map(value => ['read', { value }])],
    [trail, [
      ['advance', { dt: 1 }],
      ['observe', { target: 'stone', method: 'report', condition: 1 }],
      ['plan', { target: 'stone' }],
      ['change', { target: 'stone', condition: 'blocked' }],
      ['plan', { target: 'reed' }],
      ['advance', { dt: 30 }],
      ['rescue', {}],
      ['no_such_event', {}],
      ['advance', { dt: -1 }],
    ]],
  ]) {
    for (const options of [undefined, { fields: ['bindings', 'decision_series', 'effects'] }]) {
      const asked = real.open(source);
      const plain = real.open(source);
      try {
        for (const [event, payload] of events) step(asked, plain, event, payload, options);
      } finally { asked.close(); plain.close(); }
    }
  }
});

test('a payload or field the kit refuses throws before the runtime, and the session is usable', () => {
  const session = real.open(thermostat);
  try {
    assert.throws(() => session.whatIf('read', { value: Number.NaN }), error => error instanceof CaveatError && error.kind === 'payload');
    assert.throws(() => session.whatIf('read', { value: 1 }, { fields: ['nope'] }), TypeError);
    assert.throws(() => session.whatIf(''), TypeError);
    assert.throws(() => session.whatIf('read', { value: 1 }, { agent: '' }), TypeError);
    assert.equal(session.state, 'open');
    assert.equal(session.dispatchView('read', { value: 17 }).outcome, 'accepted');
  } finally { session.close(); }
});

test('a fatal what-if throws but leaves the session usable', () => {
  class Failing {
    constructor(source) { this.inner = new Real(source); }
    static restore(source, saved) { return Real.restore(source, saved); }
    what_if_view_outcome() {
      throw JSON.stringify({ schema: 'caveat-dispatch/0.1', outcome: 'fatal', code: 'unclassified', message: 'copy failed' });
    }
  }
  for (const name of Object.getOwnPropertyNames(Real.prototype)) {
    if (name !== 'constructor' && !(name in Failing.prototype)) {
      Failing.prototype[name] = function (...args) { return this.inner[name](...args); };
    }
  }
  const session = createRuntime(Failing).open(thermostat);
  try {
    assert.throws(() => session.whatIf('read', { value: 17 }), error => error instanceof CaveatError && error.kind === 'fatal' && /copy failed/.test(error.message));
    assert.equal(session.state, 'open');
    assert.equal(session.dispatchView('read', { value: 17 }).outcome, 'accepted');
  } finally { session.close(); }
});

test('a runtime build without what-if throws a load error and leaves the session untouched', () => {
  class Older {
    constructor(source) { this.inner = new Real(source); }
  }
  for (const name of Object.getOwnPropertyNames(Real.prototype)) {
    if (name !== 'constructor' && !name.startsWith('what_if')) {
      Older.prototype[name] = function (...args) { return this.inner[name](...args); };
    }
  }
  const session = createRuntime(Older).open(thermostat);
  try {
    assert.throws(() => session.whatIf('read', { value: 17 }), error => error instanceof CaveatError && error.kind === 'load');
    assert.equal(session.state, 'open');
  } finally { session.close(); }
});

async function scenarioFiles(directory) {
  const found = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (entry.name === 'node_modules' || entry.name.startsWith('.') || entry.name === 'target' || entry.name === 'dist') continue;
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) found.push(...await scenarioFiles(full));
    else if (entry.name.endsWith('.scenarios.json')) found.push(full);
  }
  return found;
}

// Every send of every scenario in the repository, asked first. Files whose
// program needs a module library, or whose steps restore, stop at the first
// step this simple driver does not follow; each counts what it reached.
test('the corpus: every scenario event, asked first, answers what it does and changes nothing', async () => {
  let programs = 0;
  let events = 0;
  for (const file of await scenarioFiles(repo)) {
    let document;
    try { document = JSON.parse(await readFile(file, 'utf8')); } catch { continue; }
    if (document?.schema !== 'caveat-scenarios/0.1' || typeof document.source !== 'string') continue;
    let source;
    try { source = await readFile(path.resolve(path.dirname(file), document.source), 'utf8'); } catch { continue; }
    let opened = false;
    for (const scenario of document.scenarios ?? []) {
      let asked;
      let plain;
      try {
        asked = real.open(source);
        plain = real.open(source);
      } catch (error) {
        if (error instanceof CaveatError && error.kind === 'load') break;
        throw error;
      }
      opened = true;
      try {
        for (const item of scenario.steps ?? []) {
          if (!('send' in item)) {
            if ('restore' in item || 'checkpoint' in item || 'load' in item) break;
            continue;
          }
          if (typeof item.send !== 'string' || !item.send) break;
          step(asked, plain, item.send, item.payload ?? {});
          events++;
        }
      } finally { asked.close(); plain.close(); }
    }
    if (opened) programs++;
  }
  assert.ok(programs >= 20, `swept ${programs} programs`);
  assert.ok(events >= 200, `swept ${events} events`);
  console.log(`what-if corpus sweep: ${programs} programs, ${events} events`);
});

const HANDOFF = `
claim plan_ok;
evidence request from "the user's request";
evidence report from "the sub-agent's report";
readings reports from report limit 8;
decisions plan limit 4;
agent main sends delegate;
agent sub sends sub_report;
event delegate;
event sub_report n min 0 max 10;
event note;
on delegate reveal request supports plan_ok;
on delegate commit plan because enough citing qualified(1, request);
on sub_report sample reports = n;
`;

test('with an agent, the answer is what the event would do sent by that agent', () => {
  const asked = real.open(HANDOFF);
  const plain = real.open(HANDOFF);
  try {
    for (const [event, payload, options] of [
      ['delegate', {}, { agent: 'sub' }],
      ['delegate', {}, undefined],
      ['delegate', {}, { agent: 'main' }],
      ['sub_report', { n: 3 }, { agent: 'sub', fields: ['commitments'] }],
      ['sub_report', { n: 3 }, { agent: 'mallory' }],
      ['note', {}, { agent: 'main' }],
      ['note', {}, { fields: ['decision_journal'] }],
      ['sub_report', { n: 4 }, { agent: 'sub', fields: ['decision_journal'] }],
    ]) {
      const answer = step(asked, plain, event, payload, options);
      const agent = options?.agent;
      const accepted = (event === 'delegate' && agent === 'main') || (event === 'sub_report' && agent === 'sub') || event === 'note';
      assert.equal(answer.outcome, accepted ? 'accepted' : 'rejected', `${event} by ${agent}`);
      if (!accepted) assert.equal(answer.code, 'wrong_agent', `${event} by ${agent}`);
    }
    assert.equal(asked.snapshot().last_agent, 'sub');
  } finally { asked.close(); plain.close(); }
});

test('a runtime build without agents on the what-if path throws a load error for an agent and answers without one', () => {
  class Older {
    constructor(source) { this.inner = new Real(source); }
  }
  for (const name of Object.getOwnPropertyNames(Real.prototype)) {
    if (name !== 'constructor' && !name.startsWith('what_if_view_outcome_as') && !name.startsWith('what_if_view_fields_outcome_as')) {
      Older.prototype[name] = function (...args) { return this.inner[name](...args); };
    }
  }
  const session = createRuntime(Older).open(HANDOFF);
  try {
    assert.throws(() => session.whatIf('delegate', {}, { agent: 'main' }), error => error instanceof CaveatError && error.kind === 'load' && /what_if_view_outcome_as/.test(error.message));
    assert.throws(() => session.whatIf('delegate', {}, { agent: 'main', fields: ['bindings'] }), error => error instanceof CaveatError && error.kind === 'load' && /what_if_view_fields_outcome_as/.test(error.message));
    assert.equal(session.state, 'open');
    assert.equal(session.whatIf('note').outcome, 'accepted');
    assert.equal(session.whatIf('delegate').code, 'wrong_agent');
  } finally { session.close(); }
});
