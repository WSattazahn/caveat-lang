// Records what was fixed before any agent started: the pinned packages, every
// public study file, and the SHA-256 of every sealed file. Run once per stage,
// then commit the file it writes. Neither stage overwrites a file.
//
//   node register.mjs stage1 --sealed DIR --caveat TGZ --typescript TGZ
//   node register.mjs stage2 --model ID --effort LEVEL --runs-dir DIR
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const [stage, ...rest] = process.argv.slice(2);
const args = Object.fromEntries(rest.reduce((pairs, value, index, all) =>
  (index % 2 === 0 ? [...pairs, [value.replace(/^--/, ''), all[index + 1]]] : pairs), []));
const sha = async file => createHash('sha256').update(await readFile(file)).digest('hex');

export const PUBLIC = ['PROTOCOL.md', 'README.md', 'TASK.md', 'QUESTIONS.md', 'answers.schema.json', 'probes.json',
  'prompts/author.md', 'prompts/reviewer.md', 'run-cases.mjs', 'score.mjs', 'register.mjs',
  'histories/H1.jsonl', 'histories/H2.jsonl', 'histories/H3.jsonl', 'notes/H1.md', 'notes/H2.md', 'notes/H3.md',
  ...['C01-unknown-event', 'C02-out-of-range', 'C03-fraction', 'C04-missing-and-extra', 'C05-decide-nothing-recorded',
    'C06-decide-in-force', 'C07-review-opposing', 'C08-scan-opposing', 'C09-grant-other-head', 'C10-no-grant',
    'C11-fifth-review', 'C12-ninth-check', 'C13-fifth-scan', 'C14-fifth-grant', 'C15-recheck-twice',
    'C16-recheck-not-rested-on', 'C17-db-aged-twice', 'C18-refreshed-before-aged', 'C19-four-decisions',
    'C20-all-checks-withdrawn', 'C21-second-correction-while-reopened'].map(name => `cases/${name}.jsonl`)];
// Sealed until phase two is frozen: the model of the task, the expected view
// after every line of every case, the keys (with the notes' verdicts) and the
// investigator's reference programs.
export const SEALED = ['model.mjs', 'expected.json', 'keys.json', 'references/gate.cav', 'references/gate.ts'];

if (stage === 'stage1') {
  for (const key of ['sealed', 'caveat', 'typescript']) if (!args[key]) throw new Error(`--${key} is required`);
  const registration = {
    schema: 1,
    stage: 1,
    registeredAt: new Date().toISOString(),
    pins: {
      caveat: { tarball: path.basename(args.caveat), sha256: await sha(args.caveat) },
      typescript: { tarball: path.basename(args.typescript), sha256: await sha(args.typescript) },
      node: process.version,
    },
    public: Object.fromEntries(await Promise.all(PUBLIC.map(async file => [file, await sha(path.join(here, file))]))),
    sealed: Object.fromEntries(await Promise.all(SEALED.map(async file => [file, await sha(path.join(args.sealed, file))]))),
    runs: {
      'A-C': { phase: 1, arm: 'caveat', program: 'gate.cav' },
      'A-T': { phase: 1, arm: 'ts', program: 'gate.ts' },
      ...Object.fromEntries(['R-C1', 'R-C2', 'R-C3'].map(id => [id, { phase: 2, arm: 'caveat', reviews: 'A-C' }])),
      ...Object.fromEntries(['R-T1', 'R-T2', 'R-T3'].map(id => [id, { phase: 2, arm: 'ts', reviews: 'A-T' }])),
    },
  };
  await writeFile(path.join(here, 'registration.json'), `${JSON.stringify(registration, null, 2)}\n`, { flag: 'wx' });
  console.log(JSON.stringify(registration, null, 2));
} else if (stage === 'stage2') {
  for (const key of ['model', 'effort', 'runs-dir']) if (!args[key]) throw new Error(`--${key} is required`);
  const prompts = {};
  for (const id of ['A-C', 'A-T', 'R-C1', 'R-C2', 'R-C3', 'R-T1', 'R-T2', 'R-T3']) {
    prompts[id] = await sha(path.join(args['runs-dir'], id, 'PROMPT.md'));
  }
  const registration = { schema: 1, stage: 2, registeredAt: new Date().toISOString(), model: args.model, effort: args.effort, prompts };
  await writeFile(path.join(here, 'registration-2.json'), `${JSON.stringify(registration, null, 2)}\n`, { flag: 'wx' });
  console.log(JSON.stringify(registration, null, 2));
} else {
  console.error('usage: node register.mjs stage1 … | stage2 …');
  process.exitCode = 2;
}
