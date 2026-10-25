// The undo list, `withdraw every` and `because once` through the package
// (spec/caveat-untrusted-sources-0.1.md).
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { formatUndoList, undoList, UNDO_SCHEMA } from '../lib/explain.mjs';
import { real } from './helpers.mjs';

const cli = fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url));

// An inbox agent: reversible calendar entries, irreversible sends, one
// confirmation per send, and a source withdrawn whole.
const source = `claim requested;
caveat untrusted consequence high;
evidence mail from "the newsletter as read";
evidence bad from "a finding that the newsletter is malicious";
evidence user_ok from "the user confirming one pending send";
renewable user_ok limit 8;
untrusted qualifies mail;
readings asks from mail limit 8;
decisions add_event limit 4;
decisions send_a limit 1 irreversible;
decisions send_b limit 1 irreversible;
event read; event ask; event add; event confirm; event send; event send_again; event found;
on read reveal mail;
on ask sample asks = 1 supports requested;
on add when committed(add_event) and not reopened(add_event) reopen add_event because latest(asks);
on add commit add_event because enough using latest(asks);
on confirm when observed(user_ok) renew user_ok;
on confirm reveal user_ok;
on send commit send_a because enough using latest(asks) retaining untrusted because once user_ok;
on send_again commit send_b because enough using latest(asks) retaining untrusted because once user_ok;
on found reveal bad;
on found withdraw every mail because bad;
on found when committed(add_event) and not reopened(add_event) and rests_on_withdrawn(add_event)
    reopen add_event because bad;
`;
const events = [['read'], ['ask'], ['add'], ['ask'], ['add'], ['confirm'], ['send'], ['found']];

function play(list) {
  const session = real.open(source);
  try {
    const outcomes = list.map(([event, payload = {}]) => session.dispatch(event, payload));
    return { outcomes, snapshot: session.snapshot() };
  } finally { session.close(); }
}

test('a confirmation backs one send; the second waits for another confirmation', () => {
  const { outcomes } = play([...events.slice(0, 7), ['send_again'], ['confirm'], ['send_again']]);
  assert.deepEqual(outcomes.map(item => item.code ?? item.outcome),
    ['accepted', 'accepted', 'accepted', 'accepted', 'accepted', 'accepted', 'accepted', 'reason_spent', 'accepted', 'accepted']);
  assert.equal(outcomes[7].origin, 'evaluation');
});

test('withdraw every withdraws the source and everything read from it', () => {
  const { snapshot } = play(events);
  assert.deepEqual(snapshot.withdrawals.map(item => item.evidence), ['mail', 'asks@1', 'asks@2']);
});

test('the undo list holds every revision on withdrawn evidence, superseded and irreversible included', () => {
  const { snapshot } = play(events);
  const original = structuredClone(snapshot);
  const report = undoList(snapshot);
  assert.deepEqual(snapshot, original, 'the list reads the snapshot only');
  assert.equal(report.schema, UNDO_SCHEMA);
  assert.equal(report.sequence, snapshot.sequence);
  assert.equal(report.withdrawals.length, 3);
  assert.deepEqual(report.decisions.map(({ id, status, irreversible }) => ({ id, status, irreversible })), [
    { id: 'add_event@1', status: 'superseded', irreversible: undefined },
    { id: 'add_event@2', status: 'reopened', irreversible: undefined },
    { id: 'send_a@1', status: 'in force', irreversible: true },
  ]);
  assert.deepEqual(report.decisions[0].withdrawn.map(item => item.evidence), ['asks@1']);
  assert.deepEqual(report.decisions[2].withdrawn.map(item => item.evidence), ['asks@2']);
  const text = formatUndoList(report, 'inbox.cav', events.length);
  assert.match(text, /^What rests on withdrawn evidence in inbox\.cav after 8 events/);
  const order = ['In force, not reopened', 'send_a@1 = 1  irreversible', 'Reopened', 'add_event@2', 'Superseded', 'add_event@1'];
  let at = 0;
  for (const part of order) {
    const found = text.indexOf(part, at);
    assert.ok(found >= at, `${part} in order:\n${text}`);
    at = found;
  }
});

test('nothing withdrawn, nothing listed', () => {
  const { snapshot } = play(events.slice(0, 3));
  assert.deepEqual(undoList(snapshot), { schema: UNDO_SCHEMA, sequence: 3, withdrawals: [], decisions: [] });
  assert.match(formatUndoList(undoList(snapshot)), /Withdrawals\n {2}nothing/);
});

test('caveat undo-list prints the list, as text and as JSON', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-undo-'));
  try {
    const program = path.join(directory, 'inbox.cav');
    const file = path.join(directory, 'events.jsonl');
    await writeFile(program, source);
    await writeFile(file, events.map(([event]) => JSON.stringify({ event })).join('\n') + '\n');
    const text = spawnSync(process.execPath, [cli, 'undo-list', program, file], { encoding: 'utf8' });
    assert.equal(text.status, 0, text.stderr);
    assert.match(text.stdout, /rests on asks@1 withdrawn at #8 during found because bad/);
    const json = spawnSync(process.execPath, [cli, 'undo-list', '--json', program, file], { encoding: 'utf8' });
    assert.equal(json.status, 0, json.stderr);
    const report = JSON.parse(json.stdout);
    assert.equal(report.schema, UNDO_SCHEMA);
    assert.equal(report.events.length, 8);
    assert.equal(report.decisions.length, 3);
    const missing = spawnSync(process.execPath, [cli, 'undo-list', program], { encoding: 'utf8' });
    assert.equal(missing.status, 2);
  } finally { await rm(directory, { recursive: true, force: true }); }
});
