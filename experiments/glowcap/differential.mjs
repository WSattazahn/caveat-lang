// Post-hoc check, not part of the pre-registered protocol: replay seeded random
// event sequences through both implementations and report every point where
// they disagree about accepting an event or about the resulting view. The
// scenarios cover the cases their author thought of; this covers the rest.
//
//   node experiments/glowcap/differential.mjs [--sequences=2000] [--length=40] [--seed=1]
// Same entry points as harness.mjs (importing it would run its test mode).
const IMPLEMENTATIONS = { ts: { entry: 'ts/glowcap.ts' }, caveat: { entry: 'caveat/adapter.mjs' } };

const arg = (name, fallback) => Number(process.argv.find((a) => a.startsWith(`--${name}=`))?.split('=')[1] ?? fallback);
const SEQUENCES = arg('sequences', 2000);
const LENGTH = arg('length', 40);
let seed = arg('seed', 1) >>> 0;

// mulberry32: small, seedable, reproducible
function random() {
  seed = (seed + 0x6d2b79f5) >>> 0;
  let t = seed;
  t = Math.imul(t ^ (t >>> 15), t | 1);
  t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
}
const pick = (list) => list[Math.floor(random() * list.length)];

const IDS = ['cave', 'pool', 'ruin', 'grove', 'cave', 'pool', 'ruin', 'grove', 'nowhere'];
const KINDS = ['glowcap', 'duskcap', 'glowcap', 'duskcap', 'bluecap'];

// A step is one event, or a burst of ticks long enough to cross the timers.
function randomStep() {
  const roll = random();
  if (roll < 0.35) return [{ type: 'absorb', id: pick(IDS), kind: pick(KINDS) }];
  if (roll < 0.7) return [{ type: 'taste', id: pick(IDS), kind: pick(KINDS) }];
  if (roll < 0.8) return [{ type: 'tick', dt: pick([0, 0.05, 0.0625, 0.1, 0.2, -0.01]) }];
  const ticks = pick([16, 160, 320, 480, 496]); // 1, 10, 20, 30, 31 seconds
  return Array.from({ length: ticks }, () => ({ type: 'tick', dt: 0.0625 }));
}

function normalise(value) {
  if (Array.isArray(value)) return [...value].sort();
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map((key) => [key, normalise(value[key])]));
  }
  return value;
}

function firstDifference(a, b, at = '') {
  if (JSON.stringify(a) === JSON.stringify(b)) return null;
  if (a && b && typeof a === 'object' && typeof b === 'object' && !Array.isArray(a)) {
    for (const key of new Set([...Object.keys(a), ...Object.keys(b)])) {
      const found = firstDifference(a[key], b[key], at ? `${at}.${key}` : key);
      if (found) return found;
    }
  }
  return { at, ts: a, caveat: b };
}

function attempt(policy, event) {
  try {
    policy.dispatch(event);
    return 'accepted';
  } catch {
    return 'rejected';
  }
}

const factories = {};
for (const name of Object.keys(IMPLEMENTATIONS)) {
  const module = await import(new URL(IMPLEMENTATIONS[name].entry, import.meta.url));
  if (module.ready) await module.ready;
  factories[name] = module.createPolicy;
}

let events = 0;
const divergences = new Map();
for (let sequence = 0; sequence < SEQUENCES; sequence += 1) {
  const ts = factories.ts();
  const caveat = factories.caveat();
  const history = [];
  let diverged = false;
  for (let step = 0; step < LENGTH && !diverged; step += 1) {
    // Accept/reject is compared on every event; views once per step, since a
    // burst of ticks only matters where it ends.
    let difference = null;
    for (const event of randomStep()) {
      history.push(event);
      events += 1;
      const outcome = { ts: attempt(ts, event), caveat: attempt(caveat, event) };
      if (outcome.ts !== outcome.caveat) {
        difference = { at: 'accept', ts: outcome.ts, caveat: outcome.caveat };
        break;
      }
    }
    difference ??= firstDifference(normalise(ts.view()), normalise(caveat.view()));
    if (difference) {
      const entry = divergences.get(difference.at) ?? { count: 0, example: null };
      entry.count += 1;
      entry.example ??= { sequence, difference, history: [...history] };
      divergences.set(difference.at, entry);
      diverged = true;
    }
  }
  caveat.free?.();
}

console.log(`${SEQUENCES} sequences, ${events} events, ${divergences.size ? [...divergences.values()].reduce((n, d) => n + d.count, 0) : 0} diverged`);
for (const [key, { count, example }] of divergences) {
  console.log(`\n${key}: ${count} sequence(s)`);
  console.log(`  ts=${JSON.stringify(example.difference.ts)} caveat=${JSON.stringify(example.difference.caveat)}`);
  console.log(`  events (ticks elided): ${JSON.stringify(example.history.filter((e) => e.type !== 'tick'))}`);
}
process.exitCode = divergences.size ? 1 : 0;
