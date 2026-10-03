// The commands beside test and explain: validate, replay, dependents and init,
// run as a user runs them.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { guideSteps } from './guide.mjs';

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
  ]) {
    const result = caveat(['test', path.join(ledger, name)]);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.match(result.stdout, new RegExp(`^${count} passed, 0 failed `, 'm'), name);
  }
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
    const fatal = caveat(['replay', path.join(ledger, 'ledger.cav'), events]);
    assert.equal(fatal.status, 1, fatal.stderr);
    assert.equal(last(fatal).record, 'fatal');
    assert.match(last(fatal).message, /cannot qualify a value with unobserved evidence pr26_go$/);
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
    await writeFile(dividing, 'decisions untyped limit 2;\nstate share = 0;\nevent read value min 0 max 9;\non read when value > 1.5 commit untyped because enough;\non read when value > 1.5 set share = latest(untyped);');
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
