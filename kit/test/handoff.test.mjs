// A main agent and a sub-agent share one program. scopeSession keeps each to
// its own events, and replayOnto merges two sub-agents' work, reporting the
// events that no longer apply. The program is the sub-agent handoff
// example; its scenarios pass on their own.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { real, sourceReader } from './helpers.mjs';
import { CaveatError } from '../lib/session.mjs';
import { ScopedSession, replayOnto, scopeSession } from '../lib/handoff.mjs';
import { formatFileReport, parseScenarioFile, runScenarioFile } from '../lib/scenarios.mjs';

const fixture = new URL('../examples/handoff/handoff.cav', import.meta.url);
const source = await readFile(fixture, 'utf8');
const events = real.interface(source).events.map(item => item.name);
const mainEvents = events.filter(name => name.startsWith('main_'));
const subEvents = events.filter(name => name.startsWith('sub_'));

function handedOff() {
  const session = real.open(source);
  const main = scopeSession(session, mainEvents);
  for (const event of ['main_request', 'main_notify', 'main_rule_out_rewrite']) assert.equal(main.dispatch(event).outcome, 'accepted');
  assert.equal(main.dispatch('main_grant', { scope: 'edit' }).outcome, 'accepted');
  assert.equal(main.dispatch('main_handoff').outcome, 'accepted');
  return { session, main };
}

test('the prototype\'s scenarios pass', async () => {
  const doc = parseScenarioFile(await readFile(new URL('../examples/handoff/handoff.scenarios.json', import.meta.url), 'utf8'));
  const result = await runScenarioFile(doc, { runtime: real, file: 'handoff.scenarios.json', readSource: sourceReader({ 'handoff.cav': source }) });
  assert.equal(result.failed, 0, formatFileReport(result));
  assert.equal(result.passed, doc.scenarios.length);
});

test('an event outside the scope is refused and never reaches the session', () => {
  const { session, main } = handedOff();
  try {
    const sub = scopeSession(real.restore(source, main.save()), subEvents);
    const before = sub.snapshotText();
    const refused = sub.dispatch('main_return');
    assert.deepEqual(refused, {
      schema: 'caveat-dispatch/0.1', outcome: 'rejected', origin: 'input', code: 'unknown_event',
      message: `main_return is not one of this session's events: ${subEvents.join(', ')}`,
    });
    assert.equal(sub.snapshotText(), before);
    assert.equal(sub.dispatchView('main_return').code, 'unknown_event');
    assert.equal(sub.dispatchViewDelta('main_return').code, 'unknown_event');
    assert.deepEqual(sub.accepted, []);
    assert.equal(sub.view().bindings.status.plan, 'with the sub-agent');
  } finally { session.close(); }
});

test('within its scope the runtime still decides, and only accepted events are kept', () => {
  const { session, main } = handedOff();
  const inner = real.restore(source, main.save());
  try {
    const sub = scopeSession(inner, subEvents);
    const foreclosed = sub.dispatch('sub_rename');
    assert.equal(`${foreclosed.origin}/${foreclosed.code}`, 'evaluation/foreclosed');
    assert.equal(sub.dispatch('sub_rewrite').message, 'Ruled out by the main agent: no_rewrite.');
    assert.equal(sub.dispatchView('sub_edit', {}, { fields: ['bindings'] }).outcome, 'accepted');
    assert.equal(sub.dispatch('sub_report', { result: 'pass', coverage: 'linux' }).outcome, 'accepted');
    assert.deepEqual(sub.accepted, [{ event: 'sub_edit', payload: {} }, { event: 'sub_report', payload: { result: 'pass', coverage: 'linux' } }]);
    const copy = sub.accepted;
    copy[1].payload.result = 'fail';
    assert.equal(sub.accepted[1].payload.result, 'pass', 'accepted is a copy');
    assert.deepEqual(main.accepted.map(item => item.event), ['main_request', 'main_notify', 'main_rule_out_rewrite', 'main_grant', 'main_handoff']);
    assert.ok(Object.isFrozen(sub.events));
    assert.equal(sub.state, 'open');
  } finally { session.close(); inner.close(); }
});

test('a payload dispatch would refuse throws before the scope is checked', () => {
  const session = real.open(source);
  try {
    const sub = scopeSession(session, subEvents);
    assert.throws(() => sub.dispatch('main_return', { value: Number.NaN }), error => error instanceof CaveatError && error.kind === 'payload');
    assert.throws(() => sub.dispatch(''), TypeError);
  } finally { session.close(); }
});

test('a scope must be distinct non-empty names', () => {
  const session = real.open(source);
  try {
    assert.throws(() => scopeSession(session, 'sub_edit'), TypeError);
    assert.throws(() => scopeSession(session, ['sub_edit', 'sub_edit']), /duplicate event sub_edit/);
    assert.throws(() => scopeSession(session, ['']), TypeError);
    assert.throws(() => scopeSession(null, ['sub_edit']), TypeError);
    assert.ok(scopeSession(session, []) instanceof ScopedSession);
  } finally { session.close(); }
});

test('two sub-agents merge by replay; what no longer applies is a conflict, and order matters', () => {
  const { session, main } = handedOff();
  const ledger = main.save();
  const opened = [];
  const restore = () => { const next = real.restore(source, ledger); opened.push(next); return next; };
  try {
    const a = scopeSession(restore(), subEvents);
    const b = scopeSession(restore(), subEvents);
    assert.equal(a.dispatch('sub_edit').outcome, 'accepted');
    assert.equal(a.dispatch('sub_report', { result: 'pass', coverage: 'full' }).outcome, 'accepted');
    assert.equal(b.dispatch('sub_edit').outcome, 'accepted');
    assert.equal(b.dispatch('sub_object').outcome, 'accepted');
    assert.equal(b.dispatch('sub_report', { result: 'fail', coverage: 'full' }).outcome, 'accepted');

    const merged = restore();
    replayOnto(merged, a.accepted);
    const result = replayOnto(scopeSession(merged, subEvents), b.accepted);
    assert.deepEqual(result.accepted.map(item => item.event), ['sub_object', 'sub_report']);
    assert.equal(result.conflicts.length, 1);
    const [conflict] = result.conflicts;
    assert.deepEqual([conflict.index, conflict.event, conflict.payload], [0, 'sub_edit', {}]);
    assert.equal(`${conflict.outcome.origin}/${conflict.outcome.code}`, 'evaluation/decision_in_force');

    const other = restore();
    replayOnto(other, b.accepted);
    replayOnto(other, a.accepted);
    const latest = item => item.dispatch('main_ship').outcome;
    assert.equal(latest(scopeSession(merged, mainEvents)), 'rejected', 'B last: the latest report failed');
    assert.equal(latest(scopeSession(other, mainEvents)), 'accepted', 'A last: the latest report passed');
  } finally { session.close(); for (const item of opened) item.close(); }
});

test('replayOnto rejects entries that are not events', () => {
  const session = real.open(source);
  try {
    assert.throws(() => replayOnto(session, {}), TypeError);
    assert.throws(() => replayOnto(session, [null]), /events\[0\]/);
    assert.deepEqual(replayOnto(session, []), { accepted: [], conflicts: [] });
  } finally { session.close(); }
});

test('the example host keeps each agent to its events and reports the merge conflict', async () => {
  const { runHandoff } = await import('../examples/handoff/host.mjs');
  const result = runHandoff({ runtime: real, source });
  const outcome = (agent, event) => result.steps.find(step => step.agent === agent && step.event === event).outcome;
  assert.deepEqual(result.foreclosed, { drop_column: ['notify_customers@1'], rename_column: ['notify_customers@1'] });
  assert.equal(outcome('a', 'sub_rename'), 'evaluation/foreclosed');
  assert.equal(outcome('a', 'main_return'), 'input/unknown_event');
  assert.equal(outcome('b', 'sub_rewrite'), 'policy/reject');
  assert.equal(outcome('main', 'main_ship'), 'evaluation/caveated_irreversible');
  assert.equal(outcome('main', 'main_ship_on_linux'), 'accepted');
  assert.deepEqual(result.replayed, ['sub_object']);
  assert.deepEqual(result.conflicts.map(item => [item.event, item.outcome]), [['sub_edit', 'evaluation/decision_in_force']]);
  assert.deepEqual(result.plan.map(item => item.change), ['committed', 'held', 'deferred', 'released', 'reopened']);
  assert.deepEqual(result.ship, { evidence: ['reports@1', 'return_review'], caveats: ['linux_only', 'style_unchecked'] });
});
