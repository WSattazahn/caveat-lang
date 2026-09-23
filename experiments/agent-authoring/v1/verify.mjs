// Private scoring after all authors finish. Never supply failures during a run.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import init, { WebReactiveSession } from './packet/runtime/caveat_runtime.js';
import { tasks } from './private/oracles.mjs';
import { assertProjection, assertAdmission, assertGrounds, oracleStep, casesFor } from './checks.mjs';

const here = fileURLToPath(new URL('./', import.meta.url));
const sha = text => createHash('sha256').update(text).digest('hex');
const clone = value => JSON.parse(JSON.stringify(value));
const records = {};
for (const id of ['A1', 'B1', 'C1', 'A2', 'B2', 'C2']) {
  records[id] = JSON.parse(await readFile(path.join(here, 'runs', id, 'record.json'), 'utf8'));
  assert(records[id].finished, `Do not score before all authors finish: ${id}`);
}
const manifest = JSON.parse(await readFile(path.join(here, 'packet/manifest.json'), 'utf8'));
const wasm = await readFile(new URL('./packet/runtime/caveat_runtime_bg.wasm', import.meta.url));
assert.equal(sha(wasm), manifest.runtime['caveat_runtime_bg.wasm']);
await init({ module_or_path: wasm });

function runCase(source, task, test) {
  let policy;
  let shadow;
  let state = task.initial();
  let step = -1;
  const counts = { events: 0, accepted: 0, rejected: 0, restores: 0 };
  const view = session => JSON.parse(session.view());
  const resume = () => {
    const restored = WebReactiveSession.restore(source, JSON.stringify(clone(JSON.parse(policy.save()))));
    try {
      assert.deepEqual(view(restored), view(policy), 'resume changed full runtime view');
      assert.deepEqual(JSON.parse(restored.snapshot()), JSON.parse(policy.snapshot()), 'resume changed full snapshot');
    } catch (error) { restored.free(); throw error; }
    if (!shadow) shadow = policy;
    else policy.free();
    policy = restored; counts.restores++;
  };
  const dispatch = (session, event) => {
    const before = session.save();
    const beforeView = view(session);
    let accepted;
    try { session.dispatch_view(event.event, JSON.stringify(event.payload)); accepted = true; }
    catch { accepted = false; }
    if (!accepted) {
      assert.deepEqual(JSON.parse(session.save()), JSON.parse(before), 'rejected event changed saved state');
      assert.deepEqual(view(session), beforeView, 'rejected event changed runtime view');
    }
    return accepted;
  };
  try {
    policy = new WebReactiveSession(source);
    assertProjection(task, state, view(policy));
    assertGrounds(JSON.parse(policy.snapshot()));
    for (const [index, event] of test.events.entries()) {
      step = index;
      if (event.resume === true) resume();
      else {
        const expected = oracleStep(task, state, event);
        const accepted = dispatch(policy, event);
        counts.events++; counts[accepted ? 'accepted' : 'rejected']++;
        assertAdmission(accepted, expected.accepted);
        state = expected.state;
        if (shadow) {
          assertAdmission(dispatch(shadow, event), accepted);
          assert.deepEqual(view(policy), view(shadow), 'restored session diverged from uninterrupted shadow');
        }
      }
      assertProjection(task, state, view(policy));
      assertGrounds(JSON.parse(policy.snapshot()));
    }
    resume();
    return { id: test.id, kind: test.kind, pass: true, ...counts };
  } catch (error) {
    return { id: test.id, kind: test.kind, pass: false, step: step + 1, event: test.events[step], error: String(error), ...counts };
  } finally { policy?.free(); shadow?.free(); }
}

const results = { schema: 1, at: new Date().toISOString(), runtimeSha256: sha(wasm), runs: [] };
for (const [id, record] of Object.entries(records)) {
  const task = tasks[id[0]];
  assert(task, `Unknown task ${id[0]}`);
  const cases = casesFor(task, id[0]);
  const run = { id, versions: record.versions.length, runtimeChecks: record.checks.length,
    checkErrors: record.checks.filter(check => check.status === 'error').length,
    started: record.started, finished: record.finished, stages: {} };
  for (const [stage, file, expectedHash] of [
    ['first', record.versions[0].file, record.versions[0].sha256],
    ['final', 'final.cav', record.finalSha256],
  ]) {
    const source = await readFile(path.join(here, 'runs', id, file), 'utf8');
    assert.equal(sha(source), expectedHash, `Preserved candidate was changed: ${id}/${file}`);
    let error;
    try { const session = new WebReactiveSession(source); session.free(); } catch (caught) { error = String(caught); }
    const scored = error ? [] : cases.map(test => runCase(source, task, test));
    const passed = scored.filter(test => test.pass).length;
    run.stages[stage] = { sha256: expectedHash, loadError: error ?? null, passed, total: cases.length,
      pass: !error && passed === cases.length, cases: scored };
    console.log(`${id} ${stage}: ${error ? `LOAD ERROR: ${error}` : `${passed}/${cases.length} cases`}`);
  }
  results.runs.push(run);
}
results.firstPasses = results.runs.filter(run => run.stages.first.pass).length;
results.finalPasses = results.runs.filter(run => run.stages.final.pass).length;
await mkdir(path.join(here, 'results'), { recursive: true });
await writeFile(path.join(here, 'results/scored.json'), `${JSON.stringify(results, null, 2)}\n`);
console.log(`First submissions: ${results.firstPasses}/6. Final submissions: ${results.finalPasses}/6.`);
// Candidate failure is an experimental result, not a crashed verifier.
