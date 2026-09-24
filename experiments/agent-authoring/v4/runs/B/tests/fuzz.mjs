// Randomized check of ferry.cav against a plain JavaScript model of TASK.md.
// Sends random event sequences (valid and invalid), saves and restores at
// random points, and compares every outcome and the whole hud with the model.
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const source = await readFile(new URL('../ferry.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();

let seed = Number(process.argv[2] ?? 1);
const rand = () => ((seed = (seed * 1103515245 + 12345) % 2147483648) / 2147483648);
const pick = (xs) => xs[Math.floor(rand() * xs.length)];

function model() {
  return { gusts: [], waves: [], warned: false, decisions: [], reopened: false, journal: [] };
}
// Returns true when the model accepts the event (mutating m), false otherwise.
function step(m, event, payload) {
  const keys = Object.keys(payload);
  const num = (k, lo, hi) => keys.length === 1 && keys[0] === k && typeof payload[k] === 'number'
    && Number.isFinite(payload[k]) && payload[k] >= lo && payload[k] <= hi;
  const inForce = m.decisions.length > 0 && !m.reopened;
  const mayGo = inForce && m.decisions.at(-1) <= 30;
  if (event === 'gust') {
    if (!num('kt', 0, 90) || m.gusts.length >= 10) return false;
    m.gusts.push(payload.kt);
    if (payload.kt > 30 && mayGo) { m.reopened = true; m.journal.push(['reopened', `wind@${m.gusts.length}`]); }
    return true;
  }
  if (event === 'wave') {
    if (!num('m', 0, 6) || m.waves.length >= 10) return false;
    m.waves.push(payload.m);
    if (payload.m > 1.5 && mayGo) { m.reopened = true; m.journal.push(['reopened', `swell@${m.waves.length}`]); }
    return true;
  }
  if (event === 'warning') {
    if (keys.length || m.warned) return false;
    m.warned = true;
    if (mayGo) { m.reopened = true; m.journal.push(['reopened', 'storm_warning']); }
    return true;
  }
  if (event === 'decide') {
    if (keys.length || m.decisions.length >= 4 || inForce || m.gusts.length < 2 || m.waves.length < 2) return false;
    m.decisions.push(Math.max(...m.gusts)); m.reopened = false;
    m.journal.push(['committed', m.gusts.map((_, i) => `wind@${i + 1}`).join(',')]);
    return true;
  }
  return false;
}
function hud(m) {
  const last = m.decisions.at(-1);
  return {
    gusts: m.gusts.length, waves: m.waves.length,
    strongest: m.gusts.length ? Math.max(...m.gusts) : -1,
    highest: m.waves.length ? Math.max(...m.waves) : -1,
    warned: m.warned ? 1 : 0,
    decision: last === undefined ? 'none' : m.reopened ? 'review' : last <= 30 ? 'go' : 'hold',
    frozen: last ?? -1, revision: m.decisions.length,
  };
}
function randomEvent() {
  const r = rand();
  if (r < 0.35) return ['gust', { kt: pick([0, 30, 30.5, 90, Math.round(rand() * 900) / 10, 12, 45]) }];
  if (r < 0.65) return ['wave', { m: pick([0, 1.5, 1.51, 6, Math.round(rand() * 60) / 10, 0.8, 2.2]) }];
  if (r < 0.72) return ['warning', {}];
  if (r < 0.92) return ['decide', {}];
  return pick([['gust', { kt: 91 }], ['wave', { m: -1 }], ['gust', {}], ['wave', { m: 1, kt: 1 }],
    ['decide', { x: 1 }], ['calm', {}], ['gust', { kt: '3' }]]);
}

const canon = (v) => Array.isArray(v) ? v.map(canon) : v && typeof v === "object" ? Object.fromEntries(Object.keys(v).sort().map((k) => [k, canon(v[k])])) : v;
const same = (a, b) => JSON.stringify(canon(a)) === JSON.stringify(canon(b));
let failures = 0;
const runs = Number(process.argv[3] ?? 300);
for (let run = 0; run < runs && failures < 5; run++) {
  let session = runtime.open(source);
  const m = model();
  const len = 10 + Math.floor(rand() * 40);
  for (let i = 0; i < len; i++) {
    const [event, payload] = randomEvent();
    const before = session.save();
    const expected = step(m, event, payload);
    const out = session.dispatch(event, payload);
    const accepted = out.outcome === 'accepted';
    const snap = session.snapshot();
    const journal = snap.decision_journal.map((e) => [e.change, e.because.join(',')]);
    let problem = null;
    if (accepted !== expected) problem = `outcome ${out.outcome} (${out.message ?? ''}), model ${expected}`;
    else if (!accepted && session.save() !== before) problem = 'refused event changed the save';
    else if (!same(snap.bindings.hud, hud(m))) problem = `hud ${JSON.stringify(snap.bindings.hud)} vs ${JSON.stringify(hud(m))}`;
    else if (!same(journal, m.journal)) problem = `journal ${JSON.stringify(journal)} vs ${JSON.stringify(m.journal)}`;
    if (problem) { failures++; console.log(`run ${run} step ${i} ${event} ${JSON.stringify(payload)}: ${problem}`); break; }
    if (rand() < 0.15) {
      const saved = session.save();
      const restored = runtime.restore(source, saved);
      if (!same(restored.snapshot(), session.snapshot())) { failures++; console.log(`run ${run}: restore differs`); }
      session.close(); session = restored;
    }
  }
  session.close();
}
console.log(failures ? `${failures} failure(s)` : `${runs} random runs agree with the model`);
process.exit(failures ? 1 : 0);
