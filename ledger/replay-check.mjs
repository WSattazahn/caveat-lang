#!/usr/bin/env node
// The record must replay: every event in events.jsonl is accepted by
// project.cav, in order. A refused or fatal event means the record and the
// program disagree, and the ledger cannot be trusted until they agree again.
// Prints what the ledger shows at the end.
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const cli = path.join(here, '..', 'kit', 'bin', 'caveat.mjs');
const runtime = process.argv.includes('--runtime') ? ['--runtime', process.argv[process.argv.indexOf('--runtime') + 1]] : [];
const result = spawnSync(process.execPath, [cli, 'replay', ...runtime, path.join(here, 'project.cav'), path.join(here, 'events.jsonl')], { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 });
if (result.status !== 0 && result.status !== 1) {
  process.stderr.write(result.stderr || result.stdout);
  process.exit(2);
}
const records = result.stdout.trim().split('\n').filter(Boolean).map(line => JSON.parse(line));
const events = records.filter(record => record.record === 'event' || record.record === 'fatal');
const bad = events.filter(record => record.record === 'fatal' || record.outcome !== 'accepted');
for (const record of bad) {
  process.stderr.write(`line ${record.line}: ${record.event} ${JSON.stringify(record.payload ?? {})} was ${record.record === 'fatal' ? 'fatal' : `${record.outcome} (${record.origin}/${record.code}: ${record.message})`}\n`);
}
if (bad.length) {
  process.stderr.write(`${bad.length} of ${events.length} recorded events do not replay; the record and ledger/project.cav disagree.\n`);
  process.exit(1);
}
const last = events.at(-1);
const bindings = last?.snapshot?.bindings?.ledger ?? {};
process.stdout.write(`${events.length} recorded events replay.\n`);
for (const [name, value] of Object.entries(bindings)) {
  process.stdout.write(`  ledger.${name} = ${JSON.stringify(value)}\n`);
}
