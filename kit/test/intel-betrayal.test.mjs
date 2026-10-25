// The game example: intel carries its informant, and exposing the informant
// reopens the plans that rested on their word while each keeps its grounds.
// Its scenarios pass, and host.mjs prints what its README shows.
import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { real, sourceReader } from './helpers.mjs';
import { formatFileReport, parseScenarioFile, runScenarioFile } from '../lib/scenarios.mjs';
import { playIntel } from '../examples/intel-betrayal/host.mjs';

const folder = new URL('../examples/intel-betrayal/', import.meta.url);
const source = await readFile(new URL('intel.cav', folder), 'utf8');

test('the intel example\'s scenarios pass', async () => {
  const doc = parseScenarioFile(await readFile(new URL('intel.scenarios.json', folder), 'utf8'));
  const result = await runScenarioFile(doc, { runtime: real, file: 'intel.scenarios.json', readSource: sourceReader({ 'intel.cav': source }) });
  assert.equal(result.failed, 0, formatFileReport(result));
  assert.equal(result.passed, doc.scenarios.length);
});

test('the intel example loads with no warnings, unused declarations included', () => {
  assert.deepEqual(real.check(source, { unused: true }).diagnostics, []);
});

test('the host resumes its save, and the debrief names what each plan rested on', () => {
  const result = playIntel({ runtime: real, source });
  assert.equal(result.plan, 'you were played');
  assert.deepEqual(result.history, [{
    revision: 'north_plan@1', status: 'reopened',
    made_on: ['what ocelot tells you over the radio'], reopened_by: ['proof that ocelot was lying'],
  }]);
});

test('host.mjs prints what the README shows', async () => {
  const readme = await readFile(new URL('README.md', folder), 'utf8');
  const shown = /The host prints:\n\n```text\n([\s\S]*?)```/.exec(readme)[1];
  const run = spawnSync(process.execPath, [fileURLToPath(new URL('host.mjs', folder))], { encoding: 'utf8' });
  assert.equal(run.status, 0, run.stderr);
  assert.equal(run.stdout, shown);
});
