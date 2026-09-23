// Independent synthetic-domain specification. No interpreter imports or source access.
const clone = value => structuredClone(value);
const sorted = values => [...new Set(values)].sort();
const input = (event, payload = {}) => ({ event, payload });
const resume = () => ({ resume: true });
const seq = (id, ...events) => ({ id, events: events.flat() });
const pick = (rng, values) => values[Math.floor(rng() * values.length)];

function base() {
  return { sequence: 0, elapsed: 0, evidence: {}, observations: [], timers: [], decisions: [], journal: [], grounds: {} };
}

function valid(record, signatures) {
  if (!record || typeof record.event !== 'string' || !Object.hasOwn(signatures, record.event)) return false;
  const payload = record.payload;
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) return false;
  const signature = signatures[record.event];
  if (Object.keys(payload).length !== Object.keys(signature).length) return false;
  return Object.entries(signature).every(([name, [low, high, integral]]) =>
    Object.hasOwn(payload, name) && typeof payload[name] === 'number' && Number.isFinite(payload[name]) &&
    payload[name] >= low && payload[name] <= high && (!integral || Number.isInteger(payload[name])));
}

function occurrence(baseName, ordinal) { return ordinal === 1 ? baseName : `${baseName}@${ordinal}`; }
function observe(state, id, claim, relation, caveats = []) {
  state.evidence[id] = { caveats: sorted(caveats), order: state.observations.length };
  state.observations.push({ from: id, to: claim, relation });
}
function qualify(state, id, caveat) {
  state.evidence[id].caveats = sorted([...state.evidence[id].caveats, caveat]);
}
function age(state, dt) {
  state.elapsed += dt;
  const pending = [];
  for (const timer of state.timers) {
    if (state.elapsed - timer.observedAt >= timer.delay) qualify(state, timer.id, timer.caveat);
    else pending.push(timer);
  }
  state.timers = pending;
}
function head(state) { return state.decisions.at(-1); }
function observationOrder(state, evidence) {
  return sorted(evidence).sort((a, b) => state.evidence[a].order - state.evidence[b].order);
}
function commit(state, series, value, evidence, retained, event) {
  const id = `${series}@${state.decisions.length + 1}`;
  const caveats = sorted([...retained, ...evidence.flatMap(e => state.evidence[e].caveats)]);
  const grounds = { evidence: sorted(evidence), caveats };
  const decision = { id, series, value, grounds: clone(grounds), open: false, witnesses: [] };
  state.decisions.push(decision);
  state.grounds[id] = clone(grounds);
  state.journal.push({ decision: series, commitment: id, change: 'committed', sequence: state.sequence,
    event, elapsed: state.elapsed, value, because: observationOrder(state, evidence), caveats: [...caveats] });
}
function reopen(state, evidence, event) {
  const decision = head(state);
  const fresh = observationOrder(state, evidence.filter(id => !decision.witnesses.includes(id)));
  if (!fresh.length) return;
  decision.open = true;
  decision.witnesses.push(...fresh);
  state.journal.push({ decision: decision.series, commitment: decision.id, change: 'reopened',
    sequence: state.sequence, event, elapsed: state.elapsed, value: decision.value, because: fresh,
    caveats: sorted(fresh.flatMap(id => state.evidence[id].caveats)) });
}
function projection(state, hud) {
  const qualifications = Object.entries(state.evidence).flatMap(([id, evidence]) =>
    evidence.caveats.map(caveat => ({ from: caveat, to: id, relation: 'qualifies' })));
  qualifications.sort(compareRelation);
  return { hud, decision_journal: clone(state.journal), commitment_grounds: clone(state.grounds),
    observations: clone(state.observations), qualifications };
}
function compareRelation(a, b) {
  for (const key of ['from', 'to', 'relation']) {
    if (a[key] < b[key]) return -1;
    if (a[key] > b[key]) return 1;
  }
  return 0;
}
function actualProjector(claims) {
  return raw => {
    const relations = raw.relations ?? [];
    const triple = ({ from, to, relation }) => ({ from, to, relation });
    const observations = relations.filter(r => (r.relation === 'supports' || r.relation === 'opposes') && claims.includes(r.to)).map(triple);
    const observed = new Set(observations.map(r => r.from));
    const qualifications = relations.filter(r => r.relation === 'qualifies' && observed.has(r.to)).map(triple).sort(compareRelation);
    const journal = (raw.decision_journal ?? []).map(entry => Object.fromEntries(
      ['decision', 'commitment', 'change', 'sequence', 'event', 'elapsed', 'value', 'because', 'caveats'].map(key => [key, entry[key]])));
    return { hud: clone(raw.bindings?.hud ?? null), decision_journal: journal,
      commitment_grounds: clone(raw.commitment_grounds ?? {}), observations, qualifications };
  };
}
function transactional(signatures, apply) {
  return (state, record) => {
    if (record?.resume === true) return { accepted: true, state };
    if (!valid(record, signatures)) return { accepted: false, state };
    const next = clone(state);
    next.sequence++;
    if (!apply(next, record.event, record.payload)) return { accepted: false, state };
    return { accepted: true, state: next };
  };
}

const A = {
  initial: () => ({ ...base(), count: 0, temperature: 0, latest: null }),
  step: transactional({ read: { temperature: [-10, 20] }, decide: {}, advance: { dt: [0, 2] } }, (s, event, p) => {
    if (event === 'read') {
      if (s.count >= 4) return false;
      s.latest = occurrence('sensor_reading', ++s.count);
      s.temperature = p.temperature;
      observe(s, s.latest, 'dispatch_safe', p.temperature <= 4 ? 'supports' : 'opposes', ['calibration_uncertain']);
      s.timers.push({ id: s.latest, observedAt: s.elapsed, delay: 2, caveat: 'stale' });
    } else if (event === 'decide') {
      if (!s.latest || s.evidence[s.latest].caveats.includes('stale') || (head(s) && !head(s).open) || s.decisions.length >= 3) return false;
      commit(s, 'dispatch', s.temperature <= 4 ? 1 : 0, [s.latest], [], event);
    } else {
      age(s, p.dt);
      const decision = head(s);
      if (decision && !decision.open && s.evidence[decision.grounds.evidence[0]].caveats.includes('stale')) {
        reopen(s, decision.grounds.evidence, event);
      }
    }
    return true;
  }),
  project(s) {
    const decision = head(s);
    const stale = s.latest && s.evidence[s.latest].caveats.includes('stale');
    return projection(s, { readings: s.count, temperature: s.temperature,
      recommendation: !s.latest ? 'waiting' : stale ? 'stale' : s.temperature <= 4 ? 'release' : 'block',
      decision: !decision ? 'none' : decision.open ? 'review' : decision.value ? 'release' : 'block',
      frozen: decision?.value ?? -1, revision: s.decisions.length, elapsed: s.elapsed });
  },
  projectActual: actualProjector(['dispatch_safe']),
};
const read = temperature => input('read', { temperature });
const advance = dt => input('advance', { dt });
const decide = () => input('decide');
A.cases = [
  seq('empty-and-zero-clock', decide(), advance(0), resume(), decide()),
  seq('threshold-and-closed-decision', read(4), decide(), decide(), read(4.001), decide()),
  seq('block-and-frozen-number', read(20), decide(), read(-10), advance(1), resume()),
  seq('exact-expiry', read(3), decide(), advance(1), advance(1), advance(0), decide()),
  seq('old-basis-new-fresh-reading', read(2), decide(), advance(1), read(8), resume(), advance(1), decide()),
  seq('latest-expires-without-decision', read(-10), advance(2), decide(), read(4), decide()),
  seq('multiple-due-and-single-reopen', read(0), read(5), decide(), read(3), advance(2), advance(0), resume()),
  seq('all-four-readings-capacity', read(0), read(1), read(2), read(3), read(4), resume()),
  seq('three-revisions-and-fourth-rejected', read(1), decide(), advance(2), read(6), decide(), advance(2), read(2), decide(), advance(2), read(9), decide(), resume()),
  seq('atomic-input-rejection', read(2), decide(), input('read', {}), input('read', { temperature: 1, extra: 0 }), read(21), advance(-1), input('no_such_event'), advance(2)),
  seq('fractional-time-and-save-timer', advance(0.5), read(4.5), decide(), advance(0.5), resume(), advance(1), advance(0.5)),
  seq('reopened-replacement-remains-frozen', read(9), decide(), advance(2), read(-2), decide(), read(20), resume(), advance(2)),
  seq('binary64-fractional-timer-boundary', advance(0.1), advance(0.2), read(2), decide(), advance(1), resume(), advance(1), advance(0), advance(0.1)),
  seq('invalid-input-types', input('read', { temperature: '3' }), input('read', { temperature: null }), input('advance', null), input('read', { temperature: true }), read(3), decide()),
];
A.randomEvents = rng => {
  const events = [read(pick(rng, [-10, 0, 4, 4.01, 20])), decide(), advance(1), read(pick(rng, [2, 8])), resume(), advance(1), decide()];
  while (events.length < 40) events.push(pick(rng, [read(pick(rng, [-11, -10, 3.5, 4, 5, 20, 21])), advance(pick(rng, [-1, 0, 0.1, 0.5, 0.7, 1, 1.2, 2, 3])), decide(), resume(), input('read', {}), input('decide', { extra: 1 })]));
  return events;
};

const assertionIds = { 1: 'alpha_allow', 2: 'beta_allow', 3: 'alpha_deny', 4: 'beta_deny', 5: 'challenge' };
function activeAssertions(s) {
  return [1, 2, 3, 4].filter(code => s.evidence[assertionIds[code]] && !s.evidence[assertionIds[code]].caveats.some(c => c === 'expired' || c === 'revoked'));
}
function votes(s) {
  const active = activeAssertions(s);
  return { active, support: active.filter(code => code <= 2).length, opposition: active.filter(code => code >= 3).length };
}
const B = {
  initial: base,
  step: transactional({ submit: { code: [1, 5, true] }, revoke: { code: [1, 4, true] }, approve: {}, advance: { dt: [0, 2] } }, (s, event, p) => {
    if (event === 'submit') {
      const id = assertionIds[p.code];
      if (s.evidence[id]) return false;
      observe(s, id, p.code === 5 ? 'review_required' : 'access_allowed', p.code >= 3 && p.code <= 4 ? 'opposes' : 'supports', p.code === 5 ? [] : ['unverified_source']);
      if (p.code !== 5) s.timers.push({ id, observedAt: s.elapsed, delay: 2, caveat: 'expired' });
      if (p.code >= 3 && head(s)) reopen(s, [id], event);
    } else if (event === 'revoke') {
      const id = assertionIds[p.code];
      if (!s.evidence[id] || s.evidence[id].caveats.includes('revoked')) return false;
      qualify(s, id, 'revoked');
    } else if (event === 'approve') {
      const { active, support, opposition } = votes(s);
      if (!active.length || (head(s) && !head(s).open) || s.decisions.length >= 3) return false;
      commit(s, 'access', support > opposition ? 1 : 0, active.map(code => assertionIds[code]), [], event);
    } else age(s, p.dt);
    return true;
  }),
  project(s) {
    const { active, support, opposition } = votes(s);
    const decision = head(s);
    return projection(s, { support, opposition, verdict: !active.length ? 'undecided' : support > opposition ? 'allow' : 'deny',
      decision: !decision ? 'none' : decision.open ? 'review' : decision.value ? 'allow' : 'deny',
      frozen: decision?.value ?? -1, revision: s.decisions.length, elapsed: s.elapsed });
  },
  projectActual: actualProjector(['access_allowed', 'review_required']),
};
const submit = code => input('submit', { code });
const revoke = code => input('revoke', { code });
const approve = () => input('approve');
B.cases = [
  seq('empty-admission', approve(), revoke(1), advance(0), resume()),
  seq('contradiction-and-tie', submit(1), submit(3), approve(), resume()),
  seq('opposition-alone', submit(4), approve(), submit(2), approve()),
  seq('observation-order-not-alphabetical', submit(4), submit(2), submit(1), approve()),
  seq('new-opposition-reopens-and-support-does-not', submit(1), approve(), submit(2), approve(), submit(3), approve()),
  seq('multiple-reopening-witnesses', submit(2), approve(), submit(4), submit(3), submit(5), resume(), approve()),
  seq('revocation-changes-live-only', submit(1), approve(), revoke(1), approve(), submit(5), approve(), submit(2), approve()),
  seq('expiry-changes-live-only', submit(1), approve(), advance(1), resume(), advance(1), approve(), submit(5), submit(2), approve()),
  seq('revoked-and-expired', submit(3), revoke(3), advance(2), revoke(3), submit(1), approve()),
  seq('fresh-and-expired-votes', submit(1), advance(1), submit(4), advance(1), submit(2), approve(), resume()),
  seq('duplicate-and-fractional-codes', submit(1), submit(1), submit(1.5), revoke(1.5), submit(6), input('submit', {}), input('approve', { code: 1 }), approve()),
  seq('decision-capacity', submit(1), approve(), submit(3), approve(), submit(4), approve(), submit(5), approve(), resume()),
  seq('challenge-before-decision', submit(5), approve(), submit(2), approve(), submit(5), advance(2), resume()),
  seq('expired-opposition-revoked-after-approval', submit(3), approve(), advance(2), revoke(3), submit(1), submit(5), approve()),
  seq('binary64-fractional-timer-boundary', advance(0.1), advance(0.2), submit(1), approve(), advance(1), resume(), advance(1), submit(5), approve(), advance(0.1)),
  seq('invalid-input-types', input('submit', { code: '1' }), input('revoke', { code: null }), input('advance', null), input('submit', { code: true }), submit(1), approve()),
];
B.randomEvents = rng => {
  const events = [submit(pick(rng, [1, 2])), approve(), advance(pick(rng, [0, 1])), submit(pick(rng, [3, 4])), resume(), approve()];
  while (events.length < 40) events.push(pick(rng, [submit(pick(rng, [1, 2, 3, 4, 5, 1.5, 6])), revoke(pick(rng, [1, 2, 3, 4, 2.5])), approve(), advance(pick(rng, [0, 0.1, 0.5, 0.7, 1, 1.2, 2, 3])), resume(), input('submit', {})]));
  return events;
};

const C = {
  initial: () => ({ ...base(), investigation: 1, tokens: 3, phase: 'investigate', selected: 0,
    scores: { 1: -1, 2: -1 }, latest: { 1: null, 2: null }, counts: { 1: 0, 2: 0 }, failures: 0 }),
  step: transactional({ inspect: { cause: [1, 2, true], score: [0, 2, true] }, choose: { repair: [1, 2, true] }, outcome: { success: [0, 1, true] }, next: {} }, (s, event, p) => {
    if (event === 'inspect') {
      if (s.phase !== 'investigate' || s.tokens <= 0) return false;
      const baseName = p.cause === 1 ? 'bearing_check' : 'motor_check';
      const id = occurrence(baseName, ++s.counts[p.cause]);
      s.tokens--;
      s.latest[p.cause] = id;
      s.scores[p.cause] = p.score;
      observe(s, id, p.cause === 1 ? 'bearing_fault' : 'motor_fault', 'supports', ['alternative_cause']);
    } else if (event === 'choose') {
      if (s.phase !== 'investigate' || s.scores[p.repair] <= 0 || s.decisions.length >= 2) return false;
      if (head(s) && !head(s).open) return false;
      commit(s, 'repair', p.repair, [s.latest[p.repair]], ['repair_unverified'], event);
      s.selected = p.repair;
      s.phase = 'awaiting';
    } else if (event === 'outcome') {
      if (s.phase !== 'awaiting') return false;
      if (p.success === 1) s.phase = 'complete';
      else {
        const id = occurrence('failure', ++s.failures);
        observe(s, id, 'repair_effective', 'opposes');
        reopen(s, [id], event);
        s.phase = 'failed';
      }
    } else {
      if (s.phase !== 'failed' || s.investigation !== 1) return false;
      s.investigation = 2;
      s.phase = 'investigate';
      s.scores = { 1: -1, 2: -1 };
      s.latest = { 1: null, 2: null };
      s.selected = 0;
    }
    return true;
  }),
  project(s) {
    return projection(s, { investigation: s.investigation, tokens: s.tokens, bearing: s.scores[1], motor: s.scores[2],
      phase: s.phase, selected: s.selected, frozen: head(s)?.value ?? 0, revision: s.decisions.length });
  },
  projectActual: actualProjector(['bearing_fault', 'motor_fault', 'repair_effective']),
};
const inspect = (cause, score) => input('inspect', { cause, score });
const choose = repair => input('choose', { repair });
const outcome = success => input('outcome', { success });
const next = () => input('next');
C.cases = [
  seq('empty-admission', choose(1), outcome(0), next(), resume()),
  seq('success-terminal', inspect(1, 1), choose(1), outcome(1), next(), inspect(2, 2), choose(1), outcome(0)),
  seq('alternative-causes-coexist', inspect(2, 2), inspect(1, 1), choose(1), resume(), outcome(0)),
  seq('repeated-cause-latest-basis', inspect(1, 1), inspect(1, 2), choose(1), outcome(0), next(), inspect(1, 1), choose(1), outcome(1)),
  seq('latest-zero-prevents-choice', inspect(2, 2), inspect(2, 0), choose(2), inspect(1, 1), choose(1)),
  seq('three-token-boundary', inspect(1, 1), inspect(1, 1), inspect(2, 2), inspect(2, 1), choose(2), outcome(0), next(), inspect(1, 1), choose(1)),
  seq('failure-next-and-fresh-diagnosis-required', inspect(2, 1), choose(2), outcome(0), resume(), next(), choose(2), inspect(2, 2), choose(2)),
  seq('second-failure-terminal-investigation', inspect(1, 2), choose(1), outcome(0), next(), inspect(2, 1), choose(2), outcome(0), next(), choose(2)),
  seq('awaiting-does-not-spend', inspect(1, 1), choose(1), inspect(2, 2), choose(1), next(), resume(), outcome(0)),
  seq('failed-needs-next', inspect(2, 2), choose(2), outcome(0), inspect(1, 2), choose(2), outcome(0), next(), inspect(1, 2), choose(1)),
  seq('integer-and-shape-validation', inspect(1.5, 1), inspect(1, 0.5), inspect(1, 3), input('inspect', { cause: 1 }), input('next', { extra: 1 }), inspect(1, 1), choose(1.5), choose(1), outcome(0.5), outcome(1)),
  seq('repeat-motor-identity-after-save', inspect(2, 1), resume(), inspect(2, 1), choose(2), outcome(0), next(), resume(), inspect(2, 2), choose(2), outcome(0)),
  seq('zero-score-is-real-observation', inspect(1, 0), choose(1), inspect(2, 1), choose(2), outcome(0), next(), inspect(1, 2), choose(1)),
  seq('invalid-input-types', input('inspect', { cause: '1', score: 1 }), input('inspect', { cause: 1, score: null }), input('next', null), input('outcome', { success: false }), inspect(1, 1), choose(1)),
];
C.randomEvents = rng => {
  const first = pick(rng, [1, 2]);
  const second = pick(rng, [1, 2]);
  const events = [inspect(first, pick(rng, [1, 2])), choose(first), outcome(pick(rng, [0, 0, 1])), resume(), next(), inspect(second, pick(rng, [0, 1, 2])), choose(second)];
  while (events.length < 40) events.push(pick(rng, [inspect(pick(rng, [1, 2, 1.5]), pick(rng, [0, 1, 2, 3])), choose(pick(rng, [1, 2])), outcome(pick(rng, [0, 1, 0.5])), next(), resume(), input('inspect', {})]));
  return events;
};

export const tasks = { A, B, C };
