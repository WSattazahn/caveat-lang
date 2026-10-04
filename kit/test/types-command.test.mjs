// caveat types and runtime.interface (spec/caveat-interface-0.1.md), run as a
// user runs them, and the declarations kit/type-tests compiles.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { CaveatError } from '../lib/session.mjs';
import { INTERFACE_SCHEMA, declarations, typed } from '../lib/types.mjs';
import { real, thermostat } from './helpers.mjs';

const kit = fileURLToPath(new URL('../', import.meta.url));
const cli = path.join(kit, 'bin', 'caveat.mjs');
const caveat = (args, options = {}) => spawnSync(process.execPath, [cli, ...args], { encoding: 'utf8', ...options });
const gauge = path.join(kit, 'type-tests', 'gauge.cav');

test('runtime.interface returns the program interface, and a load error as open does', () => {
  const programInterface = real.interface(thermostat);
  assert.equal(programInterface.schema, INTERFACE_SCHEMA);
  assert.ok(programInterface.events.some(event => event.name === 'read'));
  const broken = 'event poke;\nbind gauge.value = missing;\n';
  let opened;
  try { real.open(broken); } catch (error) { opened = error; }
  assert.throws(() => real.interface(broken), error => error instanceof CaveatError && error.kind === 'load' && error.message === opened.message);
  assert.throws(() => real.interface(7), TypeError);
});

test('caveat types prints the declarations kit/type-tests compiles', async () => {
  const result = caveat(['types', '--from', '../lib/types.mjs', gauge]);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, await readFile(path.join(kit, 'type-tests', 'gauge.d.ts'), 'utf8'),
    'regenerate: node kit/bin/caveat.mjs types --from ../lib/types.mjs kit/type-tests/gauge.cav > kit/type-tests/gauge.d.ts');
  const plain = caveat(['types', gauge]);
  assert.match(plain.stdout, /^import type \{ TypedSession, TypedView \} from "caveat-lang\/types";$/m);
  assert.equal(plain.stdout.replace('"caveat-lang/types"', '"../lib/types.mjs"'), result.stdout);
});

test('caveat types --json prints the interface itself', async () => {
  const result = caveat(['types', '--json', gauge]);
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(JSON.parse(result.stdout), real.interface(await readFile(gauge, 'utf8')));
});

test('a program that does not load gets exit status 2 and no declarations', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-types-'));
  try {
    const program = path.join(directory, 'broken.cav');
    await writeFile(program, 'state level = 0;\n');
    const result = caveat(['types', program]);
    assert.equal(result.status, 2);
    assert.equal(result.stdout, '');
    assert.match(result.stderr, /broken\.cav does not load: reactive program must declare an event/);
  } finally { await rm(directory, { recursive: true, force: true }); }
  assert.equal(caveat(['types']).status, 2);
  assert.equal(caveat(['types', '--from']).status, 2);
});

test('declarations quote names that are not identifiers and refuse another schema', () => {
  const text = declarations({
    schema: INTERFACE_SCHEMA,
    events: [{ name: 'poke', parameters: [] }],
    states: [], cues: [], decisions: [], readings: [], evidence: [], caveats: [], claims: [],
    bindings: [{ target: 'cave', property: 'why.absorb', type: 'text', always: false }],
  }, { program: 'cave.cav' });
  assert.match(text, /^\/\/ Declarations for cave\.cav,/);
  assert.match(text, /^ {4}poke: Record<string, never>;$/m);
  assert.match(text, /^ {4}cave: \{ "why\.absorb"\?: string \};$/m);
  assert.match(text, /^export type CueName = never;$/m);
  assert.throws(() => declarations({ schema: 'caveat-check/0.1' }), /expected a caveat-interface\/0\.1 interface/);
});

test('typed returns the session itself', () => {
  const session = real.open(thermostat);
  assert.equal(typed(session), session);
  session.close();
});
