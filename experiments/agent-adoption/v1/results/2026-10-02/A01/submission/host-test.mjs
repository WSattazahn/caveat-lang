import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {loadRuntimeFromDirectory} from 'caveat-lang/node';
import {attempt} from './host.mjs';
const path='node_modules/caveat-lang/examples/agent-evidence/assessment.cav';
const source=readFileSync(path,'utf8');
const hash=createHash('sha256').update(source).digest('hex');
const runtime=await loadRuntimeFromDirectory();
const session=runtime.open(source);
const results=[];
for(const score of [85,101,92,40,85]) {
 const before=session.save();
 const result=await attempt(session,score);
 if(score===101) {
   assert.equal(session.save(),before);
   assert.equal(result.snapshot.bindings.assessment.verdict,'approved');
   assert.equal(result.operations[0].code,'bound_exceeded');
 }
 if(score===92) assert.equal(result.operations[1].message,'Already assessed.');
 if(score===40) assert.equal(result.snapshot.bindings.assessment.verdict,'reopened');
 assert.equal(session.state,'open');
 results.push({score,...result});
}
assert.deepEqual(results.map(r=>r.ok),[true,false,false,false,true]);
session.close();
for(const score of [0,69,70,100,-1,102,NaN,Infinity]) {
 const s=runtime.open(source);
 const r=await attempt(s,score);
 assert.equal(r.ok,Number.isFinite(score)&&score>=70&&score<=100);
 assert.equal(s.state,'open');
 s.close();
}
assert.equal(createHash('sha256').update(readFileSync(path)).digest('hex'),hash);
writeFileSync('host-results.json',JSON.stringify({starter:path,sha256:hash,results,boundaries:[0,69,70,100,-1,102,'NaN','Infinity']},null,2));
console.log(JSON.stringify({starterSha256:hash,results:results.map(r=>({score:r.score,ok:r.ok,verdict:r.snapshot?.bindings.assessment.verdict,operations:r.operations.map(o=>({event:o.event,outcome:o.outcome,origin:o.origin,code:o.code,message:o.message}))})),boundaries:'passed'},null,2));
