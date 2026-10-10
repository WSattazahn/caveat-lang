// Prepares round 8's directories outside the repository (PROTOCOL.md, round
// 7's "Registration notes" carried over). Each agent's directory holds only
// what the protocol gives it.
//
//   node prepare.mjs writer      the change-request writer's directory
//   node prepare.mjs reviewer    the scenario reviewer's (after the writer)
//
// --root=DIR sets the directory (default /home/claude/glowcap-r8). The authors'
// directories are added at stage 3, once rc.17 is published.
import { cpSync, existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const arg = (name, fallback) => process.argv.find((a) => a.startsWith(`--${name}=`))?.slice(name.length + 3) ?? fallback;
const ROOT = arg('root', '/home/claude/glowcap-r8');

function fresh(name) {
  const dir = path.join(ROOT, name);
  if (existsSync(dir)) throw new Error(`${dir} exists`);
  return dir;
}

function writer() {
  const dir = fresh('writer');
  cpSync(path.join(here, 'BEAT.md'), path.join(dir, 'BEAT.md'));
  cpSync(path.join(here, '..', 'scenarios.mjs'), path.join(dir, 'existing-scenarios.mjs'));
  cpSync(path.join(here, '..', 'round7', 'scenarios-r7.mjs'), path.join(dir, 'existing-scenarios-r7.mjs'));
  cpSync(path.join(here, 'workspace', 'README-writer.md'), path.join(dir, 'README.md'));
  console.log(dir);
}

function reviewer() {
  const from = path.join(ROOT, 'writer');
  const dir = fresh('reviewer');
  for (const file of ['BEAT.md', 'existing-scenarios.mjs', 'existing-scenarios-r7.mjs', 'REQUESTS.md', 'scenarios-r8.mjs']) {
    cpSync(path.join(from, file), path.join(dir, file));
  }
  cpSync(path.join(here, 'workspace', 'README-reviewer.md'), path.join(dir, 'README.md'));
  console.log(dir);
}

const steps = { writer, reviewer };
const step = steps[process.argv[2]];
if (!step) throw new Error(`usage: node prepare.mjs ${Object.keys(steps).join('|')}`);
step();
