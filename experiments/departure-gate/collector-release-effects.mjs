// Supplemental actual-WASM check: immediate departure effects are saved until
// the next accepted event; draining or restoring must not clear them early.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {loadRuntimeFromDirectory} from '../../kit/lib/node.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url));
const runtime=await loadRuntimeFromDirectory();
const hash=text=>createHash('sha256').update(text).digest('hex');
function describe(save){
 const data=JSON.parse(save), effects=data.effects??[];
 return {save_bytes:Buffer.byteLength(save),save_sha256:hash(save),
   effects_bytes:Buffer.byteLength(JSON.stringify(effects)),effects:effects.length,
   departure_effects:effects.filter(e=>e.kind==='depart').length,
   retired_dynamic_records:Object.keys(data.retired??{}).filter(n=>/@\d+$/.test(n)).length};
}
const rows=[];
for(const kind of ['self','mutual'])for(const cycles of [60,300,1000,3000]){
 const source=readFileSync(root+`experiments/departure-gate/collector-fixtures/release-${kind}.cav`,'utf8')+'\nevent collector_following;\n';
 const session=runtime.open(source);let restored;
 try{
   for(let i=0;i<cycles;i++){assert.equal(session.dispatchView('cycle').outcome,'accepted');session.drainArchive();}
   assert.equal(session.dispatchView('release').outcome,'accepted');
   const immediate=session.save(), archive=session.drainArchive();
   assert.equal(session.save(),immediate,'Drain must preserve last-event effects');
   const afterRelease=describe(immediate), expected=(cycles-1)*(kind==='mutual'?2:1);
   assert.equal(afterRelease.retired_dynamic_records,0);assert.equal(afterRelease.departure_effects,expected);
   restored=runtime.restore(source,immediate);assert.equal(restored.save(),immediate,'Restore preserves effects');
   assert.equal(session.dispatchView('collector_following').outcome,'accepted');
   assert.equal(restored.dispatchView('collector_following').outcome,'accepted');
   const afterFollowing=describe(session.save());assert.equal(restored.save(),session.save());
   assert.equal(afterFollowing.effects,0);assert.equal(afterFollowing.retired_dynamic_records,0);
   rows.push({kind,cycles,source_sha256:hash(source),after_release:afterRelease,after_following:afterFollowing,
     release_archive:{items:archive.length,records:archive.filter(a=>a.kind!=='provenance').length,
       provenance_nodes:archive.filter(a=>a.kind==='provenance').length,serialized_bytes:Buffer.byteLength(JSON.stringify(archive))}});
 }finally{session.close();restored?.close();}
}
console.log(JSON.stringify({passed:true,scope:'Actual-WASM last-event effects, drain, exact restore and next accepted no-op; serialized bytes are not process heap.',runtime:runtime.identity,node:process.version,platform:process.platform,rows},null,2));
