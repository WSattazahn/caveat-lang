import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, readdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { formatDoctor } from '../lib/doctor.mjs';
import { formatAgentDemo } from '../lib/demo.mjs';

const cli = fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url));
const run = (args, cwd) => spawnSync(process.execPath, [cli, ...args], { cwd, encoding: 'utf8' });

test('doctor and demo are discoverable and reject unsupported arguments', () => {
  const help = run(['help']);
  assert.match(help.stdout, /caveat-lang doctor/);
  assert.match(help.stdout, /caveat-lang demo agent/);
  for (const args of [['doctor', 'extra'], ['doctor', '--strict'], ['demo'],
    ['demo', 'unknown'], ['demo', 'agent', 'extra'], ['demo', 'agent', '--strict']]) {
    assert.equal(run(args).status, 2, args.join(' '));
  }
});

test('doctor and demo text reflect JSON results, run elsewhere and create no files', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-adoption-'));
  try {
    for (const [args, format] of [[['doctor'], formatDoctor], [['demo', 'agent'], formatAgentDemo]]) {
      const json = run([...args, '--json'], directory);
      assert.equal(json.status, 0, json.stdout + json.stderr);
      const result = JSON.parse(json.stdout);
      const text = run(args, directory);
      assert.equal(text.status, 0, text.stdout + text.stderr);
      assert.equal(text.stdout.trimEnd(), format(result).trimEnd());
    }
    assert.deepEqual(await readdir(directory), []);
    const failed = run(['doctor', '--runtime', path.join(directory, 'missing'), '--json'], directory);
    assert.equal(failed.status, 1, failed.stderr);
    assert.equal(JSON.parse(failed.stdout).ok, false);
    assert.equal(run(['demo', 'agent', '--runtime', path.join(directory, 'missing')], directory).status, 2);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
