import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { real } from './helpers.mjs';
import { parseScenarioFile, runScenarioFile, formatFileReport } from '../lib/scenarios.mjs';

const folder = new URL('../examples/retractions/', import.meta.url);
const source = await readFile(new URL('retraction.cav', folder), 'utf8');
const scenarios = parseScenarioFile(await readFile(new URL('retraction.scenarios.json', folder), 'utf8'));
const events = (await readFile(new URL('retraction.events.jsonl', folder), 'utf8')).trim().split('\n').map(line => JSON.parse(line));

test('the retraction example loads without warnings and its scenarios pass', async () => {
  assert.deepEqual(real.check(source).diagnostics, []);
  const result = await runScenarioFile(scenarios, {
    runtime: real, file: 'retraction.scenarios.json', readSource: async () => source,
  });
  assert.equal(result.failed, 0, formatFileReport(result));
  assert.equal(result.passed, 4);
});

test('the timeline is accepted in full', () => {
  const session = real.open(source);
  try {
    for (const { event, payload } of events) assert.equal(session.dispatch(event, payload ?? {}).outcome, 'accepted', event);
  } finally {
    session.close();
  }
});

test('without the withdrawn guard, a new pause rests on the retracted study and the scenarios fail', async () => {
  const guard = 'on pause_announced when withdrawn(lancet_registry) reject "The study it would rest on is retracted.";\n';
  assert.equal(source.split(guard).length, 2);
  const result = await runScenarioFile(scenarios, {
    runtime: real, file: 'retraction.scenarios.json', readSource: async () => source.replace(guard, ''),
  });
  assert.ok(result.failed > 0, 'R04 must catch a commit on withdrawn evidence');
});

// The authoring guide's "Results produced under a version" program, and its
// claim that `renewable` evidence would ground the new result on both versions.
const guide = await readFile(new URL('../../docs/AI_AUTHORING.md', import.meta.url), 'utf8');
const versions = /<!-- versions-as-readings -->\n```caveat\n([\s\S]*?)```/.exec(guide)?.[1];
const timeline = [{ send: 'release' }, { send: 'analyse', payload: { estimate: 7 } }, { send: 'decide' }, { send: 'erratum_v1' },
  { send: 'release' }, { send: 'analyse', payload: { estimate: 5 } }, { send: 'decide' }];
const onlyItsVersion = parseScenarioFile(JSON.stringify({
  schema: 'caveat-scenarios/0.1', source: 'versions.cav',
  scenarios: [{ id: 'V01', title: 'each conclusion rests on the version its analysis was produced under', steps: [...timeline,
    { expect: {
      '/commitment_grounds/conclude@1': { evidence: { $set: ['analysis@1', 'versions@1'] } },
      '/commitment_grounds/conclude@2': { evidence: { $set: ['analysis@2', 'versions@2'] }, caveats: [] },
    } }] }],
}));

test('the guide\'s versions program grounds each conclusion on its own version', async () => {
  assert.ok(versions, 'the authoring guide has the versions-as-readings program');
  assert.deepEqual(real.check(versions).diagnostics, []);
  const result = await runScenarioFile(onlyItsVersion, { runtime: real, file: 'versions.scenarios.json', readSource: async () => versions });
  assert.equal(result.failed, 0, formatFileReport(result));
});

test('with renewable evidence for versions, the new conclusion also rests on the withdrawn version', async () => {
  const renewable = [
    'claim effect;',
    'evidence dataset from "a released version of the dataset";',
    'evidence pipeline from "the analysis code";',
    'evidence erratum from "a finding that a version contains duplicated rows";',
    'renewable dataset limit 4;',
    'readings analysis from pipeline limit 8;',
    'state data = 0;',
    'decisions conclude limit 4;',
    'event release;', 'event analyse estimate min 0 max 100;', 'event decide;', 'event erratum_v1;',
    'on release when observed(dataset) renew dataset;',
    'on release reveal dataset;',
    'on release set data = qualified(1, dataset);',
    'on analyse sample analysis = estimate * data supports effect;',
    'on decide when committed(conclude) and not reopened(conclude) reject "In force.";',
    'on decide commit conclude because enough using latest(analysis);',
    'on erratum_v1 reveal erratum;',
    'on erratum_v1 withdraw occurrence(dataset, 1) because erratum;',
    'on erratum_v1 when committed(conclude) and not reopened(conclude) and rests_on_withdrawn(conclude) reopen conclude because erratum;',
  ].join('\n');
  const session = real.open(renewable);
  try {
    let outcome;
    for (const step of timeline) outcome = session.dispatch(step.send, step.payload ?? {});
    assert.equal(outcome.outcome, 'accepted');
    const grounds = outcome.snapshot.commitment_grounds['conclude@2'];
    assert.deepEqual([...grounds.evidence].sort(), ['analysis@2', 'dataset', 'dataset@2']);
    assert.ok(grounds.caveats.includes('withdrawn'));
  } finally {
    session.close();
  }
});
