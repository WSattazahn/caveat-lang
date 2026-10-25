// ledgers: reviewing a set of past ledgers, and the past reviews a program
// consulted (spec/caveat-ledger-sets-0.1.md).
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { CONSULTED_SOURCE, LEDGER_REVIEW_SCHEMA, LEDGER_SUMMARY_SCHEMA, consultedSources, formatConsulted, formatSummary, reviewLedgers, reviewWithConsulted, summarizeLedgers } from '../lib/ledgers.mjs';
import { real } from './helpers.mjs';

const cli = path.join(fileURLToPath(new URL('../', import.meta.url)), 'bin', 'caveat.mjs');
const caveat = args => spawnSync(process.execPath, [cli, ...args], { encoding: 'utf8' });
const sha256 = text => createHash('sha256').update(text).digest('hex');
const events = names => names.map(event => ({ event, payload: {} }));

// Round 1: a refund made on the customer's own email, which a delivery scan
// later shows was wrong.
const first = `claim refund_warranted;
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
const firstEvents = events(['read_order', 'read_email', 'decide', 'delivered_after_all']);

// Round 2: the agent reads round 1's review first, and the rule now needs
// tracking.
const second = (sha, revision = 'refund@1') => `claim refund_warranted;
evidence order_record from "the order system";
evidence customer_email from "the customer's email";
evidence tracking from "the carrier's tracking record";
evidence lesson from "caveat-review:${sha}#${revision}";
caveat self_reported consequence material;
self_reported qualifies customer_email;
decisions refund limit 4;
state price = 0 min 0 max 500;
state missing = 0 min 0 max 5;
state lost = 0 min 0 max 1;
event recall; event read_order; event read_email; event tracking_lost; event decide;
on recall reveal lesson;
on read_order reveal order_record supports refund_warranted;
on read_order set price = qualified(80, order_record);
on read_email reveal customer_email supports refund_warranted;
on read_email set missing = qualified(1, customer_email);
on tracking_lost reveal tracking supports refund_warranted;
on tracking_lost set lost = qualified(1, tracking);
on decide when missing >= 1 and lost < 1 reject "Read tracking first.";
on decide when missing >= 1 and lost >= 1 commit refund because enough using price * missing * lost retaining self_reported;
`;
const secondEvents = events(['recall', 'read_order', 'read_email', 'tracking_lost', 'decide']);
const open = source => real.open(source);

test('a consulted source names a program by its SHA-256 and a revision', () => {
  const sha = sha256(first);
  assert.deepEqual(CONSULTED_SOURCE.exec(`caveat-review:${sha}#refund@1`).slice(1), [sha, 'refund@1']);
  for (const source of [`caveat-review:${sha}`, `caveat-review:${sha.toUpperCase()}#refund@1`, `caveat-review:${sha}#refund`,
    `caveat-review:${sha}#refund@0`, ` caveat-review:${sha}#refund@1`, 'the order system']) {
    assert.equal(CONSULTED_SOURCE.test(source), false, source);
  }
  const session = real.open(second(sha));
  assert.deepEqual(consultedSources(session.snapshot()), [{ evidence: 'lesson', program_sha256: sha, revision: 'refund@1' }]);
  session.close();
});

test('reviewing a set: the revision with an outcome, and the later one that consulted its review', () => {
  const sha = sha256(first);
  const entries = reviewLedgers(open, [
    { program: 'a.cav', events_file: 'a.events.jsonl', source: first, events: firstEvents },
    { program: 'b.cav', events_file: 'b.events.jsonl', source: second(sha), events: secondEvents },
  ]);
  assert.deepEqual(entries.map(entry => [entry.schema, entry.program, entry.revision]),
    [[LEDGER_REVIEW_SCHEMA, 'a.cav', 'refund@1'], [LEDGER_REVIEW_SCHEMA, 'b.cav', 'refund@1']]);
  const [a, b] = entries;
  assert.equal(a.program_sha256, sha);
  assert.deepEqual(a.decisive.map(item => item.evidence), [['customer_email']]);
  assert.deepEqual(a.came_true, [{ caveat: 'self_reported', carried_by: ['customer_email'], decisive: true }]);
  assert.deepEqual(a.consulted, []);
  assert.equal(b.outcome, null);
  assert.deepEqual(b.decisive.map(item => item.evidence), [['customer_email'], ['tracking']]);
  assert.deepEqual(b.consulted, [{ evidence: 'lesson', program_sha256: sha, revision: 'refund@1', in_lineage: false,
    found: { program: 'a.cav', events: 'a.events.jsonl' }, came_true: [{ caveat: 'self_reported', decisive: true }] }]);
  assert.deepEqual(formatConsulted(b.consulted),
    [`Consulted: lesson, the review of refund@1 of program ${sha.slice(0, 12)}; a.cav: self_reported came true, decisive`]);
});

test('a consulted review the set does not hold, or a revision it has not reviewed, is said so', () => {
  const missing = 'f'.repeat(64);
  const [alone] = reviewLedgers(open, [{ program: 'b.cav', events_file: 'e', source: second(missing), events: secondEvents }]);
  assert.equal(alone.consulted[0].found, null);
  assert.equal(alone.consulted[0].came_true, null);
  assert.match(formatConsulted(alone.consulted)[0], /; no ledger in the set has that program$/);
  const entries = reviewLedgers(open, [
    { program: 'a.cav', events_file: 'e', source: first, events: firstEvents },
    { program: 'b.cav', events_file: 'e', source: second(sha256(first), 'refund@2'), events: secondEvents },
  ]);
  assert.deepEqual(entries[1].consulted[0].found, { program: 'a.cav', events: 'e' });
  assert.equal(entries[1].consulted[0].came_true, null);
  assert.match(formatConsulted(entries[1].consulted)[0], /; a\.cav has no such revision with an outcome$/);
});

test('consulted evidence observed after the deciding event is not listed; one in the lineage says so', () => {
  const sha = sha256(first);
  const late = reviewWithConsulted(() => real.open(second(sha)), events(['read_order', 'read_email', 'tracking_lost', 'decide', 'recall']), 'refund@1');
  assert.deepEqual(late.consulted, []);
  const gated = second(sha).replace('when missing >= 1 and lost >= 1 commit', 'when missing >= 1 and lost >= 1 and observed(lesson) commit');
  const report = reviewWithConsulted(() => real.open(gated), secondEvents, 'refund@1');
  assert.deepEqual(report.consulted, [{ evidence: 'lesson', program_sha256: sha, revision: 'refund@1', in_lineage: true }]);
  assert.match(formatConsulted(report.consulted)[0], /\(in its lineage\)$/);
});

test('a program that consults nothing and has no outcome adds no entry; one that does not load is an error entry', () => {
  const plain = first.replace(/\non delivered_after_all resolve[^\n]*/, '');
  const entries = reviewLedgers(open, [
    { program: 'plain.cav', events_file: 'e', source: plain, events: firstEvents },
    { program: 'broken.cav', events_file: 'e', source: 'claim;', events: [] },
  ]);
  assert.equal(entries.length, 1);
  assert.equal(entries[0].program, 'broken.cav');
  assert.equal(typeof entries[0].error, 'string');
});

// The harder demo's shape: one caveat on two carriers' scans, true only for one.
const carrier = (scan, wrong) => `claim refund_warranted;
evidence order_record from "the order system";
evidence ${scan} from "the delivered scan";
evidence scan_gps from "the scan's GPS";
evidence customer_reply from "the customer's follow-up";
caveat depot_scan consequence material;
depot_scan qualifies ${scan};
decisions refund limit 4;
state price = 0 min 0 max 500;
state delivered = 0 min 0 max 1;
event read_order; event read_tracking; event decide; event outcome;
on read_order reveal order_record supports refund_warranted;
on read_order set price = qualified(60, order_record);
on read_tracking reveal ${scan} opposes refund_warranted;
on read_tracking set delivered = qualified(1, ${scan});
on decide when delivered >= 1 commit refund because enough using price - price * delivered retaining depot_scan;
${wrong
    ? 'on outcome reveal scan_gps supports refund_warranted;\non outcome resolve refund = 60 because scan_gps materialized depot_scan;'
    : 'on outcome reveal customer_reply opposes refund_warranted;\non outcome resolve refund = 0 because customer_reply;'}
`;

test('the summary counts how often each caveat came true, by the evidence that carried it', () => {
  const sent = events(['read_order', 'read_tracking', 'decide', 'outcome']);
  const ledgers = [['zephyr_scan', true], ['northway_scan', false], ['zephyr_scan', true]].map(([scan, wrong], index) => ({
    program: `t${index}.cav`, events_file: `t${index}.events.jsonl`, source: carrier(scan, wrong), events: sent,
  }));
  const entries = reviewLedgers(open, [...ledgers, { program: 'broken.cav', events_file: 'b.jsonl', source: 'claim;', events: [] }]);
  const summary = summarizeLedgers(entries);
  assert.deepEqual(summary, {
    schema: LEDGER_SUMMARY_SCHEMA,
    revisions: 3,
    caveats: [{
      caveat: 'depot_scan', retained: 3, came_true: 2,
      by_evidence: [{ evidence: 'northway_scan', retained: 1, came_true: 0 }, { evidence: 'zephyr_scan', retained: 2, came_true: 2 }],
    }],
  });
  assert.deepEqual(formatSummary(summary), [
    'Across the set (3 revisions with an outcome)',
    '  depot_scan: came true in 2 of 3; carried by northway_scan, 0 of 1; carried by zephyr_scan, 2 of 2',
    '',
  ]);
  assert.deepEqual(formatSummary(summarizeLedgers([])), []);
});

test('caveat review --all finds ledgers under a directory and reports them', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-ledgers-'));
  try {
    const sha = sha256(first);
    const jsonl = list => list.map(item => JSON.stringify(item)).join('\n') + '\n';
    await writeFile(path.join(directory, 'ticket-a.cav'), first);
    await writeFile(path.join(directory, 'ticket-a.events.jsonl'), jsonl(firstEvents.map(({ event }) => ({ event }))));
    await mkdir(path.join(directory, 'ticket-b'));
    await writeFile(path.join(directory, 'ticket-b', 'program.cav'), second(sha));
    await writeFile(path.join(directory, 'ticket-b', 'events.jsonl'), jsonl(secondEvents.map(({ event }) => ({ event }))));
    await mkdir(path.join(directory, '.hidden'));
    await writeFile(path.join(directory, '.hidden', 'x.cav'), 'claim;');
    await writeFile(path.join(directory, '.hidden', 'x.events.jsonl'), '');

    const text = caveat(['review', '--all', directory]);
    assert.equal(text.status, 0, text.stderr);
    assert.match(text.stdout, /ticket-a\.cav: review of refund@1 = 80 {2}in force\n/);
    assert.match(text.stdout, /program\.cav: review of refund@1 = 80 {2}in force\n/);
    assert.match(text.stdout, /\nConsulted: lesson, the review of refund@1 of program [0-9a-f]{12}; [^\n]*ticket-a\.cav: self_reported came true, decisive\n/);
    assert.match(text.stdout, /\nAcross the set \(1 revision with an outcome\)\n {2}self_reported: came true in 1 of 1; carried by customer_email, 1 of 1\n\n2 revisions reviewed in 2 ledgers\.\n$/);

    const json = caveat(['review', '--all', '--json', directory]);
    assert.equal(json.status, 0, json.stderr);
    const all = json.stdout.trim().split('\n').map(line => JSON.parse(line));
    assert.equal(all.at(-1).schema, LEDGER_SUMMARY_SCHEMA);
    const lines = all.filter(line => line.schema === LEDGER_REVIEW_SCHEMA);
    assert.deepEqual(lines.map(line => [path.relative(directory, line.program), line.revision]), [['ticket-a.cav', 'refund@1'], [path.join('ticket-b', 'program.cav'), 'refund@1']]);
    assert.equal(lines[1].consulted[0].found.program, path.join(directory, 'ticket-a.cav'));

    const single = caveat(['review', path.join(directory, 'ticket-b', 'program.cav'), 'refund@1', path.join(directory, 'ticket-b', 'events.jsonl')]);
    assert.equal(single.status, 0, single.stderr);
    assert.match(single.stdout, new RegExp(`\\nConsulted: lesson, the review of refund@1 of program ${sha.slice(0, 12)}\\n$`));

    await writeFile(path.join(directory, 'broken.cav'), 'claim;');
    await writeFile(path.join(directory, 'broken.events.jsonl'), '');
    const broken = caveat(['review', '--all', directory]);
    assert.equal(broken.status, 2);
    assert.match(broken.stderr, /broken\.cav does not load/);
    assert.match(broken.stdout, /2 revisions reviewed in 3 ledgers\./);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
  const empty = await mkdtemp(path.join(tmpdir(), 'caveat-ledgers-'));
  try {
    const none = caveat(['review', '--all', empty]);
    assert.equal(none.status, 2);
    assert.match(none.stderr, /holds no ledger/);
  } finally {
    await rm(empty, { recursive: true, force: true });
  }
  assert.equal(caveat(['review', '--all']).status, 2);
  assert.match(caveat(['review', '--all', 'a', 'b']).stderr, /name one directory of ledgers with --all/);
});
