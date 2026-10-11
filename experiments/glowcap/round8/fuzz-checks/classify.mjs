// Post-hoc: sorts every divergence in an all-divergences.mjs dump into
// classes. order: the same evidence in another order (a decision's basis, or
// a journal entry's because). timer: a countdown one second off. session: the
// program's policy could not be created or resumed, or its view threw.
// Anything else is
// listed as other, with both values.
//
//   node fuzz-checks/classify.mjs DUMP.json > results/fuzz-classes.json
import { readFileSync } from 'node:fs';

const sameSet = (a, b) => Array.isArray(a) && Array.isArray(b) && a.length === b.length
  && JSON.stringify([...a].map((x) => JSON.stringify(x)).sort()) === JSON.stringify([...b].map((x) => JSON.stringify(x)).sort());
const journalOrderOnly = (a, b) => Array.isArray(a) && Array.isArray(b) && a.length === b.length
  && a.every((e, i) => JSON.stringify({ ...e, because: [...(e.because ?? [])].sort() }) === JSON.stringify({ ...b[i], because: [...(b[i].because ?? [])].sort() }));

function classify({ difference: { at, first, other } }) {
  // A view that threw is compared as { error }, so its fields read undefined.
  if (other === 'error' || other === undefined) return 'session';
  if (at === 'decision.basis' && sameSet(first, other)) return 'order';
  if (at === 'journal' && journalOrderOnly(first, other)) return 'order';
  if (/\.(regrowsIn|fadesIn)$/.test(at) && typeof first === 'number' && Math.abs(first - other) === 1) return 'timer';
  return 'other';
}

const out = {};
for (const r of JSON.parse(readFileSync(process.argv[2], 'utf8'))) {
  const classes = {};
  const other = [];
  for (const d of r.all) {
    const c = classify(d);
    classes[c] = (classes[c] ?? 0) + 1;
    if (c === 'other') other.push({ sequence: d.sequence, at: d.difference.at, first: d.difference.first, other: d.difference.other, lastEvents: d.history.slice(-4) });
  }
  out[r.entry.split('/').at(-3)] = { diverged: r.diverged, classes, other };
}
console.log(JSON.stringify(out, null, 1));
