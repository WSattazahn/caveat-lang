// Corpus sweep for D026 (spec/caveat-withdrawal-0.1.md, "Changes",
// 2026-10-26): runs every tracked Caveat program on two runtimes, the
// published one and the fixed one, with the same seeded events, and compares
// the snapshot and outcome after every event. A difference is expected only
// where a value read through `committed(D)`, `reopened(D)` or `in_basis(D, E)`
// now carries `withdrawn`, and in what a decision made from such a value
// retains; any other difference fails the sweep.
//
// node experiments/d026-sweep/sweep.mjs BEFORE_DIR AFTER_DIR [--write]
// Each directory holds caveat_runtime.js and caveat_runtime_bg.wasm, as the
// npm package's runtime/ or dist/pkg-reactive/ does. --write rewrites
// RESULTS.json beside this file.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../..');
const [beforeDir, afterDir] = process.argv.slice(2).filter(arg => !arg.startsWith('--'));
assert.ok(beforeDir && afterDir, 'usage: sweep.mjs BEFORE_DIR AFTER_DIR [--write]');

async function runtime(dir) {
  // A query string gives each runtime its own module instance.
  const url = pathToFileURL(resolve(dir, 'caveat_runtime.js'));
  url.search = `?${encodeURIComponent(dir)}`;
  const module = await import(url.href);
  await module.default({ module_or_path: readFileSync(resolve(dir, 'caveat_runtime_bg.wasm')) });
  return module.WebReactiveSession;
}
const Before = await runtime(beforeDir);
const After = await runtime(afterDir);

const listed = spawnSync('git', ['ls-files', '-z', '--', '*.cav'], { cwd: root, encoding: 'utf8' });
assert.equal(listed.status, 0, listed.stderr);
const programs = listed.stdout.split('\0').filter(Boolean).sort();

const RUNS = 20;
const STEPS = 30;
let seed = 0x2025;
const next = n => { seed = (seed * 1103515245 + 12345) >>> 0; return (seed >>> 8) % n; };

function payload(event) {
  const out = {};
  for (const parameter of event.parameters ?? []) {
    const members = parameter.domain?.entity?.members ?? parameter.domain?.member?.members;
    if (members) out[parameter.name] = members[next(members.length)];
    else {
      const min = parameter.min ?? 0;
      const max = parameter.max ?? 1;
      out[parameter.name] = min + next(Math.max(1, Math.floor(max - min) + 1));
    }
  }
  return JSON.stringify(out);
}

// Removes `withdrawn` wherever the fix can add it, so two snapshots that
// differ only by it compare equal: caveat lists, a commitment's retained
// caveats (with their authorship), and the `retains withdrawn` relation a
// decision made through such a value records, as one made through
// `latest(D)` already did.
const WITHDRAWN = 'withdrawn';
function withoutWithdrawn(value) {
  if (Array.isArray(value)) {
    return value
      .filter(item => !(item?.relation === 'retains' && item?.to === WITHDRAWN))
      .filter(item => item?.caveat !== WITHDRAWN)
      .map(withoutWithdrawn);
  }
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value).map(([key, item]) => [
      key,
      (key === 'caveats' || key === 'retained') && Array.isArray(item) ? item.filter(c => c !== WITHDRAWN) : withoutWithdrawn(item),
    ]));
  }
  return value;
}

const results = { runs: RUNS, steps: STEPS, programs: 0, skipped: [], identical: 0, gainedWithdrawn: [], other: [] };
for (const path of programs) {
  const source = readFileSync(join(root, path), 'utf8');
  let events;
  try {
    events = JSON.parse(new Before(source).snapshot()).events;
    new After(source);
  } catch {
    results.skipped.push(path);
    continue;
  }
  results.programs += 1;
  if (!events.length) { results.identical += 1; continue; }
  let gained = 0;
  let other = null;
  for (let run = 0; run < RUNS && !other; run += 1) {
    const before = new Before(source);
    const after = new After(source);
    const trail = [];
    for (let step = 0; step < STEPS; step += 1) {
      const event = events[next(events.length)];
      const body = payload(event);
      trail.push(`${event.name} ${body}`);
      let a; let b;
      try { a = before.dispatch_outcome(event.name, body); } catch (error) { a = `fatal ${error}`; }
      try { b = after.dispatch_outcome(event.name, body); } catch (error) { b = `fatal ${error}`; }
      if (JSON.parse(a.startsWith('fatal') ? '{}' : a).outcome !== JSON.parse(b.startsWith('fatal') ? '{}' : b).outcome) {
        other = { trail, why: 'the runtimes disagree on accepting the event' };
        break;
      }
      if (a.startsWith('fatal') || b.startsWith('fatal')) {
        if (a !== b) other = { trail, why: 'only one runtime ended the session' };
        break;
      }
      const left = before.snapshot();
      const right = after.snapshot();
      if (left === right) continue;
      if (JSON.stringify(withoutWithdrawn(JSON.parse(left))) === JSON.stringify(withoutWithdrawn(JSON.parse(right)))) {
        gained += 1;
        continue;
      }
      other = { trail, why: 'the snapshots differ by more than withdrawn' };
      break;
    }
  }
  if (other) results.other.push({ path, ...other });
  else if (gained) results.gainedWithdrawn.push({ path, events: gained });
  else results.identical += 1;
}

console.log(`${results.programs} programs run (${results.skipped.length} do not load alone), ${results.identical} identical, ${results.gainedWithdrawn.length} differ only by withdrawn, ${results.other.length} otherwise`);
for (const item of results.gainedWithdrawn) console.log(`  withdrawn only: ${item.path} (${item.events} snapshots)`);
for (const item of results.other) console.log(`  OTHER: ${item.path}: ${item.why}`);
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), `${JSON.stringify(results, null, 2)}\n`);
assert.equal(results.other.length, 0, 'a difference beyond withdrawn');
