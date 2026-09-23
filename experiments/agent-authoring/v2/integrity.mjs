import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { describeCorpus } from './corpus.mjs';

export const here = fileURLToPath(new URL('./', import.meta.url));
export const hash = value => createHash('sha256').update(value).digest('hex');
export const json = async file => JSON.parse(await readFile(path.join(here, file), 'utf8'));

export async function assertRegisteredInputs() {
  const registration = await json('registration.json');
  const manifest = await json('packet/manifest.json');
  const study = await json('study.json');
  assert.equal(registration.runtimeCommit, study.runtimeCommit);
  assert.equal(manifest.sourceCommit, study.runtimeCommit);
  assert.equal(manifest.build.revision, study.runtimeCommit);
  assert.equal(manifest.build.compiled, true);
  assert.equal(manifest.build.clean, true);
  assert.deepEqual(registration.runs, study.runs);
  const files = [];
  for (const [file, expected] of Object.entries(registration.sha256)) {
    assert.equal(hash(await readFile(path.join(here, file))), expected, `Registered file changed: ${file}`);
    files.push({ file, sha256: expected });
  }
  for (const [file, expected] of Object.entries(manifest.references)) {
    assert.equal(hash(await readFile(path.join(here, 'packet/reference', file))), expected, `Reference changed: ${file}`);
  }
  for (const [file, expected] of Object.entries(manifest.runtime)) {
    assert.equal(hash(await readFile(path.join(here, 'packet/runtime', file))), expected, `Runtime changed: ${file}`);
  }
  assert.deepEqual(await json('corpus.json'), describeCorpus(), 'Registered corpus changed');
  return { registration, manifest, files };
}

export async function assertFinishedRuns(registration) {
  const records = {};
  const summaries = [];
  for (const id of registration.runs) {
    const prefix = `runs/${id}`;
    const record = await json(`${prefix}/record.json`);
    assert.equal(record.id, id);
    assert(record.finished, `Do not score before all authors finish: ${id}`);
    assert(record.versions.length >= 1 && record.versions.length <= registration.sourceVersionsPerRun);
    assert(record.checks.length <= registration.runtimeChecksPerRun);
    assert.equal(new Set(record.versions.map(version => version.sha256)).size, record.versions.length);
    for (const [index, version] of record.versions.entries()) {
      assert.equal(version.number, index + 1);
      assert.equal(version.file, `versions/${String(index + 1).padStart(2, '0')}.cav`);
      assert.equal(hash(await readFile(path.join(here, prefix, version.file))), version.sha256, `${id}: archived source changed`);
    }
    for (const [index, check] of record.checks.entries()) {
      assert.equal(check.index, index + 1);
      assert(['validate', 'replay'].includes(check.command));
      assert.equal(check.log, `checks/${String(index + 1).padStart(2, '0')}.jsonl`);
      const version = record.versions.find(item => item.number === check.version);
      assert(version);
      assert(version.submittedAt <= check.started, `${id}: check predates source capture`);
      assert(check.started <= record.finished, `${id}: check follows final freeze`);
      const log = (await readFile(path.join(here, prefix, check.log), 'utf8')).trim().split('\n').map(JSON.parse);
      assert(log.length > 0);
      assert(['completed', 'error'].includes(check.status));
      if (check.command === 'replay' && log.some(entry => entry.kind === 'accepted' || entry.kind === 'rejected' || entry.kind === 'resumed')) {
        await readFile(path.join(here, prefix, check.log.replace('.jsonl', '.input.jsonl')));
      }
    }
    assert.equal(hash(await readFile(path.join(here, prefix, 'final.cav'))), record.finalSha256);
    assert.equal(record.versions[record.finalVersion - 1].sha256, record.finalSha256);
    assert((await readFile(path.join(here, prefix, 'notes.md'), 'utf8')).trim().length > 0);
    records[id] = record;
    summaries.push({ id, preservedVersions: record.versions.length, runtimeChecks: record.checks.length,
      sourceCaptureBeforeExecution: true, finalFrozen: true, notesPresent: true });
  }
  return { records, summaries };
}
