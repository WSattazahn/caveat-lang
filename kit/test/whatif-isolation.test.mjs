// Existing APIs, not a public whatif operation. The fixture executes an actual
// WASM unreachable at an injected dispatch boundary, not a natural Caveat trap.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import path from 'node:path';
import { loadRuntimeFromDirectory, defaultRuntimeDirectory } from '../lib/node.mjs';
import { loadRuntime } from '../lib/session.mjs';

const starter = await readFile(new URL('../examples/agent-evidence/assessment.cav', import.meta.url), 'utf8');
const source = `${starter}
identifiers limit 8;
state subject = 0;
state bounded = 1 min 0 max 2;
caveat stale consequence material;
event identify item id;
event advance dt min 0 max 10;
clock advance every 1;
event expire;
event divide denominator min 0 max 1;
event signal;
event crash;
cue note toast "branch fixture" 1;
on identify set subject = item;
on expire when not observed(tool) reveal tool supports answer_supported;
on expire qualify tool with stale;
on divide set bounded = 1 / denominator;
on signal emit note;
on crash set bounded = sqrt(0 - 1);
bind fixture.subject = id_text(subject);
`;

function capture(session) {
  return { save: session.save(), snapshot: session.snapshotText(), view: session.viewText() };
}
function unchanged(session, before) {
  assert.deepEqual(capture(session), before, 'hypothetical work changed a live session');
}
function accepted(session, event, payload = {}) {
  const outcome = session.dispatch(event, payload);
  assert.equal(outcome.outcome, 'accepted', JSON.stringify(outcome));
  return outcome;
}
function seed(session, label, confidence = 85) {
  accepted(session, 'observe', { confidence });
  accepted(session, 'assess');
  accepted(session, 'identify', { item: label });
  accepted(session, 'advance', { dt: 1 });
  accepted(session, 'signal');
}
function restoreAndContinue(runtime, session) {
  const restored = runtime.restore(source, session.save());
  try {
    assert.deepEqual(capture(restored), capture(session));
    assert.deepEqual(accepted(restored, 'advance', { dt: 1 }), accepted(session, 'advance', { dt: 1 }));
    assert.deepEqual(capture(restored), capture(session));
  } finally { restored.close(); }
}

test('separate-runtime branches preserve all live fields through accepted and rejected events', async () => {
  const runtimeA = await loadRuntimeFromDirectory();
  const runtimeB = await loadRuntimeFromDirectory();
  assert.deepEqual(runtimeB.identity, runtimeA.identity);
  const live = runtimeA.open(source);
  let branch;
  try {
    seed(live, 'live');
    const before = capture(live);
    branch = runtimeB.restore(source, before.save);
    assert.deepEqual(capture(branch), before);
    const dispatch = (event, payload, origin) => {
      const outcome = branch.dispatch(event, payload);
      assert.equal(outcome.outcome, origin ? 'rejected' : 'accepted', JSON.stringify(outcome));
      if (origin) assert.equal(outcome.origin, origin);
      if (event === 'expire') assert.ok(branch.snapshot().effects.some(effect => effect.kind === 'qualify'));
      unchanged(live, before);
    };
    dispatch('observe', { confidence: 40 });
    dispatch('assess', {}, 'policy');
    dispatch('observe', { confidence: 101 }, 'input');
    dispatch('divide', { denominator: 0.25 }, 'evaluation');
    dispatch('retract', {});
    dispatch('expire', {});
    dispatch('observe', { confidence: 90 });
    dispatch('assess', {});
    dispatch('identify', { item: 'hypothetical-only' });
    dispatch('advance', { dt: 3 });
    dispatch('signal', {});
    const changed = branch.snapshot();
    assert.equal(changed.reading_streams.observations.occurrences.length, 3);
    assert.equal(changed.decision_series.assessment.revisions.length, 2);
    assert.equal(changed.withdrawals.length, 1);
    assert.ok(changed.relations.some(edge => edge.relation === 'qualifies' && edge.from === 'stale'));
    assert.equal(changed.elapsed, 4);
    assert.equal(changed.bindings.fixture.subject, 'hypothetical-only');
    assert.ok(Array.isArray(changed.cues) && changed.cues.length > 0);
    for (let index = 3; index < 16; index++) dispatch('observe', { confidence: 90 });
    dispatch('observe', { confidence: 90 }, 'limit');
    branch.close();
    unchanged(live, before);
    restoreAndContinue(runtimeA, live);
  } finally { branch?.close(); live.close(); }
});

test('ordinary unclassified fatal ends only its branch', async () => {
  const runtimeA = await loadRuntimeFromDirectory();
  const runtimeB = await loadRuntimeFromDirectory();
  const live = [runtimeA.open(source), runtimeA.open(source)];
  let branch;
  let sibling;
  try {
    live.forEach((session, index) => seed(session, `live-${index}`));
    const before = live.map(capture);
    branch = runtimeB.restore(source, before[0].save);
    sibling = runtimeB.restore(source, before[0].save);
    assert.throws(() => branch.dispatch('crash'), error =>
      error.kind === 'fatal' && error.report?.code === 'unclassified' && /sqrt requires a nonnegative number/.test(error.message));
    assert.equal(branch.state, 'fatal');
    assert.equal(runtimeB.trapped, false);
    assert.equal(runtimeA.trapped, false);
    branch.close();
    accepted(sibling, 'advance', { dt: 1 });
    live.forEach((session, index) => {
      unchanged(session, before[index]);
      restoreAndContinue(runtimeA, session);
    });
  } finally { branch?.close(); sibling?.close(); live.forEach(session => session.close()); }
});

// (module (func (export "trap") unreachable)). The engine produces the error;
// JavaScript does not construct it. This tests adapter containment, not a
// corruption inside Caveat's own Rust/WASM memory.
const trapModule = new WebAssembly.Module(Uint8Array.from([
  0, 97, 115, 109, 1, 0, 0, 0,
  1, 4, 1, 96, 0, 0,
  3, 2, 1, 0,
  7, 8, 1, 4, 116, 114, 97, 112, 0, 0,
  10, 5, 1, 3, 0, 0, 11,
]));
let isolatedNumber = 0;
async function trapCapableRuntime(identity) {
  const directory = defaultRuntimeDirectory();
  const wasm = await readFile(path.join(directory, 'caveat_runtime_bg.wasm'));
  assert.equal(createHash('sha256').update(wasm).digest('hex'), identity.reactiveWasmSha256);
  const url = pathToFileURL(path.join(directory, 'caveat_runtime.js'));
  url.searchParams.set('whatif-isolation', String(++isolatedNumber));
  const runtime = await loadRuntime({ module: url, wasm, identity });
  const namespace = await import(url.href);
  const prototype = namespace.WebReactiveSession.prototype;
  const original = prototype.dispatch_outcome;
  const trap = new WebAssembly.Instance(trapModule).exports.trap;
  const receipt = { entered: 0, error: null };
  prototype.dispatch_outcome = function (event, payload) {
    if (event === 'injected_wasm_trap') {
      receipt.entered += 1;
      try { return trap(); } catch (error) { receipt.error = error; throw error; }
    }
    return original.call(this, event, payload);
  };
  return { runtime, namespace, receipt,
    removeInjection: () => { prototype.dispatch_outcome = original; } };
}

test('an injected actual WASM trap in isolated B preserves two A sessions and later branches', async () => {
  const runtimeA = await loadRuntimeFromDirectory();
  const live = [runtimeA.open(source), runtimeA.open(source)];
  try {
    live.forEach((session, index) => seed(session, `live-${index}`, 85 + index));
    for (let iteration = 0; iteration < 2; iteration++) {
      const before = live.map(capture);
      const isolated = await trapCapableRuntime(runtimeA.identity);
      const branch = isolated.runtime.restore(source, before[0].save);
      try {
        assert.deepEqual(capture(branch), before[0]);
        assert.throws(() => branch.dispatch('injected_wasm_trap'), error => error.kind === 'fatal');
        assert.equal(isolated.receipt.entered, 1);
        assert.ok(isolated.receipt.error instanceof WebAssembly.RuntimeError);
        assert.equal(isolated.runtime.trapped, true);
        assert.equal(branch.state, 'fatal');
        assert.throws(() => branch.save(), error => error.kind === 'fatal');
        assert.throws(() => isolated.runtime.restore(source, before[0].save), error => error.kind === 'fatal');
        await assert.rejects(loadRuntime({ module: isolated.namespace, wasm: null }),
          error => error.kind === 'fatal' && /module URL/.test(error.message));
      } finally { isolated.removeInjection(); branch.close(); }
      assert.equal(branch.state, 'closed');
      assert.equal(runtimeA.trapped, false);
      live.forEach((session, index) => {
        unchanged(session, before[index]);
        restoreAndContinue(runtimeA, session);
      });
      const fresh = await loadRuntimeFromDirectory();
      assert.deepEqual(fresh.identity, runtimeA.identity);
      const nextBranch = fresh.restore(source, live[0].save());
      try { accepted(nextBranch, 'observe', { confidence: 40 }); }
      finally { nextBranch.close(); }
    }
  } finally { live.forEach(session => session.close()); }
});

test('inverse control: the same-runtime trap invalidates live siblings', async () => {
  const identity = (await loadRuntimeFromDirectory()).identity;
  const shared = await trapCapableRuntime(identity);
  const live = [shared.runtime.open(source), shared.runtime.open(source)];
  let branch;
  try {
    live.forEach((session, index) => seed(session, `same-${index}`));
    const saved = live[0].save();
    branch = shared.runtime.restore(source, saved);
    assert.throws(() => branch.dispatch('injected_wasm_trap'), error => error.kind === 'fatal');
    assert.ok(shared.receipt.error instanceof WebAssembly.RuntimeError);
    assert.equal(shared.runtime.trapped, true);
    for (const session of live) {
      for (const read of ['snapshot', 'save', 'view']) {
        assert.throws(() => session[read](), error => error.kind === 'fatal' && /trapped/.test(error.message));
      }
      assert.throws(() => session.dispatch('advance', { dt: 1 }), error => error.kind === 'fatal');
    }
    assert.throws(() => shared.runtime.restore(source, saved), error => error.kind === 'fatal');
  } finally { shared.removeInjection(); branch?.close(); live.forEach(session => session.close()); }
});

test('appending history differs from replacing an earlier event at its checkpoint', async () => {
  const runtimeA = await loadRuntimeFromDirectory();
  const runtimeB = await loadRuntimeFromDirectory();
  const live = runtimeA.open(source);
  const start = live.save();
  let appended;
  let replaced;
  try {
    accepted(live, 'observe', { confidence: 85 });
    accepted(live, 'assess');
    const before = capture(live);
    appended = runtimeB.restore(source, before.save);
    replaced = runtimeB.restore(source, start);
    accepted(appended, 'observe', { confidence: 40 });
    accepted(replaced, 'observe', { confidence: 40 });
    const readings = session => session.snapshot().reading_streams.observations.occurrences.map(({ id, value }) => ({ id, value }));
    assert.deepEqual(readings(appended), [{ id: 'observations@1', value: 85 }, { id: 'observations@2', value: 40 }]);
    assert.deepEqual(readings(replaced), [{ id: 'observations@1', value: 40 }]);
    assert.equal(appended.snapshot().decision_series.assessment.revisions.length, 1);
    assert.equal(replaced.snapshot().decision_series.assessment.revisions.length, 0);
    unchanged(live, before);
  } finally { appended?.close(); replaced?.close(); live.close(); }
});

test('restore requires compatible source and the branch uses the same runtime artifact', async () => {
  const runtimeA = await loadRuntimeFromDirectory();
  const runtimeB = await loadRuntimeFromDirectory();
  assert.deepEqual(runtimeA.identity, runtimeB.identity);
  const live = runtimeA.open(source);
  let branch;
  try {
    seed(live, 'same-build');
    const before = capture(live);
    branch = runtimeB.restore(source, before.save);
    assert.deepEqual(capture(branch), before);
    assert.throws(() => runtimeB.restore(`${source}\nstate changed_source = 0;`, before.save),
      error => error.kind === 'restore' && /different program/.test(error.message));
    unchanged(live, before);
  } finally { branch?.close(); live.close(); }
});
