// The model of the release gate (TASK.md), written from the task text alone.
// Sealed until phase two is frozen. It decides, for every history line,
// whether the line is accepted and what the view is afterwards, and it keeps
// the record the answer keys are read from.
//
//   run(lines) -> { steps: [{ line, accepted, view }], record }

const PARAMS = {
  propose: { head: [1, 999] },
  check: { run: [1, 999], passed: [0, 1] },
  review: { approved: [0, 1], relayed: [0, 1] },
  scan: { findings: [0, 99] },
  approve: { head: [1, 999] },
  decide: {}, recheck: {}, db_aged: {}, db_refreshed: {},
};
const LIMITS = { check: 8, review: 4, scan: 4, approve: 4, decide: 4 };

function valid(entry) {
  if (!entry || typeof entry !== 'object' || Array.isArray(entry)) return false;
  const spec = PARAMS[entry.event];
  if (!spec) return false;
  const payload = entry.payload ?? {};
  if (typeof payload !== 'object' || payload === null || Array.isArray(payload)) return false;
  const want = Object.keys(spec).sort(), have = Object.keys(payload).sort();
  if (want.length !== have.length || want.some((key, index) => key !== have[index])) return false;
  return want.every(key => Number.isInteger(payload[key]) && payload[key] >= spec[key][0] && payload[key] <= spec[key][1]);
}

export function initial() {
  return {
    head: 0,
    checks: [], reviews: [], scans: [], grants: [],
    decisions: [],
    reason: '',
  };
}

const last = list => list[list.length - 1];
const latestCheck = state => [...state.checks].reverse().find(check => !check.withdrawn);
const inForce = state => { const d = last(state.decisions); return d && !d.reopened ? d : null; };

export function view(state) {
  const latest = last(state.decisions);
  return {
    status: !latest ? 'undecided' : latest.reopened ? 'reopened' : 'merge',
    head: state.head,
    revisions: state.decisions.length,
    reason: state.reason,
  };
}

const clone = state => structuredClone(state);

// Applies one history line; returns the new state, or null when refused.
export function apply(state0, entry, line) {
  if (!valid(entry)) return null;
  const state = clone(state0);
  const p = entry.payload ?? {};
  const current = inForce(state);
  const reopen = (reason, by) => {
    if (!current) return;
    current.reopened = { line: by, reason };
    state.reason = reason;
  };
  switch (entry.event) {
    case 'propose':
      state.head = p.head;
      return state;
    case 'check': {
      if (state.checks.length >= LIMITS.check) return null;
      state.checks.push({ line, run: p.run, passed: p.passed, withdrawn: null });
      if (p.passed === 0) reopen('check failed', line);
      return state;
    }
    case 'review': {
      if (state.reviews.length >= LIMITS.review) return null;
      state.reviews.push({ line, approved: p.approved, relayed: p.relayed });
      return state;
    }
    case 'scan': {
      if (state.scans.length >= LIMITS.scan) return null;
      state.scans.push({ line, findings: p.findings, dbStale: false });
      if (p.findings > 0) reopen('scan found issues', line);
      return state;
    }
    case 'approve': {
      if (state.grants.length >= LIMITS.approve) return null;
      state.grants.push({ line, head: p.head });
      return state;
    }
    case 'decide': {
      const check = latestCheck(state), review = last(state.reviews), scan = last(state.scans);
      if (!state.checks.length || !review || !scan || !check) return null;
      if (check.passed !== 1 || review.approved !== 1 || scan.findings !== 0) return null;
      if (current) return null;
      const grant = last(state.grants);
      if (!grant || grant.head !== state.head) return null;
      if (state.decisions.length >= LIMITS.decide) return null;
      const caveats = [];
      if (review.relayed === 1) caveats.push('secondhand');
      if (scan.dbStale) caveats.push('db_stale');
      state.decisions.push({
        number: state.decisions.length + 1, line, value: state.head,
        restsOn: [check.line, review.line, scan.line].sort((a, b) => a - b),
        caveats: caveats.sort(), retains: ['deps_unreviewed'], grant: grant.line,
        reopened: null,
      });
      state.reason = 'ready';
      return state;
    }
    case 'recheck': {
      const check = last(state.checks);
      if (!check || check.withdrawn) return null;
      check.withdrawn = { line };
      if (current && current.restsOn.includes(check.line)) reopen('check withdrawn', line);
      return state;
    }
    case 'db_aged': {
      const scan = last(state.scans);
      if (!scan || scan.dbStale) return null;
      scan.dbStale = true;
      if (current && current.restsOn.includes(scan.line)) reopen('scan database stale', line);
      return state;
    }
    case 'db_refreshed': {
      const scan = last(state.scans);
      if (!scan || !scan.dbStale) return null;
      for (const each of state.scans) each.dbStale = false;
      return state;
    }
    default:
      return null;
  }
}

export function run(lines) {
  let state = initial();
  const steps = [];
  lines.forEach((entry, index) => {
    const next = apply(state, entry, index + 1);
    if (next) state = next;
    steps.push({ line: index + 1, accepted: next !== null, view: view(state) });
  });
  return { steps, record: state };
}

// The answer key for one registered history and its probes.
export function key(lines, probes) {
  const { steps, record } = run(lines);
  const decisions = record.decisions.map((d, index) => ({
    decision: d.number,
    value: d.value,
    rests_on: d.restsOn,
    caveats: d.caveats,
    retains: d.retains,
    grant: d.grant,
    state: !d.reopened ? 'in_force' : index < record.decisions.length - 1 ? 'superseded' : 'reopened',
    reopened_by: d.reopened ? d.reopened.line : null,
    reopen_reason: d.reopened ? d.reopened.reason : null,
  }));
  const force = inForce(record);
  const observations = new Set([...record.checks, ...record.reviews, ...record.scans].map(item => item.line));
  const grants = new Set(record.grants.map(item => item.line));
  const q2 = probes.map(probe => {
    if (!observations.has(probe) && !grants.has(probe)) throw new Error(`probe ${probe} names no observation`);
    if (!force) return 'none_in_force';
    return force.restsOn.includes(probe) ? 'yes' : 'no';
  });
  return { refused: steps.filter(step => !step.accepted).map(step => step.line), decisions, q2 };
}
