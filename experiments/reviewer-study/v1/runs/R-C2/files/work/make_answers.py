import json
def it(a,k,t): return {"answer":a,"receipt":{"kind":k,"text":t}}
def R(t): return ("record",t)
def dec(n,value,rests,cav,ret,grant,state):
    return {"decision":n,"value":it(value,*value_r[n]),"rests_on":it(rests[0],*rests[1]),
            "caveats":it(cav[0],*cav[1]),"retains":it(ret[0],*ret[1]),"grant":it(grant[0],*grant[1]),
            "state":it(state[0],*state[1])}
E="caveat-lang explain gate.cav histories/{h}.jsonl"
J="caveat-lang replay gate.cav histories/{h}.jsonl (final snapshot decision_journal)"
out={"run":"R-C2","histories":{}}

# ---------------- H1
h="H1"
value_r={1:R(E.format(h=h)+": 'merge@1 = 17  in force'; journal: {\"commitment\": \"merge@1\", \"change\": \"committed\", \"sequence\": 6, \"value\": 17}")}
q1=[dec(1,17,
  ([3,4,5],R(E.format(h=h)+": 'based on checks@1, reviewer, scanner' / 'checks@1: observed at #3 check' / 'reviewer: observed at #4 review' / 'scanner: observed at #5 scan'; replay commitment_grounds merge@1 evidence [checks@1, reviewer, scanner]")),
  (["secondhand"],R(J.format(h=h)+": merge@1 committed seq 6 caveats [\"deps_unreviewed\", \"secondhand\"]; relations: secondhand qualifies reviewer; explain: 'reviewer supports ready  caveats: secondhand' (review #4 relayed=1). deps_unreviewed is the retained-by-choice caveat, not carried by an observation (gate.cav:158)")),
  (["deps_unreviewed"],("code","gate.cav:158 'retaining deps_unreviewed;' (only caveat retained by choice); explain: '#6 decide: committed because checks@1, reviewer, scanner; retaining: deps_unreviewed, secondhand'")),
  (2,R(E.format(h=h)+": 'permitted by grants@1 for 17'; 'grants@1 = 17 supports may_merge  (#2 approve)'; replay commitment_permissions merge@1 grant grants@1")),
  ({"state":"in_force","reopened_by":None,"reason":None},R(E.format(h=h)+": 'merge@1 = 17  in force', no reopened entry; replay view after line 7: status 'merge', reason 'ready', revisions 1")))]
q2=[{"probe":3,"answer":"yes","receipt":{"kind":"record","text":"caveat-lang dependents gate.cav checks@1 histories/H1.jsonl: 'merge@1 = 17  in force  based on checks@1' (checks@1 = line 3)"}},
    {"probe":7,"answer":"no","receipt":{"kind":"record","text":"caveat-lang dependents gate.cav checks@2 histories/H1.jsonl: 'Decisions  nothing' (checks@2 = line 7)"}},
    {"probe":2,"answer":"no","receipt":{"kind":"record","text":"caveat-lang dependents gate.cav grants@1 histories/H1.jsonl: 'merge@1 = 17  in force  permitted by grants@1' (permission, not grounds; commitment_grounds merge@1 = [checks@1, reviewer, scanner]); task: withdrawing a grant reopens nothing"}},
    {"probe":4,"answer":"yes","receipt":{"kind":"record","text":"caveat-lang dependents gate.cav reviewer histories/H1.jsonl: 'merge@1 = 17  in force  based on reviewer' (reviewer = line 4)"}}]
q3=[{"statement":1,"answer":"true","receipt":{"kind":"record","text":"explain H1: 'merge@1 = 17  in force'"}},
    {"statement":2,"answer":"true","receipt":{"kind":"record","text":"explain H1: 'based on checks@1, reviewer, scanner'; observed at #3, #4, #5"}},
    {"statement":3,"answer":"false","receipt":{"kind":"record","text":"H1 line 4 has relayed=1; explain H1: 'reviewer supports ready  caveats: secondhand'; journal merge@1 caveats [\"deps_unreviewed\", \"secondhand\"]"}},
    {"statement":4,"answer":"true","receipt":{"kind":"record","text":"explain H1: 'permitted by grants@1 for 17'; commitment_grounds merge@1 evidence [checks@1, reviewer, scanner] (no grants@1)"}},
    {"statement":5,"answer":"false","receipt":{"kind":"record","text":"dependents gate.cav checks@2 H1: 'Decisions  nothing'; merge@1 committed at #6 because checks@1, reviewer, scanner"}},
    {"statement":6,"answer":"true","receipt":{"kind":"record","text":"replay H1 view after line 7: {'head': 17, 'reason': 'ready', 'revisions': 1, 'status': 'merge'}; explain: 'merge@1 = 17  in force'"}}]
out["histories"][h]={"q1":q1,"q2":q2,"q3":q3}

# ---------------- H2
h="H2"
value_r={n:R(E.format(h=h)+f": 'merge@{n} = 31'") for n in (1,2,3)}
none_cav=lambda n,seq: ([],R(J.format(h=h)+f": merge@{n} committed seq {seq} caveats [\"deps_unreviewed\"] only (the retained one); review #4 relayed=0; no qualify/withdraw on its grounds before seq {seq}"))
ret=lambda n: (["deps_unreviewed"],("code",f"gate.cav:158 'retaining deps_unreviewed;'; explain H2 merge@{n} 'retaining: deps_unreviewed'"))
gr=lambda n: (2,R(E.format(h=h)+f": merge@{n} 'permitted by grants@1 for 31'; grants@1 '(#2 approve)'"))
q1=[dec(1,31,([3,4,5],R(E.format(h=h)+": merge@1 'based on checks@1, reviewer, scanner'; observed at #3, #4, #5")),none_cav(1,6),ret(1),gr(1),
        ({"state":"superseded","reopened_by":7,"reason":"scan found issues"},R(E.format(h=h)+": 'merge@1 = 31  superseded' / '#7 scan: reopened because scanner@2'; replay view after line 7: reason 'scan found issues', status 'reopened'"))),
    dec(2,31,([3,4,8],R(E.format(h=h)+": merge@2 'based on checks@1, reviewer, scanner@3'; 'scanner@3: observed at #8 scan'; checks@1 #3; reviewer #4")),none_cav(2,9),ret(2),gr(2),
        ({"state":"superseded","reopened_by":10,"reason":"check withdrawn"},R(E.format(h=h)+": 'merge@2 = 31  superseded' / '#10 recheck: reopened because requeue'; replay view after line 10: reason 'check withdrawn', status 'reopened'"))),
    dec(3,31,([4,8,11],R(E.format(h=h)+": merge@3 'based on checks@2, reviewer, scanner@3'; 'checks@2: observed at #11'; reviewer #4; scanner@3 #8")),none_cav(3,12),ret(3),gr(3),
        ({"state":"in_force","reopened_by":None,"reason":None},R(E.format(h=h)+": 'merge@3 = 31  in force'; replay view after line 13: status 'merge', reason 'ready', revisions 3")))]
q2=[{"probe":11,"answer":"yes","receipt":{"kind":"record","text":"dependents gate.cav checks@2 H2: 'merge@3 = 31  in force  based on checks@2' (checks@2 = line 11)"}},
    {"probe":13,"answer":"no","receipt":{"kind":"record","text":"dependents gate.cav checks@3 H2: 'Decisions  nothing' (checks@3 = line 13)"}},
    {"probe":2,"answer":"no","receipt":{"kind":"record","text":"dependents gate.cav grants@1 H2: 'merge@3 = 31  in force  permitted by grants@1' (permission only); commitment_grounds merge@3 [checks@2, reviewer, scanner@3]"}},
    {"probe":5,"answer":"no","receipt":{"kind":"record","text":"dependents gate.cav scanner H2: 'merge@1 = 31  superseded  based on scanner' / 'merge@3 = 31  in force  based on scanner@3'; line 5 is occurrence 'scanner' (renewals: scanner, scanner@2, scanner@3); merge@3 grounds exclude it (it appears only in lineage: 'could also have been influenced by ... scanner')"}}]
q3=[{"statement":1,"answer":"true","receipt":{"kind":"record","text":"explain H2: '#7 scan: reopened because scanner@2'; view after line 7 reason 'scan found issues'"}},
    {"statement":2,"answer":"false","receipt":{"kind":"record","text":"explain H2: merge@2 '#10 recheck: reopened because requeue'; view after line 10 reason 'check withdrawn'; no db_aged in H2"}},
    {"statement":3,"answer":"true","receipt":{"kind":"record","text":"explain H2: merge@2 'based on checks@1, reviewer, scanner@3' at #3, #4, #8"}},
    {"statement":4,"answer":"false","receipt":{"kind":"record","text":"replay H2 withdrawals: {evidence: checks@1, because: requeue, sequence 10}; explain: merge@1 and merge@2 'based on checks@1 ... checks@1 has since been withdrawn at #10 because requeue'"}},
    {"statement":5,"answer":"true","receipt":{"kind":"record","text":"explain H2: merge@3 'based on checks@2, reviewer, scanner@3' (#11, #4, #8)"}},
    {"statement":6,"answer":"true","receipt":{"kind":"record","text":"explain H2: 'merge@3 = 31  in force'; dependents checks@3 (line 13): 'Decisions  nothing'"}}]
out["histories"][h]={"q1":q1,"q2":q2,"q3":q3}

# ---------------- H3
h="H3"
value_r={n:R(E.format(h=h)+f": 'merge@{n} = 42'") for n in (1,2)}
q1=[dec(1,42,([3,4,5],R(E.format(h=h)+": merge@1 'based on checks@1, reviewer, scanner'; observed at #3, #4, #5")),
        ([],R(J.format(h=h)+": merge@1 committed seq 6 caveats [\"deps_unreviewed\"] only; db_stale first qualifies scanner at line 7 (instrumented carries(scanner, db_stale): 0 at line 6, 1 at line 7)")),
        (["deps_unreviewed"],("code","gate.cav:158; explain H3 '#6 decide: committed ...; retaining: deps_unreviewed'")),
        (2,R(E.format(h=h)+": merge@1 'permitted by grants@1 for 42'; grants@1 '(#2 approve)'")),
        ({"state":"superseded","reopened_by":7,"reason":"scan database stale"},R(E.format(h=h)+": 'merge@1 = 42  superseded' / '#7 db_aged: reopened because scanner (caveats: db_stale)'; replay view after line 7 reason 'scan database stale'"))),
    dec(2,42,([3,4,5],R(E.format(h=h)+": merge@2 'based on checks@1, reviewer, scanner' (#3, #4, #5); commitment_grounds merge@2 evidence [checks@1, reviewer, scanner]")),
        ([],("run","work/gate_instr.cav (gate.cav + 'bind probe.scan_carries_stale = 1 when carries(scanner, db_stale)') replayed on H3: line 8 -> 0, line 9 -> 0; replay snapshot taken_back [{evidence: scanner, caveat: db_stale, sequence: 8, event: db_refreshed}]; journal merge@2 committed seq 9 caveats [\"deps_unreviewed\"]; commitment_grounds merge@2 caveats [deps_unreviewed]. Note: explain lists merge@2 'retaining: deps_unreviewed, db_stale' and commitment_bases has kept: [db_stale] - that is lineage kept from the earlier merge record, not a caveat carried by checks@1/reviewer/scanner at line 9")),
        (["deps_unreviewed"],("code","gate.cav:158 'retaining deps_unreviewed;'; journal merge@2 caveats [\"deps_unreviewed\"]. db_stale in explain's 'retaining' line is kept lineage, not retained by choice")),
        (2,R(E.format(h=h)+": merge@2 'permitted by grants@1 for 42'; commitment_permissions merge@2 grant grants@1 (granted 42)")),
        ({"state":"in_force","reopened_by":None,"reason":None},R(E.format(h=h)+": 'merge@2 = 42  in force'; line 13 'decide  refused (policy): a merge decision is in force'; view status 'merge', reason 'ready'")))]
q2=[{"probe":5,"answer":"yes","receipt":{"kind":"record","text":"dependents gate.cav scanner H3: 'merge@2 = 42  in force  based on scanner' (scanner = line 5, only scan)"}},
    {"probe":10,"answer":"no","receipt":{"kind":"record","text":"dependents gate.cav checks@2 H3: 'Decisions  nothing' (checks@2 = line 10)"}},
    {"probe":2,"answer":"no","receipt":{"kind":"record","text":"dependents gate.cav grants@1 H3: 'merge@2 = 42  in force  permitted by grants@1' (permission, not grounds)"}},
    {"probe":4,"answer":"yes","receipt":{"kind":"record","text":"dependents gate.cav reviewer H3: 'merge@2 = 42  in force  based on reviewer' (reviewer = line 4)"}}]
q3=[{"statement":1,"answer":"true","receipt":{"kind":"record","text":"journal H3 merge@1 committed seq 6 caveats [\"deps_unreviewed\"]; explain '#6 decide: committed because checks@1, reviewer, scanner; retaining: deps_unreviewed'"}},
    {"statement":2,"answer":"true","receipt":{"kind":"record","text":"explain H3: '#7 db_aged: reopened because scanner (caveats: db_stale)'; view after line 7 reason 'scan database stale'"}},
    {"statement":3,"answer":"false","receipt":{"kind":"run","text":"work/gate_instr.cav replayed on H3: carries(scanner, db_stale) = 0 after line 8 and at line 9; snapshot taken_back records db_stale taken back from scanner at sequence 8 (db_refreshed); journal merge@2 caveats [\"deps_unreviewed\"]. explain's Evidence line 'scanner ... caveats: db_stale' still lists the (taken-back) relation, which is misleading"}},
    {"statement":4,"answer":"false","receipt":{"kind":"record","text":"explain H3: merge@2 'permitted by grants@1 for 42'; grants@1 is '#2 approve'; grants@2 (#12, head 43) permits nothing"}},
    {"statement":5,"answer":"true","receipt":{"kind":"record","text":"explain H3: merge@2 'based on checks@1, reviewer, scanner' observed at #3, #4, #5"}},
    {"statement":6,"answer":"true","receipt":{"kind":"record","text":"explain H3: '13  decide  refused (policy): a merge decision is in force'; 'merge@2 = 42  in force'; sequence stays 12"}}]
out["histories"][h]={"q1":q1,"q2":q2,"q3":q3}
json.dump(out,open('/root/rs1-runs/R-C2/ANSWERS.json','w'),indent=2)
