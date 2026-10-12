// The release gate (study run A-T).
//
// createPolicy() returns { dispatch(line), view() } as TASK.md requires, plus
// read-only helpers record() and explain() so that a reviewer can establish
// what each decision rested on, what caveats it carried, which grant permitted
// it, and how (and citing what) it was reopened.
//
// Observation identity: TASK.md identifies an observation by the 1-based line
// of the history file that recorded it. dispatch() receives only the parsed
// line, so the policy counts dispatch() calls (accepted AND refused): the n-th
// call is line n. This assumes the host sends every line of a history, in
// order, to one fresh policy. That counter is the only state a refused event
// touches; it is not the session's sequence (`seq`), which counts accepted
// events only and does not advance on refusal.
//
// Only erasable TypeScript syntax is used (no enums, namespaces or parameter
// properties), so the file also runs under Node's type stripping.
const DECLARED_CAVEATS = {
    secondhand: "material",
    db_stale: "material",
    deps_unreviewed: "low",
};
const LIMITS = { check: 8, review: 4, scan: 4, grant: 4, merge: 4 };
// Parameter schema: name -> [min, max] (inclusive integers).
const SCHEMA = {
    propose: { head: [1, 999] },
    check: { run: [1, 999], passed: [0, 1] },
    review: { approved: [0, 1], relayed: [0, 1] },
    scan: { findings: [0, 99] },
    approve: { head: [1, 999] },
    decide: {},
    recheck: {},
    db_aged: {},
    db_refreshed: {},
};
export class Refused extends Error {
    constructor(message) {
        super(message);
        this.name = "Refused";
    }
}
function refuse(msg) {
    throw new Refused(msg);
}
function validate(line) {
    if (line === null || typeof line !== "object" || Array.isArray(line))
        refuse("line is not an object");
    const obj = line;
    for (const k of Object.keys(obj)) {
        if (k !== "event" && k !== "payload")
            refuse(`unexpected field "${k}" in line`);
    }
    const event = obj.event;
    if (typeof event !== "string")
        refuse("event name missing or not a string");
    if (!Object.prototype.hasOwnProperty.call(SCHEMA, event))
        refuse(`unknown event "${event}"`);
    const params = SCHEMA[event];
    const names = Object.keys(params);
    let raw;
    if (!("payload" in obj) || obj.payload === undefined) {
        raw = {};
    }
    else if (obj.payload === null || typeof obj.payload !== "object" || Array.isArray(obj.payload)) {
        refuse(`${event}: payload is not an object`);
    }
    else {
        raw = obj.payload;
    }
    for (const k of Object.keys(raw)) {
        if (!names.includes(k))
            refuse(`${event}: extra parameter "${k}"`);
    }
    const payload = {};
    for (const n of names) {
        if (!Object.prototype.hasOwnProperty.call(raw, n))
            refuse(`${event}: missing parameter "${n}"`);
        const v = raw[n];
        const [lo, hi] = params[n];
        if (typeof v !== "number" || !Number.isInteger(v))
            refuse(`${event}: "${n}" is not an integer`);
        if (v < lo || v > hi)
            refuse(`${event}: "${n}" = ${v} is outside ${lo}..${hi}`);
        payload[n] = v === 0 ? 0 : v; // normalise -0
    }
    return { event, payload };
}
export function createPolicy() {
    let line = 0; // dispatch-call counter = history line number
    let s = { seq: 0, head: 0, observations: [], decisions: [], reason: "" };
    const refusals = [];
    const ofKind = (k) => s.observations.filter((o) => o.kind === k);
    const latest = (k, includeWithdrawn = false) => {
        const xs = ofKind(k).filter((o) => includeWithdrawn || o.withdrawn === null);
        return xs[xs.length - 1];
    };
    const activeCaveats = (o) => o.caveats.filter((c) => c.removedAtLine === null);
    const hasActive = (o, n) => activeCaveats(o).some((c) => c.name === n);
    const current = () => s.decisions[s.decisions.length - 1];
    const inForce = () => {
        const d = current();
        return d && d.reopened === null ? d : undefined;
    };
    const restsOnIds = (d) => [d.restsOn.check, d.restsOn.review, d.restsOn.scan];
    function reopenIfInForce(reason, citing, condition) {
        const d = inForce();
        if (d && condition(d)) {
            d.reopened = { atLine: line, reason, citing };
            s.reason = reason;
        }
    }
    function record(kind, payload, stance) {
        const lim = LIMITS[kind];
        if (ofKind(kind).length >= lim)
            refuse(`${kind}: at most ${lim} may be recorded`);
        const o = {
            id: `#${line}`,
            line,
            seq: s.seq,
            kind,
            payload: { ...payload },
            stance,
            caveats: [],
            withdrawn: null,
        };
        s.observations.push(o);
        return o;
    }
    const handlers = {
        propose(p) {
            s.head = p.head;
        },
        check(p) {
            const o = record("check", p, p.passed === 1 ? "supports" : "opposes");
            if (p.passed === 0)
                reopenIfInForce("check failed", o.id, () => true);
        },
        review(p) {
            const o = record("review", p, p.approved === 1 ? "supports" : "opposes");
            if (p.relayed === 1) {
                o.caveats.push({ name: "secondhand", consequence: DECLARED_CAVEATS.secondhand, addedAtLine: line, removedAtLine: null });
            }
        },
        scan(p) {
            const o = record("scan", p, p.findings === 0 ? "supports" : "opposes");
            if (p.findings > 0)
                reopenIfInForce("scan found issues", o.id, () => true);
        },
        approve(p) {
            record("grant", p, null);
        },
        decide() {
            const c = latest("check");
            const r = latest("review");
            const sc = latest("scan");
            if (ofKind("check").length === 0 || !r || !sc)
                refuse("decide: needs at least one check, one review and one scan");
            if (!c)
                refuse("decide: every recorded check is withdrawn; there is no latest check");
            if (c.stance !== "supports")
                refuse(`decide: latest check ${c.id} opposes ready`);
            if (r.stance !== "supports")
                refuse(`decide: latest review ${r.id} opposes ready`);
            if (sc.stance !== "supports")
                refuse(`decide: latest scan ${sc.id} opposes ready`);
            if (inForce())
                refuse(`decide: decision ${current().number} is in force and has not been reopened`);
            const g = latest("grant");
            if (!g)
                refuse("decide: no grant has been recorded");
            if (g.payload.head !== s.head)
                refuse(`decide: latest grant ${g.id} is for head ${g.payload.head}, not current head ${s.head}`);
            if (s.decisions.length >= LIMITS.merge)
                refuse(`decide: ${LIMITS.merge} decisions already made`);
            const caveats = [];
            for (const o of [c, r, sc]) {
                for (const cv of activeCaveats(o))
                    caveats.push({ name: cv.name, consequence: cv.consequence, on: o.id });
            }
            caveats.push({ name: "deps_unreviewed", consequence: DECLARED_CAVEATS.deps_unreviewed, on: "(retained by choice)" });
            s.decisions.push({
                number: s.decisions.length + 1,
                line,
                seq: s.seq,
                value: s.head,
                claim: "ready",
                restsOn: { check: c.id, review: r.id, scan: sc.id },
                caveats,
                retained: ["deps_unreviewed"],
                permittedBy: g.id,
                reopened: null,
            });
            s.reason = "ready";
        },
        recheck() {
            const c = latest("check", true);
            if (!c)
                refuse("recheck: no check has been recorded");
            if (c.withdrawn)
                refuse(`recheck: latest recorded check ${c.id} is already withdrawn`);
            c.withdrawn = { atLine: line, reason: `run ${c.payload.run} re-queued as flaky (recheck at line ${line})` };
            reopenIfInForce("check withdrawn", c.id, (d) => restsOnIds(d).includes(c.id));
        },
        db_aged() {
            const sc = latest("scan");
            if (!sc)
                refuse("db_aged: no scan has been recorded");
            if (hasActive(sc, "db_stale"))
                refuse(`db_aged: latest scan ${sc.id} already carries db_stale`);
            sc.caveats.push({ name: "db_stale", consequence: DECLARED_CAVEATS.db_stale, addedAtLine: line, removedAtLine: null });
            reopenIfInForce("scan database stale", sc.id, (d) => restsOnIds(d).includes(sc.id));
        },
        db_refreshed() {
            const sc = latest("scan");
            if (!sc || !hasActive(sc, "db_stale"))
                refuse("db_refreshed: latest scan does not carry db_stale");
            for (const o of s.observations) {
                for (const cv of o.caveats)
                    if (cv.name === "db_stale" && cv.removedAtLine === null)
                        cv.removedAtLine = line;
            }
        },
    };
    function view() {
        const d = current();
        return {
            status: !d ? "undecided" : d.reopened ? "reopened" : "merge",
            head: s.head,
            revisions: s.decisions.length,
            reason: s.reason,
        };
    }
    function dispatch(input) {
        line += 1;
        const snapshot = structuredClone(s);
        try {
            const { event, payload } = validate(input);
            s.seq += 1;
            handlers[event](payload);
        }
        catch (e) {
            s = snapshot; // atomic: a refused event changes nothing
            refusals.push({ line, event: input, reason: e instanceof Error ? e.message : String(e) });
            throw e;
        }
    }
    // Read-only copy of everything the program keeps.
    function recordCopy() {
        return structuredClone({ line, seq: s.seq, head: s.head, observations: s.observations, decisions: s.decisions, refusals, view: view() });
    }
    // Human-readable account of each decision.
    function explain() {
        const out = [];
        if (s.decisions.length === 0)
            out.push("No merge decision has been made.");
        for (const d of s.decisions) {
            const caveats = d.caveats.map((c) => `${c.name} (${c.consequence}) on ${c.on}`).join(", ");
            out.push(`Decision ${d.number} (line ${d.line}): merge head ${d.value}; claim ${d.claim}.`, `  rests on: check ${d.restsOn.check}, review ${d.restsOn.review}, scan ${d.restsOn.scan}`, `  caveats at decision: ${caveats}`, `  permitted by grant ${d.permittedBy} (not rested on)`, d.reopened
                ? `  reopened at line ${d.reopened.atLine}: ${d.reopened.reason}, citing ${d.reopened.citing}`
                : `  in force`);
        }
        return out.join("\n");
    }
    return { dispatch, view, record: recordCopy, explain };
}
