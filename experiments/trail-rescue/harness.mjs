// Executes the pre-registered JSON contract; contains no alternative game engine.
import assert from 'node:assert/strict';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { performance } from 'node:perf_hooks';
import { createPolicy, createSourcePolicy, ready, source } from './adapter.mjs';

await ready;
const contract = JSON.parse(await readFile(new URL('./scenarios.json', import.meta.url), 'utf8'));
const report = { schema: 1, started: new Date().toISOString(), scenarios: [], checks: [], stress: null };
const started = performance.now();
const clone = value => JSON.parse(JSON.stringify(value));
const atPointer = (value, pointer) => pointer.split('/').slice(1).reduce((object, key) => object?.[key.replace(/~1/g, '/').replace(/~0/g, '~')], value);

function send(policy, event) {
  const before = policy.view();
  const saved = policy.save();
  let accepted = false;
  try { policy.dispatch(event); accepted = true; } catch { /* Rejection is part of the contract. */ }
  if (!accepted) {
    assert.deepEqual(policy.view(), before, 'rejection changed the public view');
    assert.deepEqual(policy.save(), saved, 'rejection changed hidden or saved state');
  }
  return accepted;
}

for (const scenario of contract.scenarios) {
  const live = [];
  let stepIndex = -1;
  try {
    let policy = createPolicy();
    live.push(policy);
    assert.deepEqual(policy.view(), contract.initialView);
    for (const [index, step] of scenario.steps.entries()) {
      stepIndex = index;
      if (step.op === 'resume') {
        const restored = createPolicy(clone(policy.save()));
        assert.deepEqual(restored.view(), policy.view(), 'resume changed the full view');
        live.push(restored);
        policy = restored;
      } else {
        const results = live.map(session => send(session, step.event));
        assert.equal(results[0], !step.reject, step.reject ? 'event should reject' : 'event should accept');
        assert(results.every(value => value === results[0]), 'resumed shadow disagreed on acceptance');
        for (const shadow of live) assert.deepEqual(shadow.view(), policy.view(), 'resumed shadow diverged');
      }
      for (const [pointer, expected] of Object.entries(step.expect ?? {})) {
        assert.deepEqual(atPointer(policy.view(), pointer), expected, pointer);
      }
    }
    const final = createPolicy(clone(policy.save()));
    live.push(final);
    assert.deepEqual(final.view(), policy.view(), 'final resume changed the full view');
    report.scenarios.push({ id: scenario.id, pass: true, steps: scenario.steps.length });
    console.log(`PASS ${scenario.id} ${scenario.title}`);
  } catch (error) {
    report.scenarios.push({ id: scenario.id, pass: false, step: stepIndex + 1, error: error.message });
    console.error(`FAIL ${scenario.id} step ${stepIndex + 1}: ${error.message}`);
  } finally {
    for (const policy of live) policy.free();
  }
}

async function check(name, body) {
  try { await body(); report.checks.push({ name, pass: true }); console.log(`PASS ${name}`); }
  catch (error) { report.checks.push({ name, pass: false, error: error.message }); console.error(`FAIL ${name}: ${error.message}`); }
}

await check('Non-JSON event rejection is atomic', () => {
  const policy = createPolicy();
  try {
    for (const dt of [NaN, Infinity, -Infinity, '1', null, undefined, {}, [], -1, 30.001]) {
      assert.equal(send(policy, { type: 'tick', dt }), false);
    }
    for (const event of [null, [], true, 42, 'rescue', {}, { type: 'observe', tunnel: 'stone', method: 'scout', condition: undefined }]) {
      assert.equal(send(policy, event), false);
    }
  } finally { policy.free(); }
});

await check('Source alone controls budget, aging, caveats and physical outcomes', () => {
  const altered = source.replace('state scouts = 3', 'state scouts = 1')
    .replace('after 30;', 'after 12;')
    .replace('secondhand qualifies seen_report_stone;', '')
    .replace('if($index == 1, 1, 2) min 1 max 2', 'if($index == 1, 2, 1) min 1 max 2');
  assert.notEqual(altered, source);
  const policy = createSourcePolicy(altered);
  try {
    assert.equal(policy.view().scoutsRemaining, 1);
    policy.dispatch({ type: 'observe', tunnel: 'stone', method: 'report', condition: 'clear' });
    assert.deepEqual(policy.view().evidence[0].caveats, []);
    policy.dispatch({ type: 'plan', tunnel: 'stone' });
    policy.dispatch({ type: 'tick', dt: 12 });
    assert.equal(policy.view().decision.state, 'reopened');
    policy.dispatch({ type: 'observe', tunnel: 'reed', method: 'scout' });
    assert.equal(policy.view().tunnels.reed.status, 'clear');
    assert.equal(policy.view().scoutsRemaining, 0);
    assert.equal(send(policy, { type: 'observe', tunnel: 'stone', method: 'scout' }), false);
    policy.dispatch({ type: 'plan', tunnel: 'reed' });
    policy.dispatch({ type: 'rescue' });
    assert.equal(policy.view().outcome, 'rescued');
  } finally { policy.free(); }
});

await check('A hidden physical change does not leak into the knowledge view', () => {
  const policy = createPolicy();
  try {
    policy.dispatch({ type: 'observe', tunnel: 'stone', method: 'scout' });
    policy.dispatch({ type: 'plan', tunnel: 'stone' });
    const before = policy.view();
    policy.dispatch({ type: 'change', tunnel: 'stone', condition: 'blocked' });
    assert.deepEqual(policy.view(), before);
    policy.dispatch({ type: 'rescue' });
    assert.equal(policy.view().outcome, 'failed');
  } finally { policy.free(); }
});

await check('A source clock that permits negative time restores its journal and pending timers', () => {
  const customClock = source.replace('event advance dt min 0 max 30;', 'event advance dt min -1 max 30;');
  assert.notEqual(customClock, source);
  const original = createSourcePolicy(customClock);
  let restored;
  try {
    original.dispatch({ type: 'tick', dt: -1 });
    original.dispatch({ type: 'observe', tunnel: 'stone', method: 'scout' });
    original.dispatch({ type: 'plan', tunnel: 'stone' });
    assert.equal(original.view().decision.history[0].at, -1);
    restored = createSourcePolicy(customClock, clone(original.save()));
    assert.deepEqual(restored.view(), original.view());
    for (const event of [{ type: 'tick', dt: 29.9 }, { type: 'tick', dt: 0.1 }]) {
      original.dispatch(event); restored.dispatch(event);
      assert.deepEqual(restored.view(), original.view());
    }
    assert.equal(restored.view().decision.state, 'reopened');
    assert.deepEqual(restored.view().decision.reopenedBy, ['scout_stone_1']);
  } finally { original.free(); restored?.free(); }
});

// Seeded property checks examine invariants, not a second set of game rules.
let randomState = 0x7a11beef;
function random() { randomState ^= randomState << 13; randomState ^= randomState >>> 17; randomState ^= randomState << 5; return (randomState >>> 0) / 4294967296; }
const pick = values => values[Math.floor(random() * values.length)];
function randomEvent(index) {
  const tunnel = pick(['stone', 'reed']);
  const condition = pick(['clear', 'blocked']);
  const kind = pick(index < 25 ? ['observe', 'observe', 'plan', 'change', 'tick', 'tick'] : ['observe', 'plan', 'change', 'tick', 'rescue']);
  if (kind === 'observe') return random() < 0.5 ? { type: kind, tunnel, method: 'scout' } : { type: kind, tunnel, method: 'report', condition };
  if (kind === 'tick') return { type: kind, dt: pick([0, 0.1, 0.0013580246789999999, 1, 5, 15, 29.9, 30]) };
  if (kind === 'change') return { type: kind, tunnel, condition };
  if (kind === 'plan') return { type: kind, tunnel };
  return { type: kind };
}

await check('Seeded event and restore invariants', () => {
  const stats = { seed: '0x7a11beef', sequences: 200, stepsPerSequence: 80, accepted: 0, rejected: 0, resumes: 0 };
  for (let sequence = 0; sequence < stats.sequences; sequence++) {
    let policy = createPolicy();
    let shadow = createPolicy();
    try {
      for (let index = 0; index < stats.stepsPerSequence; index++) {
        const before = policy.view();
        const event = randomEvent(index);
        const accepted = send(policy, event);
        assert.equal(send(shadow, event), accepted);
        const after = policy.view();
        assert.deepEqual(after, shadow.view(), `restore divergence in sequence ${sequence}, step ${index}`);
        stats[accepted ? 'accepted' : 'rejected']++;
        assert.deepEqual(after.decision.history.slice(0, before.decision.history.length), before.decision.history, 'history was rewritten');
        const scouts = after.evidence.filter(record => record.method === 'scout');
        assert.equal(after.scoutsRemaining, 3 - scouts.length);
        assert(after.scoutsRemaining >= 0 && after.scoutsRemaining <= 3);
        assert(after.evidence.length <= 5);
        for (const tunnel of ['stone', 'reed']) assert(after.evidence.filter(record => record.method === 'report' && record.tunnel === tunnel).length <= 1);
        assert.equal(new Set(after.evidence.map(record => record.id)).size, after.evidence.length);
        for (const previous of before.evidence) {
          const current = after.evidence.find(record => record.id === previous.id);
          assert(current, 'old observation was removed');
          assert.deepEqual({ ...current, caveats: [] }, { ...previous, caveats: [] }, 'observation identity/content was changed');
          for (const caveat of previous.caveats) assert(current.caveats.includes(caveat), 'caveat was forgotten');
        }
        if (before.outcome !== 'pending') assert.equal(accepted, false);
        if (event.type === 'plan') assert.equal(accepted, before.tunnels[event.tunnel].canPlan);
        if (event.type === 'rescue') assert.equal(accepted, before.canRescue);
        if (accepted && event.type === 'change') assert.deepEqual(after, before, 'hidden change leaked');
        if (random() < 0.2) {
          const restored = createPolicy(clone(policy.save()));
          assert.deepEqual(restored.view(), after);
          policy.free(); policy = restored; stats.resumes++;
        }
      }
    } finally { policy.free(); shadow.free(); }
  }
  report.stress = stats;
});

report.durationMs = Math.round(performance.now() - started);
report.pass = [...report.scenarios, ...report.checks].every(result => result.pass);
await mkdir(new URL('../../test-results/', import.meta.url), { recursive: true });
await writeFile(new URL('../../test-results/trail-rescue.json', import.meta.url), `${JSON.stringify(report, null, 2)}\n`);
console.log(`${report.scenarios.filter(result => result.pass).length}/${report.scenarios.length} scenarios; ${report.checks.filter(result => result.pass).length}/${report.checks.length} additional checks; ${report.durationMs} ms`);
if (!report.pass) process.exitCode = 1;
