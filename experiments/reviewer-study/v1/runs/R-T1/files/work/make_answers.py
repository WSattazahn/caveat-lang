import json
def R(kind,text): return {"kind":kind,"text":text}
def it(a,kind,text): return {"answer":a,"receipt":R(kind,text)}
RUN="work/run.mjs (compiled copy work/gate.mjs of gate.ts, all lines replayed into one fresh createPolicy(); output in work/replay-output.txt)"
def dec(n,value,rests,cav,grant,state,exp,code_rest="gate.ts:239",):
    e="explain() via "+RUN+": "+exp
    return {"decision":n,
      "value":it(value,"record",e),
      "rests_on":it(rests,"record",e),
      "caveats":it(cav,"record",e+" (caveats on rested-on observations only; deps_unreviewed is listed separately as retained)"),
      "retains":it(["deps_unreviewed"],"code","gate.ts:241 retained: [\"deps_unreviewed\"]; explain lists 'deps_unreviewed (low) on (retained by choice)'"),
      "grant":it(grant,"record",e),
      "state":it(state,"record",e)}
def st(s,by=None,reason=None): return {"state":s,"reopened_by":by,"reason":reason}
def probe(p,a,t): return {"probe":p,"answer":a,"receipt":R("record",t)}
def stm(n,a,kind,t): return {"statement":n,"answer":a,"receipt":R(kind,t)}

H1x="'Decision 1 (line 6): merge head 17; claim ready. / rests on: check #3, review #4, scan #5 / caveats at decision: secondhand (material) on #4, deps_unreviewed (low) on (retained by choice) / permitted by grant #2 (not rested on) / in force'"
H2d1="'Decision 1 (line 6): merge head 31 / rests on: check #3, review #4, scan #5 / caveats at decision: deps_unreviewed (low) on (retained by choice) / permitted by grant #2 (not rested on) / reopened at line 7: scan found issues, citing #7'; superseded by Decision 2 (line 9)"
H2d2="'Decision 2 (line 9): merge head 31 / rests on: check #3, review #4, scan #8 / caveats at decision: deps_unreviewed (low) on (retained by choice) / permitted by grant #2 (not rested on) / reopened at line 10: check withdrawn, citing #3'; superseded by Decision 3 (line 12)"
H2d3="'Decision 3 (line 12): merge head 31 / rests on: check #11, review #4, scan #8 / caveats at decision: deps_unreviewed (low) on (retained by choice) / permitted by grant #2 (not rested on) / in force'"
H3d1="'Decision 1 (line 6): merge head 42 / rests on: check #3, review #4, scan #5 / caveats at decision: deps_unreviewed (low) on (retained by choice) / permitted by grant #2 (not rested on) / reopened at line 7: scan database stale, citing #5'; superseded by Decision 2 (line 9)"
H3d2="'Decision 2 (line 9): merge head 42 / rests on: check #3, review #4, scan #5 / caveats at decision: deps_unreviewed (low) on (retained by choice) / permitted by grant #2 (not rested on) / in force'; line 13 decide REFUSED: 'decide: decision 2 is in force and has not been reopened'"

A={"run":"R-T1","histories":{
"H1":{"q1":[dec(1,17,[3,4,5],["secondhand"],2,st("in_force"),H1x)],
 "q2":[probe(3,"yes","Decision 1 in force, "+H1x+"; #3 is its check"),
       probe(7,"no","Decision 1 rests on #3,#4,#5 only ("+H1x+"); #7 is a later check not rested on"),
       probe(2,"no","#2 is a grant: 'permitted by grant #2 (not rested on)'; TASK says withdrawing a grant reopens nothing"),
       probe(4,"yes","Decision 1 in force rests on 'review #4'")],
 "q3":[stm(1,"true","record",H1x),stm(2,"true","record",H1x),
       stm(3,"false","record","Line 4 review has relayed 1; record shows obs #4 caveats secondhand (material) addedAtLine 4; explain: 'caveats at decision: secondhand (material) on #4, deps_unreviewed ...'"),
       stm(4,"true","record","explain: 'permitted by grant #2 (not rested on)'"),
       stm(5,"false","record","explain: 'rests on: check #3, review #4, scan #5'; #7 recorded after decision at line 6 and rests-on is frozen (gate.ts:239)"),
       stm(6,"true","record","view after line 7: {\"status\":\"merge\",\"head\":17,\"revisions\":1,\"reason\":\"ready\"}; explain: 'in force'")]},
"H2":{"q1":[dec(1,31,[3,4,5],[],2,st("superseded",7,"scan found issues"),H2d1),
            dec(2,31,[3,4,8],[],2,st("superseded",10,"check withdrawn"),H2d2),
            dec(3,31,[11,4,8],[],2,st("in_force"),H2d3)],
 "q2":[probe(11,"yes","Decision 3 in force: "+H2d3),
       probe(13,"no","Decision 3 rests on #11,#4,#8 ("+H2d3+"); #13 not among them"),
       probe(2,"no","#2 is a grant: 'permitted by grant #2 (not rested on)'; withdrawing a grant reopens nothing"),
       probe(5,"no","Decision 3 rests on scan #8, not #5 ("+H2d3+")")],
 "q3":[stm(1,"true","record",H2d1+"; view after line 7 status reopened reason 'scan found issues'"),
       stm(2,"false","record",H2d2+"; no db_aged in H2"),
       stm(3,"true","record",H2d2),
       stm(4,"false","record","record: obs #3 withdrawn atLine 10; decisions 1 and 2 'rests on: check #3'"),
       stm(5,"true","record",H2d3),
       stm(6,"true","record",H2d3+"; view after line 13 status merge revisions 3")]},
"H3":{"q1":[dec(1,42,[3,4,5],[],2,st("superseded",7,"scan database stale"),H3d1),
            dec(2,42,[3,4,5],[],2,st("in_force"),H3d2)],
 "q2":[probe(5,"yes","Decision 2 in force: "+H3d2),
       probe(10,"no","Decision 2 rests on #3,#4,#5 ("+H3d2+"); #10 is later"),
       probe(2,"no","#2 is a grant: 'permitted by grant #2 (not rested on)'; withdrawing a grant reopens nothing"),
       probe(4,"yes","Decision 2 in force rests on 'review #4'")],
 "q3":[stm(1,"true","record",H3d1),
       stm(2,"true","record",H3d1),
       stm(3,"false","record","record: obs #5 db_stale addedAtLine 7 removedAtLine 8; decision 2 at line 9: "+H3d2),
       stm(4,"false","record",H3d2+"; grant #12 recorded after decision 2"),
       stm(5,"true","record",H3d2),
       stm(6,"true","record",H3d2+"; view after lines 12 and 13 identical {\"status\":\"merge\",\"head\":43,\"revisions\":2,\"reason\":\"ready\"} (head 43 came from line 11, not 13); decision 2 value 42")]}}}
json.dump(A,open("../ANSWERS.json","w"),indent=2)
