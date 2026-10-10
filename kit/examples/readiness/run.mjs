// The engineering-readiness pilot, run end to end with the integration
// starter: an approval tied to the test result for one exact revision, a
// qualification that the authored policy turns into a reopening, an unrelated
// decision left alone, and the original grounds read back after a restart and
// from the host archive. Every input is illustrative.
//
//   node run.mjs [empty-directory] [--json]
//
// Without a directory it uses a new temporary one and leaves it in place.
import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { mkdtemp, readdir, readFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { explain } from 'caveat-lang/explain';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
import { openHost } from 'caveat-lang/starter';
import { fileStore } from 'caveat-lang/starter/node';

const here = path.dirname(fileURLToPath(import.meta.url));

// A file store whose next checkpoint write can be made to fail, standing in
// for a full disk or a lost volume.
function faultyStore(store) {
  let failNext = false;
  return {
    durable: store.durable,
    failNextCheckpoint() { failNext = true; },
    readCheckpoint: () => store.readCheckpoint(),
    async writeCheckpoint(text) {
      if (failNext) {
        failNext = false;
        throw new Error('simulated storage failure: the checkpoint was not written');
      }
      return store.writeCheckpoint(text);
    },
    readChunks: () => store.readChunks(),
    appendChunks: chunks => store.appendChunks(chunks),
    close: () => store.close?.(),
  };
}

async function send(host, event, payload) {
  const sent = await host.send(event, payload);
  if (!sent.handled) throw new Error(`${event} was not handled: ${sent.error.kind}: ${sent.error.message}`);
  return sent;
}

async function accept(host, event, payload) {
  const sent = await send(host, event, payload);
  if (!sent.accepted) throw new Error(`${event} was refused: ${sent.rejection.message}`);
  if (!sent.durable) throw new Error(`${event} was accepted but not stored: ${sent.storeError}`);
  return sent;
}

function grounds(assessment) {
  return { revision: assessment.revision ?? null, status: assessment.status, change: assessment.change?.change ?? null,
    because: assessment.change?.because ?? [], caveats: assessment.change?.caveats ?? [] };
}

/**
 * Runs the pilot against `directory`, which must be empty or absent. Returns
 * the facts it showed; `say` receives one line per step.
 */
export async function runPilot({ runtime, source, directory, say = () => {} }) {
  if (existsSync(directory) && (await readdir(directory)).length > 0) throw new Error(`${directory} is not empty`);
  const report = { directory, steps: [] };
  const step = (name, facts) => { report.steps.push({ step: name, ...facts }); };

  // 1. Approve revision A on test result T.
  let host = await openHost({ runtime, source, store: await fileStore(directory) });
  const identity = { source_sha256: host.sourceSha256, runtime: runtime.identity };
  say(`Caveat source sha256 ${host.sourceSha256}`);
  say(`Runtime ${runtime.identity.revision ?? 'revision not recorded'}, wasm sha256 ${runtime.identity.reactiveWasmSha256 ?? 'not recorded'}`);
  await accept(host, 'propose', { revision: 'rev-a1b2c3' });
  await accept(host, 'test_reported', { revision: 'rev-a1b2c3', passed: 1 });
  await accept(host, 'approve_release');
  const approved = host.assessment('release');
  assert.equal(approved.status, 'in_force');
  step('approve', { application_revision: 'rev-a1b2c3', release: grounds(approved), permits: host.permits('release') });
  say(`1. Release of rev-a1b2c3 approved as ${approved.revision}, because ${approved.change.because.join(', ')}.`);

  // 2. An unrelated decision on evidence U.
  await accept(host, 'audit_reported', { findings: 0 });
  await accept(host, 'sign_off_dependencies');
  const signedOff = host.assessment('dependency_signoff');
  step('unrelated', { dependency_signoff: grounds(signedOff) });
  say(`2. Dependencies signed off as ${signedOff.revision}, because ${signedOff.change.because.join(', ')}.`);

  // A restart: the next process resumes from the directory.
  const before = host.sequence;
  await host.close();
  const store = faultyStore(await fileStore(directory));
  host = await openHost({ runtime, source, store });
  assert.equal(host.sequence, before);
  assert.equal(host.permits('release'), true);
  step('restart', { sequence: host.sequence, permits: host.permits('release') });
  say(`   Restarted at sequence ${host.sequence}; the release approval is still in force.`);

  // 3 and 4. T ran under the wrong configuration. The program qualifies it,
  // and its authored policy reopens the approval that rests on it.
  await accept(host, 'test_configuration_wrong');
  const reopened = host.assessment('release');
  assert.equal(reopened.status, 'reopened');
  assert.deepEqual(reopened.change.caveats, ['wrong_configuration']);
  step('reopen', { release: grounds(reopened), permits: host.permits('release') });
  say(`3. ${reopened.revision} reopened because ${reopened.change.because.join(', ')} carries ${reopened.change.caveats.join(', ')}.`);

  // 5. The unrelated decision is unchanged.
  const still = host.assessment('dependency_signoff');
  assert.deepEqual(grounds(still), grounds(signedOff));
  step('unchanged', { dependency_signoff: grounds(still) });
  say(`4. ${still.revision} is still in force, on ${still.change.because.join(', ')}; no policy connects it to the test result.`);

  // A refused event changes nothing.
  const atRefusal = host.sequence;
  const refused = await send(host, 'approve_release');
  assert.equal(refused.accepted, false);
  assert.equal(host.sequence, atRefusal);
  step('refusal', { origin: refused.rejection.origin, code: refused.rejection.code, message: refused.rejection.message, sequence: host.sequence });
  say(`   Re-approving on the same test result is refused (${refused.rejection.origin}/${refused.rejection.code}): ${refused.rejection.message}`);

  // A storage failure: the event is accepted but not durable until flushed.
  store.failNextCheckpoint();
  const unstored = await send(host, 'propose', { revision: 'rev-d4e5f6' });
  assert.equal(unstored.accepted, true);
  assert.equal(unstored.durable, false);
  const flushed = await host.flush();
  assert.equal(flushed.durable, true);
  step('storage_failure', { accepted: unstored.accepted, durable: unstored.durable, flushed: flushed.durable });
  say('   A failed checkpoint write left rev-d4e5f6 accepted but not durable; flush() stored it.');

  // A corrected configuration: ten more runs for rev-d4e5f6. Runs beyond the
  // window of eight leave the session for the host archive.
  for (let run = 0; run < 9; run += 1) await accept(host, 'test_reported', { revision: 'rev-d4e5f6', passed: 0 });
  const last = await accept(host, 'test_reported', { revision: 'rev-d4e5f6', passed: 1 });
  await accept(host, 'approve_release');
  const renewed = host.assessment('release');
  assert.equal(renewed.status, 'in_force');
  step('new_approval', { application_revision: 'rev-d4e5f6', release: grounds(renewed), sequence: last.sequence });
  say(`   rev-d4e5f6 approved as ${renewed.revision}, because ${renewed.change.because.join(', ')}.`);

  // 6. After another restart, the original grounds and the source identity,
  // from the current snapshot and the archive the host kept.
  await host.close();
  host = await openHost({ runtime, source, store: await fileStore(directory) });
  const archive = await host.archive();
  const explained = explain(host.snapshot(), [], { archive });
  const series = explained.decisions.find(decision => decision.name === 'release');
  const first = series.revisions.find(revision => revision.id === 'release@1');
  assert.equal(first.status, 'superseded');
  assert.deepEqual(first.grounds.evidence, approved.change.because);
  // Runs that left the session are in the archive. test_result itself left
  // its window but stays, because release@1 rests on it.
  const departed = archive.filter(entry => !('kind' in entry)).map(entry => entry.record);
  assert.ok(departed.length > 0);
  assert.ok(!departed.includes('test_result'));
  const history = first.history.map(change => ({ change: change.change, because: change.because, caveats: change.caveats }));
  step('history', { source_sha256: host.sourceSha256, archive_entries: archive.length, release_1: { status: first.status, grounds: first.grounds.evidence, history }, departed });
  assert.deepEqual(history.map(change => change.change), ['committed', 'reopened']);
  say(`5. After a restart, release@1 is ${first.status} by release@2. It was committed on ${first.grounds.evidence.join(', ')} and its history reads ${history.map(change => `${change.change}${change.caveats.length ? ` (${change.caveats.join(', ')})` : ''}`).join(', then ')}.`);
  say(`   Runs ${departed.join(', ')} left the session; the store keeps them in its archive. The session still runs source ${host.sourceSha256}.`);
  await host.close();

  report.identity = identity;
  return report;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const args = process.argv.slice(2);
  const json = args.includes('--json');
  const directory = args.find(arg => arg !== '--json') ?? await mkdtemp(path.join(os.tmpdir(), 'caveat-readiness-'));
  const runtime = await loadRuntimeFromDirectory();
  const source = await readFile(path.join(here, 'readiness.cav'), 'utf8');
  const report = await runPilot({ runtime, source, directory, say: json ? undefined : line => console.log(line) });
  if (json) console.log(JSON.stringify(report, null, 2));
  else console.log(`The store is in ${directory}.`);
}
