#!/usr/bin/env node
// Every recorded event must replay. A failed command, missing output, refused
// event or incomplete history cannot support a successful ledger check.
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { isDeepStrictEqual } from 'node:util';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseEvents } from '../kit/lib/explain.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));

const isObject = value => value !== null && typeof value === 'object' && !Array.isArray(value);

function verifySnapshot(record, sequence) {
  if (record.sequence !== sequence || !isObject(record.snapshot) || record.snapshot.sequence !== sequence) {
    throw new Error(`replay returned an invalid sequence or snapshot at sequence ${sequence}`);
  }
  const ledger = record.snapshot.bindings?.ledger;
  if (!isObject(record.snapshot.bindings) || !isObject(ledger) || Object.keys(ledger).length === 0) {
    throw new Error(`replay returned no ledger bindings at sequence ${sequence}`);
  }
}

export function verifyReplay(result, eventsText) {
  if (result.error) throw new Error(`replay could not run: ${result.error.message}`);
  if (result.signal) throw new Error(`replay stopped on ${result.signal}`);
  if (result.status !== 0 && result.status !== 1) {
    throw new Error(`replay exited with status ${result.status}: ${(result.stderr || '').trim()}`);
  }
  const expected = parseEvents(eventsText);
  if (!expected.length) throw new Error('the project event record is empty');
  // Use the CLI's parser and physical-line convention, including blank lines.
  const lines = eventsText.split(/\r?\n/).flatMap((line, index) => line.trim() ? [index + 1] : []);
  const records = (result.stdout || '').trim().split('\n').filter(Boolean).map(line => JSON.parse(line));
  if (records[0]?.record !== 'initial') throw new Error('replay did not return its initial record');
  verifySnapshot(records[0], 0);
  const events = records.slice(1);
  for (const record of events) {
    if (!isObject(record) || (record.record !== 'event' && record.record !== 'fatal')) {
      throw new Error('replay returned an unrecognized record');
    }
    if (record.record === 'fatal' || record.outcome !== 'accepted') {
      throw new Error(`line ${record.line}: ${record.event} ${JSON.stringify(record.payload ?? {})} was ${record.record === 'fatal' ? 'fatal' : `${record.outcome} (${record.origin}/${record.code}: ${record.message})`}`);
    }
  }
  if (result.status !== 0) throw new Error(`replay exited with status ${result.status}`);
  if (events.length !== expected.length) {
    throw new Error(`replay returned ${events.length} of ${expected.length} recorded events`);
  }
  events.forEach((record, index) => {
    const wanted = expected[index];
    if (record.line !== lines[index] || record.event !== wanted.event || !isDeepStrictEqual(record.payload, wanted.payload)) {
      throw new Error(`replay event ${index + 1} does not match recorded line ${lines[index]} (${wanted.event})`);
    }
    verifySnapshot(record, index + 1);
  });
  return events;
}

function main() {
  const cli = path.join(here, '..', 'kit', 'bin', 'caveat.mjs');
  const eventsFile = path.join(here, 'events.jsonl');
  const runtime = process.argv.includes('--runtime') ? ['--runtime', process.argv[process.argv.indexOf('--runtime') + 1]] : [];
  const eventsText = readFileSync(eventsFile, 'utf8');
  const result = spawnSync(process.execPath, [cli, 'replay', ...runtime, path.join(here, 'project.cav'), eventsFile], { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 });
  const events = verifyReplay(result, eventsText);
  process.stdout.write(`${events.length} recorded events replay.\n`);
  for (const [name, value] of Object.entries(events.at(-1).snapshot.bindings.ledger)) {
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
