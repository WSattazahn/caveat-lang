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
    assert.match(result.stdout, /caveat test umbrella\.scenarios\.json/);
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
    await writeFile(dividing, 'state share = 0;\nevent read value min 0 max 9;\non read set share = 1 / (value - 2);');
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
