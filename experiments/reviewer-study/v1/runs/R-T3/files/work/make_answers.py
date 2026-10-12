import json
E="explain() dump via work/run.mjs (output in work/run-output.txt): "
def it(a,kind,text): return {"answer":a,"receipt":{"kind":kind,"text":text}}
def dec(n,val,rests,cav,grant,state,rb,reason,ex):
    return {"decision":n,
     "value":it(val,"record",E+ex[0]),
     "rests_on":it(rests,"record",E+ex[1]),
     "caveats":it(cav,"record",E+ex[2]+" (the deps_unreviewed entry is the retained-by-choice caveat, not an observation caveat)"),
     "retains":it(["deps_unreviewed"],"record",E+ex[2]+"; record().decisions[].retained = [\"deps_unreviewed\"]"),
     "grant":it(grant,"record",E+ex[3]),
     "state":it({"state":state,"reopened_by":rb,"reason":reason},"record",E+ex[4])}
H={}
# H1
H["H1"]={"q1":[dec(1,17,[3,4,5],["secondhand"],2,"in_force",None,None,[
 "'Decision 1 (line 6): merge head 17; claim ready.'",
 "'rests on: check #3, review #4, scan #5'",
 "'caveats at decision: secondhand (material) on #4, deps_unreviewed (low) on (retained by choice)'",
 "'permitted by grant #2 (not rested on)'",
 "'in force'; view after line 7: {\"status\":\"merge\",\"head\":17,\"revisions\":1,\"reason\":\"ready\"}"])],
 "q2":[
  {"probe":3,"answer":"yes","receipt":{"kind":"record","text":E+"decision 1 is in force and 'rests on: check #3, review #4, scan #5'; #3 is in that set (reopen condition gate.ts:252/259 tests restsOn membership)."}},
  {"probe":7,"answer":"no","receipt":{"kind":"record","text":E+"decision 1 in force 'rests on: check #3, review #4, scan #5'; #7 (check run 102, recorded after the decision) is not in it."}},
  {"probe":2,"answer":"no","receipt":{"kind":"record","text":E+"'permitted by grant #2 (not rested on)'; TASK.md: withdrawing a grant reopens nothing."}},
  {"probe":4,"answer":"yes","receipt":{"kind":"record","text":E+"decision 1 in force 'rests on: check #3, review #4, scan #5'; #4 is in it."}}],
 "q3":[
  {"statement":1,"answer":"true","receipt":{"kind":"record","text":E+"'Decision 1 (line 6): merge head 17'"}},
  {"statement":2,"answer":"true","receipt":{"kind":"record","text":E+"'rests on: check #3, review #4, scan #5'"}},
  {"statement":3,"answer":"false","receipt":{"kind":"record","text":E+"line 4 is relayed:1; 'caveats at decision: secondhand (material) on #4, deps_unreviewed (low) on (retained by choice)'"}},
  {"statement":4,"answer":"true","receipt":{"kind":"record","text":E+"'permitted by grant #2 (not rested on)'"}},
  {"statement":5,"answer":"false","receipt":{"kind":"record","text":E+"'rests on: check #3, review #4, scan #5' (frozen at line 6; #7 not included)"}},
  {"statement":6,"answer":"true","receipt":{"kind":"record","text":E+"'in force'; view after line 7 status merge, revisions 1"}}]}
# H2
H["H2"]={"q1":[
 dec(1,31,[3,4,5],[],2,"superseded",7,"scan found issues",[
  "'Decision 1 (line 6): merge head 31; claim ready.'","'rests on: check #3, review #4, scan #5'",
  "'caveats at decision: deps_unreviewed (low) on (retained by choice)'","'permitted by grant #2 (not rested on)'",
  "'reopened at line 7: scan found issues, citing #7'; decisions 2 and 3 made later (revisions 3)"]),
 dec(2,31,[3,4,8],[],2,"superseded",10,"check withdrawn",[
  "'Decision 2 (line 9): merge head 31; claim ready.'","'rests on: check #3, review #4, scan #8'",
  "'caveats at decision: deps_unreviewed (low) on (retained by choice)'","'permitted by grant #2 (not rested on)'",
  "'reopened at line 10: check withdrawn, citing #3'; decision 3 made later at line 12"]),
 dec(3,31,[11,4,8],[],2,"in_force",None,None,[
  "'Decision 3 (line 12): merge head 31; claim ready.'","'rests on: check #11, review #4, scan #8'",
  "'caveats at decision: deps_unreviewed (low) on (retained by choice)'","'permitted by grant #2 (not rested on)'",
  "'in force'; view after line 13: {\"status\":\"merge\",\"head\":31,\"revisions\":3,\"reason\":\"ready\"}"])],
 "q2":[
  {"probe":11,"answer":"yes","receipt":{"kind":"record","text":E+"decision 3 in force 'rests on: check #11, review #4, scan #8'"}},
  {"probe":13,"answer":"no","receipt":{"kind":"record","text":E+"decision 3 in force 'rests on: check #11, review #4, scan #8'; #13 not in it"}},
  {"probe":2,"answer":"no","receipt":{"kind":"record","text":E+"'permitted by grant #2 (not rested on)'; grants reopen nothing when withdrawn (TASK.md)"}},
  {"probe":5,"answer":"no","receipt":{"kind":"record","text":E+"#5 was rested on by decision 1 only (superseded); decision 3 in force 'rests on: check #11, review #4, scan #8'"}}],
 "q3":[
  {"statement":1,"answer":"true","receipt":{"kind":"record","text":E+"'reopened at line 7: scan found issues, citing #7' (decision 1)"}},
  {"statement":2,"answer":"false","receipt":{"kind":"record","text":E+"decision 2 'reopened at line 10: check withdrawn, citing #3'; no db_aged in H2"}},
  {"statement":3,"answer":"true","receipt":{"kind":"record","text":E+"Decision 2 'rests on: check #3, review #4, scan #8'"}},
  {"statement":4,"answer":"false","receipt":{"kind":"record","text":E+"check withdrawn at line 10 is #3 (obs #3 withdrawn atLine 10); decisions 1 and 2 both 'rests on: check #3, ...'"}},
  {"statement":5,"answer":"true","receipt":{"kind":"record","text":E+"Decision 3 'rests on: check #11, review #4, scan #8'"}},
  {"statement":6,"answer":"true","receipt":{"kind":"record","text":E+"Decision 3 'in force', 'rests on: check #11, review #4, scan #8'; view after line 13 status merge"}}]}
# H3
H["H3"]={"q1":[
 dec(1,42,[3,4,5],[],2,"superseded",7,"scan database stale",[
  "'Decision 1 (line 6): merge head 42; claim ready.'","'rests on: check #3, review #4, scan #5'",
  "'caveats at decision: deps_unreviewed (low) on (retained by choice)'","'permitted by grant #2 (not rested on)'",
  "'reopened at line 7: scan database stale, citing #5'; decision 2 made at line 9"]),
 dec(2,42,[3,4,5],[],2,"in_force",None,None,[
  "'Decision 2 (line 9): merge head 42; claim ready.'","'rests on: check #3, review #4, scan #5'",
  "'caveats at decision: deps_unreviewed (low) on (retained by choice)' (obs #5 db_stale addedAtLine 7, removedAtLine 8, before line 9)","'permitted by grant #2 (not rested on)'",
  "'in force'; line 13 decide REFUSED ('decision 2 is in force and has not been reopened'); final view {\"status\":\"merge\",\"head\":43,\"revisions\":2,\"reason\":\"ready\"}"])],
 "q2":[
  {"probe":5,"answer":"yes","receipt":{"kind":"record","text":E+"decision 2 in force 'rests on: check #3, review #4, scan #5'"}},
  {"probe":10,"answer":"no","receipt":{"kind":"record","text":E+"decision 2 in force 'rests on: check #3, review #4, scan #5'; #10 not in it"}},
  {"probe":2,"answer":"no","receipt":{"kind":"record","text":E+"'permitted by grant #2 (not rested on)'; grants reopen nothing when withdrawn (TASK.md)"}},
  {"probe":4,"answer":"yes","receipt":{"kind":"record","text":E+"decision 2 in force 'rests on: check #3, review #4, scan #5'"}}],
 "q3":[
  {"statement":1,"answer":"true","receipt":{"kind":"record","text":E+"Decision 1 'caveats at decision: deps_unreviewed (low) on (retained by choice)' only"}},
  {"statement":2,"answer":"true","receipt":{"kind":"record","text":E+"Decision 1 'rests on: ... scan #5'; 'reopened at line 7: scan database stale, citing #5'"}},
  {"statement":3,"answer":"false","receipt":{"kind":"record","text":E+"obs #5 caveat db_stale addedAtLine 7, removedAtLine 8 (db_refreshed); decision 2 made at line 9 with 'caveats at decision: deps_unreviewed (low) on (retained by choice)' only"}},
  {"statement":4,"answer":"false","receipt":{"kind":"record","text":E+"Decision 2 (line 9) 'permitted by grant #2 (not rested on)'; grant #12 was recorded after decision 2"}},
  {"statement":5,"answer":"true","receipt":{"kind":"record","text":E+"Decision 2 'rests on: check #3, review #4, scan #5'"}},
  {"statement":6,"answer":"true","receipt":{"kind":"record","text":E+"'line 13: REFUSED: decide: decision 2 is in force and has not been reopened'; view unchanged; Decision 2 'merge head 42', 'in force'"}}]}
json.dump({"run":"R-T3","histories":H},open("ANSWERS.json","w"),indent=2)
