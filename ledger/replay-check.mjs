#!/usr/bin/env node
// Every recorded event must replay. A failed command, missing output, refused
// event or incomplete history cannot support a successful ledger check.
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));

export function verifyReplay(result, expectedCount) {
  if (result.error) throw new Error(`replay could not run: ${result.error.message}`);
  if (result.signal) throw new Error(`replay stopped on ${result.signal}`);
  if (result.status !== 0 && result.status !== 1) {
    throw new Error(`replay exited with status ${result.status}: ${(result.stderr || '').trim()}`);
  }
  if (!Number.isInteger(expectedCount) || expectedCount < 1) {
    throw new Error('the project event record is empty');
  }
  const records = (result.stdout || '').trim().split('\n').filter(Boolean).map(line => JSON.parse(line));
  if (records[0]?.record !== 'initial') throw new Error('replay did not return its initial record');
  const events = records.slice(1);
  for (const record of events) {
    if (record.record !== 'event' && record.record !== 'fatal') {
      throw new Error('replay returned an unrecognized record');
    }
    if (record.record === 'fatal' || record.outcome !== 'accepted') {
      throw new Error(`line ${record.line}: ${record.event} ${JSON.stringify(record.payload ?? {})} was ${record.record === 'fatal' ? 'fatal' : `${record.outcome} (${record.origin}/${record.code}: ${record.message})`}`);
    }
  }
  if (result.status !== 0) throw new Error(`replay exited with status ${result.status}`);
  if (events.length !== expectedCount) {
    throw new Error(`replay returned ${events.length} of ${expectedCount} recorded events`);
  }
  return events;
}

function main() {
  const cli = path.join(here, '..', 'kit', 'bin', 'caveat.mjs');
  const eventsFile = path.join(here, 'events.jsonl');
  const runtime = process.argv.includes('--runtime') ? ['--runtime', process.argv[process.argv.indexOf('--runtime') + 1]] : [];
  const expectedCount = readFileSync(eventsFile, 'utf8').split('\n').filter(line => line.trim()).length;
  const result = spawnSync(process.execPath, [cli, 'replay', ...runtime, path.join(here, 'project.cav'), eventsFile], { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 });
  const events = verifyReplay(result, expectedCount);
  process.stdout.write(`${events.length} recorded events replay.\n`);
  for (const [name, value] of Object.entries(events.at(-1)?.snapshot?.bindings?.ledger ?? {})) {
    process.stdout.write(`  ledger.${name} = ${JSON.stringify(value)}\n`);
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { main(); }
  catch (error) {
    process.stderr.write(`Ledger replay failed: ${error.message}\n`);
    process.exitCode = 1;
  }
}
