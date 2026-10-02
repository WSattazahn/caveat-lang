import assert from 'node:assert/strict';
import fs from 'node:fs';
import {loadRuntimeFromDirectory} from 'caveat-lang/node';
import {attempt} from './host.mjs';
const source=fs.readFileSync(new URL('./node_modules/caveat-lang/examples/agent-evidence/assessment.cav',import.meta.url),'utf8');
const runtime=await loadRuntimeFromDirectory();
const session=runtime.open(source);
const results=[];
for(const score of [85,101,92,40,85]){
 const result=await attempt(session,score);
 results.push({score,ok:result.ok,observation:result.observation.outcome,
 assessment:result.assessment?.outcome,refusal:result.assessment?.message||result.observation.message,
 snapshot:session.snapshot()});
}
assert.deepEqual(results.map(r=>r.ok),[true,false,false,false,true]);
assert.equal(results[1].snapshot.bindings.assessment.verdict,'approved');
assert.equal(results[2].assessment,'rejected');
assert.equal(results[3].snapshot.bindings.assessment.verdict,'reopened');
for(const score of [0,69,70,100,-1,101]){
 const s=runtime.open(source);
 const r=await attempt(s,score);
 assert.equal(r.ok,score>=70&&score<=100);
 s.close();
}
console.log(JSON.stringify(results,null,2));
session.close();
