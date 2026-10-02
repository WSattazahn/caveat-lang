import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {loadRuntimeFromDirectory} from 'caveat-lang/node';
import {attempt} from './host.mjs';
const source=readFileSync('node_modules/caveat-lang/examples/agent-evidence/assessment.cav','utf8');
const runtime=await loadRuntimeFromDirectory();
const session=runtime.open(source); const results=[];
for(const [score,expected] of [[85,true],[101,false],[92,false],[40,false],[85,true]]) {
  const before=session.save(); const result=await attempt(session,score);const snapshot=session.snapshot();
  assert.equal(result.ok,expected); assert.equal(session.state,'open');
  if(score===101){assert.equal(result.operations.length,1);assert.equal(result.operations[0].origin,'input');assert.equal(result.operations[0].code,'bound_exceeded');assert.equal(session.save(),before);assert.equal(snapshot.bindings.assessment.verdict,'approved');}
  if(score===92){assert.equal(result.operations[0].outcome,'accepted');assert.equal(result.operations[1].origin,'policy');assert.equal(result.operations[1].code,'reject');assert.equal(snapshot.bindings.assessment.verdict,'approved');assert.equal(snapshot.decision_series.assessment.current,'assessment@1');}
  if(score===40){assert.equal(result.operations[0].outcome,'accepted');assert.equal(result.operations[1].outcome,'rejected');assert.equal(snapshot.bindings.assessment.verdict,'reopened');}
  results.push({score,result,snapshot});
  console.log(JSON.stringify({score,ok:result.ok,operations:result.operations.map(o=>({event:o.event,outcome:o.outcome,origin:o.origin,code:o.code,message:o.message})),verdict:snapshot.bindings.assessment.verdict,current:snapshot.decision_series.assessment.current}));
}
assert.equal(session.snapshot().decision_series.assessment.current,'assessment@2');
writeFileSync('host-results.json',JSON.stringify({starterSha256:createHash('sha256').update(source).digest('hex'),runtime:runtime.identity,results},null,2)+'\n');session.close();
for(const score of [0,69,69.999,70,70.001,100,-1,101]) {
 const s=runtime.open(source); const r=await attempt(s,score);assert.equal(r.ok,score>=70 && score<=100);s.close();
}
const tracker=readFileSync('tracker.cav','utf8');const t=runtime.open(tracker);
for (const [event,payload] of [['observe',{value:85}],['assess',{}],['correct',{}],['observe',{value:92}],['assess',{}]]) {
 assert.equal(t.dispatch(event,payload).outcome,'accepted');
 const snap=t.snapshot();assert.equal(snap.relations.some(r=>['tool','correction'].includes(r.from)&&['supports','opposes'].includes(r.relation)),false);
}
t.close();
const d=runtime.open(readFileSync('diagnosis-corrected.cav','utf8'));assert.equal(d.dispatch('observe',{value:73}).outcome,'accepted');assert.equal(d.snapshot().bindings.result.value,73);assert.deepEqual(d.snapshot().binding_explanations.result.value.evidence,['tool']);writeFileSync('diagnosis-snapshot.json',JSON.stringify(d.snapshot(),null,2)+'\n');d.close();
console.log('PASS starter sequence, eight boundary scores, neutral tracker evidence, corrected diagnostic value and citation');
