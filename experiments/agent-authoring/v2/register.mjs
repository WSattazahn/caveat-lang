// Execute once after tests/smoke review and packet preparation, before author dispatch.
import assert from 'node:assert/strict';
import { readFile, writeFile, access } from 'node:fs/promises';
import path from 'node:path';
import { here, hash, json } from './integrity.mjs';
import { describeCorpus } from './corpus.mjs';

const study = await json('study.json');
const manifest = await json('packet/manifest.json');
assert.match(study.runtimeCommit, /^[0-9a-f]{40}$/);
assert.equal(manifest.sourceCommit, study.runtimeCommit);
assert.equal(manifest.build.revision, study.runtimeCommit);
assert.equal(manifest.build.clean, true);
assert.equal(manifest.build.compiled, true);
assert.deepEqual(study.runs, ['A1', 'B1', 'A2', 'B2']);
assert.deepEqual(study.tasks, ['A', 'B']);
assert.equal(study.sourceVersionsPerRun, 3);
assert.equal(study.runtimeChecksPerRun, 12);
assert.equal(study.randomSequencesPerTask, 100);
assert.equal(study.stepsPerRandomSequence, 40);
assert.deepEqual(study.seeds, { A: '0xa17001', B: '0xb17001' });
try { await access(path.join(here, 'registration.json')); throw new Error('Registration already exists; never overwrite a frozen registration'); }
catch (error) { if (error.code !== 'ENOENT') throw error; }
for (const id of study.runs) {
  try { await access(path.join(here, 'runs', id)); throw new Error(`Author directory exists before registration: ${id}`); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
}
const corpus = describeCorpus();
assert.equal(corpus.tasks.A.cases, 114);
assert.equal(corpus.tasks.B.cases, 116);
assert.equal(corpus.tasks.A.casesSha256, 'ae3a1be552d810c7231c087f14a6d9d714dd4ff6ff0e26a5e31a0786b81cce23');
assert.equal(corpus.tasks.B.casesSha256, '9acbce2b085b7d35689bd1bba28ab1468926c802f4c675b16884c9818e71b10a');
const unchanged = {
  'tasks/cold-storage.md': 'b511f465695b91976d64a3dd9ab61effbd0f3ce837faf0383070d3de8ff87b9c',
  'tasks/access-review.md': '7bc7da394257a31d1d3f66d329d8de82bec626222afbec8367739b37616414de',
  'private/oracles.mjs': '47212b223d27d79867fffc2696402a39f0d5aa7eedff3d557f5a80251fe9b5c8',
};
for (const [file, expected] of Object.entries(unchanged)) assert.equal(hash(await readFile(path.join(here, file))), expected, `v1 task/oracle changed: ${file}`);
for (const [file, expected] of Object.entries(manifest.references)) assert.equal(hash(await readFile(path.join(here, 'packet/reference', file))), expected);
for (const [file, expected] of Object.entries(manifest.runtime)) assert.equal(hash(await readFile(path.join(here, 'packet/runtime', file))), expected);
await writeFile(path.join(here, 'corpus.json'), `${JSON.stringify(corpus, null, 2)}\n`);
const files = ['.gitignore', 'runs/.gitattributes', 'PROTOCOL.md', 'README.md', 'study.json', 'prepare-packet.mjs',
  'packet-transform.mjs', 'packet/README.md', 'packet/runner.mjs', 'packet/transport.mjs', 'packet/manifest.json',
  'checks.mjs', 'checks.test.mjs', 'infra.test.mjs', 'verify.mjs', 'integrity.mjs', 'audit.mjs', 'register.mjs',
  'corpus.mjs', 'corpus.json', ...Object.keys(unchanged), ...study.runs.map(id => `prompts/${id}.md`)];
const sha256 = {};
for (const file of files) sha256[file] = hash(await readFile(path.join(here, file)));
const registration = { schema: 2, registeredAt: new Date().toISOString(), runtimeCommit: study.runtimeCommit,
  runs: study.runs, sourceVersionsPerRun: 3, runtimeChecksPerRun: 12,
  randomSequencesPerTask: 100, stepsPerRandomSequence: 40, seeds: study.seeds,
  cases: corpus.tasks, primaryIncludesSnapshotElapsedAndSequence: true, sha256,
  priorStudy: 'v1 (preserved unchanged); repeated A/B contracts and corpus; runtime and documentation changed together',
  packetBuild: manifest.build, authorIsolation: 'Fresh contexts and instructions; shared filesystem, not a security sandbox' };
await writeFile(path.join(here, 'registration.json'), `${JSON.stringify(registration, null, 2)}\n`, { flag: 'wx' });
console.log('Registered four runs. Commit every registered file before author dispatch; record that commit in launches.json.');
