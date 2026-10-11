// Records the hashes of round 8's registered files (PROTOCOL.md,
// "Registration"). Stage 1 is the approved protocol, the predictions, the
// probe and the inherited round 7 files; stage 2 the writer's and reviewer's
// inputs; stage 3 the blind requests (amendment 4); stage 4 what depends on
// rc.17 and the fuzz mode, before any author starts. No stage
// overwrites.
//
//   node experiments/glowcap/round8/register.mjs --stage=1|2|3|4
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const stage = process.argv.find((a) => a.startsWith('--stage='))?.split('=')[1];
const INHERITED = ['base', ...Array.from({ length: 16 }, (_, i) => `cr${i + 1}`)]
  .map((p) => `../round7/phases/${p}.md`);

const files = {
  1: ['PROTOCOL.md', 'PREDICTIONS.md', 'register.mjs', 'probe/instructions.md', 'probe/make-prompt.mjs',
    'probe/prompt.txt', '../round7/BEAT.md', ...INHERITED, '../round7/scenarios-r7.mjs',
    '../caveat5/glowcap.cav'],
  // Amendment 3: the writer's and reviewer's inputs, before either starts.
  2: ['PROTOCOL.md', 'register.mjs', 'build-beat.mjs', 'BEAT.md', 'prepare.mjs', 'prompts/writer.md',
    'prompts/reviewer.md', 'workspace/README-writer.md', 'workspace/README-reviewer.md', '../scenarios.mjs',
    '../round7/scenarios-r7.mjs'],
  // Amendment 4: the blind requests, before the runtime's stage.
  3: ['PROTOCOL.md', 'PREDICTIONS.md', 'register.mjs', 'REQUESTS.md', 'REVIEW.md', 'scenarios-r8.mjs',
    'build-phases.mjs', 'phases/cr17.md', 'phases/cr18.md', 'phases/cr19.md', 'phases/cr20.md', 'launches.json'],
  // Amendment 7: what depends on rc.17, the fuzz mode and every later agent's
  // inputs, before any author starts. packet/manifest.json exists only once
  // rc.17's verified publication record is on main (prepare.mjs manifest).
  4: ['PROTOCOL.md', 'register.mjs', 'lib.mjs', 'exec.mjs', 'fuzz.mjs', 'prepare.mjs', 'packet/manifest.json',
    'prompts/author-caveat.md', 'prompts/author-ts.md', 'prompts/mutator.md', 'prompts/drift.md',
    'workspace/README-author.md', 'workspace/run.mjs', 'workspace/README-mutator.md', 'workspace/README-drift.md',
    '../compare-views.mjs'],
}[stage];
if (!files) throw new Error(`stage ${stage} has no file list yet; add it when its files exist`);
const out = path.join(here, stage === '1' ? 'registration.json' : `registration-${stage}.json`);
if (existsSync(out)) throw new Error(`${path.basename(out)} exists; registration is never overwritten`);
const hashes = Object.fromEntries(files.map((f) => [f, createHash('sha256').update(readFileSync(path.join(here, f))).digest('hex')]));
writeFileSync(out, `${JSON.stringify({ stage: Number(stage), at: new Date().toISOString(), node: process.versions.node, files: hashes }, null, 2)}\n`);
console.log(`${path.basename(out)}: ${files.length} files`);
