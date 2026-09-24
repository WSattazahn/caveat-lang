// Pre-registration validation: the reference programs pass their corpora, and
// deliberately broken programs fail. Uses the repository kit and a local build.
import { mkdir, cp, readFile } from 'node:fs/promises';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
// Assemble draft + sealed modules side by side, as private/ will hold them.
const work = path.join(here, 'v3-validate');
await mkdir(work, { recursive: true });
for (const file of ['oracle.mjs', 'corpus.mjs', 'score-lib.mjs']) await cp(path.join(here, 'v3-draft/private', file), path.join(work, file));
await cp(path.join(here, 'v3-sealed/change-oracle.mjs'), path.join(work, 'change-oracle.mjs'));

// VALIDATE_KIT names an installed caveat-lang package; its bundled runtime is used.
const kitDir = process.env.VALIDATE_KIT;
const { loadRuntimeFromDirectory } = await import(pathToFileURL(path.join(kitDir ?? path.join(here, 'caveat-work/kit'), 'lib/node.mjs')).href);
const { phase1 } = await import(pathToFileURL(path.join(work, 'oracle.mjs')).href);
const { phase1Corpus } = await import(pathToFileURL(path.join(work, 'corpus.mjs')).href);
const { phase2, phase2Corpus } = await import(pathToFileURL(path.join(work, 'change-oracle.mjs')).href);
const { scoreProgram } = await import(pathToFileURL(path.join(work, 'score-lib.mjs')).href);
const runtime = kitDir ? await loadRuntimeFromDirectory() : await loadRuntimeFromDirectory(path.join(here, 'caveat-work/dist/pkg-reactive'));
if (kitDir) console.log(`runtime ${runtime.identity.revision}`);

const read = file => readFile(path.join(here, 'v3-sealed', file), 'utf8');
const phases = { 1: [phase1, phase1Corpus()], 2: [phase2, phase2Corpus()] };
const targets = process.argv[2] ? JSON.parse(await readFile(process.argv[2], 'utf8')) : null;
const runs = targets ?? [['reference-phase1.cav', 1, true], ['reference-phase2.cav', 2, true], ['reference-phase1.cav', 2, false], ['reference-phase2.cav', 1, false]];
let problems = 0;
for (const [file, phase, shouldPass] of runs) {
  const source = typeof file === 'string' && file.endsWith('.cav') ? await read(file) : file;
  const [task, cases] = phases[phase];
  const result = scoreProgram(runtime, source, task, cases);
  const ok = result.pass === shouldPass;
  if (!ok) problems++;
  console.log(`${ok ? 'ok ' : 'BAD'} phase ${phase} ${String(file).slice(0, 40).padEnd(40)} ${result.loadError ? `load error: ${result.loadError}` : `${result.passed}/${result.total}`} (expected ${shouldPass ? 'pass' : 'fail'})`);
  if (!ok || process.env.SHOW) for (const failure of result.cases.filter(test => !test.pass).slice(0, 3)) {
    console.log(`    ${failure.id} step ${failure.step} ${JSON.stringify(failure.event)}\n      ${failure.error.split('\n').slice(0, 6).join('\n      ')}`);
  }
}
process.exitCode = problems ? 1 : 0;
