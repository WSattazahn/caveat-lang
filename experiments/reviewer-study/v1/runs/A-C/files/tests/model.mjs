// Independent reference model of TASK.md, written from the task text only.
// It returns, for each line, whether it is accepted, the view, and for each
// decision the lines of the observations it rests on and their caveats.
const SPEC = {
  propose: { head: [1, 999] },
  check: { run: [1, 999], passed: [0, 1] },
  review: { approved: [0, 1], relayed: [0, 1] },
  scan: { findings: [0, 99] },
  approve: { head: [1, 999] },
  decide: {}, recheck: {}, db_aged: {}, db_refreshed: {},
};
export function model(lines) {
  const st = { head: 0, checks: [], reviews: [], scans: [], grants: [], decisions: [], reason: '' };
  const inForce = () => st.decisions.length > 0 && !st.decisions.at(-1).reopened;
  const latestCheck = () => [...st.checks].reverse().find((c) => !c.withdrawn);
  const out = [];
  lines.forEach((env, i) => {
    const line = i + 1;
    const r = step(env, line);
    out.push({ line, accepted: r === true, why: r === true ? null : r, view: view() });
  });
  return { steps: out, decisions: st.decisions };

  function view() {
    const status = st.decisions.length === 0 ? 'undecided' : inForce() ? 'merge' : 'reopened';
    return { status, head: st.head, revisions: st.decisions.length, reason: st.reason };
  }
  function reopen(reason, because) {
    if (!inForce()) return;
    const d = st.decisions.at(-1);
    d.reopened = { line: because, reason };
    st.reason = reason;
  }
  function step(env, line) {
    const spec = SPEC[env.event];
    if (!spec) return 'unknown event';
    const p = env.payload ?? {};
    const keys = Object.keys(p);
    if (keys.length !== Object.keys(spec).length || !keys.every((k) => k in spec)) return 'bad params';
    for (const [k, [lo, hi]] of Object.entries(spec)) {
      const v = p[k];
      if (typeof v !== 'number' || !Number.isInteger(v) || v < lo || v > hi) return 'bad value';
    }
    switch (env.event) {
      case 'propose': st.head = p.head; return true;
      case 'check': {
        if (st.checks.length >= 8) return 'ninth check';
        st.checks.push({ line, ok: p.passed === 1, withdrawn: false });
        if (p.passed === 0) reopen('check failed', line);
        return true;
      }
      case 'review': {
        if (st.reviews.length >= 4) return 'fifth review';
        st.reviews.push({ line, ok: p.approved === 1, caveats: p.relayed === 1 ? ['secondhand'] : [] });
        return true;
      }
      case 'scan': {
        if (st.scans.length >= 4) return 'fifth scan';
        st.scans.push({ line, ok: p.findings === 0, stale: false });
        if (p.findings > 0) reopen('scan found issues', line);
        return true;
      }
      case 'approve': {
        if (st.grants.length >= 4) return 'fifth grant';
        st.grants.push({ line, head: p.head });
        return true;
      }
      case 'recheck': {
        const c = st.checks.at(-1);
        if (!c || c.withdrawn) return 'nothing to recheck';
        c.withdrawn = true;
        if (inForce() && st.decisions.at(-1).rests.includes(c.line)) reopen('check withdrawn', line);
        return true;
      }
      case 'db_aged': {
        const s = st.scans.at(-1);
        if (!s || s.stale) return 'cannot age';
        s.stale = true;
        if (inForce() && st.decisions.at(-1).rests.includes(s.line)) reopen('scan database stale', s.line);
        return true;
      }
      case 'db_refreshed': {
        const s = st.scans.at(-1);
        if (!s || !s.stale) return 'cannot refresh';
        for (const x of st.scans) x.stale = false;
        return true;
      }
      case 'decide': {
        const c = latestCheck(), rv = st.reviews.at(-1), sc = st.scans.at(-1);
        if (st.checks.length === 0 || !rv || !sc || !c) return 'missing';
        if (!c.ok || !rv.ok || !sc.ok) return 'not supporting';
        if (inForce()) return 'in force';
        const g = st.grants.at(-1);
        if (!g || g.head !== st.head) return 'not permitted';
        if (st.decisions.length >= 4) return 'four decisions';
        const caveats = [...rv.caveats, ...(sc.stale ? ['db_stale'] : [])];
        st.decisions.push({ line, value: st.head, rests: [c.line, rv.line, sc.line].sort((a, b) => a - b),
          caveats: [...caveats, 'deps_unreviewed'].sort(), grant: g.line, reopened: null });
        st.reason = 'ready';
        return true;
      }
    }
  }
}
