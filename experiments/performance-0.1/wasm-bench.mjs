// Measurement-only baseline of the reactive runtime on WebAssembly in Node.
// run.mjs starts one process per (target, workload, mode) and passes a plan;
// this times the mode's operations for every event with
// process.hrtime.bigint() and writes the raw microsecond samples, which
// run.mjs summarizes with the same code as the native samples.
//
//   node --expose-gc experiments/performance-0.1/wasm-bench.mjs --plan=PLAN.json --out=OUT.json
//
// Paths through the runtime, from the host's side in:
//   adapter  experiments/glowcap/caveat5/adapter.mjs, imported unchanged but
//            for its three file locations: the published 51.6 µs method
//   kit      the developer kit's CaveatSession (lib/session.mjs, loaded with
//            lib/node.mjs): dispatch_outcome plus a view() call
//   raw      the wasm-bindgen class WebReactiveSession, as the web pages use it
//   abi      the same exports called directly, splitting a raw call into
//            argument copy, WebAssembly execution, result decode and free
// Nothing here changes the runtime or the kit; every file is loaded from the
// target being measured.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { sha256 } from './lib/workloads.mjs';
import { ADAPTER_LOCATIONS, rewriteAdapter } from './lib/adapter.mjs';

const arg = (name) => process.argv.find((value) => value.startsWith(`--${name}=`))?.slice(name.length + 3);
const plan = JSON.parse(await readFile(arg('plan'), 'utf8'));
const outPath = arg('out');
const now = process.hrtime.bigint;
const us = (from, to) => Number(to - from) / 1000;
const fileUrl = (file) => pathToFileURL(file).href;

const notes = [];
const extra = {};

// ---- loading -------------------------------------------------------------

// Older runtimes lack some methods (dispatch_outcome arrived with the
// dispatch outcome contract); a mode that needs one is skipped, not failed.
class Missing extends Error {}

async function loadRaw(methods = []) {
  const glue = await import(fileUrl(path.join(plan.runtimeDir, 'caveat_runtime.js')));
  const missing = methods.filter((name) => typeof glue.WebReactiveSession.prototype[name] !== 'function');
  if (missing.length) throw new Missing(`this runtime has no ${missing.join(', ')}`);
  const wasm = await readFile(path.join(plan.runtimeDir, 'caveat_runtime_bg.wasm'));
  const exports = await glue.default({ module_or_path: wasm });
  return { Session: glue.WebReactiveSession, exports };
}

async function loadKit() {
  if (!plan.kitLib) throw new Error('this target has no kit library');
  const node = await import(fileUrl(path.join(plan.kitLib, 'node.mjs')));
  const session = await import(fileUrl(path.join(plan.kitLib, 'session.mjs')));
  const runtime = await node.loadRuntimeFromDirectory(plan.runtimeDir);
  return { runtime, payloadText: session.payloadText };
}

// The adapter as published, with only its three file locations pointed at
// this target's runtime and at the plan's program (whose hash the runner
// checked against the adapter's own glowcap.cav).
async function loadAdapter() {
  if (!plan.adapter) throw new Error('this workload has no adapter');
  const original = await readFile(plan.adapter.file, 'utf8');
  const program = path.join(plan.workDir, 'program.cav');
  await writeFile(program, plan.program);
  const text = rewriteAdapter(original, {
    glueUrl: fileUrl(path.join(plan.runtimeDir, 'caveat_runtime.js')),
    wasmUrl: fileUrl(path.join(plan.runtimeDir, 'caveat_runtime_bg.wasm')),
    programUrl: fileUrl(program),
  });
  const file = path.join(plan.workDir, 'adapter.mjs');
  await writeFile(file, text);
  extra.adapter = { file: plan.adapter.file, originalSha256: sha256(original), rewrittenSha256: sha256(text), rewritten: Object.values(ADAPTER_LOCATIONS) };
  const module = await import(fileUrl(file));
  await module.ready;
  return module.createPolicy;
}

// The wasm-bindgen 0.2.104 ABI the generated glue uses, called directly.
// passStringToWasm0 and getStringFromWasm0 are mirrored from the glue so that
// each piece of a call can be timed on its own.
function abiOf(exports, methods) {
  const required = ['memory', '__wbindgen_malloc', '__wbindgen_realloc', '__wbindgen_free', ...methods.map((name) => `webreactivesession_${name}`)];
  const missing = required.filter((name) => !(name in exports));
  if (missing.length) return { missing };
  const decoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
  decoder.decode();
  const encoder = new TextEncoder();
  let cached = null;
  const memory = () => {
    if (cached === null || cached.byteLength === 0) cached = new Uint8Array(exports.memory.buffer);
    return cached;
  };
  function pass(text) {
    let length = text.length;
    let pointer = exports.__wbindgen_malloc(length, 1) >>> 0;
    const bytes = memory();
    let offset = 0;
    for (; offset < length; offset++) {
      const code = text.charCodeAt(offset);
      if (code > 0x7f) break;
      bytes[pointer + offset] = code;
    }
    if (offset !== length) {
      const rest = offset === 0 ? text : text.slice(offset);
      pointer = exports.__wbindgen_realloc(pointer, length, (length = offset + rest.length * 3), 1) >>> 0;
      const written = encoder.encodeInto(rest, memory().subarray(pointer + offset, pointer + length)).written;
      offset += written;
      pointer = exports.__wbindgen_realloc(pointer, length, offset, 1) >>> 0;
    }
    return [pointer, offset];
  }
  const decode = (pointer, length) => decoder.decode(memory().subarray(pointer >>> 0, (pointer >>> 0) + length));
  const free = (pointer, length) => exports.__wbindgen_free(pointer, length, 1);
  // A Result's error is an externref the glue takes out of its table.
  const dropError = (index) => {
    exports.__wbindgen_export_0?.get(index);
    exports.__externref_table_dealloc?.(index);
  };
  return { exports, pass, decode, free, dropError };
}

// ---- driving -------------------------------------------------------------

const prepared = plan.episodes.map((episode) => episode.map(([event, payloadText]) => ({
  event,
  payloadText,
  payload: JSON.parse(payloadText),
  adapterEvent: plan.adapter?.events === 'glowcap' ? glowcapEvent(event, JSON.parse(payloadText)) : null,
})));

function glowcapEvent(event, payload) {
  return event === 'tick' ? { type: 'tick', dt: payload.dt } : { type: event, id: payload.target, kind: payload.sort };
}

// Plays every episode `warmup + rounds` times, each from a fresh session
// (opening and closing are not timed). `step` fills one slot per op.
function drive({ ops, open, close, finish, finalsKind, step, warmup = plan.warmup, rounds = plan.rounds, collect = true }) {
  const passes = warmup + rounds;
  const timed = ops.map(() => []);
  const finals = [];
  const slots = new Array(ops.length);
  for (let pass = 0; pass < passes; pass++) {
    if (collect && globalThis.gc) globalThis.gc();
    const samples = ops.map(() => []);
    for (let repeat = 0; repeat < plan.repeats; repeat++) {
      for (const episode of prepared) {
        const session = open();
        try {
          for (const item of episode) {
            slots.fill(null);
            step(session, item, slots);
            for (let op = 0; op < ops.length; op++) samples[op].push(slots[op]);
          }
          if (pass === passes - 1 && repeat === plan.repeats - 1) finals.push(finish(session));
        } finally {
          close(session);
        }
      }
    }
    if (pass >= warmup) samples.forEach((values, op) => timed[op].push(values));
  }
  return {
    ops: Object.fromEntries(ops.map((op, index) => [op, { unit: 'us', rounds: timed[index] }])),
    finals: { kind: finalsKind, values: finals },
  };
}

function rawDriver(Session, extraOptions) {
  return {
    open: () => new Session(plan.program),
    close: (session) => session.free(),
    finish: (session) => session.save(),
    finalsKind: 'save-text',
    ...extraOptions,
  };
}

// Times `fn` in `samples` batches of `inner` calls, after `warm` calls.
function batch(fn, { samples = plan.microSamples ?? 2000, inner = 1, warm = 200 } = {}) {
  for (let index = 0; index < warm; index++) fn();
  const values = new Array(samples);
  for (let sample = 0; sample < samples; sample++) {
    const start = now();
    for (let index = 0; index < inner; index++) fn();
    values[sample] = us(start, now()) / inner;
  }
  return { unit: 'us', rounds: [values], inner };
}

// ---- modes -----------------------------------------------------------------

const modes = {
  // experiments/glowcap/harness.mjs bench(), for caveat5 alone: no warm-up,
  // no collection between rounds, 3 rounds unless the plan says otherwise.
  async 'published-method'() {
    const createPolicy = await loadAdapter();
    return drive({
      ops: ['adapter.dispatch+view'],
      warmup: 0,
      collect: false,
      open: () => createPolicy(),
      close: (policy) => policy.free?.(),
      finish: (policy) => JSON.stringify(policy.save()),
      finalsKind: 'adapter-save-json',
      step(policy, { adapterEvent }, slots) {
        const start = now();
        policy.dispatch(adapterEvent);
        policy.view();
        slots[0] = us(start, now());
      },
    });
  },

  async adapter() {
    const createPolicy = await loadAdapter();
    return drive({
      ops: ['adapter.dispatch', 'adapter.view', 'adapter.dispatch+view'],
      open: () => createPolicy(),
      close: (policy) => policy.free?.(),
      finish: (policy) => JSON.stringify(policy.save()),
      finalsKind: 'adapter-save-json',
      step(policy, { adapterEvent }, slots) {
        const t0 = now();
        policy.dispatch(adapterEvent);
        const t1 = now();
        policy.view();
        const t2 = now();
        slots[0] = us(t0, t1);
        slots[1] = us(t1, t2);
        slots[2] = us(t0, t2);
      },
    });
  },

  async 'raw.dispatch_view'() {
    const { Session } = await loadRaw(['dispatch_view', 'save']);
    return drive({
      ...rawDriver(Session),
      ops: ['js.stringify_payload', 'raw.dispatch_view', 'js.parse_view', 'raw.dispatch_view+parse'],
      step(session, { event, payload }, slots) {
        const t0 = now();
        const text = JSON.stringify(payload);
        const t1 = now();
        let view = null;
        try { view = session.dispatch_view(event, text); } catch { /* refused */ }
        const t2 = now();
        slots[0] = us(t0, t1);
        slots[1] = us(t1, t2);
        if (view !== null) {
          JSON.parse(view);
          const t3 = now();
          slots[2] = us(t2, t3);
          slots[3] = us(t0, t3);
        }
      },
    });
  },

  async 'raw.dispatch_outcome'() {
    const { Session } = await loadRaw(['dispatch_outcome', 'save']);
    return drive({
      ...rawDriver(Session),
      ops: ['raw.dispatch_outcome', 'js.parse_outcome', 'raw.dispatch_outcome+parse'],
      step(session, { event, payloadText }, slots) {
        const t0 = now();
        const text = session.dispatch_outcome(event, payloadText);
        const t1 = now();
        JSON.parse(text);
        const t2 = now();
        slots[0] = us(t0, t1);
        slots[1] = us(t1, t2);
        slots[2] = us(t0, t2);
      },
    });
  },

  async 'raw.dispatch'() {
    const { Session } = await loadRaw(['dispatch', 'save']);
    return drive({
      ...rawDriver(Session),
      ops: ['raw.dispatch'],
      step(session, { event, payloadText }, slots) {
        const t0 = now();
        try { session.dispatch(event, payloadText); } catch { /* refused */ }
        slots[0] = us(t0, now());
      },
    });
  },

  async 'abi.dispatch_view'() { return abiDispatch('dispatch_view'); },
  async 'abi.dispatch_outcome'() { return abiDispatch('dispatch_outcome'); },

  // Read-only calls on the state each event leaves, after an untimed dispatch.
  async read() {
    const { Session } = await loadRaw(['dispatch_view', 'view', 'snapshot', 'save']);
    return drive({
      ...rawDriver(Session),
      ops: ['raw.view', 'js.parse_view', 'raw.snapshot', 'js.parse_snapshot', 'raw.save'],
      step(session, { event, payloadText }, slots) {
        try { session.dispatch_view(event, payloadText); } catch { /* refused */ }
        const t0 = now();
        const view = session.view();
        const t1 = now();
        JSON.parse(view);
        const t2 = now();
        const snapshot = session.snapshot();
        const t3 = now();
        JSON.parse(snapshot);
        const t4 = now();
        session.save();
        const t5 = now();
        slots[0] = us(t0, t1);
        slots[1] = us(t1, t2);
        slots[2] = us(t2, t3);
        slots[3] = us(t3, t4);
        slots[4] = us(t4, t5);
      },
    });
  },

  async 'abi.read'() {
    const { Session, exports } = await loadRaw(['dispatch_view', 'save']);
    const abi = abiOf(exports, ['view', 'snapshot', 'save']);
    if (abi.missing) return skipped(`ABI exports missing: ${abi.missing.join(', ')}`);
    const call = (name, pointer, slots, at) => {
      const t0 = now();
      const result = abi.exports[`webreactivesession_${name}`](pointer);
      const t1 = now();
      if (result.length > 3 && result[3]) {
        abi.dropError(result[2]);
        slots[at] = us(t0, t1);
        return;
      }
      abi.decode(result[0], result[1]);
      const t2 = now();
      abi.free(result[0], result[1]);
      slots[at] = us(t0, t1);
      slots[at + 1] = us(t1, t2);
    };
    return drive({
      ...rawDriver(Session),
      ops: ['abi.view.exec', 'abi.view.decode', 'abi.snapshot.exec', 'abi.snapshot.decode', 'abi.save.exec', 'abi.save.decode'],
      step(session, { event, payloadText }, slots) {
        try { session.dispatch_view(event, payloadText); } catch { /* refused */ }
        call('view', session.__wbg_ptr, slots, 0);
        call('snapshot', session.__wbg_ptr, slots, 2);
        call('save', session.__wbg_ptr, slots, 4);
      },
    });
  },

  // The kit host path: dispatch returns the outcome with the full snapshot;
  // a host that wants the view calls view() as well.
  async kit() {
    const { runtime } = await loadKit();
    return drive({
      ops: ['kit.dispatch', 'kit.view', 'kit.dispatch+view'],
      open: () => runtime.open(plan.program),
      close: (session) => session.close(),
      finish: (session) => session.save(),
      finalsKind: 'save-text',
      step(session, { event, payload }, slots) {
        const t0 = now();
        session.dispatch(event, payload);
        const t1 = now();
        session.view();
        const t2 = now();
        slots[0] = us(t0, t1);
        slots[1] = us(t1, t2);
        slots[2] = us(t0, t2);
      },
    });
  },

  async 'kit.read'() {
    const { runtime } = await loadKit();
    return drive({
      ops: ['kit.view', 'kit.snapshot', 'kit.save'],
      open: () => runtime.open(plan.program),
      close: (session) => session.close(),
      finish: (session) => session.save(),
      finalsKind: 'save-text',
      step(session, { event, payload }, slots) {
        session.dispatch(event, payload);
        const t0 = now();
        session.view();
        const t1 = now();
        session.snapshot();
        const t2 = now();
        session.save();
        const t3 = now();
        slots[0] = us(t0, t1);
        slots[1] = us(t1, t2);
        slots[2] = us(t2, t3);
      },
    });
  },

  // Load, save and restore at the state the first episode ends in.
  async lifecycle() {
    const { Session } = await loadRaw(['dispatch_view', 'save']);
    const kit = plan.kitLib ? await loadKit() : null;
    const played = new Session(plan.program);
    for (const { event, payloadText } of prepared[0]) {
      try { played.dispatch_view(event, payloadText); } catch { /* refused */ }
    }
    const saved = played.save();
    const ops = ['raw.new', 'raw.save', 'raw.restore', ...(kit ? ['kit.open', 'kit.save', 'kit.restore'] : [])];
    const samples = ops.map(() => []);
    const kitSession = kit?.runtime.restore(plan.program, saved);
    const restoredSaves = [];
    const total = plan.lifecycleWarmup + plan.lifecycleSamples;
    if (globalThis.gc) globalThis.gc();
    for (let index = 0; index < total; index++) {
      const slot = [];
      let t0 = now();
      new Session(plan.program).free();
      slot.push(us(t0, now()));
      t0 = now();
      played.save();
      slot.push(us(t0, now()));
      t0 = now();
      const restored = Session.restore(plan.program, saved);
      slot.push(us(t0, now()));
      if (index === 0) restoredSaves.push(restored.save());
      restored.free();
      if (kit) {
        t0 = now();
        kit.runtime.open(plan.program).close();
        slot.push(us(t0, now()));
        t0 = now();
        kitSession.save();
        slot.push(us(t0, now()));
        t0 = now();
        const resumed = kit.runtime.restore(plan.program, saved);
        slot.push(us(t0, now()));
        if (index === 0) restoredSaves.push(resumed.save());
        resumed.close();
      }
      if (index >= plan.lifecycleWarmup) slot.forEach((value, op) => samples[op].push(value));
    }
    kitSession?.close();
    played.free();
    if (restoredSaves.some((text) => text !== saved)) notes.push('CORRECTNESS: a restored session saves differently from its save');
    extra.saveBytes = Buffer.byteLength(saved);
    return {
      ops: Object.fromEntries(ops.map((op, index) => [op, { unit: 'us', rounds: [samples[index]] }])),
      finals: { kind: 'first-episode-save-text', values: [saved] },
    };
  },

  // experiments/glowcap/resume-bench.mjs for caveat5: play, save, then time
  // JSON.parse(save) + createPolicy(saved) + view(); the views must match
  // and continued play must agree, checked outside the timed region.
  async 'adapter-resume'() {
    const createPolicy = await loadAdapter();
    const ops = ['adapter.resume.parse', 'adapter.resume.createPolicy', 'adapter.resume.view', 'adapter.resume'];
    const samples = ops.map(() => []);
    let saveBytes = 0;
    const finals = [];
    for (let run = 0; run < plan.resumeRuns; run += 1) {
      const policy = createPolicy();
      let resumed;
      try {
        for (const { adapterEvent } of prepared[0]) policy.dispatch(adapterEvent);
        const text = JSON.stringify(policy.save());
        saveBytes = Buffer.byteLength(text);
        const t0 = performance.now();
        const saved = JSON.parse(text);
        const t1 = performance.now();
        resumed = createPolicy(saved);
        const t2 = performance.now();
        const view = resumed.view();
        const t3 = performance.now();
        [t1 - t0, t2 - t1, t3 - t2, t3 - t0].forEach((ms, op) => samples[op].push(ms * 1000));
        assert.deepEqual(view, policy.view(), 'restored view');
        policy.dispatch({ type: 'tick', dt: 0.05 });
        resumed.dispatch({ type: 'tick', dt: 0.05 });
        assert.deepEqual(resumed.view(), policy.view(), 'continued play');
        if (run === 0) finals.push(text);
      } finally {
        policy.free?.();
        resumed?.free?.();
      }
    }
    extra.saveBytes = saveBytes;
    extra.timer = 'performance.now(), as resume-bench.mjs';
    return {
      ops: Object.fromEntries(ops.map((op, index) => [op, { unit: 'us', rounds: [samples[index]] }])),
      finals: { kind: 'first-episode-adapter-save-json', values: finals },
    };
  },

  // Synthetic isolation of single costs, on the texts the first episode's
  // final state produces: payload encoding, the bridge's string copy and
  // decode, JSON.parse, and the timer itself.
  async micro() {
    const { Session } = await loadRaw(['dispatch_view', 'view', 'snapshot', 'save']);
    const kit = plan.kitLib ? await loadKit() : null;
    const session = new Session(plan.program);
    const episode = prepared[0];
    for (const { event, payloadText } of episode.slice(0, -1)) {
      try { session.dispatch_view(event, payloadText); } catch { /* refused */ }
    }
    const last = episode[episode.length - 1];
    let outcome = null;
    if (typeof session.dispatch_outcome === 'function') outcome = session.dispatch_outcome(last.event, last.payloadText);
    else try { session.dispatch_view(last.event, last.payloadText); } catch { /* refused */ }
    const texts = { view: session.view(), ...(outcome === null ? {} : { outcome }), snapshot: session.snapshot(), save: session.save() };
    session.free();
    const encoder = new TextEncoder();
    const decoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
    const scratch = new Uint8Array(1 << 20);
    const passAscii = (text) => {
      for (let offset = 0; offset < text.length; offset++) scratch[offset] = text.charCodeAt(offset);
    };
    const ops = {
      'timer.hrtime_pair': batch(() => { const start = now(); now(); return start; }, {}),
      'js.stringify_payload.first': batch(() => JSON.stringify(episode[0].payload), { inner: 100 }),
      'js.stringify_payload.last': batch(() => JSON.stringify(last.payload), { inner: 100 }),
      'bridge.pass_ascii.payload': batch(() => passAscii(last.payloadText), { inner: 100 }),
    };
    if (kit) ops['kit.payloadText.last'] = batch(() => kit.payloadText(last.payload), { inner: 100 });
    for (const [name, text] of Object.entries(texts)) {
      const bytes = encoder.encode(text);
      ops[`bridge.decode.${name}`] = batch(() => decoder.decode(bytes.subarray(0, bytes.length)));
      ops[`bridge.pass_ascii.${name}`] = batch(() => passAscii(text));
      ops[`js.parse.${name}`] = batch(() => JSON.parse(text));
    }
    extra.bytes = Object.fromEntries(Object.entries(texts).map(([name, text]) => [name, Buffer.byteLength(text)]));
    return { ops, finals: { kind: 'none', values: [] } };
  },
};

async function abiDispatch(which) {
  const { Session, exports } = await loadRaw([which, 'save']);
  const abi = abiOf(exports, [which]);
  if (abi.missing) return skipped(`ABI exports missing: ${abi.missing.join(', ')}`);
  const fn = abi.exports[`webreactivesession_${which}`];
  let shapeChecked = false;
  return drive({
    ...rawDriver(Session),
    ops: ['abi.encode_args', 'abi.exec', 'abi.decode', 'abi.free', `abi.${which}`],
    step(session, { event, payloadText }, slots) {
      const t0 = now();
      const [eventPointer, eventLength] = abi.pass(event);
      const [payloadPointer, payloadLength] = abi.pass(payloadText);
      const t1 = now();
      const result = fn(session.__wbg_ptr, eventPointer, eventLength, payloadPointer, payloadLength);
      const t2 = now();
      if (!shapeChecked) {
        assert.ok(Array.isArray(result) && result.length === 4, `${which} returned ${JSON.stringify(result)}; expected [ptr, len, error, isError]`);
        shapeChecked = true;
      }
      slots[0] = us(t0, t1);
      slots[1] = us(t1, t2);
      if (result[3]) {
        abi.dropError(result[2]);
        return;
      }
      abi.decode(result[0], result[1]);
      const t3 = now();
      abi.free(result[0], result[1]);
      const t4 = now();
      slots[2] = us(t2, t3);
      slots[3] = us(t3, t4);
      slots[4] = us(t0, t4);
    },
  });
}

function skipped(reason) {
  notes.push(`skipped: ${reason}`);
  return { ops: {}, finals: { kind: 'none', values: [] } };
}

// ---- main ------------------------------------------------------------------

if (!modes[plan.mode]) throw new Error(`unknown WebAssembly mode ${plan.mode}`);
await mkdir(plan.workDir, { recursive: true });
const started = performance.now();
let measured;
try {
  measured = await modes[plan.mode]();
} catch (error) {
  if (error instanceof Missing) measured = skipped(error.message);
  else if (await programFailsToLoad()) measured = skipped(`the program does not load on this runtime: ${programFailsToLoad.message}`);
  else throw error;
}

// After a mode fails: is it because this (older) runtime cannot load the
// program at all? Then the mode is skipped, not failed.
async function programFailsToLoad() {
  const glue = await import(fileUrl(path.join(plan.runtimeDir, 'caveat_runtime.js')));
  await glue.default({ module_or_path: await readFile(path.join(plan.runtimeDir, 'caveat_runtime_bg.wasm')) });
  try {
    new glue.WebReactiveSession(plan.program).free();
    return false;
  } catch (error) {
    if (typeof WebAssembly !== 'undefined' && error instanceof WebAssembly.RuntimeError) return false;
    programFailsToLoad.message = String(error?.message ?? error).split('\n')[0];
    return true;
  }
}

const timer = batch(() => { const start = now(); now(); return start; }, { samples: 10000, warm: 1000 });
const sortedTimer = [...timer.rounds[0]].sort((a, b) => a - b);
await writeFile(outPath, JSON.stringify({
  schema: 'caveat-performance-raw/0.1',
  engine: 'wasm',
  workload: plan.workload,
  mode: plan.mode,
  method: {
    warmupPasses: plan.mode === 'published-method' ? 0 : plan.warmup,
    rounds: plan.rounds,
    repeats: plan.repeats,
    lifecycleSamples: plan.lifecycleSamples,
    lifecycleWarmup: plan.lifecycleWarmup,
    timer: 'process.hrtime.bigint()',
    timerOverhead: { what: 'two process.hrtime.bigint() calls', minUs: sortedTimer[0], medianUs: sortedTimer[Math.floor((sortedTimer.length - 1) / 2)] },
    gcBetweenRounds: Boolean(globalThis.gc) && plan.mode !== 'published-method',
  },
  process: {
    pid: process.pid,
    node: process.version,
    v8: process.versions.v8,
    execArgv: process.execArgv,
    availableParallelism: os.availableParallelism(),
  },
  ops: measured.ops,
  finals: measured.finals,
  notes,
  extra,
  wallSeconds: (performance.now() - started) / 1000,
}));
