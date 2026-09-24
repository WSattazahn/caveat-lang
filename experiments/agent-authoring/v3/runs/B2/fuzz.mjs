// Randomized check of pond.cav against an independent JavaScript model of
// TASK.md as amended by CHANGE.md. Usage: node fuzz.mjs [runs] [seed]
// For each run it sends a random event sequence (valid and invalid), compares
// the display, sequence, grounds and journal after every event, and part-way
// through saves, restores and continues on the restored session while the
// original keeps receiving the same events; both must agree with the model.
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const RUNS = Number(process.argv[2] ?? 500);
let seed = Number(process.argv[3] ?? 12345);
const rand = () => { seed = (seed * 1103515245 + 12345) % 2147483648; return seed / 2147483648; };
const pick = (xs) => xs[Math.floor(rand() * xs.length)];

const ORIGINAL = process.env.ORIGINAL === '1'; // check the pre-change program (no scan event, no scans property)
const source = await readFile(new URL(ORIGINAL ? './pond.original.cav' : './pond.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();

function newModel() {
  return { meas: [], scans: [], cracked: false, observed: [], decisions: [], journal: [], sequence: 0 };
}
const current = (m) => m.decisions[m.decisions.length - 1];
const isOpen = (m) => { const d = current(m); return d && !d.reopened && d.value >= 10; };
const allReadings = (m) => [...m.meas, ...m.scans].map((r) => r.value);

// Returns 'accepted' or the refusal kind; mutates the model only when accepted.
function modelDispatch(m, event, payload) {
  const keys = payload && typeof payload === 'object' && !Array.isArray(payload) ? Object.keys(payload) : null;
  if (!['measure', 'scan', 'crack', 'decide'].includes(event)) return 'input';
  if (ORIGINAL && event === 'scan') return 'input';
  if (keys === null) return 'input';
  if (event === 'measure' || event === 'scan') {
    if (keys.length !== 1 || keys[0] !== 'cm') return 'input';
    const cm = payload.cm;
    if (typeof cm !== 'number' || !Number.isFinite(cm) || cm < 0 || cm > 60) return 'input';
    const list = event === 'measure' ? m.meas : m.scans;
    if (list.length >= 8) return 'policy';
    const id = `${event === 'measure' ? 'thickness' : 'sonar_thickness'}@${list.length + 1}`;
    const caveat = event === 'measure' ? 'single_hole' : 'uncalibrated_sonar';
    const wasOpen = isOpen(m);
    list.push({ id, value: cm, caveat });
    m.observed.push(id);
    m.sequence++;
    if (cm < 10 && wasOpen) {
      const d = current(m);
      d.reopened = true;
      m.journal.push({ commitment: d.id, change: 'reopened', because: [id], caveats: [caveat], value: d.value, sequence: m.sequence });
    }
    return 'accepted';
  }
  if (keys.length !== 0) return 'input';
  if (event === 'crack') {
    if (m.cracked) return 'policy';
    const wasOpen = isOpen(m);
    m.cracked = true;
    m.observed.push('crack_report');
    m.sequence++;
    if (wasOpen) {
      const d = current(m);
      d.reopened = true;
      m.journal.push({ commitment: d.id, change: 'reopened', because: ['crack_report'], caveats: ['secondhand'], value: d.value, sequence: m.sequence });
    }
    return 'accepted';
  }
  // decide
  if (m.meas.length + m.scans.length < 3) return 'policy';
  const d0 = current(m);
  if (d0 && !d0.reopened) return 'policy';
  if (m.decisions.length >= 3) throw new Error('model reached a fourth decision');
  const value = Math.min(...allReadings(m));
  const readings = [...m.meas, ...m.scans];
  const ids = m.observed.filter((x) => x !== 'crack_report');
  const caveats = [...new Set(readings.map((r) => r.caveat))].sort();
  const d = { id: `rink@${m.decisions.length + 1}`, value, reopened: false, grounds: { evidence: [...ids].sort(), caveats } };
  m.decisions.push(d);
  m.sequence++;
  m.journal.push({ commitment: d.id, change: 'committed', because: ids, caveats, value, sequence: m.sequence });
  return 'accepted';
}

function expectedHud(m) {
  const d = current(m);
  const all = allReadings(m);
  const hud = {
    readings: m.meas.length,
    scans: m.scans.length,
    thinnest: all.length ? Math.min(...all) : -1,
    cracked: m.cracked ? 1 : 0,
    decision: !d ? 'none' : d.reopened ? 'review' : d.value >= 10 ? 'open' : 'closed',
    frozen: d ? d.value : -1,
    revision: m.decisions.length,
  };
  if (ORIGINAL) delete hud.scans;
  return hud;
}

const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const sortKeys = (o) => Object.fromEntries(Object.entries(o).sort(([a], [b]) => a.localeCompare(b)));

function compare(m, snap, where) {
  const problems = [];
  if (!same(sortKeys(snap.bindings.hud), sortKeys(expectedHud(m)))) problems.push(`hud ${JSON.stringify(snap.bindings.hud)} vs ${JSON.stringify(expectedHud(m))}`);
  if (Object.keys(snap.bindings).length !== 1) problems.push(`extra binding targets ${Object.keys(snap.bindings)}`);
  if (snap.sequence !== m.sequence) problems.push(`sequence ${snap.sequence} vs ${m.sequence}`);
  for (const d of m.decisions) {
    const g = snap.commitment_grounds[d.id];
    if (!g || !same([...g.evidence].sort(), d.grounds.evidence) || !same([...g.caveats].sort(), d.grounds.caveats)) problems.push(`grounds ${d.id} ${JSON.stringify(g)} vs ${JSON.stringify(d.grounds)}`);
  }
  if (Object.keys(snap.commitment_grounds).length !== m.decisions.length) problems.push('extra commitment grounds');
  const j = snap.decision_journal.map((e) => ({ commitment: e.commitment, change: e.change, because: e.because, caveats: [...e.caveats].sort(), value: e.value, sequence: e.sequence }));
  if (!same(j, m.journal)) problems.push(`journal ${JSON.stringify(j)} vs ${JSON.stringify(m.journal)}`);
  if (problems.length) throw new Error(`${where}: ${problems.join('; ')}`);
}

function randomStep() {
  const r = rand();
  const cm = () => pick([0, 5, 9.99, 10, 10.5, 12, 20, 45, 60, Math.round(rand() * 6000) / 100]);
  if (r < 0.33) return ['measure', { cm: cm() }];
  if (r < 0.66) return ['scan', { cm: cm() }];
  if (r < 0.74) return ['crack', {}];
  if (r < 0.92) return ['decide', {}];
  return pick([
    ['measure', { cm: 60.01 }], ['scan', { cm: -1 }], ['scan', {}], ['measure', { cm: 5, extra: 1 }],
    ['scan', { cm: '12' }], ['crack', { cm: 3 }], ['decide', { x: 1 }], ['drill', { cm: 3 }], ['scan', null],
  ]);
}

let events = 0, accepted = 0, maxDecisions = 0, reopenedByScan = 0;
for (let run = 0; run < RUNS; run++) {
  const m = newModel();
  let sessions = [runtime.open(source)];
  compare(m, sessions[0].snapshot(), `run ${run} initial`);
  const length = 5 + Math.floor(rand() * 40);
  const resumeAt = Math.floor(rand() * length);
  const trace = [];
  for (let i = 0; i < length; i++) {
    if (i === resumeAt) {
      const saved = sessions[sessions.length - 1].save();
      const restored = runtime.restore(source, saved);
      compare(m, restored.snapshot(), `run ${run} after restore`);
      sessions.push(restored);
    }
    const [event, payload] = randomStep();
    trace.push([event, payload]);
    const before = sessions.map((s) => s.save());
    const want = modelDispatch(m, event, payload);
    events++;
    for (const [k, s] of sessions.entries()) {
      let got;
      try {
        const out = s.dispatch(event, payload);
        got = out.outcome === 'accepted' ? 'accepted' : out.origin;
      } catch (err) {
        got = `threw ${err.kind}`;
      }
      if (got !== want) throw new Error(`run ${run} step ${i} session ${k}: ${event} ${JSON.stringify(payload)} got ${got}, model ${want}\ntrace ${JSON.stringify(trace)}`);
      if (got !== 'accepted' && s.save() !== before[k]) throw new Error(`run ${run} step ${i}: refused event changed the save`);
      compare(m, s.snapshot(), `run ${run} step ${i} session ${k} (${JSON.stringify(trace)})`);
    }
    if (want === 'accepted') accepted++;
    const last = m.journal[m.journal.length - 1];
    if (want === 'accepted' && event === 'scan' && last && last.change === 'reopened' && last.because[0].startsWith('sonar')) reopenedByScan++;
  }
  if (sessions.length === 2 && !same(sessions[0].snapshot(), sessions[1].snapshot())) throw new Error(`run ${run}: original and restored sessions diverged`);
  maxDecisions = Math.max(maxDecisions, m.decisions.length);
  for (const s of sessions) s.close();
}
console.log(`OK: ${RUNS} runs, ${events} events (${accepted} accepted), max decisions ${maxDecisions}, reopenings by scan ${reopenedByScan}`);
