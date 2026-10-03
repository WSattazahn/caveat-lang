// Withdrawal visibility belongs to the query, without changing authored decisions.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { dependents, formatDependents } from '../lib/explain.mjs';
import { createServer } from '../lib/serve.mjs';
import { runAuthoringOperation } from '../lib/authoring.mjs';
import { real, repo } from './helpers.mjs';

const source = `claim safe; claim misreading;
evidence ci from "checks"; evidence go from "permission";
evidence recheck from "a re-read"; evidence stray from "unrelated report";
caveat stale consequence material; stale qualifies ci;
readings checks from ci limit 4; readings approvals from go limit 4;
decisions merge limit 3 reopened by checks;
state estimate = 0;
event check value min 0 max 100; event approved; event decide;
event misread; event revoke; event unrelated;
on check sample checks = value supports safe;
on check set estimate = latest(checks);
on approved sample approvals = 1 supports safe;
on decide commit merge because enough using latest(checks) permitted by latest(approvals);
on misread reveal recheck supports misreading;
on misread withdraw latest(checks) because recheck;
on revoke reveal recheck supports misreading;
on revoke withdraw latest(approvals) because recheck;
on unrelated reveal stray supports safe;
on unrelated reveal recheck supports misreading;
on unrelated withdraw stray because recheck;
bind hud.value = estimate because estimate;
`;
const initial = [['check', { value: 85 }], ['approved'], ['decide']];
const readingWithdrawal = { evidence: 'checks@1', because: 'recheck', sequence: 4, event: 'misread' };
const grantWithdrawal = { evidence: 'approvals@1', because: 'recheck', sequence: 4, event: 'revoke' };

function after(events) {
  const session = real.open(source);
  try {
    for (const [event, payload = {}] of events) {
      const result = session.dispatch(event, payload);
      assert.equal(result.outcome, 'accepted', JSON.stringify(result));
    }
    return session.snapshot();
  } finally { session.close(); }
}

function withdrawalArrays(report) {
  return [report.withdrawals, ...['decisions', 'changes', 'values', 'displayed'].flatMap(name => report[name].map(item => item.withdrawn))];
}

test('withdrawn readings appear on direct, stream and template queries without changing status or frozen grounds', () => {
  const before = after(initial);
  const snapshot = after([...initial, ['misread'], ['unrelated']]);
  const original = structuredClone(snapshot);
  for (const subject of ['checks@1', 'checks', 'ci']) {
    const report = dependents(snapshot, subject);
    assert.deepEqual(report.withdrawals, [readingWithdrawal], subject);
    assert.deepEqual(report.decisions.map(item => [item.id, item.status, item.basis, item.via, item.withdrawn]),
      [['merge@1', 'in force', 'grounds', ['checks@1'], [readingWithdrawal]]], subject);
    assert.deepEqual(report.values.find(item => item.name === 'estimate').withdrawn, [readingWithdrawal]);
    assert.deepEqual(report.displayed.find(item => item.name === 'hud.value').withdrawn, [readingWithdrawal]);
    assert.deepEqual(report.changes.find(item => item.change === 'committed').withdrawn, [readingWithdrawal]);
    const text = formatDependents(report, 'checks.cav', 5);
    assert.match(text, /merge@1 = 85  in force  based on checks@1/);
    assert.match(text, /checks@1 withdrawn at #4 during misread because recheck/);
    assert.doesNotMatch(text, /stray|unrelated/);
  }
  assert.deepEqual(snapshot, original, 'reporting does not mutate the snapshot');
  assert.deepEqual(snapshot.commitment_grounds, before.commitment_grounds);
  assert.deepEqual(snapshot.commitment_bases, before.commitment_bases);
  assert.deepEqual(snapshot.decision_journal, before.decision_journal);
});

test('withdrawn permission stays a permission dependency and does not reopen its decision', () => {
  const snapshot = after([...initial, ['revoke'], ['unrelated']]);
  for (const subject of ['approvals@1', 'approvals', 'go']) {
    const report = dependents(snapshot, subject);
    assert.deepEqual(report.withdrawals, [grantWithdrawal]);
    assert.deepEqual(report.decisions.map(item => [item.basis, item.via, item.status, item.withdrawn]),
      [['permission', ['approvals@1'], 'in force', [grantWithdrawal]]]);
    assert.match(formatDependents(report), /permitted by approvals@1\n {4}approvals@1 withdrawn at #4 during revoke because recheck/);
  }
  const checks = dependents(snapshot, 'checks');
  assert.deepEqual(checks.withdrawals, []);
  assert.deepEqual(checks.decisions[0].withdrawn, []);
  assert.doesNotMatch(formatDependents(checks), /withdrawn|revoke|stray/);
  assert.deepEqual(snapshot.commitment_grounds['merge@1'].evidence, ['checks@1']);
});

test('historical and reopened revisions keep their status; only each reported basis is annotated', () => {
  const reopened = after([...initial, ['misread'], ['check', { value: 90 }]]);
  assert.equal(dependents(reopened, 'checks@1').decisions[0].status, 'reopened');
  const snapshot = after([...initial, ['misread'], ['check', { value: 90 }], ['decide']]);
  const old = dependents(snapshot, 'checks@1');
  assert.deepEqual(old.decisions.map(item => [item.id, item.status, item.basis, item.withdrawn]), [
    ['merge@1', 'superseded', 'grounds', [readingWithdrawal]],
    ['merge@2', 'in force', 'lineage', [readingWithdrawal]],
  ]);
  const stream = dependents(snapshot, 'checks');
  assert.deepEqual(stream.decisions.map(item => [item.id, item.basis, item.withdrawn]), [
    ['merge@1', 'grounds', [readingWithdrawal]], ['merge@2', 'grounds', []],
  ]);
  assert.deepEqual(dependents(snapshot, 'checks@2').withdrawals, []);
  assert.deepEqual(snapshot.commitment_grounds['merge@1'], reopened.commitment_grounds['merge@1']);
});

test('withdrawn evidence without dependents is still reported, and absent withdrawal fields remain valid', () => {
  const snapshot = after([['unrelated']]);
  const report = dependents(snapshot, 'stray');
  assert.deepEqual(report.withdrawals, [{ evidence: 'stray', because: 'recheck', sequence: 1, event: 'unrelated' }]);
  assert.ok(['decisions', 'changes', 'values', 'displayed'].every(key => report[key].length === 0));
  assert.match(formatDependents(report), /Withdrawals\n {2}stray withdrawn at #1 during unrelated because recheck/);
  const unwithdrawn = after(initial);
  delete unwithdrawn.withdrawals;
  const clean = dependents(unwithdrawn, 'checks');
  assert.ok(withdrawalArrays(clean).every(items => Array.isArray(items) && items.length === 0));
  const legacy = structuredClone(clean);
  delete legacy.withdrawals;
  for (const key of ['decisions', 'changes', 'values', 'displayed']) for (const item of legacy[key]) delete item.withdrawn;
  assert.equal(formatDependents(legacy), formatDependents(clean));
});

test('caveat queries associate withdrawals only with evidence carrying that caveat', () => {
  const snapshot = after([...initial, ['misread'], ['unrelated']]);
  const report = dependents(snapshot, 'stale');
  assert.deepEqual(report.withdrawals, [readingWithdrawal]);
  assert.deepEqual(report.decisions[0].withdrawn, [readingWithdrawal]);
  assert.deepEqual(report.values.find(item => item.name === 'estimate').withdrawn, [readingWithdrawal]);
  assert.doesNotMatch(formatDependents(report), /stray|unrelated/);
  const withdrawn = dependents(snapshot, 'withdrawn');
  assert.deepEqual(withdrawn.withdrawals, snapshot.withdrawals);
});

test('a query excludes unrelated withdrawals even when both sources ground the same decision', () => {
  const session = real.open(`claim ready; claim correction;
    evidence a from "one"; evidence b from "two"; evidence why from "recheck";
    decisions d limit 1; event start; event retract;
    on start reveal a supports ready; on start reveal b supports ready;
    on start commit d because enough using qualified(1, a) + qualified(1, b);
    on retract reveal why supports correction;
    on retract withdraw a because why; on retract withdraw b because why;`);
  try {
    session.dispatch('start', {}); session.dispatch('retract', {});
    const report = dependents(session.snapshot(), 'a');
    const expected = [{ evidence: 'a', because: 'why', sequence: 2, event: 'retract' }];
    assert.deepEqual(report.withdrawals, expected);
    assert.deepEqual(report.decisions[0].withdrawn, expected);
    assert.deepEqual(report.decisions[0].via, ['a']);
  } finally { session.close(); }
});

test('renewed evidence keeps the withdrawn occurrence distinct from its fresh current occurrence', () => {
  const session = real.open(`claim ready; claim correction;
    evidence memo from "notes"; evidence why from "recheck";
    renewable memo limit 2; decisions d limit 1;
    event start; event retract; event fresh;
    on start reveal memo;
    on start commit d because enough using 1 permitted by memo;
    on retract reveal why supports correction; on retract withdraw memo because why;
    on fresh renew memo; on fresh reveal memo supports ready;`);
  try {
    for (const event of ['start', 'retract', 'fresh']) session.dispatch(event, {});
    const snapshot = session.snapshot();
    const report = dependents(snapshot, 'memo');
    assert.deepEqual(report.withdrawals,
      [{ evidence: 'memo', because: 'why', sequence: 2, event: 'retract' }]);
    assert.deepEqual(report.decisions.map(item => [item.basis, item.via, item.withdrawn]),
      [['permission', ['memo'], report.withdrawals]]);
    assert.deepEqual(snapshot.commitment_grounds['d@1'].evidence, []);
    assert.equal(snapshot.commitment_permissions['d@1'].grant, 'memo');
    assert.deepEqual(dependents(snapshot, 'memo@2').withdrawals, []);
  } finally { session.close(); }
});

test('CLI JSON/text, serve and inline authoring expose the same withdrawal records', async () => {
  const events = [...initial, ['misread'], ['unrelated']];
  const directory = path.join(repo, 'test-results', 'dependents-withdrawal', String(process.pid));
  await mkdir(directory, { recursive: true });
  await writeFile(path.join(directory, 'withdrawal.cav'), source);
  const sent = events.map(([event, payload = {}]) => ({ event, payload }));
  await writeFile(path.join(directory, 'events.jsonl'), sent.map(item => JSON.stringify(item)).join('\n'));
  const cli = fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url));
  const run = args => spawnSync(process.execPath, [cli, 'dependents', ...args, 'withdrawal.cav', 'checks', 'events.jsonl'], { cwd: directory, encoding: 'utf8' });
  const json = run(['--json']);
  assert.equal(json.status, 0, json.stderr);
  const text = run([]);
  assert.equal(text.status, 0, text.stderr);
  assert.match(text.stdout, /checks@1 withdrawn at #4 during misread because recheck/);
  const server = createServer({ runtime: real, source });
  const send = request => server.handle(JSON.stringify(request)).response;
  try {
    for (const item of sent) assert.equal(send({ op: 'dispatch', ...item }).outcome, 'accepted');
    const served = send({ op: 'dependents', of: 'checks' }).report;
    const authored = await runAuthoringOperation('caveat_dependents', { source, subject: 'checks', events: sent });
    assert.equal(authored.exitCode, 0);
    const direct = dependents(after(events), 'checks');
    for (const actual of [JSON.parse(json.stdout), served, authored.report]) {
      for (const field of ['withdrawals', 'decisions', 'changes', 'values', 'displayed']) assert.deepEqual(actual[field], direct[field], field);
      assert.deepEqual(actual.withdrawals, [readingWithdrawal]);
    }
  } finally { send({ op: 'close' }); }
});
