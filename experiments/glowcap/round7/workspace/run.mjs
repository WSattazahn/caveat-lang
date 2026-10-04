// The round-7 author runner. Every execution of a candidate goes through it,
// and every use is logged. Only `test` counts toward the measurements.
//
//   node run.mjs status                 the current phase and what to read
//   node run.mjs test                   run the released scenarios for the current phase
//   node run.mjs check                  static check: caveat check (Caveat) or tsc --strict (TypeScript)
//   node run.mjs try events.jsonl       replay your own events; prints each outcome and the final view
//   node run.mjs caveat ARGS…           the caveat CLI (Caveat authors only)
//   node run.mjs commit                 end the phase (all scenarios must pass) and release the next
//   node run.mjs commit --incomplete "why"   end the phase without all passing; recorded as such
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { appendFileSync, cpSync, existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ID = '@@ID@@';
const SIDE = '@@SIDE@@';
const PRIVATE = '@@PRIVATE@@';

const here = path.dirname(fileURLToPath(import.meta.url));
const mine = path.join(PRIVATE, ID);
const statePath = path.join(mine, 'state.json');
const { runScenario, toEvent, canonical } = await import(pathToFileURL(path.join(PRIVATE, 'tooling', 'exec.mjs')));
const PHASES = JSON.parse(readFileSync(path.join(PRIVATE, 'release', 'phases.json'), 'utf8'));
const ENTRY = SIDE === 'caveat' ? 'impl/adapter.mjs' : 'impl/glowcap.ts';

const state = () => JSON.parse(readFileSync(statePath, 'utf8'));
const saveState = (value) => writeFileSync(statePath, `${JSON.stringify(value, null, 2)}\n`);

function implFiles() {
  const dir = path.join(here, 'impl');
  if (!existsSync(dir)) return [];
  return readdirSync(dir, { withFileTypes: true, recursive: true }).filter((d) => d.isFile())
    .map((d) => path.relative(dir, path.join(d.parentPath ?? d.path, d.name))).sort();
}

function implHash() {
  const hash = createHash('sha256');
  for (const file of implFiles()) hash.update(file).update(readFileSync(path.join(here, 'impl', file)));
  return hash.digest('hex').slice(0, 12);
}

function log(record) {
  const s = state();
  const entry = { at: new Date().toISOString(), author: ID, side: SIDE, phase: s.phase, hash: implHash(), ...record };
  appendFileSync(path.join(mine, 'log.jsonl'), `${JSON.stringify(entry)}\n`);
  return entry;
}

function snapshot(hash) {
  const target = path.join(mine, 'snapshots', hash);
  if (!existsSync(target) && existsSync(path.join(here, 'impl'))) cpSync(path.join(here, 'impl'), target, { recursive: true });
}

async function load() {
  const module = await import(`${pathToFileURL(path.join(here, ENTRY)).href}?h=${implHash()}`);
  if (module.ready) await module.ready;
  if (typeof module.createPolicy !== 'function') throw new Error(`${ENTRY} does not export createPolicy`);
  return module.createPolicy;
}

async function test() {
  const s = state();
  if (s.done) return console.log('All phases are committed; the round is over for you.');
  const scenarios = JSON.parse(readFileSync(path.join(here, 'scenarios', 'current.json'), 'utf8'));
  const hash = implHash();
  snapshot(hash);
  let createPolicy;
  const results = [];
  try {
    createPolicy = await load();
  } catch (error) {
    for (const sc of scenarios) results.push({ id: sc.id, failures: [{ step: 0, where: 'load', want: 'module', got: String(error?.message ?? error), explanation: false }] });
  }
  if (createPolicy) for (const sc of scenarios) results.push({ id: sc.id, failures: runScenario(createPolicy, sc) });
  const failed = results.filter((r) => r.failures.length);
  log({ kind: 'test', passed: results.length - failed.length, total: results.length, failed: failed.map((r) => r.id),
    explanationFailed: failed.filter((r) => r.failures.some((f) => f.explanation)).map((r) => r.id) });
  console.log(`${ID} ${s.phase} ${hash}  ${results.length - failed.length}/${results.length} passed`);
  for (const r of failed) {
    const f = r.failures[0];
    console.log(`  ✗ ${r.id} step ${f.step} ${f.where}: want ${JSON.stringify(f.want)} got ${JSON.stringify(f.got)}`);
  }
  process.exitCode = failed.length ? 1 : 0;
}

function run(command, args) {
  snapshot(implHash());
  const result = spawnSync(command, args, { cwd: here, encoding: 'utf8' });
  process.stdout.write(result.stdout ?? '');
  process.stderr.write(result.stderr ?? '');
  return result.status ?? 1;
}

function check() {
  let status = 0;
  if (SIDE === 'caveat') {
    for (const file of implFiles().filter((f) => f.endsWith('.cav'))) {
      status = Math.max(status, run(process.execPath, [path.join(here, 'node_modules', 'caveat-lang', 'bin', 'caveat.mjs'), 'check', path.join('impl', file)]));
    }
  } else {
    status = run(process.execPath, [path.join(here, 'node_modules', 'typescript', 'bin', 'tsc'), '-p', 'tsconfig.json']);
  }
  log({ kind: 'check', status });
  process.exitCode = status;
}

async function tryEvents(file) {
  if (!file) throw new Error('usage: node run.mjs try events.jsonl');
  const events = readFileSync(path.resolve(here, file), 'utf8').split('\n').filter((l) => l.trim()).map((l) => JSON.parse(l));
  snapshot(implHash());
  let outcome = 'ok';
  try {
    const createPolicy = await load();
    let policy = createPolicy();
    for (const event of events) {
      if (event.resume) {
        const next = createPolicy(JSON.parse(JSON.stringify(policy.save())));
        console.log(`resume  same view: ${canonical(next.view()) === canonical(policy.view())}`);
        policy = next;
        continue;
      }
      try { policy.dispatch(event); console.log(`accepted ${JSON.stringify(event)}`); }
      catch (error) { console.log(`rejected ${JSON.stringify(event)}: ${error?.message ?? error}`); }
    }
    console.log(JSON.stringify(policy.view(), null, 2));
  } catch (error) {
    outcome = String(error?.message ?? error);
    console.log(`error: ${outcome}`);
  }
  log({ kind: 'try', file, events: events.length, outcome });
}

function caveat(args) {
  if (SIDE !== 'caveat') throw new Error('The caveat CLI is for Caveat authors.');
  const status = run(process.execPath, [path.join(here, 'node_modules', 'caveat-lang', 'bin', 'caveat.mjs'), ...args]);
  log({ kind: 'caveat', args, status });
  process.exitCode = status;
}

function release(phase) {
  const from = path.join(PRIVATE, 'release');
  cpSync(path.join(from, 'phases', `${phase}.md`), path.join(here, 'phases', `${phase}.md`));
  cpSync(path.join(from, 'scenarios', `${phase}.json`), path.join(here, 'scenarios', `${phase}.json`));
  cpSync(path.join(from, 'scenarios', `${phase}.json`), path.join(here, 'scenarios', 'current.json'));
}

function commit(args) {
  const s = state();
  if (s.done) return console.log('All phases are already committed.');
  const hash = implHash();
  const incomplete = args[0] === '--incomplete' ? (args[1] || 'no reason given') : null;
  const tests = readFileSync(path.join(mine, 'log.jsonl'), 'utf8').split('\n').filter(Boolean).map((l) => JSON.parse(l))
    .filter((e) => e.kind === 'test' && e.phase === s.phase && e.hash === hash);
  const green = tests.some((e) => e.passed === e.total);
  if (!green && !incomplete) {
    console.log('This source has no all-green `test` run in this phase. Run `node run.mjs test`, or commit with --incomplete "why".');
    process.exitCode = 1;
    return;
  }
  snapshot(hash);
  const repo = path.join(mine, 'repo');
  cpSync(path.join(here, 'impl'), path.join(repo, 'impl'), { recursive: true, force: true });
  spawnSync('git', ['add', '-A'], { cwd: repo });
  spawnSync('git', ['-c', 'user.name=round7', '-c', 'user.email=round7@invalid', 'commit', '-q', '--allow-empty', '-m', `${ID} ${s.phase}${incomplete ? ' (incomplete)' : ''}`], { cwd: repo });
  log({ kind: 'commit', green, incomplete });
  const next = PHASES[PHASES.indexOf(s.phase) + 1];
  if (!next) {
    saveState({ ...s, done: true });
    console.log(`${s.phase} committed. That was the last phase: write notes.md if you have not, and stop.`);
    return;
  }
  release(next);
  saveState({ ...s, phase: next });
  console.log(`${s.phase} committed. Phase ${next} is released: read phases/${next}.md; its scenarios are scenarios/current.json.`);
}

function status() {
  const s = state();
  console.log(s.done ? 'All phases committed.' : `Current phase: ${s.phase}. Read phases/${s.phase}.md (and every earlier phase file); scenarios/current.json holds this phase's scenarios.`);
}

const [command, ...args] = process.argv.slice(2);
const commands = { status, test, check, try: () => tryEvents(args[0]), caveat: () => caveat(args), commit: () => commit(args) };
if (!commands[command]) {
  console.log(readFileSync(fileURLToPath(import.meta.url), 'utf8').split('\n').slice(0, 11).join('\n'));
  process.exitCode = 1;
} else {
  await commands[command]();
}
