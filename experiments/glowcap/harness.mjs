// Runs the frozen scenarios against one or both implementations and appends
// every run to runs.jsonl, so the iteration counts in RESULTS.md come from a
// log rather than recollection.
//
//   node experiments/glowcap/harness.mjs [--phase=base|cr1|…|cr12] [--impl=ts|caveat…|both]
//   node experiments/glowcap/harness.mjs --measure   size of each implementation
//   node experiments/glowcap/harness.mjs --bench     µs per dispatch + view, bytes shipped
import { createHash } from 'node:crypto';
import { appendFile, readFile, stat } from 'node:fs/promises';
import { gzipSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { SCENARIOS, EXPLANATION_KEYS, PHASES, applies } from './scenarios.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const arg = (name, fallback) => process.argv.find((a) => a.startsWith(`--${name}=`))?.split('=')[1] ?? fallback;

export const IMPLEMENTATIONS = {
  ts: { entry: 'ts/glowcap.ts', policy: ['ts/glowcap.ts'], glue: [] },
  caveat: { entry: 'caveat/adapter.mjs', policy: ['caveat/glowcap.cav'], glue: ['caveat/adapter.mjs'] },
  // Rescore after Explanations 0.1: the Caveat side re-authored with `because`.
  caveat2: { entry: 'caveat2/adapter.mjs', policy: ['caveat2/glowcap.cav'], glue: ['caveat2/adapter.mjs'] },
  // Round 4: grounds, reject, define, typed parameters, the view and the lean runtime.
  caveat3: { entry: 'caveat3/adapter.mjs', policy: ['caveat3/glowcap.cav'], glue: ['caveat3/adapter.mjs'] },
  // Round 5: caveat3 at CR4, then CR5-CR8 with late qualification and the decision journal.
  caveat4: { entry: 'caveat4/adapter.mjs', policy: ['caveat4/glowcap.cav'], glue: ['caveat4/adapter.mjs'] },
};

async function load(name) {
  const module = await import(new URL(IMPLEMENTATIONS[name].entry, import.meta.url));
  if (module.ready) await module.ready;
  return module.createPolicy;
}

async function contentHash(name) {
  const hash = createHash('sha256');
  for (const file of [...IMPLEMENTATIONS[name].policy, ...IMPLEMENTATIONS[name].glue]) {
    hash.update(file).update(await readFile(path.join(here, file)));
  }
  return hash.digest('hex').slice(0, 12);
}

function toEvent(step) {
  const [type, a, b] = step;
  return type === 'tick' ? { type, dt: a } : { type, id: a, kind: b };
}

const SAVE_LIMIT = 4096;

// Keys sorted, arrays in order: two views are identical only if this matches.
function canonical(value) {
  const sort = (v) => (Array.isArray(v) ? v.map(sort)
    : v && typeof v === 'object' ? Object.fromEntries(Object.keys(v).sort().map((k) => [k, sort(v[k])])) : v);
  return JSON.stringify(sort(value));
}

function compare(actual, expected, at, failures) {
  for (const [key, want] of Object.entries(expected)) {
    const got = actual?.[key];
    const where = at ? `${at}.${key}` : key;
    if (Array.isArray(want)) {
      const same = Array.isArray(got) && JSON.stringify([...got].sort()) === JSON.stringify([...want].sort());
      if (!same) failures.push({ where, want, got, explanation: EXPLANATION_KEYS.has(key) });
    } else if (want && typeof want === 'object') {
      compare(got, want, where, failures);
    } else if (got !== want) {
      failures.push({ where, want, got, explanation: false });
    }
  }
}

function runScenario(createPolicy, scenario) {
  const failures = [];
  let policy;
  try {
    policy = createPolicy();
  } catch (error) {
    return [{ step: 0, where: 'createPolicy', want: 'a policy', got: String(error.message ?? error), explanation: false }];
  }
  scenario.steps.forEach((step, index) => {
    if (failures.length) return; // the first failure makes later steps meaningless
    const tag = (list) => list.map((f) => ({ step: index + 1, ...f }));
    try {
      if (step[0] === 'expect') {
        const found = [];
        compare(policy.view(), step[1], '', found);
        failures.push(...tag(found));
      } else if (step[0] === 'resume') {
        // CR12: save, round-trip through JSON, resume, and carry on with the
        // resumed policy. The whole view must survive, not just what is asked.
        const text = JSON.stringify(policy.save());
        const next = createPolicy(JSON.parse(text));
        const bytes = Buffer.byteLength(text);
        if (bytes > SAVE_LIMIT) failures.push(...tag([{ where: 'save', want: `at most ${SAVE_LIMIT} bytes`, got: `${bytes} bytes`, explanation: false }]));
        else if (canonical(next.view()) !== canonical(policy.view())) failures.push(...tag([{ where: 'resume', want: canonical(policy.view()), got: canonical(next.view()), explanation: false }]));
        policy.free?.();
        policy = next;
      } else if (step[0] === 'reject') {
        const before = JSON.stringify(policy.view());
        let threw = false;
        try { policy.dispatch(step[1]); } catch { threw = true; }
        if (!threw) failures.push(...tag([{ where: 'reject', want: 'rejected', got: `accepted ${JSON.stringify(step[1])}`, explanation: false }]));
        else if (JSON.stringify(policy.view()) !== before) failures.push(...tag([{ where: 'reject', want: 'view unchanged', got: 'view changed', explanation: false }]));
      } else {
        const times = step[0] === 'tick' ? (step[2] ?? 1) : 1;
        for (let i = 0; i < times; i += 1) policy.dispatch(toEvent(step));
      }
    } catch (error) {
      failures.push(...tag([{ where: step[0], want: 'accepted', got: String(error.message ?? error), explanation: false }]));
    }
  });
  policy.free?.();
  return failures;
}

async function test(phase, names) {
  if (!PHASES.includes(phase)) throw new Error(`Unknown phase ${phase}`);
  const scenarios = SCENARIOS.filter((s) => applies(s, phase));
  let exitCode = 0;
  for (const name of names) {
    const hash = await contentHash(name);
    let createPolicy;
    const results = [];
    try {
      createPolicy = await load(name);
    } catch (error) {
      results.push(...scenarios.map((s) => ({ id: s.id, failures: [{ step: 0, where: 'load', want: 'module', got: String(error.message ?? error), explanation: false }] })));
    }
    if (createPolicy) for (const s of scenarios) results.push({ id: s.id, failures: runScenario(createPolicy, s) });
    const failed = results.filter((r) => r.failures.length);
    const explanationFailed = failed.filter((r) => r.failures.some((f) => f.explanation));
    await appendFile(path.join(here, 'runs.jsonl'), `${JSON.stringify({
      at: new Date().toISOString(), phase, impl: name, hash,
      passed: results.length - failed.length, total: results.length,
      failed: failed.map((r) => r.id), explanationFailed: explanationFailed.map((r) => r.id),
    })}\n`);
    console.log(`${name.padEnd(6)} ${phase.padEnd(4)} ${hash}  ${results.length - failed.length}/${results.length} passed`);
    for (const r of failed) {
      const f = r.failures[0];
      console.log(`  ✗ ${r.id} step ${f.step} ${f.where}: want ${JSON.stringify(f.want)} got ${JSON.stringify(f.got)}`);
    }
    if (failed.length) exitCode = 1;
  }
  process.exitCode = exitCode;
}

function codeLines(text) {
  return text.split('\n').map((line) => line.trim()).filter((line) => line && !line.startsWith('//')).length;
}

async function measure() {
  for (const [name, impl] of Object.entries(IMPLEMENTATIONS)) {
    for (const group of ['policy', 'glue']) {
      let lines = 0;
      let bytes = 0;
      for (const file of impl[group]) {
        const text = await readFile(path.join(here, file), 'utf8');
        lines += codeLines(text);
        bytes += Buffer.byteLength(text);
      }
      console.log(`${name.padEnd(6)} ${group.padEnd(6)} ${String(lines).padStart(4)} lines ${String(bytes).padStart(6)} bytes  ${impl[group].join(', ') || '-'}`);
    }
  }
}

async function bench() {
  const replay = [
    { type: 'absorb', id: 'cave', kind: 'glowcap' },
    { type: 'absorb', id: 'pool', kind: 'duskcap' },
    { type: 'taste', id: 'ruin', kind: 'glowcap' },
  ];
  while (replay.length < 10_000) replay.push({ type: 'tick', dt: 0.05 });
  for (const name of Object.keys(IMPLEMENTATIONS)) {
    const createPolicy = await load(name);
    const samples = [];
    for (let round = 0; round < 3; round += 1) {
      const policy = createPolicy();
      for (const event of replay) {
        const start = process.hrtime.bigint();
        policy.dispatch(event);
        policy.view();
        samples.push(Number(process.hrtime.bigint() - start) / 1000);
      }
      policy.free?.();
    }
    samples.sort((a, b) => a - b);
    const q = (p) => samples[Math.floor(p * (samples.length - 1))].toFixed(1);
    console.log(`${name.padEnd(6)} dispatch+view µs  median ${q(0.5)}  p95 ${q(0.95)}  max ${q(1)}`);
  }
  const shipped = {
    ts: ['ts/glowcap.ts'],
    caveat: ['caveat/glowcap.cav', 'caveat/adapter.mjs', '../../dist/pkg/caveat_runtime.js', '../../dist/pkg/caveat_runtime_bg.wasm'],
    caveat2: ['caveat2/glowcap.cav', 'caveat2/adapter.mjs', '../../dist/pkg/caveat_runtime.js', '../../dist/pkg/caveat_runtime_bg.wasm'],
    caveat3: ['caveat3/glowcap.cav', 'caveat3/adapter.mjs', '../../dist/pkg-reactive/caveat_runtime.js', '../../dist/pkg-reactive/caveat_runtime_bg.wasm'],
    caveat4: ['caveat4/glowcap.cav', 'caveat4/adapter.mjs', '../../dist/pkg-reactive/caveat_runtime.js', '../../dist/pkg-reactive/caveat_runtime_bg.wasm'],
  };
  for (const [name, files] of Object.entries(shipped)) {
    let raw = 0;
    let gz = 0;
    for (const file of files) {
      const bytes = await readFile(path.join(here, file));
      raw += (await stat(path.join(here, file))).size;
      gz += gzipSync(bytes).length;
    }
    console.log(`${name.padEnd(6)} shipped ${raw} bytes (${gz} gzipped)`);
  }
}

if (process.argv.includes('--measure')) await measure();
else if (process.argv.includes('--bench')) await bench();
else {
  const which = arg('impl', 'both');
  await test(arg('phase', 'base'), which === 'both' ? ['ts', 'caveat'] : [which]);
}
