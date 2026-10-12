import json
R=lambda k,t:{"kind":k,"text":t}
def item(a,k,t): return {"answer":a,"receipt":R(k,t)}
EXH1="explain gate.cav histories/H1.jsonl"
EXH2="explain gate.cav histories/H2.jsonl"
EXH3="explain gate.cav histories/H3.jsonl"
CODE_RET="gate.cav:152-158 `on decide commit merge ... retaining deps_unreviewed;` - the only caveat named in `retaining`."
def dec(n,value,rests,caveats,retains,grant,state,rcv):
    v,rs,cv,rt,g,st=rcv
    return {"decision":n,"value":item(value,*v),"rests_on":item(rests,*rs),"caveats":item(caveats,*cv),
            "retains":item(retains,*rt),"grant":item(grant,*g),"state":item(state,*st)}
H={}
# ---------------- H1
H["H1"]={"q1":[dec(1,17,[3,4,5],["secondhand"],["deps_unreviewed"],2,
  {"state":"in_force","reopened_by":None,"reason":None},
  [("record",EXH1+": `merge@1 = 17  in force`"),
   ("record",EXH1+": `based on checks@1, reviewer, scanner` / `checks@1: observed at #3 check {\"run\":101,\"passed\":1}` / `reviewer: observed at #4 review {\"approved\":1,\"relayed\":1}` / `scanner: observed at #5 scan {\"findings\":0}`"),
   ("run","work/gate_probe.cav (copy with `on decide set probe_review = qualified(0, reviewer)` and probe_scan) replayed on H1: at line 6 probe_review provenance {'evidence': ['reviewer'], 'caveats': ['secondhand']}, probe_scan {'evidence': ['scanner'], 'caveats': []}; checks@1 caveats [] (explain --json evidence). Record: explain --json merge@1 grounds caveats ['deps_unreviewed','secondhand'], of which deps_unreviewed is the retained one (gate.cav:158) and secondhand is from gate.cav:92 `on review when relayed == 1 qualify reviewer with secondhand`."),
   ("code",CODE_RET+" explain prints `retaining: deps_unreviewed, secondhand`, mixing in the grounds caveat secondhand; secondhand comes from the observation (#4), not the retaining clause."),
   ("record",EXH1+": `permitted by grants@1 for 17`; evidence `grants@1 = 17 supports may_merge  (#2 approve)`"),
   ("record",EXH1+": history only `#6 decide: committed because checks@1, reviewer, scanner; retaining: deps_unreviewed, secondhand` (no reopening); replay view after line 7 {\"status\": \"merge\", \"revisions\": 1, \"reason\": \"ready\"}")])],
 "q2":[
  {"probe":3,"answer":"yes","receipt":R("record",EXH1+": merge@1 in force, `based on checks@1, reviewer, scanner`, `checks@1: observed at #3`. Task: withdrawing an observation the decision in force rests on reopens it (gate.cav:122-123 implements this for checks via rests_on_withdrawn).")},
  {"probe":7,"answer":"no","receipt":R("record","dependents gate.cav checks histories/H1.jsonl: `merge@1 = 17  in force  based on checks@1`; checks@2 (#7) appears only under Claims. Not in grounds, so withdrawing it reopens nothing (gate.cav:122 rests_on_withdrawn false).")},
  {"probe":2,"answer":"no","receipt":R("record",EXH1+": `permitted by grants@1 for 17` - grants@1 (#2) is the grant, not in `based on checks@1, reviewer, scanner`. Task: withdrawing a grant reopens nothing.")},
  {"probe":4,"answer":"yes","receipt":R("record","dependents gate.cav reviewer histories/H1.jsonl: `merge@1 = 17  in force  based on reviewer`; explain: `reviewer: observed at #4`.")}],
 "q3":[
  {"statement":1,"answer":"true","receipt":R("record",EXH1+": `merge@1 = 17  in force`")},
  {"statement":2,"answer":"true","receipt":R("record",EXH1+": `based on checks@1, reviewer, scanner`; observed at #3, #4, #5")},
  {"statement":3,"answer":"false","receipt":R("record","Line 4 is `review {\"approved\":1,\"relayed\":1}` (relayed, not first-hand); "+EXH1+": `reviewer supports ready  caveats: secondhand`; `#6 decide: committed ... retaining: deps_unreviewed, secondhand`")},
  {"statement":4,"answer":"true","receipt":R("record",EXH1+": `permitted by grants@1 for 17`, grants@1 from `#2 approve`; grounds `based on checks@1, reviewer, scanner` exclude grants@1")},
  {"statement":5,"answer":"false","receipt":R("record","dependents gate.cav checks histories/H1.jsonl: `merge@1 = 17  in force  based on checks@1` only; checks@2 (#7) is not in the grounds, which are frozen at #6")},
  {"statement":6,"answer":"true","receipt":R("record","replay gate.cav histories/H1.jsonl, line 7 view {\"head\": 17, \"reason\": \"ready\", \"revisions\": 1, \"status\": \"merge\"}; explain `merge@1 = 17  in force`")}]}
# ---------------- H2
H["H2"]={"q1":[
 dec(1,31,[3,4,5],[],["deps_unreviewed"],2,{"state":"superseded","reopened_by":7,"reason":"scan found issues"},
  [("record",EXH2+": `merge@1 = 31  superseded`"),
   ("record",EXH2+": `based on checks@1, reviewer, scanner` / `checks@1: observed at #3` / `reviewer: observed at #4 review {\"approved\":1,\"relayed\":0}` / `scanner: observed at #5 scan {\"findings\":0}`"),
   ("record",EXH2+": `#6 decide: committed because checks@1, reviewer, scanner; retaining: deps_unreviewed`; replay snapshot commitment_grounds merge@1 caveats ['deps_unreviewed'] - only the retained caveat, so the observations carried none"),
   ("code",CODE_RET),
   ("record",EXH2+": `permitted by grants@1 for 31`, `grants@1 = 31 supports may_merge  (#2 approve)`"),
   ("record",EXH2+": `#7 scan: reopened because scanner@2`, later `merge@2` committed so merge@1 `superseded`; replay view after line 7 {\"reason\": \"scan found issues\", \"revisions\": 1, \"status\": \"reopened\"}")]),
 dec(2,31,[3,4,8],[],["deps_unreviewed"],2,{"state":"superseded","reopened_by":10,"reason":"check withdrawn"},
  [("record",EXH2+": `merge@2 = 31  superseded`"),
   ("record",EXH2+": `based on checks@1, reviewer, scanner@3` / `checks@1: observed at #3` / `reviewer: observed at #4` / `scanner@3: observed at #8 scan {\"findings\":0}`"),
   ("record",EXH2+": `#9 decide: committed because checks@1, reviewer, scanner@3; retaining: deps_unreviewed`; replay commitment_grounds merge@2 caveats ['deps_unreviewed']"),
   ("code",CODE_RET),
   ("record",EXH2+": merge@2 `permitted by grants@1 for 31` (#2 approve)"),
   ("record",EXH2+": `#10 recheck: reopened because requeue`, `checks@1 has since been withdrawn at #10 because requeue`; merge@3 committed later so `superseded`; replay view after line 10 {\"reason\": \"check withdrawn\", \"revisions\": 2, \"status\": \"reopened\"}")]),
 dec(3,31,[4,8,11],[],["deps_unreviewed"],2,{"state":"in_force","reopened_by":None,"reason":None},
  [("record",EXH2+": `merge@3 = 31  in force`"),
   ("record",EXH2+": `based on checks@2, reviewer, scanner@3` / `checks@2: observed at #11 check {\"run\":202,\"passed\":1}` / `reviewer: observed at #4` / `scanner@3: observed at #8`"),
   ("record",EXH2+": `#12 decide: committed because reviewer, scanner@3, checks@2; retaining: deps_unreviewed`; replay commitment_grounds merge@3 caveats ['deps_unreviewed']"),
   ("code",CODE_RET),
   ("record",EXH2+": merge@3 `permitted by grants@1 for 31`"),
   ("record",EXH2+": merge@3 history only `#12 decide: committed`; replay view after line 13 {\"reason\": \"ready\", \"revisions\": 3, \"status\": \"merge\"}")])],
 "q2":[
  {"probe":11,"answer":"yes","receipt":R("record",EXH2+": merge@3 in force `based on checks@2, reviewer, scanner@3`, `checks@2: observed at #11`")},
  {"probe":13,"answer":"no","receipt":R("record","dependents gate.cav checks histories/H2.jsonl: `merge@3 = 31  in force  based on checks@2`; checks@3 (#13) only under Claims")},
  {"probe":2,"answer":"no","receipt":R("record","dependents gate.cav grants histories/H2.jsonl: `merge@3 = 31  in force  permitted by grants@1` - a grant, not grounds; task: withdrawing a grant reopens nothing")},
  {"probe":5,"answer":"no","receipt":R("record","dependents gate.cav scanner histories/H2.jsonl: `merge@1 ... based on scanner` (the #5 occurrence) but `merge@3 = 31  in force  based on scanner@3` (#8); the #5 scan is in no in-force decision's grounds")}],
 "q3":[
  {"statement":1,"answer":"true","receipt":R("record",EXH2+": `#7 scan: reopened because scanner@2`; replay view after line 7 reason \"scan found issues\", status \"reopened\"")},
  {"statement":2,"answer":"false","receipt":R("record",EXH2+": `#10 recheck: reopened because requeue`; replay view after line 10 reason \"check withdrawn\". No db_aged event in H2.")},
  {"statement":3,"answer":"true","receipt":R("record",EXH2+": merge@2 `based on checks@1, reviewer, scanner@3`; observed at #3, #4, #8")},
  {"statement":4,"answer":"false","receipt":R("record","dependents gate.cav checks histories/H2.jsonl: `checks@1 withdrawn at #10`; `merge@1 = 31  superseded  based on checks@1` and `merge@2 = 31  superseded  based on checks@1`")},
  {"statement":5,"answer":"true","receipt":R("record",EXH2+": merge@3 `based on checks@2, reviewer, scanner@3`; checks@2 #11, reviewer #4, scanner@3 #8")},
  {"statement":6,"answer":"true","receipt":R("record",EXH2+": `merge@3 = 31  in force  based on checks@2, reviewer, scanner@3`; checks@3 (#13) not in grounds")}]}
# ---------------- H3
H["H3"]={"q1":[
 dec(1,42,[3,4,5],[],["deps_unreviewed"],2,{"state":"superseded","reopened_by":7,"reason":"scan database stale"},
  [("record",EXH3+": `merge@1 = 42  superseded`"),
   ("record",EXH3+": `based on checks@1, reviewer, scanner` / observed at #3, #4 (relayed 0), #5"),
   ("record",EXH3+" --json merge@1 grounds caveats ['deps_unreviewed']; journal `#6 decide: committed because checks@1, reviewer, scanner; retaining: deps_unreviewed` (db_stale only applied at #7)"),
   ("code",CODE_RET),
   ("record",EXH3+": merge@1 `permitted by grants@1 for 42` (#2 approve)"),
   ("record",EXH3+": `#7 db_aged: reopened because scanner (caveats: db_stale)`; merge@2 later, so superseded; replay view after line 7 {\"reason\": \"scan database stale\", \"status\": \"reopened\"}")]),
 dec(2,42,[3,4,5],[],["deps_unreviewed"],2,{"state":"in_force","reopened_by":None,"reason":None},
  [("record",EXH3+": `merge@2 = 42  in force`"),
   ("record",EXH3+": merge@2 `based on checks@1, reviewer, scanner` / observed at #3, #4, #5"),
   ("run","Record: explain --json merge@2 grounds caveats ['deps_unreviewed'], journal `#9 decide: ... retaining: deps_unreviewed`, snapshot taken_back [{'evidence': 'scanner', 'caveat': 'db_stale', 'sequence': 8}]. Run: work/gate_probe.cav replayed on H3, line 9 probe_scan = qualified(0, scanner) provenance {'evidence': ['scanner'], 'caveats': []}. Note: explain's text prints `retaining: deps_unreviewed, db_stale` for merge@2 because the commitment's lineage *keeps* db_stale (lineage `kept: ['db_stale']`, from a guard/record, not from the scan's live relation); grounds do not include it."),
   ("code",CODE_RET+" (the extra db_stale in explain's `retaining:` line is a lineage-kept caveat, not authored in `retaining`)"),
   ("record",EXH3+": merge@2 `permitted by grants@1 for 42`; grants@2 (#12, for 43) came after"),
   ("record",EXH3+": merge@2 history only `#9 decide: committed`; line 13 `decide  refused (policy): a merge decision is in force`; replay view after line 12 status \"merge\", revisions 2")])],
 "q2":[
  {"probe":5,"answer":"yes","receipt":R("record","dependents gate.cav scanner histories/H3.jsonl: `merge@2 = 42  in force  based on scanner`; explain `scanner: observed at #5`")},
  {"probe":10,"answer":"no","receipt":R("record",EXH3+": merge@2 `based on checks@1, reviewer, scanner`; checks@2 (#10) not in grounds")},
  {"probe":2,"answer":"no","receipt":R("record",EXH3+": merge@2 `permitted by grants@1 for 42` - a grant, not grounds; task: withdrawing a grant reopens nothing")},
  {"probe":4,"answer":"yes","receipt":R("record","dependents gate.cav reviewer histories/H3.jsonl: `merge@2 = 42  in force  based on reviewer`; observed at #4")}],
 "q3":[
  {"statement":1,"answer":"true","receipt":R("record",EXH3+": `#6 decide: committed because checks@1, reviewer, scanner; retaining: deps_unreviewed`; --json merge@1 grounds caveats ['deps_unreviewed']")},
  {"statement":2,"answer":"true","receipt":R("record",EXH3+": `#7 db_aged: reopened because scanner (caveats: db_stale)`; replay view after line 7 reason \"scan database stale\"")},
  {"statement":3,"answer":"false","receipt":R("run","replay snapshot after line 8: taken_back [{'evidence': 'scanner', 'caveat': 'db_stale', 'sequence': 8, 'event': 'db_refreshed'}]; work/gate_probe.cav on H3 line 9: qualified(0, scanner) caveats []; merge@2 grounds caveats ['deps_unreviewed']. (explain's `retaining: deps_unreviewed, db_stale` line is lineage-kept, not the scan's caveat.)")},
  {"statement":4,"answer":"false","receipt":R("record",EXH3+": merge@2 `permitted by grants@1 for 42`; grants@1 is `#2 approve`. Line 12's grant (grants@2 = 43) came after decision 2 (#9).")},
  {"statement":5,"answer":"true","receipt":R("record",EXH3+": merge@2 `based on checks@1, reviewer, scanner`; observed at #3, #4, #5")},
  {"statement":6,"answer":"true","receipt":R("record",EXH3+": `13  decide  refused (policy): a merge decision is in force`; `merge@2 = 42  in force`")}]}
json.dump({"run":"R-C1","histories":H},open("ANSWERS.json","w"),indent=1)
