// The stabilization trial (Path to 1.0, E2; rc.19 plan, S3): an installed
// caveat-lang package drives trial.cav through the integration starter with a
// file store. It covers decisions, a correction placed and taken back, a
// refusal from each rejection origin, a restart, archive drain, a refused
// restore under edited source, and the host's own resource budgets.
//
//   node experiments/stabilization-trial/run.mjs --package=DIR [--directory=EMPTY] [--json]
//
// DIR is an installed caveat-lang (for example a consumer's
// node_modules/caveat-lang). The budgets below are this trial's host budgets,
// as COMPATIBILITY.md's "Limits" asks a host to set; they are not promises of
// the package. Every input is illustrative.
import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { mkdtemp, readdir, readFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { performance } from 'node:perf_hooks';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));

export const BUDGETS = Object.freeze({
  // One send: dispatch, view, checkpoint write and archive append, synced.
  sendMs: 250,
  checkpointBytes: 64 * 1024,
  archiveBytes: 256 * 1024,
});

export async function runTrial({ packageDir, directory }) {
  const load = file => import(pathToFileURL(path.resolve(packageDir, 'lib', file)).href);
  const { loadRuntimeFromDirectory } = await load('node.mjs');
  const { openHost, StarterError } = await load('starter.mjs');
  const { fileStore } = await load('starter-node.mjs');
  const { explain } = await load('explain.mjs');
  const manifest = JSON.parse(await readFile(path.join(packageDir, 'package.json'), 'utf8'));
  if (existsSync(directory) && (await readdir(directory)).length > 0) throw new Error(`${directory} is not empty`);

  const runtime = await loadRuntimeFromDirectory();
  const source = await readFile(path.join(here, 'trial.cav'), 'utf8');
  const report = { package: `${manifest.name}@${manifest.version}`, budgets: BUDGETS, steps: [], sends: 0, maxSendMs: 0 };
  const step = (name, facts) => { report.steps.push({ step: name, ...facts }); };

  let host = await openHost({ runtime, source, store: await fileStore(directory) });
  report.source_sha256 = host.sourceSha256;
  const send = async (event, payload) => {
    const started = performance.now();
    const sent = await host.send(event, payload);
    const ms = performance.now() - started;
    report.sends += 1;
    report.maxSendMs = Math.max(report.maxSendMs, ms);
    return sent;
  };
  const accept = async (event, payload) => {
    const sent = await send(event, payload);
    assert.equal(sent.handled, true, `${event} handled`);
    assert.equal(sent.accepted, true, `${event} accepted: ${sent.rejection?.message}`);
    assert.equal(sent.durable, true, `${event} durable`);
    return sent;
  };
  // A refusal changes nothing: the sequence and the view stay as they were.
  const refuse = async (name, event, payload, origin, code) => {
    const before = { sequence: host.sequence, view: JSON.stringify(host.view()) };
    const sent = await send(event, payload);
    assert.equal(sent.handled, true, `${name}: handled`);
    assert.equal(sent.accepted, false, `${name}: refused`);
    assert.deepEqual([sent.rejection.origin, sent.rejection.code], [origin, code], `${name}: ${sent.rejection.message}`);
    assert.equal(host.sequence, before.sequence, `${name}: sequence unchanged`);
    assert.equal(JSON.stringify(host.view()), before.view, `${name}: view unchanged`);
    step(name, { origin, code, sequence: host.sequence });
  };
  const assessed = () => {
    const assessment = host.assessment('deploy');
    return { revision: assessment.revision ?? null, status: assessment.status,
      because: assessment.change?.because ?? [], caveats: assessment.change?.caveats ?? [], permits: host.permits('deploy') };
  };

  // Refusals from each origin before any decision.
  await refuse('policy', 'deploy', {}, 'policy', 'reject');
  await refuse('input_unknown_event', 'launch', {}, 'input', 'unknown_event');
  await refuse('input_payload_invalid', 'ci_reported', { passed: 1, extra: 1 }, 'input', 'payload_invalid');
  await refuse('input_bound_exceeded', 'ci_reported', { passed: 2 }, 'input', 'bound_exceeded');

  // A decision on a passing run.
  await accept('propose', { revision: 'build-1' });
  await accept('ci_reported', { passed: 1 });
  await accept('deploy');
  const deployed = assessed();
  assert.deepEqual([deployed.revision, deployed.status, deployed.permits], ['deploy@1', 'in_force', true]);
  step('decision', { deploy: deployed });

  await refuse('evaluation_decision_in_force', 'deploy', {}, 'evaluation', 'decision_in_force');
  await accept('retry');
  await accept('retry');
  await refuse('evaluation_bound_exceeded', 'retry', {}, 'evaluation', 'bound_exceeded');
  await accept('propose', { revision: 'build-2' });
  await refuse('limit_identifier_limit', 'propose', { revision: 'build-3' }, 'limit', 'identifier_limit');

  // A restart resumes from the store.
  const atClose = host.sequence;
  await host.close();
  host = await openHost({ runtime, source, store: await fileStore(directory) });
  assert.equal(host.sequence, atClose);
  assert.deepEqual(assessed(), deployed);
  step('restart', { sequence: host.sequence, permits: host.permits('deploy') });

  // The correction: a doubt reopens the deploy under the authored policy.
  await accept('runner_flaky');
  const reopened = assessed();
  assert.deepEqual([reopened.status, reopened.permits, reopened.caveats], ['reopened', false, ['flaky_runner']]);
  step('doubt', { deploy: reopened });
  await refuse('policy_in_doubt', 'deploy', {}, 'policy', 'reject');

  // The take-back: the doubt is withdrawn, recorded rather than erased, and a
  // new deploy rests on the same run without the caveat.
  await accept('runner_cleared');
  await accept('deploy');
  const redeployed = assessed();
  assert.deepEqual([redeployed.revision, redeployed.status, redeployed.permits, redeployed.caveats], ['deploy@2', 'in_force', true, []]);
  step('take_back', { deploy: redeployed });

  // More runs than the window holds: older occurrences leave for the archive.
  for (let run = 0; run < 6; run += 1) await accept('ci_reported', { passed: 1 });

  // A second restart, then the history from the snapshot and the kept archive.
  await host.close();
  host = await openHost({ runtime, source, store: await fileStore(directory) });
  const archive = await host.archive();
  assert.ok(archive.length > 0, 'runs left the session for the archive');
  const snapshot = host.snapshot();
  const explained = explain(snapshot, [], { archive });
  const series = explained.decisions.find(decision => decision.name === 'deploy');
  const first = series.revisions.find(revision => revision.id === 'deploy@1');
  const history = first.history.map(change => ({ change: change.change, caveats: change.caveats }));
  assert.deepEqual(history.map(change => change.change), ['committed', 'reopened']);
  assert.deepEqual(history[1].caveats, ['flaky_runner']);
  const takenBack = snapshot.taken_back ?? [];
  assert.ok(takenBack.length > 0, 'the take-back is recorded in the snapshot');
  step('history', { deploy_1: { status: first.status, grounds: first.grounds.evidence, history }, taken_back: takenBack.length, archive_entries: archive.length });

  // Under edited source, even a comment, the store refuses to resume.
  const store = await fileStore(directory);
  report.checkpointBytes = Buffer.byteLength((await store.readCheckpoint()) ?? '');
  report.archiveBytes = Buffer.byteLength(JSON.stringify(archive));
  await host.close();
  await assert.rejects(openHost({ runtime, source: `${source}\n# edited\n`, store }),
    error => error instanceof StarterError && error.kind === 'source');
  await store.close?.();
  step('edited_source', { refused: 'source' });

  // The host's budgets.
  assert.ok(report.maxSendMs <= BUDGETS.sendMs, `slowest send ${report.maxSendMs.toFixed(1)} ms > ${BUDGETS.sendMs} ms`);
  assert.ok(report.checkpointBytes <= BUDGETS.checkpointBytes, `checkpoint ${report.checkpointBytes} B`);
  assert.ok(report.archiveBytes <= BUDGETS.archiveBytes, `archive ${report.archiveBytes} B`);
  report.maxSendMs = Number(report.maxSendMs.toFixed(1));
  return report;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const option = name => process.argv.find(arg => arg.startsWith(`--${name}=`))?.slice(name.length + 3);
  const packageDir = option('package');
  if (!packageDir) throw new Error('pass --package=DIR, an installed caveat-lang');
  const directory = option('directory') ?? await mkdtemp(path.join(os.tmpdir(), 'caveat-trial-'));
  const report = await runTrial({ packageDir, directory });
  if (process.argv.includes('--json')) console.log(JSON.stringify(report, null, 2));
  else {
    for (const entry of report.steps) console.log(`${entry.step}: ok`);
    console.log(`${report.package}: ${report.sends} sends, slowest ${report.maxSendMs} ms, checkpoint ${report.checkpointBytes} B, archive ${report.archiveBytes} B`);
  }
}
