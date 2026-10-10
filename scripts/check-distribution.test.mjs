import assert from 'node:assert/strict';
import test from 'node:test';
import { compareDistribution, parseArguments } from './check-distribution.mjs';

const version = '0.1.0-rc.16';
const launch = { runtimeHint: 'npx', transport: { type: 'stdio' }, packageArguments: [{ type: 'positional', value: 'mcp' }] };
const serverJson = {
  name: 'io.github.WSattazahn/caveat-lang',
  description: 'Authoring bridge for Caveat: validate, check and test programs; explain decisions; find dependents.',
  version,
  packages: [{ registryType: 'npm', identifier: 'caveat-lang', version, ...launch }],
};
const registry = (overrides = {}, meta = {}) => ({
  server: { ...serverJson, ...overrides,
    // The live registry returns launch keys in its own order.
    packages: [{ registryType: 'npm', identifier: 'caveat-lang', version: overrides.version ?? version,
      runtimeHint: 'npx', transport: { type: 'stdio' }, packageArguments: [{ value: 'mcp', type: 'positional' }] }] },
  _meta: { 'io.modelcontextprotocol.registry/official': { status: 'active', isLatest: true, ...meta } },
});
const status = result => Object.fromEntries(result.rows.map(row => [row.check, row.status]));

test('every channel naming the version, with matching metadata, passes', () => {
  const result = compareDistribution({ version, serverJson, distTags: { latest: version, next: version }, registry: registry() });
  assert.equal(result.ok, true);
  assert.ok(result.rows.every(row => row.status === 'ok'));
});

test('the rc.16 state on 2026-10-10 fails on next and the registry until the owner closes out', () => {
  const state = { version, serverJson, distTags: { latest: version, next: '0.1.0-rc.15' }, registry: registry({ version: '0.1.0-rc.15' }) };
  const result = compareDistribution(state);
  assert.equal(result.ok, false);
  assert.equal(status(result)['npm-latest'], 'ok');
  assert.equal(status(result)['npm-next'], 'mismatch');
  assert.equal(status(result).registry, 'mismatch');
  assert.equal(status(result)['registry-launch'], 'ok', 'key order alone is not a launch change');

  const lagged = compareDistribution({ ...state, lag: { 'npm-next': 'owner step pending', registry: 'owner step pending' } });
  assert.equal(lagged.ok, true);
  assert.deepEqual(lagged.rows.filter(row => row.status === 'lag').map(row => [row.check, row.found, row.reason]),
    [['npm-next', '0.1.0-rc.15', 'owner step pending'], ['registry', '0.1.0-rc.15', 'owner step pending']]);
});

test('a stated lag never excuses changed registry metadata', () => {
  const lag = { registry: 'owner step pending' };
  const distTags = { latest: version, next: version };
  for (const [entry, meta, check] of [
    [registry({ description: 'Persistent sessions for agents.' }), {}, 'registry-description'],
    [registry({ name: 'io.github.wsattazahn/caveat-lang' }), {}, 'registry-name'],
    [registry(), { isLatest: false }, 'registry-is-latest'],
    [registry(), { status: 'deprecated' }, 'registry-status'],
  ]) {
    entry._meta['io.modelcontextprotocol.registry/official'] = { ...entry._meta['io.modelcontextprotocol.registry/official'], ...meta };
    const result = compareDistribution({ version, serverJson, distTags, registry: entry, lag });
    assert.equal(result.ok, false, check);
    assert.equal(status(result)[check], 'mismatch', check);
  }
  const moved = registry();
  moved.server.packages[0].packageArguments = [{ value: 'serve', type: 'positional' }];
  assert.equal(status(compareDistribution({ version, serverJson, distTags, registry: moved }))['registry-launch'], 'mismatch');
});

test('a registry entry whose server and package versions differ is a mismatch', () => {
  const split = registry();
  split.server.packages[0].version = '0.1.0-rc.15';
  const result = compareDistribution({ version, serverJson, distTags: { latest: version, next: version }, registry: split });
  assert.equal(status(result).registry, 'mismatch');
});

test('arguments name a released version and give each lag a reason', () => {
  assert.deepEqual(parseArguments(['--version', version, '--lag', 'npm-next=kept on rc.15 for one week']),
    { version, lag: { 'npm-next': 'kept on rc.15 for one week' } });
  assert.throws(() => parseArguments([]), /Name the released version/);
  assert.throws(() => parseArguments(['--version', 'main']), /Name the released version/);
  assert.throws(() => parseArguments(['--version', version, '--lag', 'next=x']), /one of npm-latest, npm-next, registry/);
  assert.throws(() => parseArguments(['--version', version, '--lag', 'npm-next']), /needs a reason/);
  assert.throws(() => parseArguments(['--version', version, '--lag', 'npm-next=  ']), /needs a reason/);
  assert.throws(() => parseArguments(['--latest']), /Unknown argument/);
});
