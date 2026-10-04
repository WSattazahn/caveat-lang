// Post-hoc: for every failing `test` run of an author, re-runs that run's
// snapshot against the scenarios of its phase and prints the first failure of
// each failed scenario (where, wanted, got), to classify first-run failures.
//
//   node fuzz-checks/first-failures.mjs ID [--phase=crN] [--root=DIR]
import { cpSync, mkdirSync, readFileSync, rmSync, existsSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { allScenarios, applies, runScenario } from '../lib.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const ID = process.argv[2];
const ROOT = process.argv.find((a) => a.startsWith('--root='))?.slice(7) ?? '/home/claude/glowcap-r7';
const only = process.argv.find((a) => a.startsWith('--phase='))?.slice(8);
const log = readFileSync(path.join(here, '..', 'runs', ID, 'log.jsonl'), 'utf8').trim().split('\n').map((l) => JSON.parse(l))
  .filter((r) => r.kind === 'test' && r.failed.length && (!only || r.phase === only));
const scenarios = await allScenarios();
const work = path.join(ROOT, 'authors', ID, 'snapcheck');
rmSync(work, { recursive: true, force: true });
mkdirSync(work, { recursive: true });
execFileSync('tar', ['xzf', path.join(here, '..', 'runs', ID, 'snapshots.tgz'), '-C', work]);
const short = (v) => { const s = typeof v === 'string' ? v : JSON.stringify(v); return process.env.FULL ? s : (s.length > 220 ? `${s.slice(0, 220)}…` : s); };
for (const run of log) {
  const dir = path.join(work, 'snapshots', run.hash);
  if (!existsSync(dir)) { console.log(`${run.phase} ${run.hash}: no snapshot`); continue; }
  const copy = path.join(work, `run-${run.hash}`);
  cpSync(dir, copy, { recursive: true });
  const entry = path.join(copy, ID.startsWith('C') ? 'adapter.mjs' : 'glowcap.ts');
  console.log(`== ${run.at} ${run.phase} ${run.hash} ${run.passed}/${run.total}`);
  let createPolicy;
  try { const m = await import(pathToFileURL(entry).href); if (m.ready) await m.ready; createPolicy = m.createPolicy; }
  catch (error) { console.log(`  load: ${short(error?.message ?? error)}`); continue; }
  for (const sc of scenarios.filter((s) => applies(s, run.phase) && run.failed.includes(s.id))) {
    const f = runScenario(createPolicy, sc)[0];
    console.log(`  ${sc.id} step ${f?.step} ${f?.where}: want ${short(f?.want)} got ${short(f?.got)}`);
  }
}
rmSync(work, { recursive: true, force: true });
