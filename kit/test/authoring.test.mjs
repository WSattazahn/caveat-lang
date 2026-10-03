import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { AUTHORING_LIMITS, AuthoringError, runAuthoringOperation, validateAuthoringArguments } from '../lib/authoring.mjs';
import { defaultRuntimeDirectory } from '../lib/node.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const cli = fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url));
const worker = fileURLToPath(new URL('../lib/authoring-worker.mjs', import.meta.url));
const source = `claim ready;
evidence probe from "supplied fixture readings";
readings values from probe limit 8;
decisions plan limit 4 reopened by values;
state touched = 0;
event read value min 0 max 100;
event assess;
event refuse;
event fault;
on read sample values = value supports ready;
on assess commit plan because enough using latest(values);
on refuse set touched = 1;
on refuse reject "fixture policy";
on fault set touched = sqrt(0 - 1);
bind display.value = touched;
`;
const events = [
  { event: 'read', payload: { value: 85 } }, { event: 'assess' }, { event: 'refuse' },
];
const scenarios = {
  schema: 'caveat-scenarios/0.1', source: 'inline.cav', scenarios: [{ id: 'S1', title: 'grounded and atomic', steps: [
    { send: 'read', payload: { value: 85 } }, { send: 'assess' },
    { expect: { '/commitment_grounds/plan@1': { evidence: ['values@1'], caveats: [] } } },
    { checkpoint: 'approved' }, { send: 'refuse', rejected: 'fixture policy' },
    { same_as: 'approved', paths: ['/values', '/commitment_grounds'] }, { resume: true },
  ] }],
};
let next = 0;
async function cliFixture(text = source, sends = events, doc = scenarios) {
  const directory = path.join(root, 'test-results', 'authoring-operations', `${Date.now()}-${next++}`);
  await mkdir(directory, { recursive: true });
  await writeFile(path.join(directory, 'inline.cav'), text);
  await writeFile(path.join(directory, 'inline.scenarios.json'), JSON.stringify(doc));
  await writeFile(path.join(directory, 'events.jsonl'), sends.map(event => JSON.stringify(event)).join('\n'));
  return args => spawnSync(process.execPath, [cli, ...args], { cwd: directory, encoding: 'utf8' });
}
const bridge = (tool, args) => runAuthoringOperation(`caveat_${tool}`, args);
const job = input => spawnSync(process.execPath, [worker, defaultRuntimeDirectory()], {
  input: typeof input === 'string' ? input : JSON.stringify(input), encoding: 'utf8', maxBuffer: 8 * 1024 * 1024,
});

test('all five inline authoring results preserve the existing CLI report shapes and values', async () => {
  const command = await cliFixture();
  for (const [operation, args, cliArgs] of [
    ['validate', { source }, ['validate', '--json', 'inline.cav']],
    ['check', { source, strict: true }, ['check', '--json', '--strict', 'inline.cav']],
    ['test', { source, scenarios }, ['test', '--json', 'inline.scenarios.json']],
    ['explain', { source, events }, ['explain', '--json', 'inline.cav', 'events.jsonl']],
    ['dependents', { source, subject: 'values@1', events }, ['dependents', '--json', 'inline.cav', 'values@1', 'events.jsonl']],
  ]) {
    const expected = command(cliArgs);
    const actual = await bridge(operation, args);
    assert.equal(actual.schema, 'caveat-authoring/0.1');
    assert.equal(actual.operation, operation);
    assert.equal(actual.exitCode, expected.status, expected.stdout + expected.stderr);
    assert.deepEqual(actual.report, JSON.parse(expected.stdout), operation);
    assert.equal(actual.sourceSha256, createHash('sha256').update(source).digest('hex'));
    assert.match(actual.runtime.reactiveWasmSha256, /^[0-9a-f]{64}$/);
  }
});

test('strict warnings, syntax errors and scenario assertion failures retain authoring exit codes', async () => {
  const warning = source.replace(' reopened by values', '');
  const warned = await bridge('check', { source: warning, strict: true });
  assert.equal(warned.exitCode, 1);
  assert.ok(warned.report.diagnostics.length > 0);
  const allowed = warning.replace('decisions plan', '# caveat check: allow C002\ndecisions plan');
  const suppressed = await bridge('check', { source: allowed, strict: true });
  assert.equal(suppressed.exitCode, 0);
  assert.ok(suppressed.report.suppressed.length > 0);
  for (const operation of ['validate', 'check']) {
    const invalid = await bridge(operation, { source: 'state invalid = ;' });
    assert.equal(invalid.exitCode, 2);
    assert.equal(invalid.report.loads, false);
    assert.equal(typeof invalid.report.error, 'string');
  }
  const failing = structuredClone(scenarios);
  failing.scenarios[0].steps[2].expect['/commitment_grounds/plan@1'].evidence = ['values@2'];
  const result = await bridge('test', { source, scenarios: failing });
  assert.equal(result.exitCode, 1);
  assert.equal(result.report.failed, 1);
  const malformed = await bridge('test', { source, scenarios: { source: 'inline.cav', schema: 'wrong', scenarios: [] } });
  assert.equal(malformed.exitCode, 2);
  assert.equal(malformed.report.valid, false);
  assert.match(malformed.report.error, /schema must be/);
});

test('policy refusal remains an outcome and fatal dispatch reports the last good snapshot', async () => {
  const sent = [...events, { event: 'fault' }, { event: 'read', payload: { value: 90 } }];
  const command = await cliFixture(source, sent);
  const result = await bridge('explain', { source, events: sent });
  const expected = command(['explain', '--json', 'inline.cav', 'events.jsonl']);
  assert.equal(result.exitCode, 1);
  assert.deepEqual(result.report, JSON.parse(expected.stdout));
  assert.deepEqual(result.report.events.map(item => item.outcome.outcome), ['accepted', 'accepted', 'rejected', 'fatal']);
  assert.equal(result.report.events[2].outcome.origin, 'policy');
  assert.equal(result.report.sequence, 2);
  assert.deepEqual(result.report.decisions[0].revisions[0].grounds, { evidence: ['values@1'], caveats: [] });
  assert.equal(result.report.displayed[0].value, 0, 'the rejected event did not keep its earlier write');
  const unknown = await bridge('dependents', { source, subject: 'missing', events });
  assert.equal(unknown.exitCode, 2);
  assert.match(unknown.report.error, /missing is not evidence/);
});

test('the bridge accepts inline source only and refuses virtual source escapes before loading', async t => {
  for (const target of ['../secret.cav', '/tmp/secret.cav', 'C:\\private\\secret.cav', 'file:///secret.cav', 'https://example.com/program.cav']) {
    await t.test(target, async () => {
      const external = structuredClone(scenarios);
      external.source = target;
      await assert.rejects(runAuthoringOperation('caveat_test', { source, scenarios: external }, { runtimeDirectory: '/definitely-missing-runtime' }), error => error instanceof AuthoringError && error.kind === 'input' && /inline.cav/.test(error.message));
      external.source = 'inline.cav'; external.scenarios[0].source = target;
      assert.throws(() => validateAuthoringArguments('caveat_test', { source, scenarios: external }), /inline.cav/);
    });
  }
  for (const [tool, args] of [
    ['caveat_validate', { source, path: 'secret.cav' }],
    ['caveat_validate', { source, runtimeDirectory: 'custom' }],
    ['caveat_explain', { source, events: [{ event: 'read', payload: [], extra: true }] }],
    ['caveat_explain', { source, events: [{ event: 'read', payload: { value: Infinity } }] }],
    ['caveat_check', { source, strict: 'true' }],
    ['caveat_dependents', { source, subject: '' }],
    ['caveat_whatif', { source }],
    [{ toString: null }, { source }],
  ]) assert.throws(() => validateAuthoringArguments(tool, args), error => error instanceof AuthoringError && error.kind === 'input');
});

test('source, data, event, scenario, step, expanded-send and depth limits apply before runtime loading', async () => {
  const doc = steps => ({ schema: 'caveat-scenarios/0.1', source: 'inline.cav', scenarios: [{ id: 'S', title: 'limits', steps }] });
  let nested = {};
  for (let i = 0; i < 65; i++) nested = { nested };
  const over = [
    ['caveat_validate', { source: 'é'.repeat(AUTHORING_LIMITS.sourceBytes / 2 + 1) }],
    ['caveat_explain', { source, events: [{ event: 'read', payload: { value: 'x'.repeat(AUTHORING_LIMITS.dataBytes) } }] }],
    ['caveat_explain', { source, events: Array.from({ length: 1001 }, () => ({ event: 'assess' })) }],
    ['caveat_test', { source, scenarios: { ...doc([]), note: 'x'.repeat(AUTHORING_LIMITS.dataBytes) } }],
    ['caveat_test', { source, scenarios: { ...doc([]), scenarios: Array.from({ length: 33 }, (_, i) => ({ id: `${i}`, title: 'n', steps: [{ resume: true }] })) } }],
    ['caveat_test', { source, scenarios: doc(Array.from({ length: 2001 }, () => ({ resume: true }))) }],
    ['caveat_test', { source, scenarios: doc([{ send: 'read', payload: { value: 1 }, repeat: 1001 }]) }],
    ['caveat_explain', { source, events: [{ event: 'read', payload: nested }] }],
  ];
  for (const [tool, args] of over) {
    await assert.rejects(runAuthoringOperation(tool, args, { runtimeDirectory: '/definitely-missing-runtime' }), error => error instanceof AuthoringError && error.kind === 'limit');
  }
  assert.doesNotThrow(() => validateAuthoringArguments('caveat_validate', { source: 'x'.repeat(AUTHORING_LIMITS.sourceBytes) }));
  assert.doesNotThrow(() => validateAuthoringArguments('caveat_test', { source, scenarios: doc([{ send: 'read', repeat: 1000 }]) }));
});

test('one-shot worker distinguishes completed authoring failure from bridge failure without stray stdout', () => {
  const completed = job({ tool: 'caveat_validate', arguments: { source: 'invalid source;' } });
  assert.equal(completed.status, 0, completed.stderr);
  assert.equal(JSON.parse(completed.stdout).exitCode, 2);
  assert.equal(completed.stdout.trim().split('\n').length, 1);
  for (const input of ['not json', '{}', JSON.stringify({ tool: 'caveat_validate', arguments: { source, path: '/secret' } })]) {
    const invalid = job(input);
    assert.equal(invalid.status, 1, invalid.stderr);
    assert.equal(JSON.parse(invalid.stdout).schema, 'caveat-authoring-error/0.1');
    assert.equal(JSON.parse(invalid.stdout).error.kind, 'input');
  }
  const oversized = job(' '.repeat(AUTHORING_LIMITS.messageBytes + 1));
  assert.equal(oversized.status, 1, oversized.stderr);
  assert.equal(JSON.parse(oversized.stdout).error.kind, 'limit');
  const fresh = job({ tool: 'caveat_validate', arguments: { source } });
  assert.equal(fresh.status, 0, fresh.stderr);
  assert.equal(JSON.parse(fresh.stdout).report.loads, true);
});
