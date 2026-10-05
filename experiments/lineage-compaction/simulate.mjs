// Simulation behind spec/caveat-lineage-compaction-0.1.md. Not a runtime
// measurement: it plays round 7's C3 program with windows for 60 longplay
// cycles on an installed caveat-lang package, then rewrites each raw save as
// if every retired record had departed, in two marker shapes, and prints the
// raw save bytes.
//
//   node experiments/lineage-compaction/simulate.mjs --package=DIR
//
// DIR is an unpacked caveat-lang package (for rc.15:
// `npm pack caveat-lang@0.1.0-rc.15 && tar xzf caveat-lang-0.1.0-rc.15.tgz`,
// then DIR is ./package). The C3 copy gets `renewable … window 2` in place of
// `limit 64` and `journal window 6`; the cycles are longplay.mjs's.
import { cpSync, mkdirSync, mkdtempSync, readFileSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { inflateRawSync } from 'node:zlib';

const here = path.dirname(fileURLToPath(import.meta.url));
const packageDir = process.argv.find((a) => a.startsWith('--package='))?.slice(10);
if (!packageDir) throw new Error('usage: simulate.mjs --package=DIR');

const work = mkdtempSync(path.join(tmpdir(), 'lineage-compaction-'));
mkdirSync(path.join(work, 'node_modules'));
symlinkSync(path.resolve(packageDir), path.join(work, 'node_modules', 'caveat-lang'), 'dir');
const impl = path.join(work, 'impl');
cpSync(path.join(here, '../glowcap/round7/runs/C3/impl'), impl, { recursive: true });
const cav = path.join(impl, 'glowcap.cav');
const source = readFileSync(cav, 'utf8')
  .replace(/renewable (\w+_\$m) limit 64;/g, 'renewable $1 window 2;')
  .replace('decisions trust limit 256;', 'decisions trust limit 256;\njournal window 6;');
writeFileSync(cav, source);

const adapter = await import(pathToFileURL(path.join(impl, 'adapter.mjs')).href);
const policy = adapter.createPolicy();
const ids = ['cave', 'pool', 'ruin', 'grove', 'pit'];
const saves = {};
const adapterBytes = {};
for (let cycle = 1; cycle <= 60; cycle += 1) {
  ids.forEach((m, i) => {
    policy.dispatch({ type: 'witness', id: m, kind: (cycle + i) % 2 ? 'glowcap' : 'duskcap' });
  });
  for (let t = 0; t < 46 * 16; t += 1) policy.dispatch({ type: 'tick', dt: 0.0625 });
  if ([10, 30, 60].includes(cycle)) {
    const packed = policy.save();
    adapterBytes[cycle] = Buffer.byteLength(JSON.stringify(packed));
    saves[cycle] = JSON.parse(inflateRawSync(Buffer.from(packed, 'base64')).toString('utf8'));
  }
}

const bytes = (value) => Buffer.byteLength(JSON.stringify(value));
const historyOf = (name) => name.split('@')[0];
const numberOf = (name) => (name.includes('@') ? Number(name.split('@')[1]) : 1);

// Every retired record departs. A provenance keeps its other names and, for
// the departed ones, either one marker per record or one per history.
function depart(save, shape) {
  const s = structuredClone(save);
  const retired = s.retired ?? {};
  const gone = (name) => Object.hasOwn(retired, name);
  const compact = (provenance) => {
    if (!provenance?.evidence) return provenance;
    const departed = provenance.evidence.filter(gone);
    const out = { ...provenance, evidence: provenance.evidence.filter((n) => !gone(n)) };
    if (!departed.length) return out;
    if (shape === 'record') {
      out.departed = departed.map((name) => ({ name, departed_at: retired[name] }));
    } else {
      const markers = new Map();
      for (const name of departed) {
        const history = historyOf(name);
        const m = markers.get(history) ?? { history, from: Infinity, through: 0, departed_at: 0 };
        m.from = Math.min(m.from, numberOf(name));
        m.through = Math.max(m.through, numberOf(name));
        m.departed_at = Math.max(m.departed_at, retired[name]);
        markers.set(history, m);
      }
      out.departed = [...markers.values()];
    }
    return out;
  };
  // Every provenance in the save: lineages, grounds, bases, selection and
  // reopening qualifications, journal entries.
  const walk = (value) => {
    if (Array.isArray(value)) return value.map(walk);
    if (!value || typeof value !== 'object') return value;
    const out = Object.fromEntries(Object.entries(value).map(([k, v]) => [k, walk(v)]));
    return Array.isArray(out.evidence) ? compact(out) : out;
  };
  s.graph.nodes = s.graph.nodes.filter((node) => !gone(node.name));
  s.graph.relations = s.graph.relations.filter(([from, , to]) => !gone(from) && !gone(to));
  for (const key of ['observation_qualifications', 'reopening_qualifications']) {
    if (s[key]) s[key] = Object.fromEntries(Object.entries(s[key]).filter(([name]) => !gone(name)));
  }
  s.decision_journal = s.decision_journal.filter((_, i) => !gone(`journal@${i + 1}`));
  const strip = (value) => {
    if (Array.isArray(value)) return value.filter((v) => !(typeof v === 'string' && gone(v))).map(strip);
    if (value && typeof value === 'object') return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, strip(v)]));
    return value;
  };
  s.renewals = strip(s.renewals);
  for (const key of Object.keys(s)) if (key !== 'graph' && key !== 'retired') s[key] = walk(s[key]);
  const counts = {};
  for (const name of Object.keys(retired)) counts[historyOf(name)] = (counts[historyOf(name)] ?? 0) + 1;
  s.retired = counts;
  return s;
}

const rows = {};
for (const [cycle, save] of Object.entries(saves)) {
  const perHistory = depart(save, 'history');
  rows[cycle] = {
    adapterBytes: adapterBytes[cycle],
    raw: bytes(save),
    perRecord: bytes(depart(save, 'record')),
    perHistory: bytes(perHistory),
    perHistoryParts: Object.fromEntries(Object.entries(perHistory).map(([k, v]) => [k, bytes(v)]).filter(([, n]) => n > 100)),
  };
}
console.log(JSON.stringify(rows, null, 2));
