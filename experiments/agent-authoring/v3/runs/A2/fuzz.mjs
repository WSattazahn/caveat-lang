// Randomized differential test: pond.cav against a plain-JS model of TASK.md
// as amended by CHANGE.md. Usage: node fuzz.mjs [runs] [seed] [source]
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const runs = Number(process.argv[2] ?? 2000);
let seed = Number(process.argv[3] ?? 12345);
const file = process.argv[4] ?? 'pond.cav';
const withScans = !file.includes('original');
const source = await readFile(file, 'utf8');
const runtime = await loadRuntimeFromDirectory();

function rand() { seed = (seed * 1103515245 + 12345) % 2147483648; return seed / 2147483648; }
const pick = (xs) => xs[Math.floor(rand() * xs.length)];

function newModel() {
  return { auger: [], sonar: [], order: [], cracked: false, decisions: [], journal: [], seq: 0 };
}
const current = (m) => m.decisions[m.decisions.length - 1];
const isOpen = (m) => { const d = current(m); return d && !d.reopened && d.value >= 10; };
const allReadings = (m) => [...m.auger, ...m.sonar];

// Returns 'accepted' or 'rejected'; mutates the model only when accepted.
function step(m, event, payload) {
  if (event === 'measure' || event === 'scan') {
    const list = event === 'measure' ? m.auger : m.sonar;
    if (list.length >= 8) return 'rejected';
    const id = `${event === 'measure' ? 'thickness' : 'sonar_thickness'}@${list.length + 1}`;
    list.push(payload.cm);
    m.order.push(id);
    if (payload.cm < 10 && isOpen(m)) {
      const d = current(m); d.reopened = true;
      m.journal.push({ commitment: d.id, change: 'reopened', because: [id], value: d.value });
    }
  } else if (event === 'crack') {
    if (m.cracked) return 'rejected';
    m.cracked = true;
    if (isOpen(m)) {
      const d = current(m); d.reopened = true;
      m.journal.push({ commitment: d.id, change: 'reopened', because: ['crack_report'], value: d.value });
    }
  } else if (event === 'decide') {
    if (m.auger.length + m.sonar.length < 3) return 'rejected';
    const d = current(m);
    if (d && !d.reopened) return 'rejected';
    const value = Math.min(...allReadings(m));
    const nd = { id: `rink@${m.decisions.length + 1}`, value, reopened: false, grounds: [...m.order] };
    m.decisions.push(nd);
    m.journal.push({ commitment: nd.id, change: 'committed', because: [...m.order], value });
  }
  m.seq += 1;
  return 'accepted';
}

function expectedHud(m) {
  const d = current(m);
  const r = allReadings(m);
  const hud = {
    readings: m.auger.length,
    thinnest: r.length ? Math.min(...r) : -1,
    cracked: m.cracked ? 1 : 0,
    decision: !d ? 'none' : d.reopened ? 'review' : d.value >= 10 ? 'open' : 'closed',
    frozen: d ? d.value : -1,
    revision: m.decisions.length,
  };
  if (withScans) hud.scans = m.sonar.length;
  return hud;
}

const sortKeys = (o) => JSON.stringify(Object.fromEntries(Object.entries(o).sort()));
const cmValues = [0, 3, 9, 9.99, 10, 10.01, 12, 15, 25, 60];
function randomEvent() {
  const r = rand();
  if (r < 0.05) return pick([
    ['measure', { cm: 60.5 }], ['scan', { cm: -1 }], ['measure', {}], ['scan', { cm: 5, x: 1 }],
    ['crack', { cm: 1 }], ['decide', { a: 1 }], ['drill', {}], ['measure', { cm: '9' }],
  ]).concat(['input']);
  const events = withScans ? ['measure', 'scan', 'crack', 'decide', 'decide'] : ['measure', 'crack', 'decide', 'decide'];
  const e = pick(events);
  return [e, e === 'measure' || e === 'scan' ? { cm: pick(cmValues) } : {}, 'normal'];
}

let failures = 0, accepted = 0, rejected = 0, restores = 0;
const coverage = { reopen_thickness: 0, reopen_sonar: 0, reopen_crack: 0, third_revision: 0, closed_first: 0 };
for (let run = 0; run < runs && failures < 5; run++) {
  let session = runtime.open(source);
  const m = newModel();
  const log = [];
  const length = 5 + Math.floor(rand() * 40);
  for (let i = 0; i < length; i++) {
    if (rand() < 0.1) {
      const saved = session.save();
      const restored = runtime.restore(source, saved);
      if (JSON.stringify(restored.snapshot()) !== JSON.stringify(session.snapshot())) {
        failures++; console.log('RESTORE MISMATCH', run, log.join(' ')); break;
      }
      session.close(); session = restored; restores++; log.push('[resume]');
    }
    const [event, payload, kind] = randomEvent();
    log.push(`${event}${payload.cm !== undefined ? '(' + payload.cm + ')' : ''}`);
    const before = session.save();
    const outcome = session.dispatch(event, payload);
    const want = kind === 'input' ? 'rejected' : step(m, event, payload);
    if (outcome.outcome !== want || (kind === 'input' && outcome.origin !== 'input') || (kind === 'normal' && want === 'rejected' && outcome.origin !== 'policy')) {
      failures++; console.log('OUTCOME', run, log.join(' '), JSON.stringify(outcome).slice(0, 200), 'want', want); break;
    }
    if (outcome.outcome === 'rejected') {
      rejected++;
      if (session.save() !== before) { failures++; console.log('REJECTION CHANGED STATE', run, log.join(' ')); break; }
      continue;
    }
    accepted++;
    const s = outcome.snapshot;
    const hud = s.bindings.hud;
    const want2 = expectedHud(m);
    const problems = [];
    if (sortKeys(hud) !== sortKeys(want2)) problems.push(`hud ${sortKeys(hud)} != ${sortKeys(want2)}`);
    if (s.sequence !== m.seq) problems.push(`sequence ${s.sequence} != ${m.seq}`);
    const journal = s.decision_journal.map((e) => ({ commitment: e.commitment, change: e.change, because: e.because, value: e.value }));
    if (JSON.stringify(journal) !== JSON.stringify(m.journal)) problems.push(`journal ${JSON.stringify(journal)} != ${JSON.stringify(m.journal)}`);
    for (const d of m.decisions) {
      const g = s.commitment_grounds[d.id];
      const wantCaveats = [...new Set(d.grounds.map((id) => id.startsWith('sonar') ? 'uncalibrated_sonar' : 'single_hole'))].sort();
      if (!g || JSON.stringify([...g.evidence].sort()) !== JSON.stringify([...d.grounds].sort()) || JSON.stringify([...g.caveats].sort()) !== JSON.stringify(wantCaveats)) {
        problems.push(`grounds ${d.id} ${JSON.stringify(g)} != ${JSON.stringify(d.grounds)} ${wantCaveats}`);
      }
    }
    for (const e of s.decision_journal) {
      const wantC = e.change === 'reopened' && e.because[0] === 'crack_report' ? ['secondhand']
        : [...new Set(e.because.map((id) => id.startsWith('sonar') ? 'uncalibrated_sonar' : 'single_hole'))].sort();
      if (JSON.stringify([...e.caveats].sort()) !== JSON.stringify(wantC)) problems.push(`journal caveats ${JSON.stringify(e)}`);
    }
    if (problems.length) { failures++; console.log('MISMATCH', run, log.join(' '), '\n  ' + problems.join('\n  ')); break; }
  }
  for (const e of m.journal) if (e.change === 'reopened') coverage[e.because[0].startsWith('sonar') ? 'reopen_sonar' : e.because[0].startsWith('thickness') ? 'reopen_thickness' : 'reopen_crack']++;
  if (m.decisions.length === 3) coverage.third_revision++;
  if (m.decisions[0] && m.decisions[0].value < 10) coverage.closed_first++;
  session.close();
}
console.log('coverage', JSON.stringify(coverage));
console.log(`${file}: ${runs} runs, ${accepted} accepted, ${rejected} rejected, ${restores} restores, ${failures} failures`);
process.exit(failures ? 1 : 0);
