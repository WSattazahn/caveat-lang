// Records what was fixed before any agent started: the release candidate, every
// public study file, and the SHA-256 of every sealed file. Run once, then
// commit registration.json.
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
export const PUBLIC = ['PROTOCOL.md', 'README.md', 'tasks/ferry.md', 'prompts/author.md', 'prompts/maintainer.md',
  'private/score-lib.mjs', 'score.mjs', 'register.mjs'];
// Sealed files, by the phase after whose freeze each is committed, and the name
// each has in the sealed directory.
export const SEALED = {
  'private/oracle.mjs': [1, 'oracle.mjs'],
  'private/corpus.mjs': [1, 'corpus.mjs'],
  'tasks/ferry-change.md': [2, 'ferry-change.md'],
  'private/change-oracle.mjs': [2, 'change-oracle.mjs'],
  'references/phase1.cav': [2, 'reference-phase1.cav'],
  'references/phase2.cav': [2, 'reference-phase2.cav'],
};
const models = { A: 'claude-opus-5-5', B: 'claude-opus-5-5', C: 'claude-sonnet-5', D: 'claude-sonnet-5' };

const registration = {
  schema: 1,
  registeredAt: new Date().toISOString(),
  kit: { tarball: path.basename(args.tarball), sha256: await sha(args.tarball), tag: args.tag, commit: args.commit },
  public: Object.fromEntries(await Promise.all(PUBLIC.map(async file => [file, await sha(path.join(here, file))]))),
  sealed: Object.fromEntries(await Promise.all(Object.entries(SEALED).map(async ([file, [unsealAfter, name]]) =>
    [file, { unsealAfterPhase: unsealAfter, sha256: await sha(path.join(args.sealed, name)) }]))),
  runs: Object.fromEntries(Object.entries(models).flatMap(([id, model]) => [
    [id, { phase: 1, model }], [`${id}2`, { phase: 2, maintains: id, model }]])),
};
await writeFile(path.join(here, 'registration.json'), `${JSON.stringify(registration, null, 2)}\n`, { flag: 'wx' });
console.log(JSON.stringify(registration, null, 2));
