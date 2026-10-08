#!/usr/bin/env node
// Separate finite WASM equivalence checks; never used for allocation targets.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const options = Object.fromEntries(process.argv.slice(2).map(arg => {
  assert(arg.startsWith('--') && arg.includes('='), 'Use --name=value');
  const i = arg.indexOf('='); return [arg.slice(2, i), arg.slice(i + 1)];
}));
for (const key of ['before', 'candidate', 'candidate-revision', 'native-contract', 'output']) assert(options[key], `Missing --${key}`);
const output = path.resolve(options.output);
assert(!existsSync(output), 'Refuse to overwrite equivalence results');
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const hash = file => sha(readFileSync(file));
const contractFile = path.resolve(options['native-contract']);
const contract = JSON.parse(readFileSync(contractFile, 'utf8'));
const base = path.dirname(contractFile);
function canonical(value) {
  if (typeof value === 'number') { assert(Number.isFinite(value)); return Object.is(value, -0) ? '-0' : String(value); }
  if (value === null || typeof value === 'string' || typeof value === 'boolean') return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  assert(value && typeof value === 'object', 'Unexpected comparison type');
  return `{${Object.keys(value).sort().map(key => `${JSON.stringify(key)}:${canonical(value[key])}`).join(',')}}`;
}
const artifacts = {};
async function load(name, directory, revision) {
  const modulePath = path.join(path.resolve(directory), 'caveat_runtime.js');
  const wasmPath = path.join(path.resolve(directory), 'caveat_runtime_bg.wasm');
  const buildInfoPath = path.join(path.dirname(path.resolve(directory)), 'build-info.json');
  const info = JSON.parse(readFileSync(buildInfoPath, 'utf8'));
  assert.equal(info.revision, revision); assert.equal(info.clean, true); assert.equal(info.compiled, true);
  const wasmSha256 = hash(wasmPath), glueSha256 = hash(modulePath);
  assert.equal(info.runtimeArtifacts['pkg-reactive/caveat_runtime_bg.wasm'], wasmSha256);
  assert.equal(info.runtimeArtifacts['pkg-reactive/caveat_runtime.js'], glueSha256);
  if (name === 'before') {
    assert.equal(wasmSha256, '81c048ac5197f2ba85a27dcb077e930ac7a03663d69b803971017d47a13ea3eb');
    assert.equal(glueSha256, 'a0fff7c2bc9df767502cb66f978bcc944fc65701cdeea14d7e37171c75118056');
  }
  artifacts[name] = { revision, modulePath, wasmPath, buildInfoPath, wasmSha256, glueSha256, buildInfoSha256: hash(buildInfoPath), runtimeSourceFingerprint: info.runtimeSourceFingerprint };
  const module = await import(`${pathToFileURL(modulePath).href}?index-compaction=${name}`);
  await module.default({ module_or_path: readFileSync(wasmPath) });
  return module.WebReactiveSession;
}
const Before = await load('before', options.before, '3d77aa27bcffa16f39818b2684e9afb3b102ed17');
const Candidate = await load('candidate', options.candidate, options['candidate-revision']);
const augmentation = '\nevent profile_probe;\nevidence profile_side from "profile side observation"; renewable profile_side window 1;\nevent profile_mutate;\non profile_mutate renew profile_side;\non profile_mutate reveal profile_side supports seen;\n';
const regular = contract.workloads.map(workload => ({ ...workload, sequence: [
  ...(workload.mode === 'initialize' ? [{ event: 'initialize' }] : []),
  ...Array.from({ length: 60 }, () => ({ event: 'cycle' })),
  ...Array.from({ length: 3 }, () => ({ event: 'profile_probe' })),
  ...Array.from({ length: 10 }, () => ({ event: 'profile_mutate' })),
  ...(workload.mode === 'release' ? [...Array.from({ length: 3 }, () => ({ event: 'fail', expected: 'rejected' })), { event: 'release' }] : []),
  { event: 'profile_probe' },
] }));
const high = contract.workloads.find(workload => workload.name === 'high-degree');
const staged = { ...high, name: 'high-degree-staged', sequence: [
  { event: 'create' }, { event: 'duplicate' }, { event: 'duplicate' }, { event: 'retire' },
  { event: 'same' }, { event: 'trim' }, { event: 'expand' }, { event: 'trim' },
  { event: 'refuse', payload: { index: 15 }, expected: 'rejected' },
  ...Array.from({ length: 15 }, (_, i) => ({ event: 'clear', payload: { index: i + 1 } })),
  { event: 'fail', expected: 'rejected' }, { event: 'release_single' }, { event: 'profile_probe' },
] };
const report = {
  schema: 'caveat-index-compaction-exact-equivalence/1', startedAt: new Date().toISOString(), passed: false,
  driverSha256: hash(fileURLToPath(import.meta.url)), nativeContractSha256: hash(contractFile), artifacts,
  protocol: 'Six fixtures: 60 growth events, 3 unchanged, 10 unrelated renewal, three final-binding failures and final release where authored, then no-op. Plus staged high-degree partial release. Each trace twice: archive drained after every attempt or only at end. At steps 0, midpoint and final, restore exact saves into separate sessions, compare full snapshot/view/save and a following no-op, then drop those branches without altering main trace.',
  equality: 'Complete snapshot, dispatch outcome and view objects: key order canonicalized only; array order, types and negative zero preserved. Save text compared byte-for-byte. Complete ordered archive contents compared with the same value equality and rolling digests. No computational projection/excluded fields.',
  limits: 'Finite same-source sequences only, not general proof or a timing/allocation benchmark. Native target harness remains unchanged. No f5 exact-history comparison: original f5 intentionally retains different histories.', rows: [], counts: { attempts: 0, rejected: 0, snapshots: 0, views: 0, saves: 0, archiveDrains: 0, restoreBranches: 0 },
};
let context;
try {
  for (const plan of [...regular, staged]) for (const drainEvery of [true, false]) {
    const file = path.join(base, plan.file);
    assert.equal(hash(file), plan.sha256);
    const source = readFileSync(file, 'utf8') + augmentation;
    let a = new Before(source), b = new Candidate(source);
    const digest = createHash('sha256');
    const row = { name: plan.name, fixtureSha256: plan.sha256, sourceSha256: sha(source), drainEvery, attempts: 0, rejected: 0 };
    const same = (left, right, label) => {
      const text = canonical(left); assert.equal(text, canonical(right), `${JSON.stringify(context)} ${label}`);
      digest.update(label).update('\0').update(text).update('\n');
    };
    const inspect = (left, right) => {
      same(JSON.parse(left.snapshot()), JSON.parse(right.snapshot()), 'full snapshot'); report.counts.snapshots++;
      same(JSON.parse(left.view()), JSON.parse(right.view()), 'full view'); report.counts.views++;
      const saved = left.save(); assert.equal(saved, right.save(), `${JSON.stringify(context)} save text`); report.counts.saves++;
      digest.update('save\0').update(saved).update('\n');
      return saved;
    };
    const drain = (left, right) => {
      same(JSON.parse(left.drain_archive()), JSON.parse(right.drain_archive()), 'ordered archive');
      report.counts.archiveDrains++;
    };
    const restoreBranch = saved => {
      const x = Before.restore(source, saved), y = Candidate.restore(source, saved);
      inspect(x, y); assert.equal(x.save(), saved);
      same(JSON.parse(x.dispatch_outcome('profile_probe', '{}')), JSON.parse(y.dispatch_outcome('profile_probe', '{}')), 'restored following outcome');
      inspect(x, y); drain(x, y); x.free(); y.free(); report.counts.restoreBranches++;
    };
    context = { name: plan.name, drainEvery, step: 'load' };
    let saved = inspect(a, b); restoreBranch(saved);
    for (let i = 0; i < plan.sequence.length; i++) {
      const step = plan.sequence[i]; context = { name: plan.name, drainEvery, step: i, event: step.event };
      const oldA = a.save(), oldB = b.save();
      const pendingA = a.undrained(), pendingB = b.undrained();
      const payload = JSON.stringify(step.payload ?? {});
      const outA = JSON.parse(a.dispatch_outcome(step.event, payload));
      const outB = JSON.parse(b.dispatch_outcome(step.event, payload));
      same(outA, outB, 'full dispatch outcome');
      assert.equal(outA.outcome, step.expected ?? 'accepted');
      if (outA.outcome === 'rejected') {
        assert.equal(a.save(), oldA); assert.equal(b.save(), oldB);
        assert.equal(a.undrained(), pendingA); assert.equal(b.undrained(), pendingB);
        row.rejected++; report.counts.rejected++;
      }
      saved = inspect(a, b);
      if (drainEvery) drain(a, b);
      row.attempts++; report.counts.attempts++;
      if (i === Math.floor(plan.sequence.length / 2) || i === plan.sequence.length - 1) restoreBranch(saved);
    }
    drain(a, b); assert.equal(a.undrained(), 0); assert.equal(b.undrained(), 0);
    a.free(); b.free(); a = null; b = null;
    row.equalResultSha256 = digest.digest('hex'); report.rows.push(row);
  }
  for (const artifact of Object.values(artifacts)) {
    assert.equal(hash(artifact.modulePath), artifact.glueSha256);
    assert.equal(hash(artifact.wasmPath), artifact.wasmSha256);
    assert.equal(hash(artifact.buildInfoPath), artifact.buildInfoSha256);
  }
  for (const workload of contract.workloads) assert.equal(hash(path.join(base, workload.file)), workload.sha256);
  report.passed = true;
} catch (error) {
  report.failure = { context, message: String(error.stack ?? error) }; process.exitCode = 1;
} finally {
  report.finishedAt = new Date().toISOString();
  writeFileSync(output, JSON.stringify(report, null, 2) + '\n', { flag: 'wx' });
  console.log(JSON.stringify({ passed: report.passed, rows: report.rows.length, counts: report.counts, failure: report.failure, output }, null, 2));
}
