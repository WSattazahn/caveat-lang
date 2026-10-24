import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { real } from './helpers.mjs';
import { parseScenarioFile, runScenarioFile, formatFileReport } from '../lib/scenarios.mjs';

const folder = new URL('../examples/procedure-handles/', import.meta.url);
const source = await readFile(new URL('handles.cav', folder), 'utf8');
const scenarios = parseScenarioFile(await readFile(new URL('handles.scenarios.json', folder), 'utf8'));

test('the two procedure argument forms preserve distinct reopening reasons across restore', async () => {
  assert.deepEqual(real.check(source).diagnostics, []);
  const result = await runScenarioFile(scenarios, {
    runtime: real, file: 'handles.scenarios.json', readSource: async () => source,
  });
  assert.equal(result.failed, 0, formatFileReport(result));
  assert.equal(result.passed, 2);
});

for (const [label, before, after, message] of [
  ['an occurrence is not a declared evidence name', 'call by_evidence(correction_note)', 'call by_evidence(latest(samples))',
    /procedure by_evidence parameter e takes the name of a declared evidence/],
  ['a bare stream is not a declared evidence name', 'call by_evidence(correction_note)', 'call by_evidence(samples)',
    /procedure by_evidence parameter e takes the name of a declared evidence/],
  ['a reading expression is not a stream name', 'call by_reading(samples)', 'call by_reading(latest(samples))',
    /procedure by_reading parameter s takes the name of a declared reading stream/],
  ['an evidence name is not a stream name', 'call by_reading(samples)', 'call by_reading(measurement)',
    /procedure by_reading parameter s takes the name of a declared reading stream/],
]) test(label, () => {
  assert.equal(source.split(before).length, 2);
  const invalid = source.replace(before, after);
  assert.throws(() => real.open(invalid), error => error.kind === 'load' && message.test(error.message));
  assert.throws(() => real.check(invalid), message);
});

test('observing and passing the template instead changes the reason and fails the scenarios', async () => {
  const wrong = source.replace('on measure call by_reading(samples);',
    'on measure reveal measurement;\non measure call by_evidence(measurement);');
  const session = real.open(wrong);
  session.close();
  const result = await runScenarioFile(scenarios, {
    runtime: real, file: 'handles.scenarios.json', readSource: async () => wrong,
  });
  assert.ok(result.failed > 0, 'the scenario must distinguish a template from an occurrence');
});

test('compact first-use guidance is bounded and quotes the tested procedures', async () => {
  const markdown = await readFile(new URL('README.md', folder), 'utf8');
  const section = markdown.slice(markdown.indexOf('## Choose the parameter kind'), markdown.indexOf('## Run the example')).trim();
  assert.ok(Buffer.byteLength(section, 'utf8') <= 1200);
  const code = /```caveat\n([\s\S]*?)\n```/.exec(section)?.[1];
  assert.ok(code && source.includes(code));
  assert.ok(source.includes('call by_evidence(correction_note)'));
  assert.ok(source.includes('call by_reading(samples)'));
});
