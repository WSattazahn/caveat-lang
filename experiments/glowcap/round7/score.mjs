// Scores round 7 from the private store: the runner logs, the phase commits
// and each author's final program. Measures 1-5; mutation and drift are
// scored separately (mutate.mjs, DRIFT.md).
//
//   node experiments/glowcap/round7/score.mjs [--root=DIR] [--bench] > results/score.json
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { gzipSync } from 'node:zlib';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { PHASES } from './lib.mjs';

const arg = (name, fallback) => process.argv.find((a) => a.startsWith(`--${name}=`))?.slice(name.length + 3) ?? fallback;
const ROOT = arg('root', '/home/claude/glowcap-r7');
const AUTHORS = ['C1', 'C2', 'C3', 'C4', 'T1', 'T2', 'T3'];
const BLIND = ['cr13', 'cr14', 'cr15', 'cr16'];
const INHERITED = PHASES.slice(0, PHASES.indexOf('cr13'));

// harness.mjs's counting rule: trimmed, non-blank, not starting with //.
const isCode = (line) => { const t = line.trim(); return t !== '' && !t.startsWith('//'); };
const codeLines = (text) => text.split('\n').filter(isCode).length;

function git(repo, args) { return execFileSync('git', args, { cwd: repo, encoding: 'utf8', maxBuffer: 1 << 26 }); }

function phaseCommits(repo) {
  const out = {};
  for (const line of git(repo, ['log', '--reverse', '--format=%H %s']).trim().split('\n')) {
    const [hash, , phase, incomplete] = line.split(' ');
    out[phase] = { hash, incomplete: Boolean(incomplete) };
  }
  return out;
}

function changeCost(repo, from, to) {
  const args = from ? ['diff', '-U0', from, to] : ['show', '-U0', '--format=', to];
  let added = 0, removed = 0, raw = 0;
  for (const line of git(repo, args).split('\n')) {
    if (line.startsWith('+++') || line.startsWith('---')) continue;
    if (line.startsWith('+') || line.startsWith('-')) {
      raw += 1;
      if (isCode(line.slice(1))) line.startsWith('+') ? (added += 1) : (removed += 1);
    }
  }
  return { code: added + removed, added, removed, raw };
}

function scenariosAt(phase) {
  return JSON.parse(readFileSync(path.join(ROOT, 'private', 'release', 'scenarios', `${phase}.json`), 'utf8')).map((s) => s.id);
}

function side(id) { return id.startsWith('C') ? 'caveat' : 'ts'; }

function implFiles(dir) {
  return readdirSync(dir, { withFileTypes: true, recursive: true }).filter((d) => d.isFile())
    .map((d) => path.join(d.parentPath ?? d.path, d.name)).sort();
}

async function bench(id) {
  const dir = path.join(ROOT, 'authors', id);
  const entry = path.join(dir, 'impl', side(id) === 'caveat' ? 'adapter.mjs' : 'glowcap.ts');
  const module = await import(pathToFileURL(entry).href);
  if (module.ready) await module.ready;
  const replay = [
    { type: 'absorb', id: 'cave', kind: 'glowcap' },
    { type: 'absorb', id: 'pool', kind: 'duskcap' },
    { type: 'taste', id: 'ruin', kind: 'glowcap' },
  ];
  while (replay.length < 10_000) replay.push({ type: 'tick', dt: 0.05 });
  const samples = [];
  for (let round = 0; round < 3; round += 1) {
    const policy = module.createPolicy();
    for (const event of replay) {
      const start = process.hrtime.bigint();
      policy.dispatch(event);
      policy.view();
      samples.push(Number(process.hrtime.bigint() - start) / 1000);
    }
    policy.free?.();
  }
  samples.sort((a, b) => a - b);
  const q = (p) => Number(samples[Math.floor(p * (samples.length - 1))].toFixed(1));
  // Ten minutes of play, then save, JSON round trip, resume and first view (resume-bench.mjs).
  const policy = module.createPolicy();
  policy.dispatch({ type: 'absorb', id: 'cave', kind: 'glowcap' });
  for (let tick = 0; tick < 9600; tick += 1) policy.dispatch({ type: 'tick', dt: 0.0625 });
  const text = JSON.stringify(policy.save());
  const start = performance.now();
  const resumed = module.createPolicy(JSON.parse(text));
  resumed.view();
  const resumeMs = Number((performance.now() - start).toFixed(2));
  policy.free?.();
  resumed.free?.();
  return { medianUs: q(0.5), p95Us: q(0.95), resumeMs, saveBytes: Buffer.byteLength(text) };
}

function shipped(id) {
  const dir = path.join(ROOT, 'authors', id);
  const files = implFiles(path.join(dir, 'impl'));
  if (side(id) === 'caveat') {
    const kit = path.join(dir, 'node_modules', 'caveat-lang');
    files.push(path.join(kit, 'runtime', 'caveat_runtime.js'), path.join(kit, 'runtime', 'caveat_runtime_bg.wasm'));
    const glue = files.filter((f) => f.endsWith('.mjs')).map((f) => readFileSync(f, 'utf8')).join('\n');
    for (const lib of ['session', 'node']) if (glue.includes(`caveat-lang/${lib}`)) files.push(path.join(kit, 'lib', `${lib}.mjs`));
    if (glue.includes('caveat-lang/node')) files.push(path.join(kit, 'lib', 'session.mjs'));
  }
  return [...new Set(files)].reduce((n, f) => n + gzipSync(readFileSync(f)).length, 0);
}

const results = {};
for (const id of AUTHORS) {
  const mine = path.join(ROOT, 'private', id);
  if (!existsSync(path.join(mine, 'state.json'))) continue;
  const state = JSON.parse(readFileSync(path.join(mine, 'state.json'), 'utf8'));
  const log = readFileSync(path.join(mine, 'log.jsonl'), 'utf8').split('\n').filter(Boolean).map((l) => JSON.parse(l));
  const commits = phaseCommits(path.join(mine, 'repo'));
  const phases = {};
  let previous = null;
  for (const phase of PHASES) {
    if (!commits[phase]) break;
    const tests = log.filter((e) => e.kind === 'test' && e.phase === phase);
    const first = tests[0];
    const greenAt = tests.findIndex((e) => e.passed === e.total);
    const earlier = previous ? new Set(scenariosAt(previous)) : new Set();
    const regressed = new Set(tests.flatMap((e) => e.failed).filter((s) => earlier.has(s)));
    phases[phase] = {
      runs: tests.length,
      firstRunGreen: Boolean(first && first.passed === first.total),
      firstRun: first ? `${first.passed}/${first.total}` : null,
      runsToGreen: greenAt < 0 ? null : greenAt + 1,
      failingRuns: tests.filter((e) => e.passed !== e.total).length,
      incomplete: commits[phase].incomplete,
      explanationFailedIds: [...new Set(tests.flatMap((e) => e.explanationFailed))],
      regressions: [...regressed],
      checks: log.filter((e) => e.kind === 'check' && e.phase === phase).length,
      change: changeCost(path.join(mine, 'repo'), previous ? commits[previous].hash : null, commits[phase].hash),
    };
    previous = phase;
  }
  const finalDir = path.join(ROOT, 'authors', id, 'impl');
  const files = implFiles(finalDir);
  const size = { policy: { lines: 0, bytes: 0 }, glue: { lines: 0, bytes: 0 } };
  for (const f of files) {
    const group = side(id) === 'caveat' && !f.endsWith('.cav') ? 'glue' : 'policy';
    const text = readFileSync(f, 'utf8');
    size[group].lines += codeLines(text);
    size[group].bytes += Buffer.byteLength(text);
  }
  const sum = (list, f) => list.reduce((n, p) => n + (phases[p] ? f(phases[p]) : 0), 0);
  const count = (list, f) => list.filter((p) => phases[p] && f(phases[p])).length;
  results[id] = {
    side: side(id), done: Boolean(state.done),
    blind: {
      firstRunGreen: count(BLIND, (p) => p.firstRunGreen), runs: sum(BLIND, (p) => p.runs),
      failingRuns: sum(BLIND, (p) => p.failingRuns),
      changeCode: sum(BLIND, (p) => p.change.code), changeRaw: sum(BLIND, (p) => p.change.raw),
      regressions: sum(BLIND, (p) => p.regressions.length),
      explanationFailures: sum(BLIND, (p) => p.explanationFailedIds.length),
      incomplete: count(BLIND, (p) => p.incomplete),
    },
    inherited: {
      firstRunGreen: count(INHERITED, (p) => p.firstRunGreen), phases: INHERITED.length, runs: sum(INHERITED, (p) => p.runs),
      failingRuns: sum(INHERITED, (p) => p.failingRuns),
      changeCode: sum(INHERITED, (p) => p.change.code), explanationFailures: sum(INHERITED, (p) => p.explanationFailedIds.length),
      incomplete: count(INHERITED, (p) => p.incomplete),
    },
    size: { lines: size.policy.lines + size.glue.lines, bytes: size.policy.bytes + size.glue.bytes, ...size, files: files.map((f) => path.relative(finalDir, f)) },
    shippedGzipBytes: shipped(id),
    phases,
  };
  if (process.argv.includes('--bench')) results[id].bench = await bench(id);
}
console.log(JSON.stringify(results, null, 2));
