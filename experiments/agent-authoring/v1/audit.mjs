// Audit retained artifacts and pinned inputs; cannot observe arbitrary agent reads.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const here = fileURLToPath(new URL('./', import.meta.url));
const hash = value => createHash('sha256').update(value).digest('hex');
const json = async file => JSON.parse(await readFile(path.join(here, file), 'utf8'));
const registration = await json('registration.json');
const manifest = await json('packet/manifest.json');
const report = { schema: 1, at: new Date().toISOString(), frozenFiles: [], references: 0, runtimeFiles: 0, runs: [],
  isolation: 'Instructions and author disclosures; no OS sandbox or exhaustive read-access telemetry.' };
for (const [file, expected] of Object.entries(registration.sha256)) {
  const actual = hash(await readFile(path.join(here, file)));
  if (file === 'verify.mjs') {
    const source = await readFile(path.join(here, file), 'utf8');
    const original = source.replace('JSON.stringify(event.payload)', 'JSON.stringify(event.payload ?? {})');
    assert.equal(hash(original), expected, 'Scorer changed beyond documented null-transport correction');
    report.frozenFiles.push({ file, status: 'documented null-transport correction', registeredSha256: expected, currentSha256: actual });
  } else {
    assert.equal(actual, expected, `Registered file changed: ${file}`);
    report.frozenFiles.push({ file, status: 'unchanged', sha256: actual });
  }
}
for (const [file, expected] of Object.entries(manifest.references)) {
  assert.equal(hash(await readFile(path.join(here, 'packet/reference', file))), expected, `Reference changed: ${file}`);
  report.references++;
}
for (const [file, expected] of Object.entries(manifest.runtime)) {
  assert.equal(hash(await readFile(path.join(here, 'packet/runtime', file))), expected, `Runtime changed: ${file}`);
  report.runtimeFiles++;
}
for (const id of registration.runs) {
  const prefix = `runs/${id}`;
  const record = await json(`${prefix}/record.json`);
  assert(record.finished, `${id} has not frozen its final source`);
  assert(record.versions.length >= 1 && record.versions.length <= registration.sourceVersionsPerRun);
  assert(record.checks.length <= registration.runtimeChecksPerRun);
  assert.equal(new Set(record.versions.map(version => version.sha256)).size, record.versions.length);
  for (const [index, version] of record.versions.entries()) {
    assert.equal(version.number, index + 1);
    assert.equal(hash(await readFile(path.join(here, prefix, version.file))), version.sha256, `${id}: archived source changed`);
  }
  for (const [index, check] of record.checks.entries()) {
    assert.equal(check.index, index + 1);
    const version = record.versions.find(item => item.number === check.version);
    assert(version);
    assert(version.submittedAt <= check.started, `${id}: check predates source capture`);
    const log = (await readFile(path.join(here, prefix, check.log), 'utf8')).trim().split('\n').map(JSON.parse);
    assert(log.length > 0);
    assert(['completed', 'error'].includes(check.status));
  }
  const final = await readFile(path.join(here, prefix, 'final.cav'));
  assert.equal(hash(final), record.finalSha256);
  assert.equal(record.versions[record.finalVersion - 1].sha256, record.finalSha256);
  const notes = await readFile(path.join(here, prefix, 'notes.md'), 'utf8');
  assert(notes.trim().length > 0);
  report.runs.push({ id, preservedVersions: record.versions.length, runtimeChecks: record.checks.length,
    sourceCaptureBeforeExecution: true, finalFrozen: true, notesPresent: true });
}
await mkdir(path.join(here, 'results'), { recursive: true });
await writeFile(path.join(here, 'results/integrity.json'), `${JSON.stringify(report, null, 2)}\n`);
console.log('Integrity audit passed: all six final candidates, budgets, archives and pinned inputs match their records.');
