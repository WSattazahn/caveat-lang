// caveat check (spec/caveat-check-0.1.md), run as a user runs it, and
// runtime.check from the library.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { CaveatError, createRuntime } from '../lib/session.mjs';
import { Real, real } from './helpers.mjs';

const kit = fileURLToPath(new URL('../', import.meta.url));
const cli = path.join(kit, 'bin', 'caveat.mjs');
const caveat = args => spawnSync(process.execPath, [cli, ...args], { encoding: 'utf8' });

// Line 8 declares the decision; lines 12-13 repeat lines 10-11.
const FROST = `claim frost_risk;
evidence probe from "a soil probe";
evidence drone from "a survey drone";
readings soil from probe limit 12;
readings aerial from drone limit 12;
event probe_read celsius min -40 max 60;
event drone_read celsius min -40 max 60;
decisions uncover limit 4;
event decide;
on probe_read when celsius <= 2 sample soil = celsius supports frost_risk;
on probe_read when celsius > 2 sample soil = celsius opposes frost_risk;
on drone_read when celsius <= 2 sample aerial = celsius supports frost_risk;
on drone_read when celsius > 2 sample aerial = celsius opposes frost_risk;
on decide when not committed(uncover) commit uncover because enough using latest(soil);
`;

async function withProgram(source, body) {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-check-'));
  const program = path.join(directory, 'frost.cav');
  await writeFile(program, source);
  try { return await body(program); } finally { await rm(directory, { recursive: true, force: true }); }
}

test('check prints each warning with its place, what it found and what to consider', async () => {
  await withProgram(FROST, async program => {
    const result = caveat(['check', program]);
    assert.equal(result.status, 0, result.stderr);
    const lines = result.stdout.trim().split('\n');
    assert.match(lines[0], /^frost\.cav:8:1: warning C002 no-reopening-path: `uncover` is decided on readings from `soil`, and no reopening path was detected/);
    assert.match(lines[1], /^ {2}Confirm that keeping `uncover` fixed is intended\./);
    assert.equal(lines[2], 'frost.cav:12:1: warning C001 repeated-rules: the 2 rules on `drone_read` repeat the rules on `probe_read` (line 10), with `aerial` for `soil`');
    assert.match(lines[3], /^ {2}If both events should keep following the same rules/);
    assert.equal(lines[4], '  see line 10: the rules on `probe_read`');
    assert.equal(lines.at(-1), '2 warnings. A warning points at a pattern worth a second look; it is not an error.');
  });
});

test('--strict fails on a warning; --json gives the report', async () => {
  await withProgram(FROST, async program => {
    assert.equal(caveat(['check', '--strict', program]).status, 1);
    const result = caveat(['check', '--json', '--strict', program]);
    assert.equal(result.status, 1);
    const report = JSON.parse(result.stdout);
    assert.deepEqual([report.schema, report.program, report.loads, report.strict],
      ['caveat-check/0.1', program, true, true]);
    assert.deepEqual(report.diagnostics.map(warning => [warning.code, warning.name, warning.severity, warning.line, warning.column]),
      [['C002', 'no-reopening-path', 'warning', 8, 1], ['C001', 'repeated-rules', 'warning', 12, 1]]);
    assert.deepEqual(report.diagnostics[1].related, [{ line: 10, column: 1, note: 'the rules on `probe_read`' }]);
    assert.deepEqual(report.suppressed, []);
  });
});

test('an allow comment silences a warning, which is still listed', async () => {
  const source = FROST
    .replace('decisions uncover', '# caveat check: allow no-reopening-path\ndecisions uncover')
    .replace('on drone_read when celsius <= 2', '// caveat check: allow C001\non drone_read when celsius <= 2');
  await withProgram(source, async program => {
    const result = caveat(['check', '--strict', program]);
    assert.equal(result.status, 0, result.stdout);
    assert.deepEqual(result.stdout.trim().split('\n'), [
      'frost.cav:9:1: allowed C002 no-reopening-path (an allow comment silences it)',
      'frost.cav:14:1: allowed C001 repeated-rules (an allow comment silences it)',
      'frost.cav: no warnings.',
    ]);
    assert.deepEqual(JSON.parse(caveat(['check', '--json', program]).stdout).suppressed.map(warning => warning.code), ['C002', 'C001']);
  });
});

test('a program that does not load, or a bundle, is exit 2', async () => {
  await withProgram(`${FROST}on decide sample nowhere = 1 supports frost_risk;\n`, async program => {
    const text = caveat(['check', program]);
    assert.equal(text.status, 2);
    assert.match(text.stderr, /frost\.cav does not load: .*nowhere must name a declared reading stream/);
    const json = caveat(['check', '--json', program]);
    assert.equal(json.status, 2);
    const report = JSON.parse(json.stdout);
    assert.deepEqual([report.schema, report.loads], ['caveat-check/0.1', false]);
    assert.match(report.error, /nowhere must name a declared reading stream/);
  });
  await withProgram(`#caveat-bundle 1\n${FROST}`, async program => {
    const result = caveat(['check', program]);
    assert.equal(result.status, 2);
    assert.match(result.stderr, /is a bundle; check reads a single-file program/);
  });
  assert.equal(caveat(['check']).status, 2);
  assert.match(caveat(['validate', '--strict', path.join(kit, 'templates', 'umbrella.cav')]).stderr, /unknown option --strict for validate/);
});

test('the getting-started program checks clean', () => {
  const result = caveat(['check', path.join(kit, 'templates', 'umbrella.cav')]);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout.trim(), 'umbrella.cav: no warnings.');
});

test('runtime.check returns the report and throws a load error as open does', () => {
  assert.equal(real.check(FROST).diagnostics.length, 2);
  assert.throws(() => real.check('event;'), error => error instanceof CaveatError && error.kind === 'load');
  // A runtime built before check says so rather than failing obscurely.
  class Older { constructor(source) { this.inner = new Real(source); } }
  assert.throws(() => createRuntime(Older).check(FROST), /this runtime build has no check/);
});
