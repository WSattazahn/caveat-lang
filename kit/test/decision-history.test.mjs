// decisionHistory reads every revision of a decision series from the journal,
// the same from a view, a View 0.2 and a snapshot, lists a revision a windowed
// journal retired rather than dropping it, and the starter host answers it
// from its current view.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { real } from './helpers.mjs';
import { decisionHistory } from '../lib/session.mjs';
import { StarterError, memoryStore, openHost } from '../lib/starter.mjs';

const source = await readFile(new URL('./fixtures/starter/assessment.cav', import.meta.url), 'utf8');
const windowed = await readFile(new URL('./fixtures/decision-history/windowed.cav', import.meta.url), 'utf8');
const events = [['observe', { confidence: 80 }], ['assess', {}], ['retract', {}], ['observe', { confidence: 90 }], ['assess', {}]];

function sessionAfter(count) {
  const session = real.open(source);
  for (const [event, payload] of events.slice(0, count)) assert.equal(session.dispatch(event, payload).outcome, 'accepted');
  return session;
}

test('each revision keeps what it was committed on and what reopened it', () => {
  const session = sessionAfter(events.length);
  try {
    const history = decisionHistory(session.view(), 'assessment');
    assert.deepEqual(history.map(item => [item.revision, item.status, item.retired]), [['assessment@1', 'superseded', false], ['assessment@2', 'in_force', false]]);
    const [first, second] = history;
    assert.equal(first.committed.event, 'assess');
    assert.equal(first.committed.value, 80);
    assert.equal(first.reopened.event, 'retract');
    assert.deepEqual(first.reopened.because, ['recheck']);
    assert.equal(second.committed.value, 90);
    assert.equal(second.reopened, null);
    const view = session.view();
    for (const entry of view.decision_journal.filter(item => item.change === 'committed')) {
      assert.deepEqual(history.find(item => item.revision === entry.commitment).committed.because, entry.because);
    }
    assert.deepEqual(decisionHistory(session.view({ schema: '0.2' }), 'assessment'), history);
    assert.deepEqual(decisionHistory(session.snapshot(), 'assessment'), history);
  } finally { session.close(); }
});

test('the guide\'s join finds the past revision resting on withdrawn evidence', () => {
  const session = sessionAfter(events.length);
  try {
    const withdrawn = new Set((session.snapshot().withdrawals ?? []).map(item => item.evidence));
    const restingOnWithdrawn = decisionHistory(session.view(), 'assessment')
      .filter(revision => revision.committed?.because.some(name => withdrawn.has(name)));
    assert.deepEqual(restingOnWithdrawn.map(item => item.revision), ['assessment@1']);
  } finally { session.close(); }
});

test('a revision whose journal entries retired is listed, not dropped', () => {
  const session = real.open(windowed);
  try {
    for (const event of ['read', 'decide', 'redo', 'read', 'decide', 'redo', 'read', 'decide']) {
      assert.equal(session.dispatch(event, {}).outcome, 'accepted');
    }
    const view = session.view();
    assert.equal(view.decision_journal.length, 2, 'the window kept two entries');
    const history = decisionHistory(view, 'verdict');
    assert.deepEqual(history.map(item => [item.revision, item.status, item.retired]),
      [['verdict@1', 'superseded', true], ['verdict@2', 'superseded', true], ['verdict@3', 'in_force', false]]);
    assert.equal(history[0].committed, null);
    assert.equal(history[0].reopened, null);
    assert.equal(history[1].committed, null);
    assert.equal(history[1].reopened.event, 'redo', 'a reopening the window kept is still shown');
    assert.equal(history[2].committed.event, 'decide');
  } finally { session.close(); }
});

test('the current revision reads reopened until it is committed again', () => {
  const session = sessionAfter(3);
  try {
    const history = decisionHistory(session.view(), 'assessment');
    assert.deepEqual(history.map(item => item.status), ['reopened']);
    assert.deepEqual(decisionHistory(session.view({ schema: '0.2' }), 'assessment').map(item => item.status), ['reopened']);
  } finally { session.close(); }
});

test('a decision with no revision has an empty history, and an unknown name throws', () => {
  const session = sessionAfter(0);
  try {
    assert.deepEqual(decisionHistory(session.view(), 'assessment'), []);
    assert.throws(() => decisionHistory(session.view(), 'missing'), TypeError);
  } finally { session.close(); }
});

test('the history does not share arrays with the view', () => {
  const session = sessionAfter(2);
  try {
    const view = session.view();
    decisionHistory(view, 'assessment')[0].committed.because.push('changed');
    assert.deepEqual(decisionHistory(view, 'assessment')[0].committed.because, view.decision_journal[0].because);
  } finally { session.close(); }
});

test('the starter host answers history from its current view', async () => {
  const host = await openHost({ runtime: real, source, store: memoryStore() });
  try {
    for (const [event, payload] of events) assert.equal((await host.send(event, payload)).accepted, true);
    assert.deepEqual(host.history('assessment'), decisionHistory(host.view(), 'assessment'));
    assert.equal(host.history('assessment').at(-1).status, host.assessment('assessment').status);
    assert.throws(() => host.history('missing'), error => error instanceof StarterError && error.kind === 'unknown');
  } finally { await host.close(); }
});
