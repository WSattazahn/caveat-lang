// The preserved spec evidence (experiments/scenario-format) run through the
// real runner and CLI.
import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readdir, readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { formatFileReport, parseScenarioFile, runScenarioFile, ScenarioFileError } from '../lib/scenarios.mjs';
import { real } from './helpers.mjs';

const evidence = fileURLToPath(new URL('../../experiments/scenario-format/', import.meta.url));
const cli = fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url));

// file -> [expected result, failure kind or invalid-message pattern]
const expected = {
  'examples/thermostat_history': ['pass'],
  'examples/trail_rescue': ['pass'],
  'faults/neg-pass-lineage-as-set': ['pass'],
  'faults/neg-evaluation-bound': ['pass'],
  'faults/neg-repeat': ['pass'],
  'faults/neg-wrong-value': ['fail', 'expect'],
  'faults/neg-journal-order': ['fail', 'expect'],
  'faults/neg-lineage-order-matters': ['fail', 'expect'],
  'faults/neg-bare-rejected-on-input': ['fail', 'send'],
  'faults/neg-wrong-policy-message': ['fail', 'send'],
  'faults/neg-expected-accept-got-reject': ['fail', 'send'],
  'faults/neg-same-as-before-changed': ['fail', 'same_as'],
  'faults/neg-fatal-not-a-rejection': ['fail', 'fatal'],
  'faults/neg-size-bound': ['fail', 'size'],
  'faults/neg-invalid-unknown-field': ['invalid', /unknown field "extra"/],
  'faults/neg-invalid-host-origin': ['invalid', /origin "host"/],
  'faults/neg-invalid-input-message': ['invalid', /only a policy rejection/],
  'faults/neg-invalid-absent-false': ['invalid', /\$absent takes true/],
  'faults/neg-invalid-mixed-matcher': ['invalid', /only member/],
};

for (const [name, [result, detail]] of Object.entries(expected)) {
  test(`${name} ${result === 'invalid' ? 'is refused' : result === 'pass' ? 'passes' : `fails at ${detail}`}`, async () => {
    const file = path.join(evidence, `${name}.scenarios.json`);
    const text = await readFile(file, 'utf8');
    if (result === 'invalid') {
      assert.throws(() => parseScenarioFile(text), error => error instanceof ScenarioFileError && detail.test(error.message));
      return;
    }
    const outcome = await runScenarioFile(parseScenarioFile(text), {
      runtime: real, file: name, readSource: relative => readFile(path.resolve(path.dirname(file), relative), 'utf8'),
    });
    if (result === 'pass') {
      assert.equal(outcome.failed, 0, formatFileReport(outcome));
    } else {
      const failed = outcome.scenarios.filter(scenario => !scenario.pass);
      assert.equal(failed.length, 1, formatFileReport(outcome));
      assert.equal(failed[0].failure.kind, detail, formatFileReport(outcome));
    }
  });
}

test('the CLI exits 0, 1 or 2 and prints what differed', () => {
  const run = (...files) => spawnSync(process.execPath, [cli, 'test', ...files.map(file => path.join(evidence, `${file}.scenarios.json`))], { encoding: 'utf8' });
  const pass = run('examples/thermostat_history', 'examples/trail_rescue');
  assert.equal(pass.status, 0, pass.stderr);
  assert.match(pass.stdout, /PASS R01 /);
  const fail = run('examples/trail_rescue', 'faults/neg-bare-rejected-on-input');
  assert.equal(fail.status, 1);
  assert.match(fail.stdout, /FAIL R01 step 8 send \[primary\]: expected a policy rejection; got input\/bound_exceeded "dt must be finite and in 0\.\.30"/);
  const invalid = run('examples/trail_rescue', 'faults/neg-invalid-host-origin');
  assert.equal(invalid.status, 2);
  assert.equal(invalid.stdout, '', 'nothing runs when a file is invalid');
  assert.match(invalid.stderr, /INVALID .*neg-invalid-host-origin/);
  const json = spawnSync(process.execPath, [cli, 'test', '--json', path.join(evidence, 'examples/trail_rescue.scenarios.json')], { encoding: 'utf8' });
  const report = JSON.parse(json.stdout);
  assert.equal(report.schema, 'caveat-scenario-report/0.1');
  assert.equal(report.dispatchSchema, 'caveat-dispatch/0.1');
  assert.match(report.runtime.reactiveWasmSha256, /^[0-9a-f]{64}$/);
  assert.match(report.files[0].sources['../../../game/trail_rescue.cav'], /^[0-9a-f]{64}$/);
  const usage = spawnSync(process.execPath, [cli, 'bogus'], { encoding: 'utf8' });
  assert.equal(usage.status, 2);
});

test('every examples/*.scenarios.json passes', async () => {
  const directory = fileURLToPath(new URL('../../examples/', import.meta.url));
  const files = (await readdir(directory)).filter(name => name.endsWith('.scenarios.json'));
  assert.ok(files.length >= 1);
  for (const name of files) {
    const file = path.join(directory, name);
    const outcome = await runScenarioFile(parseScenarioFile(await readFile(file, 'utf8')), {
      runtime: real, file: name, readSource: relative => readFile(path.resolve(directory, relative), 'utf8'),
    });
    assert.equal(outcome.failed, 0, formatFileReport(outcome));
  }
});
