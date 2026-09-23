import assert from 'node:assert/strict';

export function assertProjection(task, state, raw) {
  assert.deepEqual(task.projectActual(raw), task.project(state), 'domain contract differs');
}

export function assertAdmission(actual, expected) {
  assert.equal(actual, expected, 'event acceptance differs');
}

export function assertGrounds(snapshot) {
  const checkSubset = (grounds, lineage, label) => {
    for (const key of ['evidence', 'caveats']) {
      for (const name of grounds[key]) assert(lineage[key].includes(name), `${label}: ${key} ${name} is outside lineage`);
    }
  };
  for (const [name, grounds] of Object.entries(snapshot.value_grounds)) {
    checkSubset(grounds, snapshot.qualified_values[name].provenance, `state ${name}`);
  }
  for (const [name, grounds] of Object.entries(snapshot.commitment_grounds)) {
    checkSubset(grounds, snapshot.commitment_bases[name].provenance, `decision ${name}`);
  }
}

export function oracleStep(task, state, event) {
  const before = structuredClone(state);
  const result = task.step(state, event);
  assert.deepEqual(state, before, 'oracle mutated its input state');
  assert.equal(typeof result.accepted, 'boolean');
  if (!result.accepted) assert.deepEqual(result.state, before, 'oracle rejection mutated state');
  return result;
}

export function seeded(seed) {
  let state = seed >>> 0;
  return () => {
    state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
    return (state >>> 0) / 4294967296;
  };
}

export const SEEDS = { A: 0xa17001, B: 0xb17001, C: 0xc17001 };
export function casesFor(task, id) {
  const rng = seeded(SEEDS[id]);
  return [...task.cases.map(test => ({ ...test, kind: 'held-out' })),
    ...Array.from({ length: 100 }, (_, index) => ({ id: `seed-${SEEDS[id].toString(16)}-${index + 1}`, kind: 'seeded', events: task.randomEvents(rng) }))];
}
