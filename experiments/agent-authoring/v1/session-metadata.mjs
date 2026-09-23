// Post-score supplement: the registered projection omitted session metadata.
// Keep the registered scorer and its original results unchanged.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import init, { WebReactiveSession } from './packet/runtime/caveat_runtime.js';
import { tasks } from './private/oracles.mjs';
import { casesFor, oracleStep, assertAdmission } from './checks.mjs';

const read = file => readFile(new URL(file, import.meta.url));
const json = async file => JSON.parse(await read(file));
const sha = value => createHash('sha256').update(value).digest('hex');
const metadata = (actual, expected) => {
  assert.equal(actual.elapsed, expected.elapsed, 'runtime elapsed differs');
  assert.equal(actual.sequence, expected.sequence, 'runtime sequence differs');
};
// Demonstrate that each omitted invariant is now checked independently.
metadata({ elapsed: 0, sequence: 0 }, { elapsed: 0, sequence: 0 });
assert.throws(() => metadata({ elapsed: 123, sequence: 0 }, { elapsed: 0, sequence: 0 }), /elapsed differs/);
assert.throws(() => metadata({ elapsed: 0, sequence: 999 }, { elapsed: 0, sequence: 0 }), /sequence differs/);

const registration = await json('./registration.json');
const records = {};
for (const id of registration.runs) {
  records[id] = await json(`./runs/${id}/record.json`);
  assert(records[id].finished, `Unfinished author: ${id}`);
}
const manifest = await json('./packet/manifest.json');
const wasm = await read('./packet/runtime/caveat_runtime_bg.wasm');
assert.equal(sha(wasm), manifest.runtime['caveat_runtime_bg.wasm']);
await init({ module_or_path: wasm });

function check(source, task, test) {
  let session;
  let state = task.initial();
  let step = -1;
  let checks = 0;
  const inspect = () => {
    metadata(JSON.parse(session.snapshot()), state);
    checks++;
  };
  const resume = () => {
    const restored = WebReactiveSession.restore(source, JSON.stringify(JSON.parse(session.save())));
    session.free();
    session = restored;
  };
  try {
    session = new WebReactiveSession(source);
    inspect();
    for (const [index, event] of test.events.entries()) {
      step = index;
      if (event.resume === true) resume();
      else {
        const expected = oracleStep(task, state, event);
        let accepted;
        try { session.dispatch_view(event.event, JSON.stringify(event.payload)); accepted = true; }
        catch { accepted = false; }
        assertAdmission(accepted, expected.accepted);
        state = expected.state;
      }
      inspect();
    }
    resume();
    inspect();
    return { id: test.id, pass: true, checks };
  } catch (error) {
    return { id: test.id, pass: false, step: step + 1, error: String(error), checks };
  } finally { session?.free(); }
}

const result = { schema: 1, at: new Date().toISOString(),
  scope: 'Post-score supplemental runtime elapsed/sequence checks; original registered scores preserved.',
  corruptionChecks: 2, runtimeSha256: sha(wasm), runs: [] };
for (const [id, record] of Object.entries(records)) {
  const task = tasks[id[0]];
  const cases = casesFor(task, id[0]);
  const run = { id, stages: {} };
  for (const [stage, file, hash] of [
    ['first', record.versions[0].file, record.versions[0].sha256],
    ['final', 'final.cav', record.finalSha256],
  ]) {
    const source = (await read(`./runs/${id}/${file}`)).toString();
    assert.equal(sha(source), hash);
    let loadError = null;
    try { const session = new WebReactiveSession(source); session.free(); }
    catch (error) { loadError = String(error); }
    const checked = loadError ? [] : cases.map(test => check(source, task, test));
    const passed = checked.filter(test => test.pass).length;
    run.stages[stage] = { sha256: hash, loadError, passed, total: cases.length,
      pass: !loadError && passed === cases.length, cases: checked };
    console.log(`${id} ${stage}: ${loadError ? 'load error (unchanged)' : `${passed}/${cases.length} metadata cases`}`);
  }
  result.runs.push(run);
}
await writeFile(new URL('./results/session-metadata.json', import.meta.url), `${JSON.stringify(result, null, 2)}\n`);
