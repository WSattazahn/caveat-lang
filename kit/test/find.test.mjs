// find: past decisions in a set of ledgers, by the words of a question
// (spec/caveat-find-0.1.md).
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { FIND_SCHEMA, findDecisions, formatFound, queryWords } from '../lib/ledgers.mjs';
import { real } from './helpers.mjs';

const cli = path.join(fileURLToPath(new URL('../', import.meta.url)), 'bin', 'caveat.mjs');
const caveat = args => spawnSync(process.execPath, [cli, ...args], { encoding: 'utf8' });
const events = names => names.map(event => ({ event, payload: {} }));
const open = source => real.open(source);

// A refund made on the customer's own email, which a delivery scan later
// shows was wrong (as in spec/caveat-ledger-sets-0.1.md).
const refund = `claim refund_warranted;
evidence order_record from "the order system";
evidence customer_email from "the customer's email";
evidence delivery_scan from "the carrier's signed delivery scan";
caveat self_reported consequence material;
self_reported qualifies customer_email;
decisions refund limit 4;
state price = 0 min 0 max 500;
state missing = 0 min 0 max 5;
event read_order; event read_email; event decide; event delivered_after_all;
on read_order reveal order_record supports refund_warranted;
on read_order set price = qualified(80, order_record);
on read_email reveal customer_email supports refund_warranted;
on read_email set missing = qualified(1, customer_email);
on decide when missing >= 1 commit refund because enough using price * missing retaining self_reported;
on delivered_after_all reveal delivery_scan opposes refund_warranted;
on delivered_after_all resolve refund = 0 because delivery_scan materialized self_reported;
`;
const refundEvents = events(['read_order', 'read_email', 'decide', 'delivered_after_all']);

// An inbox agent whose source is withdrawn whole (as in undo-list.test.mjs):
// a superseded revision, a reopened one, and a send still in force on
// withdrawn evidence.
const inbox = `claim requested;
caveat untrusted consequence high;
evidence mail from "the newsletter as read";
evidence bad from "a finding that the newsletter is malicious";
evidence user_ok from "the user confirming one pending send";
renewable user_ok limit 8;
untrusted qualifies mail;
readings asks from mail limit 8;
decisions add_event limit 4;
decisions send_a limit 1 irreversible;
event read; event ask; event add; event confirm; event send; event found;
on read reveal mail;
on ask sample asks = 1 supports requested;
on add when committed(add_event) and not reopened(add_event) reopen add_event because latest(asks);
on add commit add_event because enough using latest(asks);
on confirm when observed(user_ok) renew user_ok;
on confirm reveal user_ok;
on send commit send_a because enough using latest(asks) retaining untrusted because once user_ok;
on found reveal bad;
on found withdraw every mail because bad;
on found when committed(add_event) and not reopened(add_event) and rests_on_withdrawn(add_event)
    reopen add_event because bad;
`;
const inboxEvents = events(['read', 'ask', 'add', 'ask', 'add', 'confirm', 'send', 'found']);

const set = () => [
  { program: 'ticket-a.cav', events_file: 'ticket-a.events.jsonl', source: refund, events: refundEvents },
  { program: 'inbox.cav', events_file: 'inbox.events.jsonl', source: inbox, events: inboxEvents },
];
const ids = report => report.found.map(item => `${item.program} ${item.revision}`);

test('a query is its words, lowercased; it needs one', () => {
  assert.deepEqual(queryWords('  Refund\tEMAIL\n'), ['refund', 'email']);
  assert.throws(() => findDecisions(open, set(), '  '), /at least one word/);
});

test('a revision is found when every word occurs in its terms, ignoring case', () => {
  const report = findDecisions(open, set(), 'Refund');
  assert.equal(report.schema, FIND_SCHEMA);
  assert.deepEqual(report.words, ['refund']);
  assert.deepEqual(ids(report), ['ticket-a.cav refund@1']);
  const [item] = report.found;
  assert.deepEqual(item.matched, { refund: ['decision refund', 'revision refund@1', 'claim refund_warranted'] });
  assert.equal(item.program_sha256, createHash('sha256').update(refund).digest('hex'));
  assert.deepEqual(item.grounds.evidence, ['customer_email', 'order_record']);
  assert.deepEqual(item.sources, { customer_email: "the customer's email", order_record: 'the order system' });
  assert.deepEqual(item.retained, ['self_reported']);
  assert.deepEqual(item.decided, { sequence: 3, event: 'decide' });
  assert.equal(item.status, 'in force');
  assert.equal(item.current, true);
  assert.deepEqual([item.outcome.value, item.outcome.because, item.outcome.materialized], [0, ['delivery_scan'], ['self_reported']]);
  // Every word must match, each anywhere.
  assert.deepEqual(ids(findDecisions(open, set(), 'customer self_reported')), ['ticket-a.cav refund@1']);
  assert.deepEqual(ids(findDecisions(open, set(), 'customer newsletter')), []);
  // An evidence source, a claim and the program's name match too.
  assert.deepEqual(ids(findDecisions(open, set(), 'carrier')), ['ticket-a.cav refund@1']);
  assert.deepEqual(ids(findDecisions(open, set(), 'requested')), ['inbox.cav add_event@1', 'inbox.cav add_event@2', 'inbox.cav send_a@1']);
  assert.deepEqual(ids(findDecisions(open, set(), 'ticket-a')), ['ticket-a.cav refund@1']);
  // A name given for matching replaces the program's path.
  const named = set().map((ledger, index) => ({ ...ledger, name: `case-${index}` }));
  assert.deepEqual(ids(findDecisions(open, named, 'ticket-a')), []);
  assert.deepEqual(ids(findDecisions(open, named, 'case-1 send')), ['inbox.cav send_a@1']);
});

test('every revision is searched, with its standing: superseded, reopened, or in force on withdrawn evidence', () => {
  const report = findDecisions(open, set(), 'newsletter');
  assert.deepEqual(report.found.map(item => [item.revision, item.status, item.current, item.withdrawn.map(entry => entry.evidence)]), [
    ['add_event@1', 'superseded', false, ['asks@1']],
    ['add_event@2', 'reopened', true, ['asks@2']],
    ['send_a@1', 'in force', true, ['asks@2']],
  ]);
  assert.deepEqual(report.searched.map(entry => [entry.program, entry.revisions, entry.fatal]), [['ticket-a.cav', 1, null], ['inbox.cav', 3, null]]);
  const text = formatFound(report);
  assert.match(text, /^inbox\.cav: add_event@2 = 1 {2}reopened\n {2}based on asks@2 \("the newsletter as read"\); caveats: untrusted\n {2}decided at #5 add\n {2}#8 found: reopened because bad\n {2}withdrawn since: asks@2\n {2}matched newsletter: source the newsletter as read$/m);
  assert.match(text, /\n3 decisions found in 2 ledgers\.$/);
  assert.match(formatFound(findDecisions(open, set(), 'refund')), /\n {2}turned out 0 at #4 delivered_after_all because delivery_scan; came true: self_reported\n/);
  assert.equal(formatFound(findDecisions(open, set(), 'nothing-here')), '0 decisions found in 2 ledgers.');
});

test('a fatal event is reported per ledger; a program that does not load is an error entry', () => {
  // A session whose dispatch of `boom` fails fatally, as a trapped runtime would.
  const fatalOpen = source => {
    const session = real.open(source);
    return { snapshot: () => session.snapshot(), close: () => session.close(),
      dispatch: (event, payload) => { if (event === 'boom') throw Object.assign(new Error('trapped'), { kind: 'trap' }); return session.dispatch(event, payload); } };
  };
  const report = findDecisions(fatalOpen, [
    { program: 'broken.cav', events_file: 'e', source: 'claim;', events: [] },
    { program: 'ticket-a.cav', events_file: 'e', source: refund, events: [...refundEvents.slice(0, 3), { event: 'boom', payload: {} }, refundEvents[3]] },
  ], 'refund');
  assert.equal(report.searched[0].program, 'broken.cav');
  assert.equal(typeof report.searched[0].error, 'string');
  assert.deepEqual(report.searched[1].fatal, { index: 3, event: 'boom', message: 'trapped' });
  // Its decisions are of the session before the fatal event: no outcome yet.
  assert.deepEqual(ids(report), ['ticket-a.cav refund@1']);
  assert.equal(report.found[0].outcome, null);
  assert.match(formatFound(report), /\n1 decision found in 1 ledger\.$/);
});

test('caveat find searches the ledgers under a directory', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-find-'));
  try {
    const lines = list => list.map(item => JSON.stringify(item)).join('\n');
    await writeFile(path.join(directory, 'ticket-a.cav'), refund);
    await writeFile(path.join(directory, 'ticket-a.events.jsonl'), lines(refundEvents));
    await mkdir(path.join(directory, 'inbox'));
    await writeFile(path.join(directory, 'inbox', 'program.cav'), inbox);
    await writeFile(path.join(directory, 'inbox', 'events.jsonl'), lines(inboxEvents));
    const text = caveat(['find', directory, 'self_reported', 'Customer']);
    assert.equal(text.status, 0, text.stderr);
    assert.match(text.stdout, /ticket-a\.cav: refund@1 = 80 {2}in force\n/);
    assert.match(text.stdout, /\n1 decision found in 2 ledgers\.\n$/);
    // Words match a program's path under the directory, not the directory.
    const json = caveat(['find', '--json', directory, 'inbox', path.basename(directory)]);
    assert.equal(json.status, 0, json.stderr);
    const report = JSON.parse(json.stdout);
    assert.equal(report.schema, FIND_SCHEMA);
    assert.deepEqual(report.found, []);
    const inboxOnly = JSON.parse(caveat(['find', '--json', directory, 'inbox']).stdout);
    assert.deepEqual(inboxOnly.found.map(item => [path.relative(directory, item.program), item.revision]),
      [[path.join('inbox', 'program.cav'), 'add_event@1'], [path.join('inbox', 'program.cav'), 'add_event@2'], [path.join('inbox', 'program.cav'), 'send_a@1']]);
    // A program that does not load: exit 2, the rest still searched.
    await writeFile(path.join(directory, 'broken.cav'), 'claim;');
    await writeFile(path.join(directory, 'broken.events.jsonl'), '');
    const broken = caveat(['find', directory, 'refund']);
    assert.equal(broken.status, 2);
    assert.match(broken.stderr, /broken\.cav does not load/);
    assert.match(broken.stdout, /1 decision found in 2 ledgers\./);
  } finally { await rm(directory, { recursive: true, force: true }); }
  const empty = await mkdtemp(path.join(tmpdir(), 'caveat-find-'));
  try {
    const none = caveat(['find', empty, 'refund']);
    assert.equal(none.status, 2);
    assert.match(none.stderr, /holds no ledger/);
  } finally { await rm(empty, { recursive: true, force: true }); }
  assert.match(caveat(['find', 'ledgers']).stderr, /name a directory of ledgers and at least one word to find/);
});
