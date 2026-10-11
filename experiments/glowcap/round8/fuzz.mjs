// The round-8 differential fuzz, registered before any author starts
// (PROTOCOL.md, "Registration"). It is ../round7/fuzz.mjs with CR18-CR20's
// events added: doubt and undoubt (an evidence id, mostly a witness the
// sequence may have seen) and retaste. CR17's fields need no event. It replays
// seeded random sequences through two or more finished programs and reports
// where each disagrees with the first about accepting an event or about the
// view, comparing like round 7's: decisions in order, each journal entry's
// caveats as a set.
//
//   node fuzz.mjs --impl=PATH --impl=PATH [--impl=PATH…] [--sequences=300] [--length=40] [--seed=7] [--dump=FILE]
//
// PATH is a program's entry (an adapter.mjs or a glowcap.ts).
import { writeFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { normaliseView } from '../compare-views.mjs';

const values = (name) => process.argv.filter((a) => a.startsWith(`--${name}=`)).map((a) => a.slice(name.length + 3));
const arg = (name, fallback) => Number(values(name)[0] ?? fallback);
const ENTRIES = values('impl');
if (ENTRIES.length < 2) throw new Error('give at least two --impl=PATH');
const SEQUENCES = arg('sequences', 300);
const LENGTH = arg('length', 40);
let seed = arg('seed', 7) >>> 0;

// mulberry32, as in ../differential.mjs
function random() {
  seed = (seed + 0x6d2b79f5) >>> 0;
  let t = seed;
  t = Math.imul(t ^ (t >>> 15), t | 1);
  t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
}
const pick = (list) => list[Math.floor(random() * list.length)];

const IDS = ['cave', 'pool', 'ruin', 'grove', 'pit', 'cave', 'pool', 'ruin', 'grove', 'pit', 'nowhere'];
const KINDS = ['glowcap', 'duskcap', 'glowcap', 'duskcap', 'bluecap'];
// Witness ids for lives 1-3 of every mushroom, plus ids no doubt may name.
const EVIDENCE = [
  ...['cave', 'pool', 'ruin', 'grove', 'pit'].flatMap((id) => [`witness_${id}`, `witness_${id}`, `witness_${id}_2`, `witness_${id}_3`]),
  'absorb_cave', 'taste_pool', 'witness_nowhere', '',
];

function randomStep() {
  const roll = random();
  if (roll < 0.06) return 'resume';
  if (roll < 0.17) return [{ type: 'witness', id: pick(IDS), kind: pick(KINDS) }];
  if (roll < 0.24) return Array.from({ length: pick([720, 240]) }, () => ({ type: 'tick', dt: 0.0625 }));
  if (roll < 0.30) return [{ type: pick(['mark', 'mark', 'unmark']), id: pick(IDS) }];
  if (roll < 0.37) return [{ type: pick(['doubt', 'doubt', 'undoubt']), evidence: pick(EVIDENCE) }];
  if (roll < 0.42) return [{ type: 'retaste', id: pick(IDS), kind: pick(KINDS) }];
  if (roll < 0.60) return [{ type: 'absorb', id: pick(IDS), kind: pick(KINDS) }];
  if (roll < 0.78) return [{ type: 'taste', id: pick(IDS), kind: pick(KINDS) }];
  if (roll < 0.88) return [{ type: 'tick', dt: pick([0, 0.05, 0.0625, 0.1, 0.2, -0.01]) }];
  const ticks = pick([16, 160, 320, 480, 496]);
  return Array.from({ length: ticks }, () => ({ type: 'tick', dt: 0.0625 }));
}

// ../compare-views.mjs plus the journal: entries in order, caveats a set.
function normalise(view) {
  const out = normaliseView({ ...view, journal: undefined });
  if (Array.isArray(view?.journal)) {
    out.journal = view.journal.map((entry) => normaliseView({ ...entry, caveats: Array.isArray(entry?.caveats) ? [...entry.caveats].sort() : entry?.caveats }));
  }
  return out;
}

function firstDifference(a, b, at = '') {
  if (JSON.stringify(a) === JSON.stringify(b)) return null;
  if (a && b && typeof a === 'object' && typeof b === 'object' && !Array.isArray(a) && !Array.isArray(b)) {
    for (const key of new Set([...Object.keys(a), ...Object.keys(b)])) {
      const found = firstDifference(a[key], b[key], at ? `${at}.${key}` : key);
      if (found) return found;
    }
  }
  return { at, first: a, other: b };
}

function attempt(policy, event) {
  try {
    policy.dispatch(event);
    return 'accepted';
  } catch {
    return 'rejected';
  }
}

const factories = [];
for (const entry of ENTRIES) {
  const module = await import(pathToFileURL(path.resolve(entry)).href);
  if (module.ready) await module.ready;
  factories.push(module.createPolicy);
}

let events = 0;
const report = ENTRIES.slice(1).map((entry) => ({ entry, diverged: 0, kinds: {}, example: null }));
for (let sequence = 0; sequence < SEQUENCES; sequence += 1) {
  const live = ENTRIES.map((_, i) => i);
  let policies = factories.map((make) => { try { return make(); } catch (error) { return { error }; } });
  const history = [];
  for (let step = 0; step < LENGTH && live.length > 1; step += 1) {
    const burst = randomStep();
    if (burst === 'resume') {
      history.push({ type: 'resume' });
      policies = policies.map((p, i) => {
        if (!live.includes(i)) return p;
        try {
          const next = factories[i](JSON.parse(JSON.stringify(p.save())));
          p.free?.();
          return next;
        } catch (error) { return { error }; }
      });
    }
    const outcomes = policies.map(() => []);
    for (const event of burst === 'resume' ? [] : burst) {
      history.push(event);
      events += 1;
      for (const i of live) outcomes[i].push(policies[i].error ? 'error' : attempt(policies[i], event));
    }
    const views = policies.map((p, i) => {
      if (!live.includes(i) || p.error) return { error: String(p.error?.message ?? 'error') };
      try { return normalise(p.view()); } catch (error) { return { error: String(error?.message ?? error) }; }
    });
    for (const i of [...live].slice(1)) {
      let difference = null;
      const at = outcomes[i].findIndex((o, k) => o !== outcomes[0][k]);
      if (at >= 0) difference = { at: 'accept', first: outcomes[0][at], other: outcomes[i][at] };
      difference ??= firstDifference(views[0], views[i]);
      if (difference) {
        const r = report[i - 1];
        r.diverged += 1;
        r.kinds[difference.at] = (r.kinds[difference.at] ?? 0) + 1;
        r.example ??= { sequence, difference, history: [...history] };
        live.splice(live.indexOf(i), 1);
      }
    }
  }
  for (const p of policies) p.free?.();
}

console.log(`${SEQUENCES} sequences, ${events} events; each program against ${ENTRIES[0]}`);
for (const r of report) console.log(`${r.entry}: ${r.diverged} diverged ${JSON.stringify(r.kinds)}`);
const dump = values('dump')[0];
if (dump) writeFileSync(dump, `${JSON.stringify(report, null, 1)}\n`);
