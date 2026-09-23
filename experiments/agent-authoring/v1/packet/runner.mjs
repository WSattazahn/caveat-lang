// Public author tool. Snapshots every new source version before any execution.
import assert from 'node:assert/strict';
import { readFile, writeFile, appendFile, mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import path from 'node:path';
import init, { WebReactiveSession } from './runtime/caveat_runtime.js';

const [id, command, input] = process.argv.slice(2);
if (!(id === 'SMOKE' || /^[ABC][12]$/.test(id ?? '')) || !['validate', 'replay', 'finish'].includes(command)) {
  throw new Error('Usage: node packet/runner.mjs A1|A2|B1|B2|C1|C2 validate|replay|finish [events.jsonl]');
}
const runs = fileURLToPath(new URL('../runs/', import.meta.url));
const directory = path.join(runs, id);
await mkdir(path.join(directory, 'versions'), { recursive: true });
await mkdir(path.join(directory, 'checks'), { recursive: true });
const source = await readFile(path.join(directory, 'candidate.cav'), 'utf8');
const sha256 = createHash('sha256').update(source).digest('hex');
let record;
try { record = JSON.parse(await readFile(path.join(directory, 'record.json'), 'utf8')); }
catch (error) { if (error.code !== 'ENOENT') throw error; record = { id, started: new Date().toISOString(), versions: [], checks: [] }; }
if (record.finished) throw new Error('This submission is finished and frozen.');
let version = record.versions.find(item => item.sha256 === sha256);
if (!version) {
  if (record.versions.length >= 3) throw new Error('Three distinct submitted versions already used. Finish a preserved version.');
  const number = record.versions.length + 1;
  version = { number, sha256, submittedAt: new Date().toISOString(), file: `versions/${String(number).padStart(2, '0')}.cav` };
  await writeFile(path.join(directory, version.file), source, { flag: 'wx' });
  record.versions.push(version);
}
const persist = () => writeFile(path.join(directory, 'record.json'), `${JSON.stringify(record, null, 2)}\n`);
await persist();
if (command === 'finish') {
  await writeFile(path.join(directory, 'final.cav'), source, { flag: 'wx' });
  record.finished = new Date().toISOString(); record.finalVersion = version.number; record.finalSha256 = sha256;
  await persist();
  console.log(JSON.stringify({ id, status: 'frozen', version: version.number, sha256, runtimeChecks: record.checks.length }));
} else {
  if (record.checks.length >= 12) throw new Error('Twelve runtime checks already used. Finish your submission.');
  const index = record.checks.length + 1;
  const prefix = `checks/${String(index).padStart(2, '0')}`;
  const check = { index, command, version: version.number, started: new Date().toISOString(), log: `${prefix}.jsonl` };
  record.checks.push(check); await persist();
  const log = async value => {
    await appendFile(path.join(directory, check.log), `${JSON.stringify(value)}\n`);
    // Keep useful host output readable; complete snapshots remain in the log.
    const { snapshot, ...rest } = value;
    console.log(JSON.stringify(snapshot ? { ...rest, bindings: snapshot.bindings, commitment_grounds: snapshot.commitment_grounds,
      decision_journal: snapshot.decision_journal, relations: snapshot.relations } : rest));
  };
  let session;
  const start = performance.now();
  try {
    await init({ module_or_path: await readFile(new URL('./runtime/caveat_runtime_bg.wasm', import.meta.url)) });
    session = new WebReactiveSession(source);
    await log({ kind: 'validated', version: version.number, snapshot: JSON.parse(session.snapshot()) });
    if (command === 'replay') {
      const inputPath = path.resolve(directory, input ?? 'events.jsonl');
      if (!inputPath.startsWith(`${directory}${path.sep}`)) throw new Error('Replay input must be inside your own run directory.');
      const events = await readFile(inputPath, 'utf8');
      await writeFile(path.join(directory, `${prefix}.input.jsonl`), events, { flag: 'wx' });
      for (const [index, line] of events.split(/\r?\n/).entries()) {
        if (!line.trim()) continue;
        const event = JSON.parse(line);
        if (event.resume === true) {
          const restored = WebReactiveSession.restore(source, session.save());
          try { assert.deepEqual(JSON.parse(restored.snapshot()), JSON.parse(session.snapshot())); }
          catch (error) { restored.free(); throw error; }
          session.free(); session = restored;
          await log({ kind: 'resumed', line: index + 1, snapshot: JSON.parse(session.snapshot()) });
        } else {
          try {
            const snapshot = JSON.parse(session.dispatch(event.event, JSON.stringify(event.payload ?? {})));
            await log({ kind: 'accepted', line: index + 1, event, snapshot });
          } catch (error) {
            await log({ kind: 'rejected', line: index + 1, event, error: String(error), snapshot: JSON.parse(session.snapshot()) });
          }
        }
      }
    }
    check.status = 'completed';
  } catch (error) {
    check.status = 'error'; check.error = String(error);
    await log({ kind: 'error', message: String(error) });
    process.exitCode = 1;
  } finally {
    session?.free(); check.durationMs = Math.round(performance.now() - start); await persist();
  }
}
