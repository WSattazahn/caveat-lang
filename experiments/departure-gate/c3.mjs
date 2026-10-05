// The departure gate's C3 measurement (spec/caveat-departure-0.1.md, "Gate";
// spec/caveat-lineage-compaction-0.1.md, "The C3 gate"). Plays round 7's C3
// program with windows, as simulate.mjs derives it, for longplay's 60 cycles
// through the C3 adapter, against the package in DIR, and prints:
//   - the adapter's save bytes and the raw runtime save bytes at every 10th cycle;
//   - the microseconds each adapter dispatch took (median, p99, mean), over
//     all events and over the first and last ten cycles;
//   - every dispatch outcome as a count and a digest, so two packages can be
//     compared outcome for outcome;
//   - the effects reported, by kind, and the archive left undrained.
//
//   node experiments/departure-gate/c3.mjs --package=DIR
//
// DIR is an unpacked caveat-lang package (or kit/ after `npm run build`).
import { cpSync, mkdirSync, mkdtempSync, readFileSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { performance } from 'node:perf_hooks';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import { inflateRawSync } from 'node:zlib';

const here = path.dirname(fileURLToPath(import.meta.url));
const packageDir = process.argv.find((a) => a.startsWith('--package='))?.slice(10);
if (!packageDir) throw new Error('usage: c3.mjs --package=DIR');

const work = mkdtempSync(path.join(tmpdir(), 'departure-gate-'));
mkdirSync(path.join(work, 'node_modules'));
symlinkSync(path.resolve(packageDir), path.join(work, 'node_modules', 'caveat-lang'), 'dir');
const impl = path.join(work, 'impl');
cpSync(path.join(here, '../glowcap/round7/runs/C3/impl'), impl, { recursive: true });
const cav = path.join(impl, 'glowcap.cav');
const windowed = readFileSync(path.join(here, 'c3-windows.cav'), 'utf8');
writeFileSync(cav, windowed);

// Count every effect the runtime reports, and keep the session to read its
// undrained archive, by wrapping the package's own session class.
const { CaveatSession } = await import(pathToFileURL(path.join(work, 'node_modules/caveat-lang/lib/session.mjs')).href);
const effects = {};
let session;
const dispatchView = CaveatSession.prototype.dispatchView;
CaveatSession.prototype.dispatchView = function wrapped(...args) {
  session = this;
  const outcome = dispatchView.apply(this, args);
  for (const effect of outcome.view?.effects ?? []) effects[effect.kind] = (effects[effect.kind] ?? 0) + 1;
  return outcome;
};

const adapter = await import(pathToFileURL(path.join(impl, 'adapter.mjs')).href);
const policy = adapter.createPolicy();
const ids = ['cave', 'pool', 'ruin', 'grove', 'pit'];
const digest = createHash('sha256');
const outcomes = { accepted: 0, refused: 0 };
const times = [];
const byCycle = [];
const send = (event) => {
  const started = performance.now();
  let result = 'accepted';
  try {
    policy.dispatch(event);
  } catch (error) {
    result = String(error.message).replace(/\): .*/, ')');
  }
  const elapsed = (performance.now() - started) * 1000;
  times.push(elapsed);
  byCycle.at(-1).push(elapsed);
  outcomes[result === 'accepted' ? 'accepted' : 'refused'] += 1;
  digest.update(`${JSON.stringify(event)} ${result}\n`);
};
const sizes = {};
for (let cycle = 1; cycle <= 60; cycle += 1) {
  byCycle.push([]);
  ids.forEach((m, i) => send({ type: 'witness', id: m, kind: (cycle + i) % 2 ? 'glowcap' : 'duskcap' }));
  for (let t = 0; t < 46 * 16; t += 1) send({ type: 'tick', dt: 0.0625 });
  if (cycle % 10 === 0) {
    const packed = policy.save();
    sizes[cycle] = {
      adapter: Buffer.byteLength(JSON.stringify(packed)),
      raw: inflateRawSync(Buffer.from(packed, 'base64')).length,
    };
  }
}

const stats = (samples) => {
  const sorted = [...samples].sort((a, b) => a - b);
  const at = (q) => sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))];
  const round = (n) => Math.round(n * 10) / 10;
  return { events: sorted.length, median: round(at(0.5)), p99: round(at(0.99)), mean: round(sorted.reduce((a, b) => a + b, 0) / sorted.length) };
};
console.log(JSON.stringify({
  package: JSON.parse(readFileSync(path.join(packageDir, 'package.json'), 'utf8')).version,
  sizes,
  dispatch_us: {
    all: stats(times),
    cycles_1_10: stats(byCycle.slice(0, 10).flat()),
    cycles_51_60: stats(byCycle.slice(50).flat()),
  },
  outcomes: { ...outcomes, sha256: digest.digest('hex') },
  effects,
  undrained: typeof session?.undrained === 'number' ? session.undrained : null,
}, null, 2));
