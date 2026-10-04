// The commands beside test and explain: validate, replay, dependents and init,
// run as a user runs them.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { guideSteps } from './guide.mjs';
import { provenanceOverflow } from './fatal-fixture.mjs';

const kit = fileURLToPath(new URL('../', import.meta.url));
const cli = path.join(kit, 'bin', 'caveat.mjs');
const caveat = (args, options = {}) => spawnSync(process.execPath, [cli, ...args], { encoding: 'utf8', ...options });
const templates = ['umbrella.cav', 'umbrella.scenarios.json', 'events.jsonl'];

async function inDirectory(body) {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-commands-'));
  try { return await body(directory); } finally { await rm(directory, { recursive: true, force: true }); }
}

test('help identifies CAVEAT Language, and version works without loading a runtime', async () => {
  const manifest = JSON.parse(await readFile(path.join(kit, 'package.json'), 'utf8'));
  assert.deepEqual(manifest.bin, { caveat: 'bin/caveat.mjs', 'caveat-lang': 'bin/caveat.mjs' });
  const help = caveat(['help']);
  assert.equal(help.status, 0, help.stderr);
  assert.match(help.stdout, /^CAVEAT Language — Programs that remember why\./);
  assert.match(help.stdout, /caveat-lang --version/);
  assert.match(help.stdout, /Both caveat-lang and caveat invoke this CLI/);
  for (const argument of ['version', '--version', '-v']) {
    const result = caveat([argument], { cwd: tmpdir() });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, `CAVEAT Language ${manifest.version}\n`);
  }
});

test('init writes exactly the files the getting-started guide has you write', async () => {
  const steps = guideSteps(await readFile(path.join(kit, 'docs', 'GETTING_STARTED.md'), 'utf8'));
  for (const name of templates) {
    const step = steps.find(item => item.kind === 'file' && item.name === name);
    assert.equal(await readFile(path.join(kit, 'templates', name), 'utf8'), step.content, name);
  }
  await inDirectory(async directory => {
    const target = path.join(directory, 'new', 'project');
    const result = caveat(['init', target]);
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual((await readdir(target)).sort(), [...templates].sort());
    assert.match(result.stdout, /caveat-lang test umbrella\.scenarios\.json/);
    const tested = caveat(['test', 'umbrella.scenarios.json'], { cwd: target });
    assert.equal(tested.status, 0, tested.stdout + tested.stderr);

    await writeFile(path.join(target, 'umbrella.cav'), 'mine');
    const again = caveat(['init', target]);
    assert.equal(again.status, 2);
    assert.match(again.stderr, /already exist; nothing written/);
    assert.equal(await readFile(path.join(target, 'umbrella.cav'), 'utf8'), 'mine');
  });
});

test('validate lists what a program declares, and fails on one that does not load', async () => {
  await inDirectory(async directory => {
    const program = path.join(kit, 'templates', 'umbrella.cav');
    const text = caveat(['validate', program]);
    assert.equal(text.status, 0, text.stderr);
    assert.equal(text.stdout.trim(), [
      'umbrella.cav loads.',
      '  events: clear_sky, leave, read_forecast (chance 0..100)',
      '  reading streams: rain_chance from forecast, limit 4',
      '  decision series: umbrella, limit 4',
      '  displayed: advice.text',
    ].join('\n'));
    const json = JSON.parse(caveat(['validate', '--json', program]).stdout);
    assert.deepEqual([json.schema, json.loads, json.reading_streams, json.decision_series],
      ['caveat-validate/0.1', true, { rain_chance: { from: 'forecast', limit: 4 } }, { umbrella: { limit: 4 } }]);

    const broken = path.join(directory, 'broken.cav');
    await writeFile(broken, 'this is not caveat;');
    const failed = caveat(['validate', broken]);
    assert.equal(failed.status, 2);
    assert.match(failed.stderr, /broken\.cav does not load: /);
    const failedJson = caveat(['validate', '--json', broken]);
    assert.equal(failedJson.status, 2);
    assert.equal(JSON.parse(failedJson.stdout).loads, false);

    // Typed parameters read as the source declares them.
    const trail = caveat(['validate', path.join(kit, '..', 'game', 'trail_rescue.cav')]);
    assert.match(trail.stdout, /observe \(target kind tunnel, method in report scout, condition 0\.\.2\)/);
    const ledger = caveat(['validate', path.join(kit, '..', 'experiments', 'agent-ledger', 'ledger-identifiers.cav')]);
    assert.match(ledger.stdout, /pushed \(target kind pr, commit id\)/);
  });
});

// The ledger's scenarios are its regression tests, including the ones that put
// several pull requests in one scenario.
test('the agent ledger scenarios pass', () => {
  const ledger = path.join(kit, '..', 'experiments', 'agent-ledger');
  for (const [name, count] of [
    ['ledger.scenarios.json', 5],
    ['ledger-identifiers.scenarios.json', 11],
    ['ledger-approved-head.scenarios.json', 16],
    ['release.scenarios.json', 8],
  ]) {
    const result = caveat(['test', path.join(ledger, name)]);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.match(result.stdout, new RegExp(`^${count} passed, 0 failed `, 'm'), name);
  }
});

// The release gates, driven from facts through `caveat serve`. rc.11's own
// history: published by hand, so nothing supports an attestation and the
// release record may not name provenance. Merges after the publish do not
// reopen it.
test('the release ledger replays rc.11 from its facts', async () => {
  const ledger = path.join(kit, '..', 'experiments', 'agent-ledger');
  const { readFacts, run, eventsOf } = await import(pathToFileURL(path.join(ledger, 'release-ledger.mjs')).href);
  const facts = await readFacts(path.join(ledger, 'release-rc11.facts.jsonl'));
  const { steps, report } = await run(facts);
  const refused = steps.filter(step => step.outcome === 'rejected')
    .map(step => [step.fact.fact, step.origin, step.code, step.message]);
  assert.deepEqual(refused, [['record', 'policy', 'reject', 'No verification found an attestation.']]);
  assert.deepEqual(steps.filter(step => step.outcome === 'skipped').map(step => step.fact.fact), ['plan']);
  const shown = Object.fromEntries(report.displayed.map(({ name, value }) => [name, value]));
  assert.equal(shown['repo.release'], 'published by hand');
  assert.equal(shown['repo.attestation'], 'unsupported');
  assert.equal(shown['repo.npm'], 'serves the tested tarball');
  assert.equal(shown['repo.tag'], '8e7805a57269c2084224df487fe186f0c6f6a4a9');

  // The events the driver sends, replayed by `caveat explain`, give the same
  // decisions, evidence and displayed values as the serve session.
  await inDirectory(async directory => {
    const events = path.join(directory, 'events.jsonl');
    await writeFile(events, eventsOf(facts).filter(step => !step.skip)
      .map(({ event, payload }) => JSON.stringify({ event, payload })).join('\n'));
    const explained = caveat(['explain', '--json', path.join(ledger, 'release.cav'), events]);
    assert.equal(explained.status, 0, explained.stderr);
    const replayed = JSON.parse(explained.stdout);
    for (const key of ['sequence', 'decisions', 'evidence', 'displayed']) assert.deepEqual(replayed[key], report[key], key);
  });
});

// rc.12's live run: its facts still give the report attached to the release
// record.
test('the release ledger replays rc.12 to its attached report', async () => {
  const ledger = path.join(kit, '..', 'experiments', 'agent-ledger');
  const { readFacts, run } = await import(pathToFileURL(path.join(ledger, 'release-ledger.mjs')).href);
  const { steps, report } = await run(await readFacts(path.join(ledger, 'release-rc12.facts.jsonl')));
  assert.deepEqual(steps.filter(step => step.outcome === 'rejected'), []);
  const attached = JSON.parse(await readFile(path.join(ledger, 'release-rc12.explain.json'), 'utf8'));
  for (const key of ['sequence', 'decisions', 'evidence', 'displayed']) assert.deepEqual(report[key], attached[key], key);
});

// ledger-approved-head.cav is ledger.cav corrected so that no pull request
// starts with a go-ahead; ledger.cav stays as recorded. Apart from comments,
// the two differ in that one line, the corrected scenarios include the
// recorded ones unchanged, and the merge that is fatal in ledger.cav is
// refused by the corrected copy.
test('the corrected agent ledger differs from the recorded one only where a go-ahead starts', async () => {
  const ledger = path.join(kit, '..', 'experiments', 'agent-ledger');
  const code = text => text.split('\n').filter(line => line.trim() && !line.trim().startsWith('#'));
  const recorded = code(await readFile(path.join(ledger, 'ledger.cav'), 'utf8'));
  const corrected = code(await readFile(path.join(ledger, 'ledger-approved-head.cav'), 'utf8'));
  const before = '    state $p_approved_head = 0 min 0 max 4294967295;';
  assert.equal(recorded.filter(line => line === before).length, 1);
  assert.deepEqual(corrected,
    recorded.map(line => (line === before ? '    state $p_approved_head = -1 min -1 max 4294967295;' : line)));

  const scenarios = async name => JSON.parse(await readFile(path.join(ledger, name), 'utf8')).scenarios;
  const own = await scenarios('ledger-approved-head.scenarios.json');
  for (const scenario of await scenarios('ledger.scenarios.json')) {
    assert.deepEqual(own.find(item => item.id === scenario.id), scenario, scenario.id);
  }

  await inDirectory(async directory => {
    const events = path.join(directory, 'events.jsonl');
    await writeFile(events, [
      { event: 'checks', payload: { target: 'pr26', commit: 0, result: 'passed' } },
      { event: 'merge', payload: { target: 'pr26' } },
    ].map(line => JSON.stringify(line)).join('\n'));
    const last = result => JSON.parse(result.stdout.trim().split('\n').at(-1));
    // The recorded ledger's merge was fatal through rc.10; it now refuses
    // (spec/caveat-dispatch-0.1.md), and the merge still does not happen.
    const unobserved = caveat(['replay', path.join(ledger, 'ledger.cav'), events]);
    assert.equal(unobserved.status, 0, unobserved.stderr);
    assert.deepEqual([last(unobserved).outcome, last(unobserved).origin, last(unobserved).code],
      ['rejected', 'evaluation', 'unobserved_evidence']);
    assert.match(last(unobserved).message, /cannot qualify a value with unobserved evidence pr26_go$/);
    const refused = caveat(['replay', path.join(ledger, 'ledger-approved-head.cav'), events]);
    assert.equal(refused.status, 0, refused.stderr);
    const { outcome, origin, message } = last(refused);
    assert.deepEqual([outcome, origin, message], ['rejected', 'policy', 'No go-ahead for the current head.']);
  });
});

// Routed repetition (spec/caveat-routed-repetition-0.1.md section 5): the
// ledger converted to `routed by target`, run against the same scenarios.
test('the agent ledger converted to a routed block passes the same scenarios', async () => {
  const ledger = path.join(kit, '..', 'experiments', 'agent-ledger');
  const today = await readFile(path.join(ledger, 'ledger-identifiers.cav'), 'utf8');
  const occurrences = (text, part) => text.split(part).length - 1;
  assert.equal(occurrences(today, 'target == $index and '), 20);
  const unguarded = today.replaceAll('target == $index and ', '');
  assert.equal(occurrences(unguarded, ' when target == $index'), 11);
  const routed = unguarded.replaceAll(' when target == $index', '')
    .replace('for pr as $p {', 'for pr as $p routed by target {');
  assert.equal(occurrences(routed, 'routed by target'), 1);
  assert.equal(occurrences(routed, 'target == $index'), 0);
  await inDirectory(async directory => {
    // The scenario file names its program, so the converted one keeps the name.
    await writeFile(path.join(directory, 'ledger-identifiers.cav'), routed);
    await writeFile(path.join(directory, 'ledger-identifiers.scenarios.json'),
      await readFile(path.join(ledger, 'ledger-identifiers.scenarios.json'), 'utf8'));
    const result = caveat(['test', path.join(directory, 'ledger-identifiers.scenarios.json')]);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.match(result.stdout, /^11 passed, 0 failed /m);
  });
});

test('replay prints one record per event, keeps file line numbers and stops at a fatal event', async () => {
  await inDirectory(async directory => {
    const events = path.join(directory, 'events.jsonl');
    await writeFile(events, '{"event":"read_forecast","payload":{"chance":70}}\n\n{"event":"leave"}\r\n{"event":"leave"}\n');
    const result = caveat(['replay', path.join(kit, 'templates', 'umbrella.cav'), events]);
    assert.equal(result.status, 0, result.stderr);
    const records = result.stdout.trim().split('\n').map(line => JSON.parse(line));
    assert.deepEqual(records.map(({ record, line, outcome, sequence, code }) => [record, line, outcome, sequence, code]), [
      ['initial', undefined, undefined, 0, undefined],
      ['event', 1, 'accepted', 1, undefined],
      ['event', 3, 'accepted', 2, undefined],
      ['event', 4, 'rejected', 2, 'reject'],
    ]);
    assert.ok(records[1].snapshot && !records[3].snapshot && !('schema' in records[1]));

    const dividing = path.join(directory, 'dividing.cav');
    await writeFile(dividing, 'state share = 0;\nevent read value min 0 max 9;\non read when value > 1.5 set share = 1;\n' + provenanceOverflow('on read when value > 1.5'));
    await writeFile(events, [1, 2, 3].map(value => JSON.stringify({ event: 'read', payload: { value } })).join('\n'));
    const fatal = caveat(['replay', dividing, events]);
    assert.equal(fatal.status, 1);
    const last = fatal.stdout.trim().split('\n').map(line => JSON.parse(line));
    assert.deepEqual(last.map(record => record.record), ['initial', 'event', 'fatal']);
    assert.equal(last[2].line, 2);

    assert.equal(caveat(['replay', dividing]).status, 2);
    assert.equal(caveat(['replay', '--json', dividing, events]).status, 2);
  });
});

// Repetition counts every entity the loader declares (#47), so a comment in an
// entity statement does not move an event to another member's copy.
test('a comment in an entity statement does not move a replayed read to another plot', async () => {
  const plain = [
    'place field kind field;',
    'entity north # the first plot',
    '    kind plot at field;',
    'entity south kind plot at field;',
    'state north_n = 0;',
    'state south_n = 0;',
    'event read target kind plot;',
    'for plot as $p {',
    '    on read when target == $index set $p_n = $p_n + 1;',
    '};',
    '',
  ].join('\n');
  const routed = plain.replace('for plot as $p {', 'for plot as $p routed by target {')
    .replace(' when target == $index', '');
  await inDirectory(async directory => {
    for (const [name, text] of [['plain.cav', plain], ['routed.cav', routed]]) {
      const program = path.join(directory, name);
      await writeFile(program, text);
      for (const [target, counts] of [['north', [1, 0]], ['south', [0, 1]]]) {
        const events = path.join(directory, 'events.jsonl');
        await writeFile(events, JSON.stringify({ event: 'read', payload: { target } }) + '\n');
        const result = caveat(['replay', program, events]);
        assert.equal(result.status, 0, result.stderr);
        const [initial, read] = result.stdout.trim().split('\n').map(line => JSON.parse(line));
        const signature = initial.snapshot.events.find(event => event.name === 'read');
        assert.deepEqual(signature.parameters[0].domain.entity.members, ['north', 'south']);
        assert.equal(read.outcome, 'accepted');
        assert.deepEqual([read.snapshot.values.north_n, read.snapshot.values.south_n], counts, `${name}: read ${target}`);
      }
    }
  });
});

test('dependents answers for the guide program, as text and as JSON', () => {
  const program = path.join(kit, 'templates', 'umbrella.cav');
  const events = path.join(kit, 'templates', 'events.jsonl');
  const text = caveat(['dependents', program, 'sky', events]);
  assert.equal(text.status, 0, text.stderr);
  assert.match(text.stdout, /^What rests on sky in umbrella\.cav after 4 events \(sequence 3\)/);
  assert.match(text.stdout, /Decision changes\n {2}#2 clear_sky: umbrella@1 reopened because sky\n/);
  assert.match(text.stdout, /advice\.text = "Think again: the sky has cleared" {2}cites sky/);

  const json = JSON.parse(caveat(['dependents', '--json', program, 'rain_chance', events]).stdout);
  assert.equal(json.schema, 'caveat-dependents/0.1');
  assert.equal(json.events.length, 4);
  assert.deepEqual(json.decisions.map(item => [item.id, item.status, item.basis, item.via]),
    [['umbrella@1', 'reopened', 'grounds', ['rain_chance@1']]]);

  const unknown = caveat(['dependents', program, 'nowhere']);
  assert.equal(unknown.status, 2);
  assert.match(unknown.stderr, /nowhere is not evidence, a reading stream or a caveat/);
});
