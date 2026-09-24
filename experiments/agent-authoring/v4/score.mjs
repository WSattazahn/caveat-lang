// Scores the frozen programs of one phase. Never run while an agent is working.
//
//   node score.mjs --phase 1|2 --tarball PATH [--output FILE]
//
// Checks the tarball, every public file and the files unsealed for this phase
// against registration.json, installs the tarball into a fresh directory, and
// runs each frozen runs/<id>/ferry.cav through the registered cases with the
// kit's session library.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) =>
  (index % 2 === 0 ? [...pairs, [value.replace(/^--/, ''), all[index + 1]]] : pairs), []));
const phase = Number(args.phase);
assert(phase === 1 || phase === 2, '--phase must be 1 or 2');
assert(args.tarball, '--tarball is required');

const sha = data => createHash('sha256').update(data).digest('hex');
const registration = JSON.parse(await readFile(path.join(here, 'registration.json'), 'utf8'));
assert.equal(sha(await readFile(args.tarball)), registration.kit.sha256, 'the tarball is not the registered release candidate');
const files = { ...registration.public };
for (const [file, sealed] of Object.entries(registration.sealed)) {
  if (sealed.unsealAfterPhase <= phase) files[file] = sealed.sha256;
}
for (const [file, expected] of Object.entries(files)) {
  assert.equal(sha(await readFile(path.join(here, file))), expected, `${file} differs from its registration`);
}

// A fresh install, used only through the package, as an agent used it.
const install = path.join(here, '..', '..', '..', 'test-results', 'agent-authoring-v4', `scorer-phase${phase}`);
await rm(install, { recursive: true, force: true });
await mkdir(install, { recursive: true });
const npm = process.platform === 'win32' ? 'npm.cmd' : 'npm';
execFileSync(npm, ['init', '-y'], { cwd: install, stdio: 'ignore', shell: process.platform === 'win32' });
execFileSync(npm, ['install', '--offline', '--no-audit', '--no-fund', path.resolve(args.tarball)],
  { cwd: install, stdio: 'ignore', shell: process.platform === 'win32' });
const kit = path.join(install, 'node_modules', 'caveat-lang');
const { loadRuntimeFromDirectory } = await import(pathToFileURL(path.join(kit, 'lib', 'node.mjs')).href);
const runtime = await loadRuntimeFromDirectory();

const { scoreProgram } = await import('./private/score-lib.mjs');
const [task, cases] = phase === 1
  ? [(await import('./private/oracle.mjs')).phase1, (await import('./private/corpus.mjs')).phase1Corpus()]
  : await import('./private/change-oracle.mjs').then(module => [module.phase2, module.phase2Corpus()]);

const results = { schema: 1, phase, at: new Date().toISOString(), kit: registration.kit, runs: {} };
for (const [id, run] of Object.entries(registration.runs).filter(([, run]) => run.phase === phase)) {
  const record = JSON.parse(await readFile(path.join(here, 'runs', id, 'record.json'), 'utf8'));
  const source = await readFile(path.join(here, 'runs', id, 'ferry.cav'), 'utf8');
  assert.equal(sha(source), record.sha256, `runs/${id}/ferry.cav changed after it was frozen`);
  const scored = scoreProgram(runtime, source, task, cases);
  results.runs[id] = { ...run, sha256: record.sha256, ...scored };
  console.log(`${id}: ${scored.loadError ? `does not load: ${scored.loadError}` : `${scored.passed}/${scored.total} cases`}${scored.pass ? ' — PASS' : ''}`);
}
const output = args.output ?? path.join(here, 'results', `phase${phase}.json`);
await mkdir(path.dirname(output), { recursive: true });
await writeFile(output, `${JSON.stringify(results, null, 2)}\n`, { flag: args.output ? 'w' : 'wx' });
// A failing program is a result, not a scorer error.
