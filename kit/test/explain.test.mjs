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
import { faultingRuntimeDirectory } from './fatal-fixture.mjs';

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
      .map(({ change, sequence, event, because, caveats }) => ({ change, sequence, event, because, caveats,
        source_sha256: snapshot.source_sha256 })));
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
  // D007: the reading carries calibration_offset, so the author did not retain it.
  assert.doesNotMatch(text, /retaining: calibration_offset/);
});

test('a revision lists the thresholds its commit named, and others list none', () => {
  // spec/caveat-thresholds-0.1.md: the graded-belief note's robot.
  const source = `claim door_open; evidence estimator from "the state estimator";
    readings beliefs from estimator limit 8; decisions plan limit 8 reopened by beliefs;
    threshold go_bar = 0.9; threshold look_bar = 0.3; event belief p min 0 max 1;
    on belief sample beliefs = p supports door_open;
    on belief when not committed(plan) or reopened(plan) commit plan because enough
      using if(latest(beliefs) >= go_bar, 2, if(latest(beliefs) >= look_bar, 1, 0));`;
  const session = real.open(source);
  session.dispatch('belief', { p: 0.55 });
  const report = explain(session.snapshot(), []);
  session.close();
  const [revision] = report.decisions[0].revisions;
  assert.deepEqual(revision.thresholds, { go_bar: 0.9, look_bar: 0.3 });
  assert.match(formatExplanation(report, 'robot'),
    /plan@1 = 1 {2}in force\n {6}based on beliefs@1\n {6}thresholds: go_bar = 0\.9, look_bar = 0\.3\n/);

  const plain = real.open(thermostat);
  plain.dispatch('read', { value: 17 });
  const without = explain(plain.snapshot(), []);
  plain.close();
  assert.ok(without.decisions[0].revisions.every(item => !('thresholds' in item)));
  assert.doesNotMatch(formatExplanation(without, 'thermostat'), /thresholds:/);
});

test('each observation is described once, however many revisions rest on it', () => {
  const source = `claim ok;
evidence gauge from "a gauge";
readings level from gauge limit 8;
decisions verdict limit 8;
event read v min 0 max 100;
event decide;
event recheck;
on read sample level = v supports ok;
on decide commit verdict because enough using fold_history(level, latest(level), max);
on recheck when committed(verdict) and not reopened(verdict) reopen verdict because latest(level);`;
  const session = real.open(source);
  const sent = [];
  for (const [event, payload] of [['read', { v: 3 }], ['read', { v: 5 }], ['decide', {}], ['recheck', {}], ['decide', {}]]) {
    const { snapshot, ...outcome } = session.dispatch(event, payload);
    sent.push({ event, payload, outcome, sequence: snapshot.sequence, effects: snapshot.effects });
  }
  const text = formatExplanation(explain(session.snapshot(), sent), 'probe.cav');
  session.close();
  // Both revisions rest on both readings; each reading's observation is
  // described under the first revision only, and both list it in their grounds.
  assert.equal(text.match(/level@1: observed at #1 read \{"v":3\}/g)?.length, 1);
  assert.equal(text.match(/level@2: observed at #2 read \{"v":5\}/g)?.length, 1);
  assert.equal(text.match(/based on level@1, level@2/g)?.length, 2);
  assert.ok(text.indexOf('level@1: observed at') < text.indexOf('verdict@2 ='), text);
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

test('prompt-injection limit 6: after a refusal, an accepted event names the #N the decisions cite', async () => {
  const [first, second, third] = reads.split('\n');
  await withFiles({ 'events.jsonl': `${first}\n${second}\n{"event":"read","payload":{"value":99}}\n${third}\n` }, async file => {
    const result = run(example, file('events.jsonl'));
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /^thermostat_history\.cav after 4 events \(sequence 3\)/);
    assert.match(result.stdout, / {4}1 {2}read \{"value":17\} {2}accepted\n {4}2 {2}read \{"value":25\} {2}accepted\n/);
    assert.match(result.stdout, / {4}3 {2}read \{"value":99\} {2}refused \(input\/bound_exceeded\): [^\n]*\n {4}4 {2}read \{"value":17\} {2}accepted as #3\n/);
    assert.match(result.stdout, /#3 read: /);
    assert.doesNotMatch(result.stdout, /#4 /);
  });
});

test('without events, the command explains the initial session', () => {
  const result = run(example);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /after 0 events \(sequence 0\)\n\nDecisions\n {2}heating: 0 of at most 8\n/);
});

const threeReads = '{"event":"read","payload":{"value":1}}\n{"event":"read","payload":{"value":2}}\n{"event":"read","payload":{"value":3}}\n';

test('a full history refuses the event and the replay continues', async () => {
  const limited = 'claim c;\nevidence e from "a";\nreadings r from e limit 2;\nevent read value min 0 max 9;\non read sample r = value supports c;';
  await withFiles({ 'limited.cav': limited, 'events.jsonl': threeReads }, async file => {
    const result = run(file('limited.cav'), file('events.jsonl'));
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, / {4}3 {2}read \{"value":3\} {2}refused \(limit\/history_limit\): /);
    assert.match(result.stdout, /r@2 = 2 supports c/);
    assert.doesNotMatch(result.stdout, /r@3/);
  });
});

test('a fatal event stops the replay and the explanation is of the session before it', async () => {
  // The fatal event is the fixture runtime's: no program reaches one.
  const dividing = 'claim c;\nevidence e from "a";\nreadings r from e limit 4;\nstate share = 0;\nevent read value min 0 max 9;\nevent fault;\non read sample r = value supports c;\non read when value > 1.5 set share = 1;\n';
  const faulted = [{ event: 'read', payload: { value: 1 } }, { event: 'fault' }, { event: 'read', payload: { value: 3 } }].map(event => JSON.stringify(event)).join('\n');
  await withFiles({ 'dividing.cav': dividing, 'events.jsonl': faulted }, async file => {
    const result = run('--runtime', await faultingRuntimeDirectory(), file('dividing.cav'), file('events.jsonl'));
    assert.equal(result.status, 1);
    assert.match(result.stdout, / {4}2 {2}fault .*failed: /);
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

test('a withdrawn observation is marked where a decision rests on it, which keeps its grounds', () => {
  const session = real.open(`claim safe; claim misreading;
    evidence ci from "checks"; evidence recheck from "a re-read";
    readings checks from ci limit 4; decisions merge limit 2;
    event check; event decide; event misread;
    on check sample checks = 1 supports safe;
    on decide commit merge because enough using latest(checks);
    on misread reveal recheck supports misreading;
    on misread withdraw latest(checks) because recheck;`);
  for (const event of ['check', 'decide', 'misread']) session.dispatch(event, {});
  const snapshot = session.snapshot();
  session.close();
  const report = explain(snapshot);
  const [revision] = report.decisions[0].revisions;
  assert.deepEqual(revision.grounds.evidence, ['checks@1']);
  assert.deepEqual(revision.withdrawn, [{ evidence: 'checks@1', because: 'recheck', sequence: 3, event: 'misread' }]);
  assert.deepEqual(report.evidence.find(item => item.id === 'checks@1').withdrawn, { because: 'recheck', sequence: 3, event: 'misread' });
  const text = formatExplanation(report, 'ledger');
  assert.match(text, /based on checks@1\n {6}checks@1 has since been withdrawn at #3 because recheck/);
  assert.match(text, /checks@1 = 1 supports safe .*withdrawn at #3 because recheck/);
});

const LATER = `claim effect;
    evidence study from "a study"; evidence critique from "a critique";
    caveat disputed consequence material;
    state ok = 0; decisions conclude limit 2;
    event publish; event decide; event dispute; event recheck;
    on publish reveal study supports effect;
    on publish set ok = qualified(1, study);
    on decide commit conclude because enough using ok;
    on dispute reveal critique;
    on dispute qualify study with disputed;
    on recheck when committed(conclude) reopen conclude because critique;
    on recheck when committed(conclude) and reopened(conclude) commit conclude because enough using ok;`;

test('with laterCaveats, explain lists live caveat names absent from the decision record', async () => {
  const session = real.open(LATER);
  for (const event of ['publish', 'decide', 'dispute', 'recheck']) session.dispatch(event, {});
  const snapshot = session.snapshot();
  session.close();
  const plain = explain(snapshot);
  assert.equal(plain.decisions[0].revisions[0].qualified_since, undefined, 'off by default');
  assert.doesNotMatch(formatExplanation(plain), /absent from this record/);

  const report = explain(snapshot, [], { laterCaveats: true });
  const [first, second] = report.decisions[0].revisions;
  assert.deepEqual(first.grounds.caveats, [], 'the record keeps its own caveats');
  assert.deepEqual(first.qualified_since, [{ evidence: 'study', caveats: ['disputed'] }]);
  assert.deepEqual(second.grounds.caveats, ['disputed']);
  assert.deepEqual(second.qualified_since, [], 'a decision made after the qualify already holds it');
  assert.match(formatExplanation(report), /based on study\n {6}study now carries disputed, absent from this record/);

  await withFiles({ 'later.cav': LATER, 'events.jsonl': ['publish', 'decide', 'dispute'].map(event => JSON.stringify({ event })).join('\n') }, file => {
    const before = run(file('later.cav'), file('events.jsonl'));
    const after = run('--later-caveats', file('later.cav'), file('events.jsonl'));
    assert.equal(after.status, 0, after.stderr);
    assert.doesNotMatch(before.stdout, /absent from this record/);
    assert.match(after.stdout, /study now carries disputed, absent from this record/);
    assert.equal(after.stdout.replace(/\n {6}study now carries disputed, absent from this record/, ''), before.stdout);
    const refused = spawnSync(process.execPath, [cli, 'check', '--later-caveats', file('later.cav')], { encoding: 'utf8' });
    assert.equal(refused.status, 2);
    assert.match(refused.stderr, /unknown option --later-caveats for check/);
    const mistyped = run('--later-caveatz', file('later.cav'));
    assert.equal(mistyped.status, 2);
    assert.match(mistyped.stderr, /did you mean --later-caveats\?/);
    const json = JSON.parse(run('--json', file('later.cav'), file('events.jsonl')).stdout);
    const enabled = JSON.parse(run('--json', '--later-caveats', file('later.cav'), file('events.jsonl')).stdout);
    for (const decision of enabled.decisions) for (const revision of decision.revisions) delete revision.qualified_since;
    assert.deepEqual(enabled, json);
  });
});

const REDECIDED = `claim effect;
    evidence old_study from "the first study"; evidence retraction from "its retraction";
    evidence new_study from "a later study";
    state old_ok = 0; state new_ok = 0; decisions conclude limit 2;
    event publish; event decide; event retract; event replicate; event redecide;
    on publish reveal old_study supports effect;
    on publish set old_ok = qualified(1, old_study);
    on decide commit conclude because enough using old_ok;
    on retract reveal retraction;
    on retract withdraw old_study because retraction;
    on retract when committed(conclude) and rests_on_withdrawn(conclude) reopen conclude because retraction;
    on replicate reveal new_study supports effect;
    on replicate set new_ok = qualified(1, new_study);
    on redecide when reopened(conclude) commit conclude because enough using new_ok;`;

test('with lineageNotes, lineage that came with the revision before is marked with it', async () => {
  const session = real.open(REDECIDED);
  for (const event of ['publish', 'decide', 'retract', 'replicate', 'redecide']) session.dispatch(event, {});
  const snapshot = session.snapshot();
  session.close();
  const plain = explain(snapshot);
  assert.equal(plain.decisions[0].revisions[1].lineage_from, undefined, 'off by default');
  assert.match(formatExplanation(plain), /could also have been influenced by old_study, retraction\n/);

  const report = explain(snapshot, [], { lineageNotes: true });
  const [first, second] = report.decisions[0].revisions;
  assert.deepEqual(first.lineage_from, []);
  assert.deepEqual(second.grounds.evidence, ['new_study'], 'the grounds are unchanged');
  assert.deepEqual(second.lineage_from, [
    { evidence: 'old_study', revision: 'conclude@1', as: 'grounds' },
    { evidence: 'retraction', revision: 'conclude@1', as: 'reopening' },
  ]);
  assert.match(formatExplanation(report),
    /could also have been influenced by old_study \(conclude@1 rested on it\), retraction \(it reopened conclude@1\)\n/);

  await withFiles({ 'redecided.cav': REDECIDED, 'events.jsonl': ['publish', 'decide', 'retract', 'replicate', 'redecide'].map(event => JSON.stringify({ event })).join('\n') }, file => {
    const before = run(file('redecided.cav'), file('events.jsonl'));
    const after = run('--lineage-notes', file('redecided.cav'), file('events.jsonl'));
    assert.equal(after.status, 0, after.stderr);
    assert.equal(after.stdout.replace(' (conclude@1 rested on it)', '').replace(' (it reopened conclude@1)', ''), before.stdout);
  });
});

test('a permission is shown as what permitted a decision, apart from what it rests on', async () => {
  const { dependents } = await import('../lib/explain.mjs');
  const session = real.open(`identifiers limit 8; claim ready; claim may_merge;
    evidence ci from "checks"; evidence go from "the go-ahead";
    readings checks from ci limit 4; readings approvals from go limit 4; decisions merge limit 2;
    state head = 0;
    event pushed commit id; event check; event approved commit id; event merge;
    on pushed set head = commit;
    on check sample checks = 1 supports ready;
    on approved sample approvals = commit supports may_merge;
    on merge commit merge because enough using latest(checks) permitted by latest(approvals) for head;`);
  session.dispatch('pushed', { commit: 'abc' });
  session.dispatch('check', {});
  session.dispatch('approved', { commit: 'abc' });
  session.dispatch('merge', {});
  const snapshot = session.snapshot();
  session.close();
  const [revision] = explain(snapshot).decisions[0].revisions;
  assert.deepEqual(revision.grounds.evidence, ['checks@1']);
  assert.deepEqual(revision.permission, { grant: 'approvals@1', scope: { granted: 1, required: 1 } });
  assert.match(formatExplanation(explain(snapshot), 'ledger'), /based on checks@1\n {6}permitted by approvals@1 for 1\n/);
  for (const subject of ['approvals@1', 'approvals', 'go']) {
    const report = dependents(snapshot, subject);
    assert.deepEqual(report.decisions.map(item => [item.id, item.basis, item.via]), [['merge@1', 'permission', ['approvals@1']]], subject);
  }
  assert.deepEqual(dependents(snapshot, 'checks@1').decisions.map(item => item.basis), ['grounds']);
});

// A retired record is shown as it was, with the sequence at which it retired,
// and dependents lists what still cites it (spec/caveat-windows-0.1.md). Once
// nothing pins it, it departs: explain shows the marker that replaced it, and
// dependents of the departed name lists what may rest on it
// (spec/caveat-departure-0.1.md, spec/caveat-lineage-compaction-0.1.md).
test('explain and dependents show retired and departed records', async () => {
  const { dependents, formatDependents } = await import('../lib/explain.mjs');
  const source = `claim seen; evidence glimpse from "a glimpse";
    readings sighting from glimpse window 2; decisions trust limit 4; journal window 2;
    event look; event decide;
    on look sample sighting = 1 supports seen;
    on decide when committed(trust) reopen trust because latest(sighting);
    on decide commit trust because enough using history_count(sighting);`;
  const session = real.open(source);
  for (const event of ['look', 'decide', 'look', 'look']) session.dispatch(event, {});
  const snapshot = session.snapshot();
  // The journal's entry for trust@1 still cites sighting@1, so it stays.
  assert.deepEqual(snapshot.retired, { 'sighting@1': 4 });
  assert.deepEqual(snapshot.windows, ['journal', 'sighting']);
  const report = explain(snapshot);
  assert.equal(report.evidence.find(item => item.id === 'sighting@1').retired_at, 4);
  assert.equal(report.evidence.find(item => item.id === 'sighting@3').retired_at, undefined);
  const text = formatExplanation(report);
  assert.match(text, /sighting@1 = 1 supports seen {2}\(#1 look\) {2}retired at #4/);
  const rests = dependents(snapshot, 'sighting');
  assert.deepEqual(rests.retired, [{ record: 'sighting@1', sequence: 4 }]);
  assert.ok(rests.decisions.some(item => item.id === 'trust@1' && item.via.includes('sighting@1')),
    'the revision still rests on the retired reading');
  assert.match(formatDependents(rests), /Retired\n {2}sighting@1 retired at #4/);

  // The next decision retires that journal entry; nothing pins sighting@1, and
  // it departs.
  session.dispatch('decide', {});
  const later = session.snapshot();
  assert.equal(later.retired, undefined);
  assert.deepEqual(later.effects.filter(effect => effect.kind === 'depart').map(effect => effect.record),
    ['journal@1', 'sighting@1']);
  assert.match(formatExplanation(explain(later)), /trust@1 = 1 {2}superseded\n {6}based on departed sighting@1 \(at #5\)/);
  const departed = dependents(later, 'sighting@1');
  assert.equal(departed.departed, true);
  assert.deepEqual(departed.decisions.map(item => [item.id, item.basis]),
    [['trust@1', 'may rest on'], ['trust@2', 'may rest on']]);
  assert.match(formatDependents(departed), /sighting@1 has departed/);
  const undrained = session.undrained;
  const archive = session.drainArchive();
  assert.equal(undrained, archive.length);
  const records = archive.filter(entry => entry.kind !== 'provenance');
  assert.deepEqual(records.map(entry => entry.record), ['journal@1', 'sighting@1']);
  assert.equal(session.undrained, 0);
  assert.deepEqual(session.drainArchive(), []);
  assert.ok(records[1].holders.some(holder => holder.kind === 'commitment' && holder.name === 'trust@1' && holder.in === 'grounds'));
  session.close();
});

test('a reversible decision forecloses while in force, and explain says so', async () => {
  const source = `claim out_of_scope;
evidence review from "a review";
evidence recheck from "a second look";
decisions no_rewrite limit 4 forecloses rewrite;
decisions rewrite limit 4;
event rule_out;
event rewrite_it;
event reconsider;
on rule_out reveal review supports out_of_scope;
on rule_out commit no_rewrite because enough citing qualified(1, review);
on rewrite_it commit rewrite because enough using 1;
on reconsider reveal recheck opposes out_of_scope;
on reconsider reopen no_rewrite because recheck;
`;
  const events = ['rule_out', 'rewrite_it'].map(event => JSON.stringify({ event })).join('\n');
  const lifted = ['rule_out', 'reconsider', 'rewrite_it'].map(event => JSON.stringify({ event })).join('\n');
  await withFiles({ 'rule.cav': source, 'events.jsonl': events, 'lifted.jsonl': lifted }, async file => {
    const result = run(file('rule.cav'), file('events.jsonl'));
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, / {2}no_rewrite: 1 of at most 4, forecloses rewrite\n/);
    assert.match(result.stdout, / {2}rewrite: 0 of at most 4, foreclosed by no_rewrite@1\n/);
    assert.match(result.stdout, /rewrite_it {2}refused \(evaluation\/foreclosed\): .*rewrite is foreclosed by no_rewrite@1/);
    const json = JSON.parse(run('--json', file('rule.cav'), file('events.jsonl')).stdout);
    const series = Object.fromEntries(json.decisions.map(item => [item.name, item]));
    assert.equal('irreversible' in series.no_rewrite, false);
    assert.deepEqual(series.no_rewrite.forecloses, ['rewrite']);
    assert.deepEqual(series.rewrite.foreclosed_by, ['no_rewrite@1']);
    const after = JSON.parse(run('--json', file('rule.cav'), file('lifted.jsonl')).stdout);
    const reopened = Object.fromEntries(after.decisions.map(item => [item.name, item]));
    assert.equal('foreclosed_by' in reopened.rewrite, false);
    assert.equal(reopened.rewrite.current, 'rewrite@1');
  });
});

test('an irreversible decision shows what it forecloses, and a refused commit names it', async () => {
  const source = `scene "Dropping the old table can't be undone.";
claim safe;
evidence check from "a complete dry run";
readings runs from check limit 4;
decisions drop limit 1 irreversible forecloses rollback, undo;
decisions rollback limit 4;
event dry_run;
event go;
event back;
on dry_run sample runs = 1 supports safe;
on go commit drop because enough using latest(runs);
on back commit rollback because enough using 1;
on back commit undo because enough;
`;
  const events = ['dry_run', 'go', 'back'].map(event => JSON.stringify({ event })).join('\n');
  await withFiles({ 'drop.cav': source, 'events.jsonl': events }, async file => {
    const result = run(file('drop.cav'), file('events.jsonl'));
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, / {2}drop: 1 of at most 1, irreversible, forecloses rollback, undo\n/);
    assert.match(result.stdout, / {2}rollback: 0 of at most 4, foreclosed by drop@1\n/);
    assert.match(result.stdout, /back {2}refused \(evaluation\/foreclosed\): .*rollback is foreclosed by drop@1/);
    const json = JSON.parse(run('--json', file('drop.cav'), file('events.jsonl')).stdout);
    const series = Object.fromEntries(json.decisions.map(item => [item.name, item]));
    assert.equal(series.drop.irreversible, true);
    assert.deepEqual(series.drop.forecloses, ['rollback', 'undo']);
    assert.deepEqual(series.rollback.foreclosed_by, ['drop@1']);
    assert.equal('irreversible' in series.rollback, false);
  });
});
