import { writeFileSync } from "node:fs";
const R = (kind, text) => ({ kind, text });
const it = (answer, receipt) => ({ answer, receipt });
const RUN = "work/run.mjs (replays the history through a tsc-compiled copy of gate.ts and prints view(), explain() and record()); output in work/run-output.txt";
function dec(h, n, value, rests, cav, cavTxt, grant, state, explainLines) {
  const ex = R("record", `${h} explain() via ${RUN}: ` + explainLines.join(" | "));
  return {
    decision: n,
    value: it(value, ex),
    rests_on: it(rests, ex),
    caveats: it(cav, R("record", `${h} explain(): "${cavTxt}". deps_unreviewed is reported here but is the caveat retained by choice (on "(retained by choice)"), not one carried by an observation, so it is excluded from (c).`)),
    retains: it(["deps_unreviewed"], R("record", `${h} record() decision ${n}: "retained":["deps_unreviewed"]; set unconditionally at gate.ts:241`)),
    grant: it(grant, ex),
    state: it(state, ex),
  };
}
const probe = (h, p, a, why) => ({ probe: p, answer: a, receipt: R("run", `${RUN}, ${h} "--- probes": ${why}. Rule applied is the program's own reopen condition for a withdrawal (gate.ts:252, reopenIfInForce only if the decision in force has the id in restsOn).`) });
const st = (n, a, kind, text) => ({ statement: n, answer: a, receipt: R(kind, text) });

const H1x = ["Decision 1 (line 6): merge head 17; claim ready.", "rests on: check #3, review #4, scan #5", "caveats at decision: secondhand (material) on #4, deps_unreviewed (low) on (retained by choice)", "permitted by grant #2 (not rested on)", "in force"];
const H2d1 = ["Decision 1 (line 6): merge head 31; claim ready.", "rests on: check #3, review #4, scan #5", "caveats at decision: deps_unreviewed (low) on (retained by choice)", "permitted by grant #2 (not rested on)", "reopened at line 7: scan found issues, citing #7", "(Decision 2 made at line 9, so superseded)"];
const H2d2 = ["Decision 2 (line 9): merge head 31; claim ready.", "rests on: check #3, review #4, scan #8", "caveats at decision: deps_unreviewed (low) on (retained by choice)", "permitted by grant #2 (not rested on)", "reopened at line 10: check withdrawn, citing #3", "(Decision 3 made at line 12, so superseded)"];
const H2d3 = ["Decision 3 (line 12): merge head 31; claim ready.", "rests on: check #11, review #4, scan #8", "caveats at decision: deps_unreviewed (low) on (retained by choice)", "permitted by grant #2 (not rested on)", "in force"];
const H3d1 = ["Decision 1 (line 6): merge head 42; claim ready.", "rests on: check #3, review #4, scan #5", "caveats at decision: deps_unreviewed (low) on (retained by choice)", "permitted by grant #2 (not rested on)", "reopened at line 7: scan database stale, citing #5", "(Decision 2 made at line 9, so superseded)"];
const H3d2 = ["Decision 2 (line 9): merge head 42; claim ready.", "rests on: check #3, review #4, scan #5", "caveats at decision: deps_unreviewed (low) on (retained by choice)", "permitted by grant #2 (not rested on)", "in force"];

const A = { run: "R-T2", histories: {
 H1: {
  q1: [dec("H1",1,17,[3,4,5],["secondhand"],"caveats at decision: secondhand (material) on #4, deps_unreviewed (low) on (retained by choice)",2,{state:"in_force",reopened_by:null,reason:null},H1x)],
  q2: [probe("H1",3,"yes","probe #3 (check): yes; decision in force: 1 (rests on check #3)"),
       probe("H1",7,"no","probe #7 (check): no; decision in force: 1 (#7 recorded after decision at line 6)"),
       probe("H1",2,"no","probe #2 (grant): no; grant is permittedBy, never in restsOn"),
       probe("H1",4,"yes","probe #4 (review): yes; decision in force: 1 (rests on review #4)")],
  q3: [
   st(1,"true","record","H1 explain(): \"Decision 1 (line 6): merge head 17; claim ready.\""),
   st(2,"true","record","H1 explain(): \"rests on: check #3, review #4, scan #5\""),
   st(3,"false","record","H1 record(): \"#4 review {\\\"approved\\\":1,\\\"relayed\\\":1} ... caveats=[secondhand, addedAtLine 4]\"; explain(): \"caveats at decision: secondhand (material) on #4, deps_unreviewed ...\". The review was relayed (secondhand), and decision 1 carried secondhand."),
   st(4,"true","record","H1 explain(): \"permitted by grant #2 (not rested on)\"; rests on #3,#4,#5 only."),
   st(5,"false","record","H1 explain(): \"rests on: check #3, review #4, scan #5\"; #7 was recorded at line 7 after the decision at line 6 and the basis is frozen."),
   st(6,"true","record","H1 view after line 7: {\"status\":\"merge\",\"head\":17,\"revisions\":1,\"reason\":\"ready\"}; explain(): \"in force\"")]
 },
 H2: {
  q1: [dec("H2",1,31,[3,4,5],[],"caveats at decision: deps_unreviewed (low) on (retained by choice)",2,{state:"superseded",reopened_by:7,reason:"scan found issues"},H2d1),
       dec("H2",2,31,[3,4,8],[],"caveats at decision: deps_unreviewed (low) on (retained by choice)",2,{state:"superseded",reopened_by:10,reason:"check withdrawn"},H2d2),
       dec("H2",3,31,[11,4,8],[],"caveats at decision: deps_unreviewed (low) on (retained by choice)",2,{state:"in_force",reopened_by:null,reason:null},H2d3)],
  q2: [probe("H2",11,"yes","probe #11 (check): yes; decision in force: 3 (rests on check #11)"),
       probe("H2",13,"no","probe #13 (check): no; decision in force: 3 (#13 recorded after decision 3)"),
       probe("H2",2,"no","probe #2 (grant): no; grants are never rested on"),
       probe("H2",5,"no","probe #5 (scan): no; decision in force 3 rests on scan #8, not #5")],
  q3: [
   st(1,"true","record","H2 explain(): \"Decision 1 ... reopened at line 7: scan found issues, citing #7\""),
   st(2,"false","record","H2 explain(): \"Decision 2 ... reopened at line 10: check withdrawn, citing #3\"; no db_aged event in H2."),
   st(3,"true","record","H2 explain(): \"Decision 2 (line 9) ... rests on: check #3, review #4, scan #8\""),
   st(4,"false","record","H2 record(): \"#3 check ... withdrawn={atLine:10,...}\"; explain(): decisions 1 and 2 both \"rests on: check #3\"."),
   st(5,"true","record","H2 explain(): \"Decision 3 (line 12) ... rests on: check #11, review #4, scan #8\""),
   st(6,"true","record","H2 explain(): Decision 3 \"in force\", \"rests on: check #11, review #4, scan #8\"; view after line 13: {\"status\":\"merge\",\"revisions\":3,\"reason\":\"ready\"}")]
 },
 H3: {
  q1: [dec("H3",1,42,[3,4,5],[],"caveats at decision: deps_unreviewed (low) on (retained by choice)",2,{state:"superseded",reopened_by:7,reason:"scan database stale"},H3d1),
       dec("H3",2,42,[3,4,5],[],"caveats at decision: deps_unreviewed (low) on (retained by choice); db_stale on #5 was added at line 7 and removed at line 8 (record(): addedAtLine 7, removedAtLine 8), before the decide at line 9",2,{state:"in_force",reopened_by:null,reason:null},H3d2)],
  q2: [probe("H3",5,"yes","probe #5 (scan): yes; decision in force: 2 (rests on scan #5)"),
       probe("H3",10,"no","probe #10 (check): no; #10 recorded after decision 2 (line 9)"),
       probe("H3",2,"no","probe #2 (grant): no; grants are never rested on"),
       probe("H3",4,"yes","probe #4 (review): yes; decision in force: 2 (rests on review #4)")],
  q3: [
   st(1,"true","record","H3 explain(): Decision 1 \"caveats at decision: deps_unreviewed (low) on (retained by choice)\"; record(): \"retained\":[\"deps_unreviewed\"]"),
   st(2,"true","record","H3 explain(): \"Decision 1 ... rests on: check #3, review #4, scan #5 ... reopened at line 7: scan database stale, citing #5\""),
   st(3,"false","record","H3 record(): \"#5 scan ... caveats=[{name:db_stale, addedAtLine:7, removedAtLine:8}]\"; decision 2 made at line 9 with \"caveats at decision: deps_unreviewed (low) on (retained by choice)\" only."),
   st(4,"false","record","H3 explain(): Decision 2 \"permitted by grant #2 (not rested on)\"; grant #12 was recorded at line 12, after decision 2 (line 9)."),
   st(5,"true","record","H3 explain(): \"Decision 2 (line 9) ... rests on: check #3, review #4, scan #5\""),
   st(6,"true","record","H3 run: \"line 13 {\\\"event\\\":\\\"decide\\\"} -> REFUSED: decide: decision 2 is in force and has not been reopened | view {status:merge, head:43, revisions:2, reason:ready}\" (view identical to after line 12); explain(): Decision 2 \"merge head 42\", \"in force\".")]
 }
}};
writeFileSync(new URL("../ANSWERS.json", import.meta.url), JSON.stringify(A, null, 2) + "\n");
