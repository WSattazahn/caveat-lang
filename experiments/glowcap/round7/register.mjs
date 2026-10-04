// Records the hashes of round 7's registered files. Stage 1 is the protocol
// and everything agents receive; stage 2 adds CR13-CR16, their scenarios and
// the round-7 fuzz mode, before any author starts. Stage 3 registers the
// wave-2 prompts and amendments 2-3. No stage overwrites.
//
//   node experiments/glowcap/round7/register.mjs --stage=1|2|3
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const stage = process.argv.find((a) => a.startsWith('--stage='))?.split('=')[1];
const list = (dir) => readdirSync(path.join(here, dir)).sort().map((f) => `${dir}/${f}`);
const NEW_PHASES = ['cr13', 'cr14', 'cr15', 'cr16'].map((p) => `phases/${p}.md`);

const files = {
  1: ['PROTOCOL.md', 'BEAT.md', 'build-phases.mjs', 'exec.mjs', 'lib.mjs', 'prepare.mjs', 'register.mjs',
    'packet/manifest.json', ...list('phases').filter((f) => !NEW_PHASES.includes(f)), ...list('prompts'), ...list('workspace'),
    '../scenarios.mjs', '../compare-views.mjs', '../harness.mjs'],
  3: ['PROTOCOL.md', 'prompts/author-caveat-wave2.md', 'prompts/author-ts-wave2.md', 'prepare.mjs', 'launches.json', 'register.mjs'],
  2: ['REQUESTS.md', 'REVIEW.md', 'scenarios-r7.mjs', 'fuzz.mjs', 'PREDICTIONS.md', ...NEW_PHASES,
    'PROTOCOL.md', 'BEAT.md', 'build-phases.mjs', 'phases/cr10.md', 'register.mjs'],
}[stage];
if (!files) throw new Error('usage: register.mjs --stage=1|2|3');
const out = path.join(here, stage === '1' ? 'registration.json' : `registration-${stage}.json`);
if (existsSync(out)) throw new Error(`${path.basename(out)} exists; registration is never overwritten`);
const hashes = Object.fromEntries(files.map((f) => [f, createHash('sha256').update(readFileSync(path.join(here, f))).digest('hex')]));
writeFileSync(out, `${JSON.stringify({ stage: Number(stage), at: new Date().toISOString(), node: process.versions.node, files: hashes }, null, 2)}\n`);
console.log(`${path.basename(out)}: ${files.length} files`);
