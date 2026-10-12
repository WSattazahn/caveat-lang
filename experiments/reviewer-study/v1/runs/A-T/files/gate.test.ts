// Run: node --test gate.test.ts
// Expectations below were derived by hand from TASK.md, not copied from program output.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createPolicy } from "./gate.ts";

type V = [string, number, number, string]; // status, head, revisions, reason
type D = { line: number; value: number; rests: [string, string, string]; caveats: string[]; grant: string; reopened: null | [number, string, string] };

function run(file: string) {
  const p = createPolicy();
  const lines = readFileSync(file, "utf8").split("\n").filter((l) => l.trim() !== "");
  const refused: number[] = [];
  const views: V[] = [];
  lines.forEach((t, i) => {
    const before = JSON.stringify(p.view());
    const recBefore = JSON.stringify({ ...p.record(), line: 0, refusals: [] });
    try { p.dispatch(JSON.parse(t)); } catch {
      refused.push(i + 1);
      assert.equal(JSON.stringify(p.view()), before, `${file}:${i + 1} refusal changed view`);
      assert.equal(JSON.stringify({ ...p.record(), line: 0, refusals: [] }), recBefore, `${file}:${i + 1} refusal changed record`);
    }
    const v = p.view();
    views.push([v.status, v.head, v.revisions, v.reason]);
  });
  const decisions: D[] = p.record().decisions.map((d) => ({
    line: d.line, value: d.value,
    rests: [d.restsOn.check, d.restsOn.review, d.restsOn.scan],
    caveats: d.caveats.map((c) => `${c.name}@${c.on}`),
    grant: d.permittedBy,
    reopened: d.reopened ? [d.reopened.atLine, d.reopened.reason, d.reopened.citing] : null,
  }));
  return { p, refused, views, decisions, final: views[views.length - 1] };
}

const DEPS = "deps_unreviewed@(retained by choice)";
const U = (h: number): V => ["undecided", h, 0, ""];

test("H1", () => {
  const r = run("histories/H1.jsonl");
  assert.deepEqual(r.refused, []);
  assert.deepEqual(r.views, [U(17), U(17), U(17), U(17), U(17), ["merge", 17, 1, "ready"], ["merge", 17, 1, "ready"]]);
  assert.deepEqual(r.decisions, [{ line: 6, value: 17, rests: ["#3", "#4", "#5"], caveats: ["secondhand@#4", DEPS], grant: "#2", reopened: null }]);
});

test("H2", () => {
  const r = run("histories/H2.jsonl");
  assert.deepEqual(r.refused, []);
  assert.deepEqual(r.views.slice(5), [
    ["merge", 31, 1, "ready"], ["reopened", 31, 1, "scan found issues"], ["reopened", 31, 1, "scan found issues"],
    ["merge", 31, 2, "ready"], ["reopened", 31, 2, "check withdrawn"], ["reopened", 31, 2, "check withdrawn"],
    ["merge", 31, 3, "ready"], ["merge", 31, 3, "ready"],
  ]);
  assert.deepEqual(r.decisions, [
    { line: 6, value: 31, rests: ["#3", "#4", "#5"], caveats: [DEPS], grant: "#2", reopened: [7, "scan found issues", "#7"] },
    { line: 9, value: 31, rests: ["#3", "#4", "#8"], caveats: [DEPS], grant: "#2", reopened: [10, "check withdrawn", "#3"] },
    { line: 12, value: 31, rests: ["#11", "#4", "#8"], caveats: [DEPS], grant: "#2", reopened: null },
  ]);
  const c3 = r.p.record().observations.find((o) => o.id === "#3")!;
  assert.equal(c3.withdrawn!.atLine, 10); // retained, marked withdrawn
});

test("H3", () => {
  const r = run("histories/H3.jsonl");
  assert.deepEqual(r.refused, [13]);
  assert.deepEqual(r.views.slice(5), [
    ["merge", 42, 1, "ready"], ["reopened", 42, 1, "scan database stale"], ["reopened", 42, 1, "scan database stale"],
    ["merge", 42, 2, "ready"], ["merge", 42, 2, "ready"], ["merge", 43, 2, "ready"], ["merge", 43, 2, "ready"], ["merge", 43, 2, "ready"],
  ]);
  assert.deepEqual(r.decisions, [
    { line: 6, value: 42, rests: ["#3", "#4", "#5"], caveats: [DEPS], grant: "#2", reopened: [7, "scan database stale", "#5"] },
    { line: 9, value: 42, rests: ["#3", "#4", "#5"], caveats: [DEPS], grant: "#2", reopened: null },
  ]);
  const scan = r.p.record().observations.find((o) => o.id === "#5")!;
  assert.deepEqual(scan.caveats.map((c) => [c.name, c.addedAtLine, c.removedAtLine]), [["db_stale", 7, 8]]);
});

const refusedOnly: Record<string, [number[], V]> = {
  "C01-unknown-event": [[6], U(5)],
  "C02-out-of-range": [[2, 3, 4, 5], U(5)],
  "C03-fraction": [[2, 3, 4], U(5)],
  "C04-missing-and-extra": [[2, 3, 4, 5], U(5)],
  "C05-decide-nothing-recorded": [[3], U(5)],
  "C06-decide-in-force": [[7], ["merge", 5, 1, "ready"]],
  "C10-no-grant": [[5], U(5)],
  "C11-fifth-review": [[5], U(0)],
  "C12-ninth-check": [[9], U(0)],
  "C13-fifth-scan": [[5], U(0)],
  "C14-fifth-grant": [[5], U(4)],
  "C18-refreshed-before-aged": [[6, 8], ["merge", 5, 1, "ready"]],
};
for (const [name, [refused, final]] of Object.entries(refusedOnly)) {
  test(name, () => {
    const r = run(`cases/${name}.jsonl`);
    assert.deepEqual(r.refused, refused);
    assert.deepEqual(r.final, final);
  });
}

const dec = (line: number, rests: [string, string, string], grant = "#2", reopened: D["reopened"] = null, caveats = [DEPS]): D =>
  ({ line, value: 5, rests, caveats, grant, reopened });

test("C07 review opposing", () => {
  const r = run("cases/C07-review-opposing.jsonl");
  assert.deepEqual(r.refused, [7]);
  assert.deepEqual(r.decisions, [dec(9, ["#3", "#8", "#6"], "#2", null, ["secondhand@#8", DEPS])]);
});
test("C08 scan opposing", () => {
  const r = run("cases/C08-scan-opposing.jsonl");
  assert.deepEqual(r.refused, [6]);
  assert.deepEqual(r.decisions, [dec(8, ["#3", "#4", "#7"])]);
});
test("C09 grant for other head", () => {
  const r = run("cases/C09-grant-other-head.jsonl");
  assert.deepEqual(r.refused, [6]);
  assert.deepEqual(r.decisions, [dec(8, ["#3", "#4", "#5"], "#7")]);
});
test("C15 recheck twice", () => {
  const r = run("cases/C15-recheck-twice.jsonl");
  assert.deepEqual(r.refused, [8]);
  assert.deepEqual(r.views[6], ["reopened", 5, 1, "check withdrawn"]);
  assert.deepEqual(r.decisions, [dec(6, ["#3", "#4", "#5"], "#2", [7, "check withdrawn", "#3"]), dec(10, ["#9", "#4", "#5"])]);
  assert.deepEqual(r.final, ["merge", 5, 2, "ready"]);
});
test("C16 recheck of a check not rested on", () => {
  const r = run("cases/C16-recheck-not-rested-on.jsonl");
  assert.deepEqual(r.refused, []);
  assert.deepEqual(r.views.slice(5), [["merge", 5, 1, "ready"], ["merge", 5, 1, "ready"], ["merge", 5, 1, "ready"], ["merge", 6, 1, "ready"]]);
  assert.deepEqual(r.decisions, [dec(6, ["#3", "#4", "#5"])]);
});
test("C17 db_aged twice", () => {
  const r = run("cases/C17-db-aged-twice.jsonl");
  assert.deepEqual(r.refused, [8]);
  assert.deepEqual(r.decisions, [
    dec(6, ["#3", "#4", "#5"], "#2", [7, "scan database stale", "#5"]),
    dec(10, ["#3", "#4", "#5"], "#2", [11, "scan database stale", "#5"]), // caveats frozen: no db_stale
  ]);
  assert.deepEqual(r.final, ["reopened", 5, 2, "scan database stale"]);
});
test("C19 four decisions", () => {
  const r = run("cases/C19-four-decisions.jsonl");
  assert.deepEqual(r.refused, [18]);
  assert.deepEqual(r.decisions, [
    dec(6, ["#3", "#4", "#5"], "#2", [7, "check failed", "#7"]),
    dec(9, ["#8", "#4", "#5"], "#2", [10, "scan found issues", "#10"]),
    dec(12, ["#8", "#4", "#11"], "#2", [13, "check withdrawn", "#8"]),
    dec(15, ["#14", "#4", "#11"], "#2", [16, "check failed", "#16"]),
  ]);
  assert.deepEqual(r.final, ["reopened", 5, 4, "check failed"]);
});
test("C20 all checks withdrawn", () => {
  const r = run("cases/C20-all-checks-withdrawn.jsonl");
  assert.deepEqual(r.refused, [7]);
  assert.deepEqual(r.decisions, [dec(9, ["#8", "#4", "#5"])]);
});
test("C21 second correction while reopened", () => {
  const r = run("cases/C21-second-correction-while-reopened.jsonl");
  assert.deepEqual(r.refused, []);
  for (let i = 6; i <= 11; i++) assert.deepEqual(r.views[i], ["reopened", 5, 1, "check failed"], `line ${i + 1}`);
  assert.deepEqual(r.decisions, [dec(6, ["#3", "#4", "#5"], "#2", [7, "check failed", "#7"]), dec(13, ["#11", "#4", "#12"])]);
  const rec = r.p.record();
  assert.equal(rec.observations.find((o) => o.id === "#8")!.caveats[0].name, "db_stale");
  assert.equal(rec.observations.find((o) => o.id === "#7")!.withdrawn!.atLine, 10);
});

test("input validation edge cases", () => {
  const p = createPolicy();
  const bad: unknown[] = [
    null, [], "propose", { payload: { head: 1 } }, { event: "toString" }, { event: "propose", payload: { head: "5" } },
    { event: "propose", payload: { head: true } }, { event: "propose", payload: { head: 0 } }, { event: "propose", payload: null },
    { event: "propose", payload: { head: 1 }, extra: 1 }, { event: "propose", payload: { head: NaN } }, { event: "propose", payload: { head: Infinity } },
    { event: "decide", payload: null }, { event: "check", payload: { run: 1, passed: 1, foo: undefined } },
  ];
  for (const b of bad) assert.throws(() => p.dispatch(b), undefined, JSON.stringify(b));
  assert.deepEqual(p.view(), { status: "undecided", head: 0, revisions: 0, reason: "" });
  assert.equal(p.record().seq, 0); // sequence did not advance
  p.dispatch({ event: "propose", payload: { head: 999 } });
  p.dispatch({ event: "scan", payload: { findings: 0 } });
  p.dispatch({ event: "db_aged", payload: {} }); // empty payload accepted for a no-parameter event
  assert.equal(p.record().seq, 3);
  assert.equal(p.record().observations[0].id, "#16"); // ids count every dispatched line, refused or not
});
