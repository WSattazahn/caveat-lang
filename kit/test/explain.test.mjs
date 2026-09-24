// caveat explain: the library reads a snapshot faithfully, and the command
// replays events, lists refusals, stops at a fatal event and refuses bad input.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { explain, formatExplanation, parseEvents, EXPLAIN_SCHEMA } from '../lib/explain.mjs';
import { real, repo, thermostat } from './helpers.mjs';

const cli = fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url));
const example = path.join(repo, 'examples', 'thermostat_history.cav');
const run = (...args) => spawnSync(process.execPath, [cli, 'explain', ...args], { encoding: 'utf8' });

async function withFiles(files, body) {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-explain-'));
  try {
    for (const [name, text] of Object.entries(files)) await writeFile(path.join(directory, name), text);
    return await body(name => path.join(directory, name));
  } finally { await rm(directory, { recursive: true, force: true }); }
}

const reads = [17, 25, 17].map(value => JSON.stringify({ event: 'read', payload: { value } })).join('\n');

test('explain reports what the snapshot records, decision by decision', () => {
  const session = real.open(thermostat);
  const events = [17, 25, 17].map(value => {
    const { snapshot: _, ...outcome } = session.dispatch('read', { value });
    return { event: 'read', payload: { value }, outcome };
  });
  const snapshot = session.snapshot();
  const report = explain(snapshot, events);
  session.close();

  assert.equal(report.schema, EXPLAIN_SCHEMA);
  assert.equal(report.sequence, snapshot.sequence);
  const [heating] = report.decisions;
  assert.equal(heating.name, 'heating');
  assert.deepEqual(heating.revisions.map(revision => [revision.id, revision.value, revision.status]),
    [['heating@1', 1, 'superseded'], ['heating@2', 0, 'superseded'], ['heating@3', 1, 'in force']]);
  for (const revision of heating.revisions) {
    assert.deepEqual(revision.grounds, snapshot.commitment_grounds[revision.id]);
    assert.deepEqual(revision.lineage, snapshot.commitment_bases[revision.id].provenance);
    assert.deepEqual(revision.history, snapshot.decision_journal.filter(entry => entry.commitment === revision.id)
      .map(({ change, sequence, event, because, caveats }) => ({ change, sequence, event, because, caveats })));
  }
  assert.deepEqual(report.evidence.map(item => [item.id, item.value, item.relation, item.caveats]),
    [['temperature@1', 17, 'opposes', ['calibration_offset']], ['temperature@2', 25, 'supports', ['calibration_offset']],
      ['temperature@3', 17, 'opposes', ['calibration_offset']]]);
  const shown = Object.fromEntries(report.displayed.map(item => [item.name, item]));
  assert.equal(shown['temperature.text'].value, snapshot.bindings.temperature.text);
  assert.deepEqual(shown['temperature.text'].cites, snapshot.binding_explanations.temperature.text);

  const text = formatExplanation(report, 'thermostat');
  assert.match(text, /heating@3 = 1 {2}in force\n {6}based on temperature@3 \(caveats: calibration_offset\)\n {6}could also have been influenced by temperature@1, temperature@2/);
  assert.match(text, /#2 read: reopened because temperature@2 \(caveats: calibration_offset\)/);
});

test('a session with no decisions, evidence or bindings says so', () => {
  const session = real.open('state count = 0;\nevent poke;\non poke set count = count + 1;');
  const text = formatExplanation(explain(session.snapshot()), 'empty');
  session.close();
  assert.match(text, /^empty after 0 events \(sequence 0\)\n\nDecisions\n {2}none declared\n\nEvidence\n {2}none observed\n\nDisplayed\n {2}nothing bound$/);
});

test('the command replays events and lists refusals, which change nothing', async () => {
  await withFiles({ 'events.jsonl': `${reads}\n\n{"event":"read","payload":{"value":99}}\n{"event":"warm"}\n` }, async file => {
    const result = run(example, file('events.jsonl'));
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /^thermostat_history\.cav after 5 events \(sequence 3\)/);
    assert.match(result.stdout, / {4}4 {2}read \{"value":99\} {2}refused \(input\/bound_exceeded\): /);
    assert.match(result.stdout, / {4}5 {2}warm {2}refused \(input\/unknown_event\): /);
    assert.match(result.stdout, /heating: 3 of at most 8/);

    const json = JSON.parse(run('--json', example, file('events.jsonl')).stdout);
    assert.equal(json.schema, EXPLAIN_SCHEMA);
    assert.equal(json.program, example);
    assert.deepEqual(json.events.map(entry => entry.outcome.outcome), ['accepted', 'accepted', 'accepted', 'rejected', 'rejected']);
    assert.equal(json.decisions[0].revisions.length, 3);
  });
});

test('without events, the command explains the initial session', () => {
  const result = run(example);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /after 0 events \(sequence 0\)\n\nDecisions\n {2}heating: 0 of at most 8\n/);
});

test('a fatal event stops the replay and the explanation is of the session before it', async () => {
  const limited = 'claim c;\nevidence e from "a";\nreadings r from e limit 1;\nevent read value min 0 max 9;\non read sample r = value supports c;';
  await withFiles({ 'limited.cav': limited, 'events.jsonl': '{"event":"read","payload":{"value":1}}\n{"event":"read","payload":{"value":2}}\n{"event":"read","payload":{"value":3}}\n' }, async file => {
    const result = run(file('limited.cav'), file('events.jsonl'));
    assert.equal(result.status, 1);
    assert.match(result.stdout, / {4}2 {2}read \{"value":2\} {2}failed: /);
    assert.doesNotMatch(result.stdout, / {4}3 {2}read/);
    assert.match(result.stdout, /r@1 = 1 supports c/);
    assert.doesNotMatch(result.stdout, /r@2/);
    assert.match(result.stdout, /Event 2 failed; the explanation above is of the session before it\./);
  });
});

test('bad input is refused before anything runs', async () => {
  await withFiles({ 'broken.cav': 'this is not caveat;', 'bad.jsonl': '{"event":"read","payload":{"value":1}}\nnot json\n',
    'extra.jsonl': '{"event":"read","when":1}\n' }, async file => {
    let result = run(file('broken.cav'));
    assert.equal(result.status, 2);
    assert.match(result.stderr, /broken\.cav does not load: /);
    result = run(example, file('bad.jsonl'));
    assert.equal(result.status, 2);
    assert.match(result.stderr, /INVALID .*bad\.jsonl: line 2: not JSON/);
    assert.equal(result.stdout, '');
    result = run(example, file('extra.jsonl'));
    assert.match(result.stderr, /line 1: unknown field when/);
    result = run();
    assert.equal(result.status, 2);
    assert.match(result.stderr, /name a program and, optionally, an events file/);
  });
});

test('parseEvents accepts blank lines and CRLF, and defaults the payload', () => {
  assert.deepEqual(parseEvents('{"event":"a"}\r\n\r\n{"event":"b","payload":{"x":1}}\r\n'),
    [{ event: 'a', payload: {} }, { event: 'b', payload: { x: 1 } }]);
  assert.throws(() => parseEvents('[]'), /line 1: expected an object/);
  assert.throws(() => parseEvents('{"payload":{}}'), /line 1: "event" must name an event/);
});
