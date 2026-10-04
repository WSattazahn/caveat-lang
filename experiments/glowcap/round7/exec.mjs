// Scenario execution for round 7. compare(), canonical(), the save limit and
// runScenario() are copied from ../harness.mjs at main 4500dec without change
// in behaviour (that file runs on import, so it cannot be imported). One
// addition: a step for an event other than absorb, taste, witness or tick may
// be written [type, payloadObject]. This file imports nothing, so the author
// runner can use it without seeing any scenario that is not yet released.

const EXPLANATION_KEYS = new Set(['because', 'supportedBy', 'contradictedBy', 'basis', 'reopenedBy', 'caveats', 'history']);

export function toEvent(step) {
  const [type, a, b] = step;
  if (a && typeof a === 'object') return { type, ...a };
  return type === 'tick' ? { type, dt: a } : { type, id: a, kind: b };
}

const SAVE_LIMIT = 4096;

export function canonical(value) {
  const sort = (v) => (Array.isArray(v) ? v.map(sort)
    : v && typeof v === 'object' ? Object.fromEntries(Object.keys(v).sort().map((k) => [k, sort(v[k])])) : v);
  return JSON.stringify(sort(value));
}

function compare(actual, expected, at, failures) {
  for (const [key, want] of Object.entries(expected)) {
    const got = actual?.[key];
    const where = at ? `${at}.${key}` : key;
    if (Array.isArray(want)) {
      const same = Array.isArray(got) && JSON.stringify([...got].sort()) === JSON.stringify([...want].sort());
      if (!same) failures.push({ where, want, got, explanation: EXPLANATION_KEYS.has(key) });
    } else if (want && typeof want === 'object') {
      compare(got, want, where, failures);
    } else if (got !== want) {
      failures.push({ where, want, got, explanation: false });
    }
  }
}

export function runScenario(createPolicy, scenario) {
  const failures = [];
  let policy;
  try {
    policy = createPolicy();
  } catch (error) {
    return [{ step: 0, where: 'createPolicy', want: 'a policy', got: String(error?.message ?? error), explanation: false }];
  }
  scenario.steps.forEach((step, index) => {
    if (failures.length) return; // the first failure makes later steps meaningless
    const tag = (list) => list.map((f) => ({ step: index + 1, ...f }));
    try {
      if (step[0] === 'expect') {
        const found = [];
        compare(policy.view(), step[1], '', found);
        failures.push(...tag(found));
      } else if (step[0] === 'resume') {
        const text = JSON.stringify(policy.save());
        const next = createPolicy(JSON.parse(text));
        const bytes = Buffer.byteLength(text);
        if (bytes > SAVE_LIMIT) failures.push(...tag([{ where: 'save', want: `at most ${SAVE_LIMIT} bytes`, got: `${bytes} bytes`, explanation: false }]));
        else if (canonical(next.view()) !== canonical(policy.view())) failures.push(...tag([{ where: 'resume', want: canonical(policy.view()), got: canonical(next.view()), explanation: false }]));
        policy.free?.();
        policy = next;
      } else if (step[0] === 'reject') {
        const before = JSON.stringify(policy.view());
        let threw = false;
        try { policy.dispatch(step[1]); } catch { threw = true; }
        if (!threw) failures.push(...tag([{ where: 'reject', want: 'rejected', got: `accepted ${JSON.stringify(step[1])}`, explanation: false }]));
        else if (JSON.stringify(policy.view()) !== before) failures.push(...tag([{ where: 'reject', want: 'view unchanged', got: 'view changed', explanation: false }]));
      } else {
        const times = step[0] === 'tick' && typeof step[1] !== 'object' ? (step[2] ?? 1) : 1;
        for (let i = 0; i < times; i += 1) policy.dispatch(toEvent(step));
      }
    } catch (error) {
      failures.push(...tag([{ where: step[0], want: 'accepted', got: String(error?.message ?? error), explanation: false }]));
    }
  });
  policy?.free?.();
  return failures;
}
