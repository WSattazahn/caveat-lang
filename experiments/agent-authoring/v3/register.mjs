// Records what was fixed before any author started: the release candidate,
// every public study file, and the sealed change request with its model,
// cases and the reference programs. Run once, then commit registration.json.
//
//   node register.mjs --tarball PATH --tag TAG --commit SHA --sealed DIR
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) =>
  (index % 2 === 0 ? [...pairs, [value.replace(/^--/, ''), all[index + 1]]] : pairs), []));
for (const key of ['tarball', 'tag', 'commit', 'sealed']) if (!args[key]) throw new Error(`--${key} is required`);

const sha = async file => createHash('sha256').update(await readFile(file)).digest('hex');
export const PUBLIC = ['PROTOCOL.md', 'tasks/pond.md', 'prompts/author.md', 'prompts/maintainer.md',
  'private/oracle.mjs', 'private/corpus.mjs', 'private/score-lib.mjs', 'score.mjs', 'register.mjs'];
export const SEALED = ['tasks/pond-change.md', 'private/change-oracle.mjs', 'references/phase1.cav', 'references/phase2.cav'];
const sealedSources = { 'tasks/pond-change.md': 'CHANGE.md', 'private/change-oracle.mjs': 'change-oracle.mjs',
  'references/phase1.cav': 'reference-phase1.cav', 'references/phase2.cav': 'reference-phase2.cav' };

const registration = {
  schema: 1,
  registeredAt: new Date().toISOString(),
  kit: { tarball: path.basename(args.tarball), sha256: await sha(args.tarball), tag: args.tag, commit: args.commit },
  public: Object.fromEntries(await Promise.all(PUBLIC.map(async file => [file, await sha(path.join(here, file))]))),
  sealed: Object.fromEntries(await Promise.all(SEALED.map(async file => [file, await sha(path.join(args.sealed, sealedSources[file]))]))),
  runs: { A: { phase: 1 }, B: { phase: 1 }, A2: { phase: 2, maintains: 'A' }, B2: { phase: 2, maintains: 'B' } },
};
await writeFile(path.join(here, 'registration.json'), `${JSON.stringify(registration, null, 2)}\n`, { flag: 'wx' });
console.log(JSON.stringify(registration, null, 2));
