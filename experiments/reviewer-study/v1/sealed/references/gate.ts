// Reference TypeScript program for the release gate (investigator's; sealed).
// Replaces an author's program in phase two only if that program fails.
type Line = { event: string; payload?: Record<string, unknown> };
type View = { status: 'undecided' | 'merge' | 'reopened'; head: number; revisions: number; reason: string };
type Check = { line: number; run: number; passed: number; withdrawn: number | null };
type Review = { line: number; approved: number; relayed: number };
type Scan = { line: number; findings: number; dbStale: boolean };
type Grant = { line: number; head: number };
type Decision = { line: number; value: number; restsOn: number[]; caveats: string[]; retains: string[]; grant: number; reopened: { line: number; reason: string } | null };

const PARAMS: Record<string, Record<string, [number, number]>> = {
  propose: { head: [1, 999] }, check: { run: [1, 999], passed: [0, 1] },
  review: { approved: [0, 1], relayed: [0, 1] }, scan: { findings: [0, 99] },
  approve: { head: [1, 999] }, decide: {}, recheck: {}, db_aged: {}, db_refreshed: {},
};

class Refused extends Error {}
const refuse = (why: string): never => { throw new Refused(why); };

export function createPolicy(): { dispatch(line: Line): void; view(): View; record(): unknown } {
  let line = 0; // the session's sequence: accepted lines only
  let head = 0, reason = '';
  let checks: Check[] = [], reviews: Review[] = [], scans: Scan[] = [], grants: Grant[] = [], decisions: Decision[] = [];
  const last = <T>(list: T[]): T | undefined => list[list.length - 1];
  const inForce = (): Decision | undefined => { const d = last(decisions); return d && !d.reopened ? d : undefined; };

  function validate(entry: Line): Record<string, number> {
    const spec = PARAMS[entry.event] ?? refuse(`unknown event ${entry.event}`);
    const payload = entry.payload ?? {};
    const want = Object.keys(spec).sort(), have = Object.keys(payload).sort();
    if (want.join() !== have.join()) refuse('wrong parameters');
    for (const key of want) {
      const value = payload[key];
      if (typeof value !== 'number' || !Number.isInteger(value) || value < spec[key][0] || value > spec[key][1]) refuse(`${key} out of range`);
    }
    return payload as Record<string, number>;
  }

  function dispatch(entry: Line): void {
    const p = validate(entry);
    const at = line + 1;
    const snapshot = JSON.stringify({ head, reason, checks, reviews, scans, grants, decisions });
    try {
      const current = inForce();
      const reopen = (why: string): void => { if (current) { current.reopened = { line: at, reason: why }; reason = why; } };
      switch (entry.event) {
        case 'propose': head = p.head; break;
        case 'check':
          if (checks.length >= 8) refuse('at most eight checks');
          checks.push({ line: at, run: p.run, passed: p.passed, withdrawn: null });
          if (p.passed === 0) reopen('check failed');
          break;
        case 'review':
          if (reviews.length >= 4) refuse('at most four reviews');
          reviews.push({ line: at, approved: p.approved, relayed: p.relayed });
          break;
        case 'scan':
          if (scans.length >= 4) refuse('at most four scans');
          scans.push({ line: at, findings: p.findings, dbStale: false });
          if (p.findings > 0) reopen('scan found issues');
          break;
        case 'approve':
          if (grants.length >= 4) refuse('at most four grants');
          grants.push({ line: at, head: p.head });
          break;
        case 'decide': {
          const check = [...checks].reverse().find(c => c.withdrawn === null);
          const review = last(reviews), scan = last(scans), grant = last(grants);
          if (!check || !review || !scan) refuse('nothing to decide on');
          if (check!.passed !== 1 || review!.approved !== 1 || scan!.findings !== 0) refuse('not ready');
          if (current) refuse('a decision is in force');
          if (!grant || grant.head !== head) refuse('no grant for the current head');
          if (decisions.length >= 4) refuse('four decisions made');
          const caveats = [...(review!.relayed === 1 ? ['secondhand'] : []), ...(scan!.dbStale ? ['db_stale'] : [])];
          decisions.push({ line: at, value: head, restsOn: [check!.line, review!.line, scan!.line].sort((a, b) => a - b),
            caveats, retains: ['deps_unreviewed'], grant: grant!.line, reopened: null });
          reason = 'ready';
          break;
        }
        case 'recheck': {
          const check = last(checks);
          if (!check || check.withdrawn !== null) refuse('nothing to withdraw');
          check!.withdrawn = at;
          if (current && current.restsOn.includes(check!.line)) reopen('check withdrawn');
          break;
        }
        case 'db_aged': {
          const scan = last(scans);
          if (!scan || scan.dbStale) refuse('no scan to age');
          scan!.dbStale = true;
          if (current && current.restsOn.includes(scan!.line)) reopen('scan database stale');
          break;
        }
        case 'db_refreshed': {
          const scan = last(scans);
          if (!scan || !scan.dbStale) refuse('nothing to refresh');
          for (const each of scans) each.dbStale = false;
          break;
        }
      }
      line = at;
    } catch (error) {
      ({ head, reason, checks, reviews, scans, grants, decisions } = JSON.parse(snapshot));
      throw error;
    }
  }

  return {
    dispatch,
    view: (): View => {
      const latest = last(decisions);
      return { status: !latest ? 'undecided' : latest.reopened ? 'reopened' : 'merge', head, revisions: decisions.length, reason };
    },
    record: () => ({ checks, reviews, scans, grants, decisions }),
  };
}
