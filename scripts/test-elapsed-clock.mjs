import assert from 'node:assert/strict';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import init, { WebReactiveSession } from '../dist/pkg-reactive/caveat_runtime.js';

const wasm = await readFile(new URL('../dist/pkg-reactive/caveat_runtime_bg.wasm', import.meta.url));
await init({ module_or_path: wasm });
const snapshot = session => JSON.parse(session.snapshot());
const dispatch = (session, event, payload = {}) => JSON.parse(session.dispatch_view(event, JSON.stringify(payload)));
const source = `
claim safe;
evidence reading from "sensor";
caveat stale consequence low;
decisions plan limit 1;
event read;
event advance dt min 0 max 2;
clock advance every 1;
on read reveal reading supports safe;
on read qualify reading with stale after 2;
on advance when carries(reading, stale) and not committed(plan)
    commit plan because enough using qualified(elapsed(), reading);
on advance when dt == 2 reject "test rollback";
bind hud.elapsed = elapsed();
bind hud.stale = carries(reading, stale);
`;
let session = new WebReactiveSession(source);
try {
  assert.equal(snapshot(session).bindings.hud.elapsed, 0);
  // Clock-only events must update bindings in the release WASM build.
  for (const dt of [0.1, 0.2]) dispatch(session, 'advance', { dt });
  assert.equal(snapshot(session).bindings.hud.elapsed, 0.1 + 0.2);
  dispatch(session, 'read');
  dispatch(session, 'advance', { dt: 1 });
  const before = session.save();
  assert.throws(() => dispatch(session, 'advance', { dt: 2 }));
  assert.equal(session.save(), before, 'late rejection rolls back clock, timer and commitment');
  const restored = WebReactiveSession.restore(source, before);
  assert.deepEqual(snapshot(restored), snapshot(session));
  session.free(); session = restored;
  const boundary = dispatch(session, 'advance', { dt: 1 });
  assert.equal(boundary.bindings.hud.elapsed, 2.3);
  assert.equal(boundary.bindings.hud.stale, false, 'timer uses subtraction with exact binary64 values');
  const expired = dispatch(session, 'advance', { dt: 0.1 });
  assert.equal(expired.bindings.hud.stale, true);
  assert.equal(expired.decision_journal[0].elapsed, expired.bindings.hud.elapsed);
  assert.equal(expired.decision_journal[0].value, expired.bindings.hud.elapsed);
  assert.deepEqual(expired.commitment_grounds['plan@1'], { evidence: ['reading'], caveats: ['stale'] });
} finally { session.free(); }

const largeSource = 'event advance dt min 0 max 2; clock advance every 1; bind hud.elapsed = elapsed();';
const initial = new WebReactiveSession(largeSource);
const manufactured = JSON.parse(initial.save());
initial.free();
// Boundary probe via a deliberately manufactured valid save, not a long replay.
manufactured.elapsed = 1e12 + 0.25;
session = WebReactiveSession.restore(largeSource, JSON.stringify(manufactured));
try {
  assert.equal(snapshot(session).bindings.hud.elapsed, manufactured.elapsed);
  const view = dispatch(session, 'advance', { dt: 0.25 });
  assert.equal(view.bindings.hud.elapsed, 1e12 + 0.5);
  const restored = WebReactiveSession.restore(largeSource, session.save());
  try { assert.deepEqual(snapshot(restored), snapshot(session)); }
  finally { restored.free(); }
} finally { session.free(); }

const signedSource = 'event advance dt min 0 max 1; clock advance every 1; bind hud.angle = atan2(elapsed(), -1);';
const signedInitial = new WebReactiveSession(signedSource);
// Preserve negative zero on the wire; JSON.stringify(-0) would erase it.
const signedSave = signedInitial.save().replace(/"elapsed":0(?:\.0)?(?=[,}])/, '"elapsed":-0.0');
signedInitial.free();
session = WebReactiveSession.restore(signedSource, signedSave);
try {
  assert.equal(snapshot(session).bindings.hud.angle, -Math.PI);
  assert.equal(dispatch(session, 'advance', { dt: 0 }).bindings.hud.angle, Math.PI);
} finally { session.free(); }

const result = { passed: true, runtimeSha256: createHash('sha256').update(wasm).digest('hex'),
  checks: ['release binding invalidation', 'fractional timer boundary', 'late rejection atomicity',
    'journal clock/value agreement', 'restore identity', 'manufactured elapsed above state limit',
    'signed-zero clock invalidation'] };
await mkdir(new URL('../test-results/', import.meta.url), { recursive: true });
await writeFile(new URL('../test-results/elapsed-clock-wasm.json', import.meta.url), `${JSON.stringify(result, null, 2)}\n`);
console.log('Elapsed clock WASM: all 7 checks passed.');
