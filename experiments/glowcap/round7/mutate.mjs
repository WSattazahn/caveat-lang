// Runs every mutant the mutator wrote and records which catcher rejects it
// first: static (caveat check does not load / tsc --strict fails), runtime
// (the program fails to load or throws on an event the scenarios expect to be
// accepted), or suite (any other scenario failure at CR16). Survivors are
// replayed against their unmutated program with the round-7 fuzz; a survivor
// it cannot tell apart is labelled possibly equivalent.
//
//   node experiments/glowcap/round7/mutate.mjs [--root=DIR] [--fuzz-sequences=100] > results/mutation.json
import { spawnSync } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { PHASES, allScenarios, applies } from './lib.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const arg = (name, fallback) => process.argv.find((a) => a.startsWith(`--${name}=`))?.slice(name.length + 3) ?? fallback;
const ROOT = arg('root', '/home/claude/glowcap-r7');
const FUZZ = Number(arg('fuzz-sequences', 100));
const sites = JSON.parse(readFileSync(path.join(ROOT, 'mutator', 'sites.json'), 'utf8'));
const scenarios = (await allScenarios()).filter((s) => applies(s, PHASES.at(-1)));
const scenarioFile = path.join(ROOT, 'private', 'mutation-scenarios.json');
writeFileSync(scenarioFile, JSON.stringify(scenarios));

const run = (cmd, args, cwd) => spawnSync(cmd, args, { cwd, encoding: 'utf8', maxBuffer: 1 << 26, timeout: 15 * 60 * 1000 });

function stage(program, dir, files) {
  const work = path.join(ROOT, 'mut-run', program, dir);
  rmSync(work, { recursive: true, force: true });
  mkdirSync(path.join(work, 'impl'), { recursive: true });
  for (const f of files) cpSync(f.from, path.join(work, 'impl', f.name));
  symlinkSync(path.join(ROOT, 'authors', program, 'node_modules'), path.join(work, 'node_modules'));
  writeFileSync(path.join(work, 'package.json'), '{"type":"module"}\n');
  const tsconfig = path.join(ROOT, 'authors', program, 'tsconfig.json');
  if (existsSync(tsconfig)) cpSync(tsconfig, path.join(work, 'tsconfig.json'));
  return work;
}

// Scenario run in a child process, so a hang or crash is contained.
const CHILD = `
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
const { runScenario } = await import(pathToFileURL(process.argv[1]).href);
const scenarios = JSON.parse(readFileSync(process.argv[2], 'utf8'));
let createPolicy;
try {
  const m = await import(pathToFileURL(process.argv[3]).href);
  if (m.ready) await m.ready;
  createPolicy = m.createPolicy;
} catch (error) { console.log(JSON.stringify({ load: String(error?.message ?? error) })); process.exit(0); }
const failed = [];
for (const s of scenarios) { const f = runScenario(createPolicy, s); if (f.length) failed.push({ id: s.id, ...f[0] }); }
console.log(JSON.stringify({ failed }));
`;
const childFile = path.join(ROOT, 'private', 'mutation-child.mjs');
writeFileSync(childFile, CHILD);

function judge(program, work) {
  const caveat = program.startsWith('C');
  const entry = path.join(work, 'impl', caveat ? 'adapter.mjs' : 'glowcap.ts');
  const result = { static: false, runtime: false, suite: false, checkWarnings: 0, detail: '' };
  if (caveat) {
    for (const f of readdirSync(path.join(work, 'impl')).filter((n) => n.endsWith('.cav'))) {
      const r = run(process.execPath, [path.join(work, 'node_modules', 'caveat-lang', 'bin', 'caveat.mjs'), 'check', '--json', path.join('impl', f)], work);
      if (r.status === 2) { result.static = true; result.detail = (r.stdout + r.stderr).slice(0, 300); }
      else { try { result.checkWarnings += JSON.parse(r.stdout).diagnostics.length; } catch { result.checkWarnings = null; } }
    }
  } else {
    const r = run(process.execPath, [path.join(work, 'node_modules', 'typescript', 'bin', 'tsc'), '-p', 'tsconfig.json'], work);
    if (r.status !== 0) { result.static = true; result.detail = r.stdout.slice(0, 300); }
  }
  const r = run(process.execPath, ['--input-type=module', '-e', readFileSync(childFile, 'utf8'), path.join(here, 'exec.mjs'), scenarioFile, entry], work);
  let out;
  try { out = JSON.parse(r.stdout.trim().split('\n').at(-1)); } catch { out = { load: `child ${r.status} ${r.signal ?? ''} ${r.stderr.slice(0, 200)}` }; }
  if (out.load) { result.runtime = true; result.detail ||= out.load.slice(0, 300); }
  else if (out.failed.length) {
    const thrown = out.failed.find((f) => f.want === 'accepted' || f.where === 'createPolicy');
    if (thrown) result.runtime = true;
    else result.suite = true;
    result.failedScenarios = out.failed.map((f) => f.id);
    result.detail ||= JSON.stringify(out.failed[0]).slice(0, 300);
  }
  result.caught = result.static || result.runtime || result.suite;
  result.firstCatcher = result.static ? 'static' : result.runtime ? 'runtime' : result.suite ? 'suite' : null;
  return { result, entry };
}

const originals = {};
for (const program of [...new Set(sites.map((s) => s.program))]) {
  const dir = path.join(ROOT, 'authors', program, 'impl');
  const work = stage(program, 'original', readdirSync(dir).map((name) => ({ name, from: path.join(dir, name) })));
  const { result, entry } = judge(program, work);
  if (result.caught) throw new Error(`${program}'s unmutated program is not clean: ${result.detail}`);
  originals[program] = entry;
}

const records = [];
for (const site of sites) {
  const dir = path.join(ROOT, 'mutator', 'mutants', site.program, `${site.operator}-${site.site}`);
  if (!existsSync(dir)) { records.push({ ...site, missing: true }); continue; }
  const files = readdirSync(dir).filter((n) => n !== 'CHANGE.md').map((name) => ({ name, from: path.join(dir, name) }));
  const work = stage(site.program, `${site.operator}-${site.site}`, files);
  const { result, entry } = judge(site.program, work);
  if (!result.caught) {
    const f = run(process.execPath, [path.join(here, 'fuzz.mjs'), `--impl=${originals[site.program]}`, `--impl=${entry}`, `--sequences=${FUZZ}`, '--seed=11'], work);
    const m = /: (\d+) diverged/.exec(f.stdout);
    result.fuzzDiverged = m ? Number(m[1]) : null;
    result.possiblyEquivalent = result.fuzzDiverged === 0;
  }
  records.push({ ...site, ...result });
  console.error(site.program, site.operator, site.site, result.firstCatcher ?? (result.possiblyEquivalent ? 'survived (possibly equivalent)' : 'survived'));
}
console.log(JSON.stringify(records, null, 2));
