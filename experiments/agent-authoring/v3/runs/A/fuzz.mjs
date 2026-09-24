// Randomised differential test: pond.cav against an independent JS model of TASK.md,
// with save/restore at random points and atomicity checks on every refusal.
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const source = await readFile(new URL('./pond.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();

let seed = Number(process.argv[2] ?? 12345);
const rand = () => { seed = (seed * 1103515245 + 12345) & 0x7fffffff; return seed / 0x7fffffff; };
const pick = (xs) => xs[Math.floor(rand() * xs.length)];

function newModel() {
  return { readings: [], cracked: false, decisions: [], journal: [] };
}
function current(m) { return m.decisions[m.decisions.length - 1]; }
function isOpen(m) { const d = current(m); return d && !d.reopened && d.value >= 10; }

// Returns 'accepted', 'policy' or 'input'; mutates model only when accepted.
function modelStep(m, event, payload) {
  const keys = Object.keys(payload);
  if (event === 'measure') {
    if (keys.length !== 1 || keys[0] !== 'cm' || typeof payload.cm !== 'number') return 'input';
    if (!(payload.cm >= 0 && payload.cm <= 60)) return 'input';
    if (m.readings.length >= 8) return 'policy';
    m.readings.push(payload.cm);
    const id = `thickness@${m.readings.length}`;
    if (payload.cm < 10 && isOpen(m)) {
      current(m).reopened = true;
      m.journal.push({ commitment: current(m).id, change: 'reopened', because: [id], caveats: ['single_hole'] });
    }
    return 'accepted';
  }
  if (event === 'crack') {
    if (keys.length !== 0) return 'input';
    if (m.cracked) return 'policy';
    m.cracked = true;
    if (isOpen(m)) {
      current(m).reopened = true;
      m.journal.push({ commitment: current(m).id, change: 'reopened', because: ['crack_report'], caveats: ['secondhand'] });
    }
    return 'accepted';
  }
  if (event === 'decide') {
    if (keys.length !== 0) return 'input';
    if (m.readings.length < 3) return 'policy';
    const d = current(m);
    if (d && !d.reopened) return 'policy';
    const value = Math.min(...m.readings);
    const id = `rink@${m.decisions.length + 1}`;
    const because = m.readings.map((_, i) => `thickness@${i + 1}`);
    m.decisions.push({ id, value, reopened: false, because });
    m.journal.push({ commitment: id, change: 'committed', value, because, caveats: ['single_hole'] });
    return 'accepted';
  }
  return 'input';
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

function randomEvent() {
  const r = rand();
  if (r < 0.55) {
    const q = rand();
    if (q < THICK) return ['measure', { cm: 10 + Math.round(rand() * 5000) / 100 }];     // thick
    if (q < 0.75) return ['measure', { cm: Math.round(rand() * 999) / 100 }];         // thin
    if (q < 0.8) return ['measure', { cm: pick([0, 10, 9.999, 60, 10.0001]) }];
    if (q < 0.85) return ['measure', { cm: pick([-1, 60.01, 100, -0.0001]) }];
    if (q < 0.9) return ['measure', {}];
    if (q < 0.95) return ['measure', { cm: 12, extra: 1 }];
    return ['measure', { cm: pick(['12', null, true, [12]]) }];
  }
  if (r < 0.7) return rand() < 0.9 ? ['crack', {}] : ['crack', { cm: 1 }];
  if (r < 0.97) return rand() < 0.95 ? ['decide', {}] : ['decide', { x: 0 }];
  return [pick(['thaw', 'Measure', 'reopen']), {}];
}

const THICK = Number(process.env.THICK ?? 0.4);
const deepEq = (a, b) => JSON.stringify(a) === JSON.stringify(b);
let failures = 0;
const runs = Number(process.argv[3] ?? 300);
let totals = { accepted: 0, policy: 0, input: 0, restores: 0, maxRevision: 0 };

for (let run = 0; run < runs && failures < 5; run++) {
  const m = newModel();
  let sessions = [runtime.open(source)];
  const steps = 5 + Math.floor(rand() * 30);
  const log = [];
  try {
    for (let i = 0; i < steps; i++) {
      if (rand() < 0.15) {
        const saved = sessions[sessions.length - 1].save();
        const restored = runtime.restore(source, saved);
        if (!deepEq(restored.snapshot(), sessions[sessions.length - 1].snapshot())) throw new Error('restore changed the snapshot');
        sessions.push(restored);
        totals.restores++;
      }
      const [event, payload] = randomEvent();
      log.push([event, payload]);
      const expected = modelStep(m, event, payload);
      const outcomes = [];
      for (const s of sessions) {
        const beforeSave = s.save();
        const beforeSnap = JSON.stringify(s.snapshot());
        const o = s.dispatch(event, payload);
        const got = o.outcome === 'accepted' ? 'accepted' : o.origin;
        outcomes.push(got);
        if (got !== expected) throw new Error(`step ${i} ${event} ${JSON.stringify(payload)}: expected ${expected}, got ${got} ${o.code ?? ''} ${o.message ?? ''}`);
        if (got !== 'accepted') {
          if (s.save() !== beforeSave || JSON.stringify(s.snapshot()) !== beforeSnap) throw new Error(`step ${i}: refusal changed the session`);
        }
      }
      totals[expected]++;
      const snaps = sessions.map((s) => s.snapshot());
      for (const snap of snaps.slice(1)) if (!deepEq(snap, snaps[0])) throw new Error(`step ${i}: restored session diverged`);
      const snap = snaps[0];
      const hud = modelHud(m);
      if (!deepEq(snap.bindings, { hud: Object.fromEntries(Object.keys(snap.bindings.hud).sort().map((k) => [k, hud[k]])) }) || Object.keys(snap.bindings.hud).length !== 6)
        throw new Error(`step ${i}: hud ${JSON.stringify(snap.bindings)} vs model ${JSON.stringify(hud)}`);
      const journal = snap.decision_journal.map((e) => {
        const o = { commitment: e.commitment, change: e.change };
        if (e.change === 'committed') o.value = e.value;
        o.because = e.because; o.caveats = e.caveats;
        return o;
      });
      if (!deepEq(journal, m.journal)) throw new Error(`step ${i}: journal ${JSON.stringify(journal)} vs model ${JSON.stringify(m.journal)}`);
      for (const d of m.decisions) {
        const g = snap.commitment_grounds[d.id];
        if (!deepEq(g, { evidence: [...d.because].sort(), caveats: ['single_hole'] }) && !deepEq(g, { evidence: d.because, caveats: ['single_hole'] }))
          throw new Error(`step ${i}: grounds of ${d.id} ${JSON.stringify(g)}`);
      }
      totals.maxRevision = Math.max(totals.maxRevision, m.decisions.length);
    }
  } catch (err) {
    failures++;
    console.log(`run ${run} FAILED: ${err.message}`);
    console.log('  events:', JSON.stringify(log));
  } finally {
    for (const s of sessions) s.close();
  }
}
console.log(JSON.stringify(totals));
console.log(failures ? `${failures} failing runs` : `all ${runs} runs agree with the model`);
process.exit(failures ? 1 : 0);
