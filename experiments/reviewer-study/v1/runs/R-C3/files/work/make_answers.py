import json
def it(a, kind, text): return {"answer": a, "receipt": {"kind": kind, "text": text}}
def R(t): return ("record", t)
EX = "caveat explain gate.cav histories/%s.jsonl (saved work/%s.explain.txt)"
JR = "caveat replay gate.cav histories/%s.jsonl, final snapshot (saved work/%s.replay.jsonl)"

def dec(h, n, value, rests, rests_txt, cav, cav_txt, grant, grant_txt, state, state_txt):
    ex = EX % (h, h); jr = JR % (h, h)
    return {
      "decision": n,
      "value": it(value, "record", f"{ex}: 'merge@{n} = {value}'; replay decision_journal committed entry for merge@{n}: \"value\": {value}"),
      "rests_on": it(rests, "record", f"{ex}: {rests_txt}"),
      "caveats": it(cav, "record", cav_txt),
      "retains": it(["deps_unreviewed"], "record", f"gate.cav:158 'retaining deps_unreviewed'; {jr}: commitment_grounds merge@{n} caveats include deps_unreviewed, which no observation carries (gate.cav:18, no qualify of it)"),
      "grant": it(grant, "record", f"{jr}: commitment_permissions merge@{n} {{\"grant\": \"grants@1\", scope granted=required}}; {ex}: 'grants@1 = ... (#2 approve)'; {grant_txt}"),
      "state": it(state, "record", state_txt),
    }

H1 = {"q1": [dec("H1",1,17,[3,4,5],
  "'based on checks@1, reviewer, scanner' / 'checks@1: observed at #3' / 'reviewer: observed at #4' / 'scanner: observed at #5'; commitment_grounds merge@1 evidence [checks@1, reviewer, scanner]",
  ["secondhand"], "replay commitment_grounds merge@1 caveats [deps_unreviewed, secondhand]; journal committed entry caveats [deps_unreviewed, secondhand]; explain: 'reviewer supports ready  caveats: secondhand' (line 4 relayed=1, gate.cav:92). deps_unreviewed is retained by choice, not carried, so it is listed under retains.",
  2, "grants@1 is line 2 (only grant).",
  {"state":"in_force","reopened_by":None,"reason":None},
  "explain H1: 'merge@1 = 17  in force'; journal has only the committed entry; view after line 7: status 'merge', reason 'ready'")],
 "q2": [
  {"probe":3,"answer":"yes","receipt":{"kind":"record","text":"explain H1: merge@1 in force 'based on checks@1, reviewer, scanner', 'checks@1: observed at #3'. TASK: withdrawing an observation the decision in force rests on reopens it (program: gate.cav:122-123 rests_on_withdrawn)."}},
  {"probe":7,"answer":"no","receipt":{"kind":"run","text":"work/H1_plus_recheck.jsonl (H1 + recheck) replayed: recheck withdraws checks@2 (line 7), effects [withdraw checks@2], view stays status 'merge', reason 'ready', revisions 1. Also explain: merge@1 not based on checks@2."}},
  {"probe":2,"answer":"no","receipt":{"kind":"record","text":"line 2 is grants@1, the grant: dependents H1 grants: 'merge@1 = 17 in force  permitted by grants@1' (not 'based on'); commitment_grounds merge@1 evidence excludes grants@1. TASK: withdrawing a grant reopens nothing."}},
  {"probe":4,"answer":"yes","receipt":{"kind":"record","text":"dependents H1 reviewer: 'merge@1 = 17  in force  based on reviewer'; explain 'reviewer: observed at #4'."}}],
 "q3": [
  {"statement":1,"answer":"true","receipt":{"kind":"record","text":"explain H1: 'merge@1 = 17  in force'"}},
  {"statement":2,"answer":"true","receipt":{"kind":"record","text":"explain H1: 'based on checks@1, reviewer, scanner', observed at #3, #4, #5"}},
  {"statement":3,"answer":"false","receipt":{"kind":"record","text":"line 4 has relayed=1; explain H1: 'reviewer supports ready  caveats: secondhand', 'retaining: deps_unreviewed, secondhand'; commitment_grounds merge@1 caveats [deps_unreviewed, secondhand]"}},
  {"statement":4,"answer":"true","receipt":{"kind":"record","text":"explain H1: 'permitted by grants@1 for 17', grants@1 (#2 approve); grounds exclude grants@1"}},
  {"statement":5,"answer":"false","receipt":{"kind":"record","text":"explain H1: merge@1 'based on checks@1, reviewer, scanner'; checks@2 (#7) is not in the basis; grounds frozen at line 6"}},
  {"statement":6,"answer":"true","receipt":{"kind":"record","text":"replay H1 line 7 view: status 'merge', reason 'ready'; explain 'merge@1 = 17  in force'"}}]}

H2 = {"q1": [
 dec("H2",1,31,[3,4,5],"merge@1 'based on checks@1, reviewer, scanner'; checks@1 #3, reviewer #4, scanner #5",
  [], "replay H2 commitment_grounds merge@1 caveats [deps_unreviewed] only (retained by choice); journal committed entry caveats [deps_unreviewed]; review line 4 relayed=0, scan not stale",
  2,"only grant is line 2",
  {"state":"superseded","reopened_by":7,"reason":"scan found issues"},
  "explain H2: 'merge@1 = 31  superseded', '#7 scan: reopened because scanner@2'; replay view after line 7: reason 'scan found issues', status 'reopened'"),
 dec("H2",2,31,[3,4,8],"merge@2 'based on checks@1, reviewer, scanner@3'; checks@1 #3, reviewer #4, scanner@3 #8",
  [], "replay H2 commitment_grounds merge@2 caveats [deps_unreviewed] only; journal committed entry (seq 9) caveats [deps_unreviewed]; checks@1 was withdrawn only later, at line 10",
  2,"only grant is line 2",
  {"state":"superseded","reopened_by":10,"reason":"check withdrawn"},
  "explain H2: 'merge@2 = 31  superseded', '#10 recheck: reopened because requeue'; replay view after line 10: reason 'check withdrawn', status 'reopened'"),
 dec("H2",3,31,[11,4,8],"merge@3 'based on checks@2, reviewer, scanner@3'; checks@2 #11, reviewer #4, scanner@3 #8",
  [], "replay H2 commitment_grounds merge@3 caveats [deps_unreviewed] only; journal committed entry (seq 12) caveats [deps_unreviewed]",
  2,"only grant is line 2",
  {"state":"in_force","reopened_by":None,"reason":None},
  "explain H2: 'merge@3 = 31  in force', no reopen entry; view after line 13: status 'merge', reason 'ready'")],
 "q2": [
  {"probe":11,"answer":"yes","receipt":{"kind":"record","text":"explain H2: merge@3 in force 'based on checks@2, reviewer, scanner@3', 'checks@2: observed at #11'"}},
  {"probe":13,"answer":"no","receipt":{"kind":"run","text":"work/H2_plus_recheck.jsonl replayed: recheck withdraws checks@3 (line 13), view stays status 'merge', reason 'ready', revisions 3; merge@3 not based on checks@3"}},
  {"probe":2,"answer":"no","receipt":{"kind":"record","text":"line 2 is grants@1, the grant: commitment_permissions merge@3 grant grants@1; commitment_grounds merge@3 evidence [checks@2, reviewer, scanner@3] excludes it. TASK: withdrawing a grant reopens nothing."}},
  {"probe":5,"answer":"no","receipt":{"kind":"record","text":"dependents H2 scanner: 'merge@3 = 31  in force  based on scanner@3' (#8); scanner (#5) is basis only of superseded merge@1"}}],
 "q3": [
  {"statement":1,"answer":"true","receipt":{"kind":"record","text":"explain H2: merge@1 '#7 scan: reopened because scanner@2'; view after line 7 reason 'scan found issues'"}},
  {"statement":2,"answer":"false","receipt":{"kind":"record","text":"explain H2: merge@2 '#10 recheck: reopened because requeue'; view after line 10 reason 'check withdrawn'; no db_aged in H2"}},
  {"statement":3,"answer":"true","receipt":{"kind":"record","text":"explain H2: merge@2 'based on checks@1, reviewer, scanner@3' (#3, #4, #8)"}},
  {"statement":4,"answer":"false","receipt":{"kind":"record","text":"dependents H2 checks: 'checks@1 withdrawn at #10'; 'merge@1 ... based on checks@1', 'merge@2 ... based on checks@1' (checks@1 is line 3)"}},
  {"statement":5,"answer":"true","receipt":{"kind":"record","text":"explain H2: merge@3 'based on checks@2, reviewer, scanner@3' (#11, #4, #8)"}},
  {"statement":6,"answer":"true","receipt":{"kind":"record","text":"explain H2: 'merge@3 = 31  in force', basis excludes checks@3 (#13)"}}]}

H3 = {"q1": [
 dec("H3",1,42,[3,4,5],"merge@1 'based on checks@1, reviewer, scanner'; #3, #4, #5",
  [], "replay H3 commitment_grounds merge@1 caveats [deps_unreviewed]; journal committed entry (seq 6) caveats [deps_unreviewed]; db_stale was added only at line 7",
  2,"grants@1 is line 2; grants@2 (line 12) came later",
  {"state":"superseded","reopened_by":7,"reason":"scan database stale"},
  "explain H3: 'merge@1 = 42  superseded', '#7 db_aged: reopened because scanner (caveats: db_stale)'; view after line 7 reason 'scan database stale'"),
 dec("H3",2,42,[3,4,5],"merge@2 'based on checks@1, reviewer, scanner'; #3, #4, #5",
  [], "replay H3: taken_back [{scanner, db_stale, sequence 8, db_refreshed}] before the decide at line 9; commitment_grounds merge@2 caveats [deps_unreviewed]; journal committed entry (seq 9) caveats [deps_unreviewed]; run work/H3_first8.jsonl: a db_refreshed right after line 8 is refused 'the latest scan does not carry db_stale'. Note: commitment_bases merge@2 lists db_stale as 'kept' and explain prints 'retaining: deps_unreviewed, db_stale'; that db_stale is inherited from merge@1's frozen reopening record, not carried by any observation at line 9.",
  2,"grants@1 is line 2; decision 2 at line 9 precedes grants@2 (line 12)",
  {"state":"in_force","reopened_by":None,"reason":None},
  "explain H3: 'merge@2 = 42  in force', no reopen entry; line 13 decide 'refused (policy): a merge decision is in force'")],
 "q2": [
  {"probe":5,"answer":"yes","receipt":{"kind":"record","text":"dependents H3 scanner: 'merge@2 = 42  in force  based on scanner'; scanner observed at #5"}},
  {"probe":10,"answer":"no","receipt":{"kind":"run","text":"work/H3_plus_recheck.jsonl replayed: recheck withdraws checks@2 (line 10), view stays status 'merge', reason 'ready', revisions 2"}},
  {"probe":2,"answer":"no","receipt":{"kind":"record","text":"line 2 is grants@1, the grant: commitment_permissions merge@2 grant grants@1; commitment_grounds merge@2 evidence [checks@1, reviewer, scanner]. TASK: withdrawing a grant reopens nothing."}},
  {"probe":4,"answer":"yes","receipt":{"kind":"record","text":"explain H3: merge@2 'based on checks@1, reviewer, scanner', 'reviewer: observed at #4'"}}],
 "q3": [
  {"statement":1,"answer":"true","receipt":{"kind":"record","text":"replay H3 commitment_grounds merge@1 caveats [deps_unreviewed]; journal seq 6 caveats [deps_unreviewed]; gate.cav:158 retaining deps_unreviewed"}},
  {"statement":2,"answer":"true","receipt":{"kind":"record","text":"explain H3: '#7 db_aged: reopened because scanner (caveats: db_stale)'; view after line 7 reason 'scan database stale'"}},
  {"statement":3,"answer":"false","receipt":{"kind":"record","text":"replay H3 taken_back: scanner db_stale taken back at sequence 8 (db_refreshed), before decide at 9; grounds/journal for merge@2 caveats [deps_unreviewed]; run work/H3_first8.jsonl: second db_refreshed refused 'the latest scan does not carry db_stale'"}},
  {"statement":4,"answer":"false","receipt":{"kind":"record","text":"replay H3 commitment_permissions merge@2 grant 'grants@1' (line 2); grants@2 (line 12) recorded after decision 2"}},
  {"statement":5,"answer":"true","receipt":{"kind":"record","text":"explain H3: merge@2 'based on checks@1, reviewer, scanner' (#3, #4, #5)"}},
  {"statement":6,"answer":"true","receipt":{"kind":"record","text":"explain H3: '13 decide refused (policy): a merge decision is in force'; 'merge@2 = 42  in force'"}}]}

json.dump({"run":"R-C3","histories":{"H1":H1,"H2":H2,"H3":H3}}, open("/root/rs1-runs/R-C3/ANSWERS.json","w"), indent=2)
