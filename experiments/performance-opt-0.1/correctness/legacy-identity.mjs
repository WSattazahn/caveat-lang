// The exports that existed before dispatch_view_outcome give byte-identical
// output on the base and on this branch: dispatch_outcome, the legacy
// dispatch_view and the legacy dispatch, each on its own pair of sessions,
// with snapshot(), view() and save() compared after every event. Returned
// text and thrown text must match exactly. Streams: a seeded random stream
// over every tracked program that opens, and the recorded Glowcap replay,
// agent ledger and Trail Rescue streams.
//
//   node experiments/performance-opt-0.1/correctness/legacy-identity.mjs \
//     --base=PATH/TO/base/dist [--branch=dist] [--events=120] [--out=FILE]
//
// Each dist holds pkg/ and pkg-reactive/ as `npm run build` writes them; both
// variants are compared.
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { loadManifest, loadWorkload } from '../../performance-0.1/lib/workloads.mjs';

const repo = fileURLToPath(new URL('../../../', import.meta.url));
const option = (name, fallback) => process.argv.find(arg => arg.startsWith(`--${name}=`))?.slice(name.length + 3) ?? fallback;
const base = option('base');
if (!base) throw new Error('--base=DIR is required: the dist/ of a build of the base revision');
const branch = option('branch', path.join(repo, 'dist'));
const EVENTS = Number(option('events', 120));
const out = option('out');
const SEED = 20260929;
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');

let instance = 0;
async function load(directory) {
  const wasm = await readFile(path.join(directory, 'caveat_runtime_bg.wasm'));
  instance += 1;
  const namespace = await import(`${pathToFileURL(path.join(directory, 'caveat_runtime.js')).href}?instance=${instance}`);
  await namespace.default({ module_or_path: wasm });
  return { Session: namespace.WebReactiveSession, wasmSha256: sha256(wasm), wasmBytes: wasm.length };
}

// The generator and event mix of kit/test/dispatch-view-differential.test.mjs.
function mulberry(seed) {
  let state = seed >>> 0;
  const next = () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let t = state;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return (t ^ (t >>> 14)) >>> 0;
  };
  return { below: n => next() % Math.max(1, n), chance: p => next() / 2 ** 32 < p, unit: () => next() / 2 ** 32 };
}
const TEXTS = ['a', 'b', 'c', '602bdbec0a047a5319f53e83f336b9f7aec0e5ed', 'pr-21', 'x'.repeat(40)];
function value(random, { min, max, domain }) {
  if (domain?.identifier !== undefined) return random.chance(0.05) ? random.below(3) + 1 : TEXTS[random.below(TEXTS.length)];
  const members = domain?.entity?.members ?? domain?.member?.members;
  if (members) {
    if (random.chance(0.05)) return 'nobody';
    if (random.chance(0.05)) return 1.5;
    if (random.chance(0.2)) return random.below(members.length + 1) + 1;
    return members[random.below(members.length)];
  }
  if (random.chance(0.05)) return max + 1;
  const number = min + random.unit() * Math.min(max - min, 1e6);
  return random.chance(0.5) ? Math.round(number) : number;
}
function randomStep(random, events) {
  if (random.chance(0.03) || !events.length) return ['no_such_event', '{}'];
  const { name, parameters } = events[random.below(events.length)];
  const payload = Object.fromEntries(parameters.map(parameter => [parameter.name, value(random, parameter)]));
  if (parameters.length && random.chance(0.03)) delete payload[parameters[random.below(parameters.length)].name];
  if (random.chance(0.03)) payload.extra = 1;
  return [name, JSON.stringify(payload)];
}

const call = run => {
  try { return { returned: run() }; } catch (error) {
    return { thrown: typeof error === 'string' ? error : `${error?.constructor?.name}: ${error?.message}` };
  }
};

const METHODS = ['dispatch_outcome', 'dispatch_view', 'dispatch'];

function compare(Base, Branch, source, next, count, label, tally) {
  const open = () => METHODS.map(() => [new Base(source), new Branch(source)]);
  let pairs = open();
  const close = () => { for (const pair of pairs) for (const session of pair) session.free(); };
  try {
    for (let index = 0; index < count; index += 1) {
      const [event, payload] = next(index);
      let fatal = false;
      for (const [slot, method] of METHODS.entries()) {
        const [a, b] = pairs[slot];
        const left = call(() => a[method](event, payload));
        const right = call(() => b[method](event, payload));
        tally.calls += 1;
        if (JSON.stringify(left) !== JSON.stringify(right)) {
          throw new Error(`${label}, event ${index + 1}: ${method}(${event}, ${payload}) differs\nbase:   ${JSON.stringify(left).slice(0, 400)}\nbranch: ${JSON.stringify(right).slice(0, 400)}`);
        }
        if (left.thrown) tally.thrown[method] += 1;
        // A fatal outcome ends the dispatch_outcome sessions; start all again.
        if (method === 'dispatch_outcome' && left.thrown?.includes('"outcome":"fatal"')) { fatal = true; continue; }
        for (const read of ['snapshot', 'view', 'save']) {
          if (JSON.stringify(call(() => a[read]())) !== JSON.stringify(call(() => b[read]()))) {
            throw new Error(`${label}, event ${index + 1}: ${read}() after ${method} differs`);
          }
          tally.reads += 1;
        }
      }
      tally.events += 1;
      if (fatal) { tally.fatal += 1; close(); pairs = open(); }
    }
  } finally { close(); }
}

const files = spawnSync('git', ['ls-files', '-z', '--', '*.cav'], { cwd: repo, encoding: 'utf8' }).stdout.split('\0').filter(Boolean).sort();
const manifest = await loadManifest();
const buildInfo = async directory => { try { return JSON.parse(await readFile(path.join(directory, 'build-info.json'), 'utf8')); } catch { return null; } };
const report = {
  schema: 'caveat-legacy-identity/0.1', seed: SEED, eventsPerProgram: EVENTS,
  // The branch build is identified by its bytes: a build of uncommitted
  // sources still names the base revision in its build-info.json.
  base: { buildInfo: await buildInfo(base) }, variants: {},
};
for (const variant of ['pkg-reactive', 'pkg']) {
  const left = await load(path.join(base, variant));
  const right = await load(path.join(branch, variant));
  const tally = { programs: 0, events: 0, calls: 0, reads: 0, fatal: 0, thrown: Object.fromEntries(METHODS.map(method => [method, 0])), workloads: {} };
  for (const [index, file] of files.entries()) {
    const source = await readFile(path.join(repo, file), 'utf8');
    let probe;
    try { probe = new right.Session(source); } catch { continue; }
    const { events } = JSON.parse(probe.snapshot());
    probe.free();
    const random = mulberry(SEED + index);
    compare(left.Session, right.Session, source, () => randomStep(random, events), EVENTS, `${variant} ${file}`, tally);
    tally.programs += 1;
  }
  for (const id of ['glowcap-replay', 'ledger-session', 'trail-rescue-scenarios']) {
    const workload = await loadWorkload(manifest, id);
    const before = tally.events;
    for (const [number, episode] of workload.episodes.entries()) {
      compare(left.Session, right.Session, workload.program.source, index => episode[index], episode.length, `${variant} ${id} episode ${number + 1}`, tally);
    }
    tally.workloads[id] = tally.events - before;
  }
  report.variants[variant] = {
    base: { wasmSha256: left.wasmSha256, wasmBytes: left.wasmBytes },
    branch: { wasmSha256: right.wasmSha256, wasmBytes: right.wasmBytes },
    ...tally,
  };
  console.log(`${variant}: byte-identical over ${tally.events} events (${tally.programs} programs; ${JSON.stringify(tally.workloads)}), ${tally.calls} dispatch calls and ${tally.reads} reads`);
}
if (out) await writeFile(out, `${JSON.stringify(report, null, 2)}\n`);
