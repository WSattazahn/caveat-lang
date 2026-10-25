// review: which evidence a decision hinged on, found by replaying its events
// without the events that observed its grounds (spec/caveat-review-0.1.md).
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { formatReview, review, REVIEW_SCHEMA } from '../lib/review.mjs';
import { real } from './helpers.mjs';

// The spec's example.
const vendor = `claim vendor_reliable;
evidence references from "two customer references";
evidence uptime_report from "vendor's own uptime report";
evidence trial from "two-week trial";
evidence analyst from "analyst note";
evidence incident from "incident report";
caveat self_reported consequence material;
caveat short_trial consequence low;
self_reported qualifies uptime_report;
short_trial qualifies trial;
decisions choose limit 4;
state support = 0 min 0 max 10;
event refs; event uptime; event run_trial; event analyst_note; event decide; event failed;
on refs reveal references supports vendor_reliable;
on refs set support = support + qualified(1, references);
on uptime reveal uptime_report supports vendor_reliable;
on uptime set support = support + qualified(2, uptime_report);
on run_trial reveal trial supports vendor_reliable;
on run_trial set support = support + qualified(1, trial);
on analyst_note reveal analyst supports vendor_reliable;
on decide when support >= 3 commit choose because enough using support
  retaining self_reported, short_trial;
on decide when support < 3 reject "Not enough support to choose.";
on failed reveal incident opposes vendor_reliable;
on failed resolve choose = 0 because incident materialized self_reported;
`;
const vendorEvents = ['refs', 'uptime', 'run_trial', 'analyst_note', 'decide', 'failed']
  .map(event => ({ event, payload: {} }));
const opener = source => () => real.open(source);

test('the example: one decisive item, a decisive pair, and the caveat that came true', () => {
  const report = review(opener(vendor), vendorEvents, 'choose@1');
  assert.equal(report.schema, REVIEW_SCHEMA);
  assert.equal(report.value, 4);
  assert.equal(report.status, 'in force');
  assert.deepEqual(report.decided_by, { sequence: 5, event: 'decide' });
  assert.deepEqual(report.decisive.map(item => item.evidence), [['uptime_report'], ['references', 'trial']]);
  assert.equal(report.decisive[0].because.outcome, 'rejected');
  assert.equal(report.decisive[0].because.message, 'Not enough support to choose.');
  assert.deepEqual(report.decisive[0].without, [{ sequence: 2, event: 'uptime' }]);
  assert.deepEqual(report.decisive[0].caveats, ['self_reported']);
  assert.deepEqual(report.value_only.map(item => [item.evidence, item.value]), [[['references'], 3], [['trial'], 3]]);
  assert.deepEqual(report.not_decisive, []);
  assert.deepEqual(report.control, []);
  assert.deepEqual(report.outside_grounds, ['analyst']);
  assert.equal(report.outcome.value, 0);
  assert.deepEqual(report.came_true, [{ caveat: 'self_reported', carried_by: ['uptime_report'], decisive: true }]);
  assert.deepEqual(report.search, { mode: 'exhaustive', units: 3, replays: 4, limit: 8 });
  assert.equal(report.fatal, null);
  const text = formatReview(report, 'vendor.cav');
  assert.match(text, /^vendor\.cav: review of choose@1 = 4 {2}in force/);
  assert.match(text, /references \+ trial \(caveats: short_trial\): decide is refused/);
  assert.match(text, /Came true: self_reported \(carried by uptime_report, decisive\); the revision is still in force/);
});

test('a program that cannot decide without its evidence says so', async () => {
  const directory = new URL('../examples/reviewer-decision/', import.meta.url);
  const source = await readFile(new URL('rely.cav', directory), 'utf8');
  const events = (await readFile(new URL('rely.events.jsonl', directory), 'utf8'))
    .split('\n').filter(Boolean).map(line => ({ payload: {}, ...JSON.parse(line) }));
  const report = review(opener(source), events, 'rely@1');
  assert.deepEqual(report.decisive.map(item => item.evidence), [['registry_api'], ['npm_view'], ['publication_record']]);
  for (const item of report.decisive) assert.equal(item.because.code, 'unobserved_evidence');
  assert.deepEqual(report.outside_grounds, ['socket_page']);
  assert.equal(report.outcome, null);
  assert.deepEqual(report.came_true, []);
});

const grouped = `claim ok;
evidence a from "a"; evidence b from "b"; evidence c from "c";
decisions go limit 2;
state s = 0 min 0 max 10;
event both; event third;
on both reveal a supports ok;
on both reveal b supports ok;
on both set s = qualified(1, a) + qualified(1, b);
on third reveal c supports ok;
on third when s >= 2 commit go because enough using s + qualified(1, c);
on third when s < 2 reject "Too little to go on.";
`;

test('grounds observed by one event are one unit; grounds observed by the deciding event are entangled', () => {
  const report = review(opener(grouped), [{ event: 'both', payload: {} }, { event: 'third', payload: {} }], 'go@1');
  assert.deepEqual(report.decisive.map(item => item.evidence), [['a', 'b']]);
  assert.deepEqual(report.decisive[0].without, [{ sequence: 1, event: 'both' }]);
  assert.equal(report.decisive[0].because.message, 'Too little to go on.');
  assert.deepEqual(report.entangled, ['c']);
  assert.deepEqual(report.search, { mode: 'exhaustive', units: 1, replays: 1, limit: 8 });
  assert.match(formatReview(report), /1 replay searched every combination of 1 observing event\./);
});

// The why-demo finding: evidence read only by the deciding rule's guard is
// control, not grounds, and the decision can still hinge on it.
const guarded = `claim refund_warranted;
evidence order_record from "the order system";
evidence customer_email from "the customer's own email";
evidence photo from "a photo the customer attached";
evidence risk_flag from "the fraud screen";
evidence chargeback from "the bank's fraud report";
caveat self_reported consequence material;
caveat unverified_photo consequence low;
self_reported qualifies customer_email;
unverified_photo qualifies photo;
decisions refund limit 4;
state support = 0 min 0 max 10;
state amount = 0 min 0 max 200;
event read_order; event read_email; event read_photo; event read_risk; event decide; event fraud_found;
on read_order reveal order_record supports refund_warranted;
on read_order set support = support + qualified(1, order_record);
on read_order set amount = amount + qualified(40, order_record);
on read_email reveal customer_email supports refund_warranted;
on read_email set support = support + qualified(2, customer_email);
on read_photo reveal photo supports refund_warranted;
on read_photo set support = support + qualified(1, photo);
on read_photo set amount = amount + qualified(40, photo);
on read_risk reveal risk_flag opposes refund_warranted;
on decide when support >= 3 commit refund because enough using amount
  retaining self_reported, unverified_photo;
on decide when support < 3 reject "Not enough support to refund.";
on fraud_found reveal chargeback opposes refund_warranted;
on fraud_found resolve refund = 0 because chargeback materialized self_reported;
`;

test('evidence read only by the deciding guard is replayed too, as control', () => {
  const events = ['read_order', 'read_email', 'read_photo', 'read_risk', 'decide', 'fraud_found']
    .map(event => ({ event, payload: {} }));
  const report = review(opener(guarded), events, 'refund@1');
  assert.deepEqual(report.grounds.map(item => item.evidence), ['order_record', 'photo']);
  assert.deepEqual(report.control, ['customer_email']);
  assert.deepEqual(report.decisive.map(item => item.evidence), [['customer_email'], ['order_record', 'photo']]);
  assert.equal(report.decisive[0].because.message, 'Not enough support to refund.');
  assert.deepEqual(report.decisive[0].caveats, ['self_reported']);
  assert.deepEqual(report.value_only.map(item => [item.evidence, item.value]), [[['order_record'], 40], [['photo'], 40]]);
  assert.deepEqual(report.outside_grounds, ['risk_flag']);
  assert.deepEqual(report.came_true, [{ caveat: 'self_reported', carried_by: ['customer_email'], decisive: true }]);
  assert.deepEqual(report.search, { mode: 'exhaustive', units: 3, replays: 4, limit: 8 });
  const text = formatReview(report, 'refund.cav');
  assert.match(text, /\n {2}control outside its grounds: customer_email\n/);
  assert.match(text, /Observed before the decision, outside its lineage: risk_flag/);
  assert.match(text, /Came true: self_reported \(carried by customer_email, decisive\)/);
});

test('past the limit, each unit is replayed alone and the report says so', () => {
  const report = review(opener(vendor), vendorEvents, 'choose@1', { limit: 2 });
  assert.deepEqual(report.search, { mode: 'single', units: 3, replays: 3, limit: 2 });
  assert.deepEqual(report.decisive.map(item => item.evidence), [['uptime_report']]);
  assert.deepEqual(report.value_only.map(item => item.evidence), [['references'], ['trial']]);
  assert.match(formatReview(report), /combinations are not searched/);
});

test('an unknown revision is an error', () => {
  assert.throws(() => review(opener(vendor), vendorEvents, 'choose@2'), /no revision choose@2/);
  assert.throws(() => review(opener(vendor), vendorEvents, 'support'), /no revision support/);
});

test('a review is the same each time and opens a fresh session per replay', () => {
  let opened = 0;
  const open = () => { opened += 1; return real.open(vendor); };
  const first = review(open, vendorEvents, 'choose@1');
  // The full run, then each replay; sets holding uptime_report are not replayed.
  assert.equal(opened, 1 + first.search.replays);
  assert.deepEqual(review(opener(vendor), vendorEvents, 'choose@1'), first);
});
