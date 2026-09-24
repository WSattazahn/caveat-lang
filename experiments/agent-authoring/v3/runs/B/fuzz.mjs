// Randomized check of pond.cav against an independent model of TASK.md.
// Usage: node fuzz.mjs [runs] [seed]
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const RUNS = Number(process.argv[2] ?? 400);
let seed = Number(process.argv[3] ?? 12345);
const rand = () => ((seed = (seed * 1103515245 + 12345) % 2147483648) / 2147483648);
const pick = (xs) => xs[Math.floor(rand() * xs.length)];

const source = await readFile(new URL('./pond.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();

// ---- reference model -------------------------------------------------------
function newModel() {
  return { readings: [], cracked: false, decisions: [], journal: [] };
}
function current(m) { return m.decisions[m.decisions.length - 1]; }
function rinkOpen(m) { const d = current(m); return d && !d.reopened && d.value >= 10; }
function modelDispatch(m, event, payload) {
  const keys = payload && typeof payload === 'object' && !Array.isArray(payload) ? Object.keys(payload) : null;
  if (!['measure', 'crack', 'decide'].includes(event)) return 'input';
  if (event === 'measure') {
    if (!keys || keys.length !== 1 || keys[0] !== 'cm') return 'input';
    const cm = payload.cm;
    if (typeof cm !== 'number' || !Number.isFinite(cm) || cm < 0 || cm > 60) return 'input';
    if (m.readings.length >= 8) return 'policy';
    m.readings.push(cm);
    const id = `thickness@${m.readings.length}`;
    if (cm < 10 && rinkOpen(m)) {
      const d = current(m); d.reopened = true;
      m.journal.push({ commitment: d.id, change: 'reopened', because: [id], caveats: ['single_hole'], value: d.value });
    }
    return 'accepted';
  }
  if (!keys || keys.length !== 0) return 'input';
  if (event === 'crack') {
    if (m.cracked) return 'policy';
    m.cracked = true;
    if (rinkOpen(m)) {
      const d = current(m); d.reopened = true;
      m.journal.push({ commitment: d.id, change: 'reopened', because: ['crack_report'], caveats: ['secondhand'], value: d.value });
    }
    return 'accepted';
  }
  // decide
  if (m.readings.length < 3) return 'policy';
  const d = current(m);
  if (d && !d.reopened) return 'policy';
  if (m.decisions.length >= 3) throw new Error('model: fourth decision reached, policy says impossible');
  const value = Math.min(...m.readings);
  const grounds = m.readings.map((_, i) => `thickness@${i + 1}`);
  const nd = { id: `rink@${m.decisions.length + 1}`, value, reopened: false, grounds };
  m.decisions.push(nd);
  m.journal.push({ commitment: nd.id, change: 'committed', because: grounds, caveats: ['single_hole'], value });
  return 'accepted';
}
function modelHud(m) {
  const d = current(m);
  return {
    readings: m.readings.length,
    thinnest: m.readings.length ? Math.min(...m.readings) : -1,
    cracked: m.cracked ? 1 : 0,
    decision: !d ? 'none' : d.reopened ? 'review' : d.value >= 10 ? 'open' : 'closed',
    frozen: d ? d.value : -1,
    revision: m.decisions.length,
  };
}

// ---- random events ---------------------------------------------------------
function randomEvent() {
  const r = rand();
  if (r < 0.5) {
    const cm = pick([
      () => Math.round(rand() * 600) / 10,        // 0..60, one decimal
      () => 8 + Math.round(rand() * 40) / 10,     // around the threshold
      () => pick([0, 10, 9.999, 10.001, 60]),
      () => pick([-0.1, 60.1, -5, 100]),          // out of range
    ])();
    return ['measure', { cm }];
  }
  if (r < 0.62) return ['crack', {}];
  if (r < 0.85) return ['decide', {}];
  return pick([
    ['measure', {}], ['measure', { cm: 12, extra: 1 }], ['measure', { cm: '12' }], ['measure', { depth: 12 }],
    ['crack', { cm: 1 }], ['decide', { x: 0 }], ['thaw', {}], ['measure', { cm: null }],
  ]);
}

const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const sortedHud = (h) => Object.fromEntries(Object.keys(h).sort().map((k) => [k, h[k]]));
let failures = 0, events = 0, restores = 0;
const stats = { accepted: 0, policy: 0, input: 0, maxDecisions: 0, reopenCrack: 0, reopenMeasure: 0 };

for (let run = 0; run < RUNS && failures < 5; run++) {
  const model = newModel();
  let sessions = [runtime.open(source)];
  const log = [];
  const fail = (msg) => { failures++; console.log(`FAIL run ${run}: ${msg}\n  events: ${JSON.stringify(log)}`); };
  const length = 5 + Math.floor(rand() * 30);
  let ok = true;
  for (let step = 0; step < length && ok; step++) {
    if (rand() < 0.1) {
      const saved = sessions[sessions.length - 1].save();
      const restored = runtime.restore(source, saved);
      restores++;
      if (!same(restored.snapshot(), sessions[sessions.length - 1].snapshot())) { fail('restore changed snapshot'); ok = false; break; }
      sessions.push(restored);
    }
    const [event, payload] = randomEvent();
    log.push([event, payload]);
    events++;
    const expected = modelDispatch(model, event, payload);
    stats[expected]++;
    const before = sessions.map((s) => s.save());
    let snaps = [];
    for (const [i, s] of sessions.entries()) {
      let out;
      try { out = s.dispatch(event, payload); } catch (e) { fail(`session ${i} threw ${e.kind}: ${e.message}`); ok = false; break; }
      const got = out.outcome === 'accepted' ? 'accepted' : out.origin;
      if (got !== expected) { fail(`session ${i}: ${event} ${JSON.stringify(payload)} expected ${expected}, got ${got} ${out.code ?? ''} ${out.message ?? ''}`); ok = false; break; }
      if (got !== 'accepted' && s.save() !== before[i]) { fail(`session ${i}: rejected event changed the save`); ok = false; break; }
      snaps.push(s.snapshot());
    }
    if (!ok) break;
    for (const snap of snaps.slice(1)) if (!same(snap, snaps[0])) { fail('restored session diverged'); ok = false; break; }
    if (!ok) break;
    const snap = snaps[0];
    const hud = snap.bindings.hud;
    if (!same(sortedHud(hud), sortedHud(modelHud(model)))) { fail(`hud ${JSON.stringify(hud)} != model ${JSON.stringify(modelHud(model))}`); ok = false; break; }
    if (Object.keys(snap.bindings).length !== 1) { fail(`extra binding targets ${Object.keys(snap.bindings)}`); ok = false; break; }
    const journal = snap.decision_journal.map((e) => ({ commitment: e.commitment, change: e.change, because: e.because, caveats: e.caveats, value: e.value }));
    if (!same(journal, model.journal)) { fail(`journal ${JSON.stringify(journal)} != model ${JSON.stringify(model.journal)}`); ok = false; break; }
    for (const d of model.decisions) {
      const g = snap.commitment_grounds[d.id];
      if (!same(g, { evidence: d.grounds, caveats: ['single_hole'] })) { fail(`grounds of ${d.id}: ${JSON.stringify(g)}`); ok = false; break; }
    }
  }
  stats.maxDecisions = Math.max(stats.maxDecisions, model.decisions.length);
  for (const e of model.journal) if (e.change === 'reopened') stats[e.because[0] === 'crack_report' ? 'reopenCrack' : 'reopenMeasure']++;
  for (const s of sessions) s.close();
}
console.log(`${RUNS} runs, ${events} events, ${restores} restores, ${failures} failures`);
console.log(JSON.stringify(stats));
process.exit(failures ? 1 : 0);
