// Agents 0.1: the host says which agent sent each event, the program says
// which agents may send which events, and the records carry the answer. See
// spec/caveat-agents-0.1.md.
import test from 'node:test';
import assert from 'node:assert/strict';
import { real, sourceReader } from './helpers.mjs';
import { CaveatError } from '../lib/session.mjs';
import { replayOnto, scopeSession } from '../lib/handoff.mjs';
import { explain, formatExplanation, parseEvents, dependents } from '../lib/explain.mjs';
import { review } from '../lib/review.mjs';
import { createServer } from '../lib/serve.mjs';
import { formatFileReport, runScenarioFile } from '../lib/scenarios.mjs';

const source = `
claim plan_ok; claim done;
evidence request from "the user's request";
evidence report from "the sub-agent's report";
evidence recheck from "the main agent's check of the report";
readings reports from report limit 8;
decisions plan limit 4;
decisions accept limit 4;
agent main sends delegate, main_return;
agent sub sends sub_report;
agent reviewer;
event delegate;
event sub_report n min 0 max 10;
event main_return;
event note;
on delegate reveal request supports plan_ok;
on delegate commit plan because enough citing qualified(1, request);
on sub_report sample reports = n supports done;
on sub_report commit accept because enough using latest(reports);
on main_return when not observed(recheck) reveal recheck opposes done;
on main_return withdraw latest(reports) because recheck;
`;

test('an event goes only from an agent that sends it, and its records name that agent', () => {
  const session = real.open(source);
  try {
    const before = session.snapshotText();
    for (const agent of ['sub', 'reviewer', 'mallory']) {
      const refused = session.dispatch('delegate', {}, { agent });
      assert.equal(refused.outcome, 'rejected');
      assert.deepEqual([refused.origin, refused.code], ['input', 'wrong_agent'], agent);
    }
    assert.equal(session.dispatch('delegate').code, 'wrong_agent', 'a restricted event needs an agent');
    assert.equal(session.snapshotText(), before);
    assert.equal(session.dispatch('delegate', {}, { agent: 'main' }).outcome, 'accepted');
    const viewed = session.dispatchView('sub_report', { n: 3 }, { agent: 'sub' });
    assert.equal(viewed.outcome, 'accepted');
    assert.equal(viewed.view.decision_journal.at(-1).agent, 'sub');
    const snapshot = session.snapshot();
    assert.equal(snapshot.last_agent, 'sub');
    assert.equal(snapshot.reading_streams.reports.occurrences[0].agent, 'sub');
    assert.equal(snapshot.decision_journal[0].agent, 'main');
    assert.equal(session.dispatchViewDelta('note', {}, { agent: 'reviewer' }).outcome, 'accepted');
    assert.equal(session.dispatch('note').outcome, 'accepted');
    assert.equal(session.snapshot().last_agent, undefined);
  } finally {
    session.close();
  }
});

test('the agent option is checked before the runtime is called', () => {
  const session = real.open(source);
  try {
    assert.throws(() => session.dispatch('note', {}, { agent: '' }), TypeError);
    assert.throws(() => session.dispatch('note', {}, { agent: 3 }), TypeError);
    assert.throws(() => session.dispatch('note', {}, 'main'), TypeError);
    assert.equal(session.snapshot().sequence, 0);
  } finally {
    session.close();
  }
});

test('a program that declares no agents refuses one and is otherwise unchanged', () => {
  const plain = 'claim c; evidence e from "e"; event go; on go when not observed(e) reveal e supports c;';
  const session = real.open(plain);
  try {
    assert.equal(session.dispatch('go', {}, { agent: 'main' }).code, 'wrong_agent');
    const accepted = session.dispatch('go');
    assert.equal(accepted.outcome, 'accepted');
    assert.equal('last_agent' in accepted.snapshot, false);
    assert.equal('agents' in real.interface(plain), false);
  } finally {
    session.close();
  }
});

test('the interface lists the agents and what each sends', () => {
  assert.deepEqual(real.interface(source).agents, [
    { name: 'main', sends: ['delegate', 'main_return'] },
    { name: 'reviewer', sends: [] },
    { name: 'sub', sends: ['sub_report'] },
  ]);
});

test('a save keeps the agents; a compact save of an agent-sent session is the state save', () => {
  const session = real.open(source);
  try {
    session.dispatch('delegate', {}, { agent: 'main' });
    session.dispatch('sub_report', { n: 4 }, { agent: 'sub' });
    const compact = JSON.parse(session.save({ compact: true }));
    assert.equal(compact.schema, 'caveat-reactive-save/0.1');
    const restored = real.restore(source, JSON.stringify(compact));
    assert.equal(restored.snapshotText(), session.snapshotText());
    restored.close();
    compact.decision_journal[0].agent = 'sub';
    assert.throws(() => real.restore(source, JSON.stringify(compact)), CaveatError);
  } finally {
    session.close();
  }
});

test('a scope sends as its agent, and replayOnto sends each entry as the agent it names', () => {
  const session = real.open(source);
  const main = scopeSession(session, ['delegate', 'main_return', 'note'], { agent: 'main' });
  assert.equal(main.agent, 'main');
  assert.equal(main.dispatch('delegate').outcome, 'accepted');
  assert.throws(() => main.dispatch('note', {}, { agent: 'sub' }), TypeError);
  assert.equal(main.dispatch('note', {}, { agent: 'main' }).outcome, 'accepted');
  const sub = scopeSession(real.restore(source, main.save()), ['sub_report', 'main_return'], { agent: 'sub' });
  assert.equal(sub.dispatch('main_return').code, 'wrong_agent', 'the program keeps the boundary, not only the scope');
  assert.equal(sub.dispatch('sub_report', { n: 2 }).outcome, 'accepted');
  assert.deepEqual(sub.accepted, [{ event: 'sub_report', payload: { n: 2 }, agent: 'sub' }]);
  const merged = replayOnto(session, sub.accepted);
  assert.deepEqual(merged.conflicts, []);
  assert.equal(session.snapshot().reading_streams.reports.occurrences[0].agent, 'sub');
  const forged = replayOnto(session, [{ event: 'sub_report', payload: { n: 1 }, agent: 'main' }]);
  assert.equal(forged.conflicts[0].outcome.code, 'wrong_agent');
  assert.equal(forged.conflicts[0].agent, 'main');
  session.close();
});

test('explain and dependents show who sent each event, and a plain program explains as before', () => {
  const events = parseEvents([
    '{"event":"delegate","agent":"main"}',
    '{"event":"sub_report","payload":{"n":3},"agent":"sub"}',
    '{"event":"main_return","agent":"main"}',
  ].join('\n'));
  assert.equal(events[1].agent, 'sub');
  assert.throws(() => parseEvents('{"event":"note","agent":""}'), /agent/);
  const session = real.open(source);
  const sent = events.map(({ event, payload, agent }) => {
    const { snapshot, ...outcome } = session.dispatch(event, payload, { agent });
    return { event, payload, agent, outcome, sequence: snapshot.sequence, effects: snapshot.effects };
  });
  const snapshot = session.snapshot();
  session.close();
  const report = explain(snapshot, sent);
  const accept = report.decisions.find(series => series.name === 'accept').revisions[0];
  assert.equal(accept.history[0].agent, 'sub');
  assert.deepEqual(accept.observed_by.map(item => item.agent), ['sub']);
  assert.equal(report.evidence.find(item => item.id === 'reports@1').agent, 'sub');
  assert.equal(report.evidence.find(item => item.id === 'reports@1').withdrawn.agent, 'main');
  const text = formatExplanation(report, 'handoff');
  assert.match(text, /2 {2}sub_report by sub \{"n":3\} {2}accepted/);
  assert.match(text, /#2 sub_report by sub: committed/);
  assert.match(text, /reports@1: observed at #2 sub_report by sub/);
  const rests = dependents(snapshot, 'reports@1');
  assert.equal(rests.changes[0].agent, 'sub');
  assert.equal(rests.withdrawals[0].agent, 'main');

  const plain = real.open(source);
  plain.dispatch('note');
  const quiet = explain(plain.snapshot(), [{ event: 'note', payload: {}, outcome: { outcome: 'accepted' } }]);
  plain.close();
  assert.doesNotMatch(JSON.stringify(quiet), /"agent"/);
  assert.doesNotMatch(formatExplanation(quiet), / by /);
});

test('review replays each event as the agent that sent it', () => {
  const events = [
    { event: 'delegate', agent: 'main' },
    { event: 'sub_report', payload: { n: 3 }, agent: 'sub' },
  ];
  const report = review(() => real.open(source), events, 'accept@1');
  assert.deepEqual(report.decided_by, { sequence: 2, event: 'sub_report', agent: 'sub' });
  assert.equal(report.decisive.length, 0, 'the deciding event observed its own grounds');
});

test('serve takes an agent on a dispatch request', () => {
  const server = createServer({ runtime: real, source });
  const send = request => server.handle(JSON.stringify(request)).response;
  try {
    assert.equal(send({ op: 'dispatch', event: 'delegate', agent: 'sub' }).code, 'wrong_agent');
    assert.equal(send({ op: 'dispatch', event: 'delegate', agent: 'main' }).outcome, 'accepted');
    const bad = send({ op: 'dispatch', event: 'delegate', agent: 7 });
    assert.equal(bad.ok, false);
    assert.match(bad.error.message, /agent/);
    const explained = send({ op: 'explain' });
    assert.equal(explained.report.events[0].agent, 'sub');
    assert.equal(explained.report.events[1].agent, 'main');
  } finally {
    server.close();
  }
});

test('a scenario send names its agent', async () => {
  const doc = {
    schema: 'caveat-scenarios/0.1', source: 'agents.cav',
    scenarios: [{ id: 'A1', title: 'the sub-agent cannot return the work itself', steps: [
      { send: 'delegate', agent: 'main' },
      { send: 'main_return', agent: 'sub', rejected: { origin: 'input', code: 'wrong_agent' } },
      { send: 'sub_report', agent: 'sub', payload: { n: 3 } },
      { expect: { '/last_agent': 'sub', '/decision_journal/1/agent': 'sub' } },
      { checkpoint: 'handed' },
      { resume: true },
      { send: 'main_return', agent: 'main' },
      { expect: { '/withdrawals/0/agent': 'main' } },
    ] }],
  };
  const result = await runScenarioFile(doc, { runtime: real, file: 'agents.scenarios.json', readSource: sourceReader({ 'agents.cav': source }) });
  assert.equal(result.failed, 0, formatFileReport(result));
});

test('the handoff example with agents declared keeps the boundary in the program', async () => {
  const { readFile } = await import('node:fs/promises');
  const handoff = await readFile(new URL('../examples/handoff/handoff.cav', import.meta.url), 'utf8');
  const declared = `${handoff}\nagent main sends main_request, main_handoff, main_return;\nagent sub sends sub_edit, sub_report;\n`;
  const names = real.interface(handoff).events.map(item => item.name);
  const session = real.open(declared);
  const main = scopeSession(session, names.filter(name => name.startsWith('main_')), { agent: 'main' });
  for (const event of ['main_request', 'main_notify', 'main_rule_out_rewrite']) assert.equal(main.dispatch(event).outcome, 'accepted', event);
  assert.equal(main.dispatch('main_grant', { scope: 'edit' }).outcome, 'accepted');
  assert.equal(main.dispatch('main_handoff').outcome, 'accepted');
  // The sub-agent's scope lists main_return, as a careless host might.
  const sub = scopeSession(real.restore(declared, main.save()), [...names.filter(name => name.startsWith('sub_')), 'main_return'], { agent: 'sub' });
  assert.equal(sub.dispatch('main_return').code, 'wrong_agent');
  assert.equal(sub.dispatch('sub_edit').outcome, 'accepted');
  assert.deepEqual(sub.accepted, [{ event: 'sub_edit', payload: {}, agent: 'sub' }]);
  const text = formatExplanation(explain(sub.snapshot()));
  assert.match(text, /#6 sub_edit by sub: committed/);
  session.close();
});
