// Run random event histories through the WebAssembly runtime and the Lean
// Caveat Sources model's executable, and require the same outcome and the same
// caveats on every state after every event. The model is proved to keep the
// Caveat Sources invariant (proofs/lean/Caveat/Sources.lean); this checks the
// runtime against it on sampled histories. It is not a refinement proof.
import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { delimiter, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadRuntimeFromDirectory } from '../kit/lib/node.mjs';
import { SOURCES_PROGRAM, TRANSLATION_CONTROLS, controlEvents, history, modelRequest, runtimeStates } from './lean-sources-histories.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const proofRoot = join(root, 'proofs', 'lean');

export const RUNS = 1000;
export const LENGTH = 24;
export const SEED = 0x9e3779b97f4a7c15n;

/** The model frame for one host event, reduced to what is compared. */
export function modelFrame(frame) {
  const outcome = frame.outcome.outcome === 'accepted'
    ? { outcome: 'accepted' } : { outcome: 'rejected', code: frame.outcome.code };
  return { outcome, states: frame.states };
}

export function runtimeFrame(dispatched, snapshot) {
  const outcome = dispatched.outcome === 'accepted'
    ? { outcome: 'accepted' } : { outcome: dispatched.outcome, code: dispatched.code };
  return { outcome, states: runtimeStates(snapshot) };
}

/** JSON with object keys sorted, so field order never decides a comparison. */
export function canonical(value) {
  if (Array.isArray(value)) return '[' + value.map(canonical).join(',') + ']';
  if (value !== null && typeof value === 'object') {
    return '{' + Object.keys(value).sort().map(key => JSON.stringify(key) + ':' + canonical(value[key])).join(',') + '}';
  }
  return JSON.stringify(value);
}

/** The first difference between two frame lists, or null. */
export function firstMismatch(expected, actual) {
  for (let index = 0; index < Math.max(expected.length, actual.length); index += 1) {
    if (canonical(expected[index]) !== canonical(actual[index])) {
      return { index, model: expected[index], runtime: actual[index] };
    }
  }
  return null;
}

function leanEnvironment() {
  const env = { ...process.env };
  const pathKey = Object.keys(env).find(key => key.toLowerCase() === 'path') ?? 'PATH';
  if (env.CAVEAT_LEAN_BIN) env[pathKey] = env.CAVEAT_LEAN_BIN + delimiter + (env[pathKey] ?? '');
  return env;
}

export async function verifySources({ runs = RUNS, length = LENGTH } = {}) {
  const outputDirectory = join(root, 'test-results', 'lean-sources');
  mkdirSync(outputDirectory, { recursive: true });
  const report = { status: 'failed', runs, length, startedAt: new Date().toISOString() };
  const env = leanEnvironment();
  try {
    const build = spawnSync('lake', ['build', '--wfail'], { cwd: proofRoot, env, encoding: 'utf8', timeout: 600_000 });
    if (build.status !== 0) throw new Error('lake build failed:\n' + build.stdout + build.stderr);
    const runtime = await loadRuntimeFromDirectory(join(root, 'dist/pkg-reactive'));
    report.runtime = runtime.identity;
    const binary = join(proofRoot, '.lake', 'build', 'bin', process.platform === 'win32' ? 'caveat_compare.exe' : 'caveat_compare');
    function runtimeHistory(seed) {
      const session = runtime.open(SOURCES_PROGRAM);
      const events = [];
      const actual = [];
      try {
        const generator = history(seed, length);
        let next = generator.next();
        while (!next.done) {
          const dispatched = session.dispatch(next.value);
          events.push(next.value);
          actual.push(runtimeFrame(dispatched, session.snapshot()));
          next = generator.next(dispatched.outcome === 'accepted');
        }
      } finally { session.close(); }
      return { events, actual };
    }
    function model(id, events, hostEvents) {
      const result = spawnSync(binary, [], { input: JSON.stringify(modelRequest(id, events, hostEvents)), encoding: 'utf8', timeout: 60_000 });
      if (result.status !== 0) throw new Error('Lean model failed on ' + id + ': ' + result.stderr);
      return JSON.parse(result.stdout).frames.map(modelFrame);
    }
    const seeds = [];
    let seed = SEED;
    for (let run = 0; run < runs; run += 1) {
      seed = BigInt.asUintN(64, seed * 6364136223846793005n + 1442695040888963407n);
      seeds.push(seed);
    }
    let compared = 0;
    let refused = 0;
    const histories = [];
    for (const [run, seed] of seeds.entries()) {
      const { events, actual } = runtimeHistory(seed);
      histories.push({ events, actual });
      refused += actual.filter(frame => frame.outcome.outcome !== 'accepted').length;
      const expected = model('run-' + run, events);
      const mismatch = firstMismatch(expected, actual);
      if (mismatch) {
        report.mismatch = { run, seed: seed.toString(), events: events.slice(0, mismatch.index + 1), ...mismatch };
        writeFileSync(join(outputDirectory, 'mismatch.json'), JSON.stringify(report.mismatch, null, 2) + '\n');
        throw new Error('Runtime and Lean model differ in run ' + run + ' after ' +
          events.slice(0, mismatch.index + 1).join(' ') + '\nmodel:   ' + JSON.stringify(mismatch.model) +
          '\nruntime: ' + JSON.stringify(mismatch.runtime));
      }
      compared += events.length;
    }
    // Each translation control must be told apart from the runtime somewhere.
    report.controls = TRANSLATION_CONTROLS.map(control => {
      const hostEvents = controlEvents(control);
      const run = histories.findIndex(({ events, actual }, index) =>
        events.includes(control.event) && firstMismatch(model('control-' + index, events, hostEvents), actual));
      if (run < 0) throw new Error('Translation control ' + control.id + ' (' + control.finding + ') was not detected.');
      return { id: control.id, finding: control.finding, detectedInRun: run };
    });
    Object.assign(report, { status: 'passed', compared, refused });
    console.log('Caveat Sources: runtime and Lean model agree on ' + runs + ' histories, ' + compared + ' events (' +
      refused + ' refused); ' + report.controls.length + ' translation controls detected.');
  } catch (error) {
    report.error = error.message;
    throw error;
  } finally {
    report.finishedAt = new Date().toISOString();
    writeFileSync(join(outputDirectory, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { await verifySources(); } catch (error) { console.error(error.message); process.exitCode = 1; }
}
