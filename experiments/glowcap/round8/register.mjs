// Records the hashes of round 8's registered files (PROTOCOL.md,
// "Registration"). Stage 1 is the approved protocol, the predictions, the
// probe and the inherited round 7 files; stage 2 the writer's and reviewer's
// inputs; stage 3 what depends on rc.17; stage 4 the blind requests before
// any author starts. No stage
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
}[stage];
if (!files) throw new Error(`stage ${stage} has no file list yet; add it when its files exist`);
const out = path.join(here, stage === '1' ? 'registration.json' : `registration-${stage}.json`);
if (existsSync(out)) throw new Error(`${path.basename(out)} exists; registration is never overwritten`);
const hashes = Object.fromEntries(files.map((f) => [f, createHash('sha256').update(readFileSync(path.join(here, f))).digest('hex')]));
writeFileSync(out, `${JSON.stringify({ stage: Number(stage), at: new Date().toISOString(), node: process.versions.node, files: hashes }, null, 2)}\n`);
console.log(`${path.basename(out)}: ${files.length} files`);
