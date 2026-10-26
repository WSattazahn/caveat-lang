// Save compatibility for D026: a session that withdraws, then reads or
// recommits a decision, saved on one runtime and restored on the other, in
// both directions. It must restore, accept and refuse the same events after,
// and match a fresh run on the target apart from `withdrawn`, which the
// saved, frozen bases keep as the saving runtime recorded them.
// node experiments/d026-sweep/saves.mjs BEFORE_DIR AFTER_DIR
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
async function rt(dir){const u=pathToFileURL(resolve(dir,'caveat_runtime.js'));u.search='?'+dir;const m=await import(u.href);await m.default({module_or_path:readFileSync(resolve(dir,'caveat_runtime_bg.wasm'))});return m.WebReactiveSession;}
const old = await rt(process.argv[2]);
const fix = await rt(process.argv[3]);
const cases=[
 ['docs/tutorials/agent-undo-list/hello.cav',['ci_passed','ci_was_wrong','ci_passed'],['ci_was_wrong','ci_passed']],
 ['kit/examples/agent-evidence/assessment.cav',[['observe',{confidence:85}],'assess','retract',['observe',{confidence:92}],'assess'],['retract']],
 ['kit/examples/intel-betrayal/intel.cav',[['intel',{informant:'ocelot',gate:'north',says:'clear'}],['go',{gate:'north'}],['betray',{informant:'ocelot'}]],[['intel',{informant:'eva',gate:'north',says:'clear'}],['go',{gate:'north'}]]],
];
const send=(s,e)=>{const [n,p]=Array.isArray(e)?e:[e,{}];return JSON.parse(s.dispatch_outcome(n,JSON.stringify(p))).outcome;};
for(const [path,events,more] of cases){
 const src=readFileSync(path,'utf8');
 for(const [from,to,label] of [[old,fix,'1.4.0 save -> fix'],[fix,old,'fix save -> 1.4.0']]){
  const a=new from(src); const outs=events.map(e=>send(a,e)); const save=a.save();
  let r; try{ r=to.restore(src,save);}catch(e){console.log(path,label,'RESTORE REFUSED',String(e).slice(0,200));continue;}
  const cont=more.map(e=>send(r,e)); const ref=new to(src); events.forEach(e=>send(ref,e)); more.forEach(e=>send(ref,e));
  const W='withdrawn';const strip=v=>Array.isArray(v)?v.filter(i=>!(i?.relation==='retains'&&i?.to===W)&&i?.caveat!==W).map(strip):v&&typeof v==='object'?Object.fromEntries(Object.entries(v).map(([k,x])=>[k,(k==='caveats'||k==='retained')&&Array.isArray(x)?x.filter(c=>c!==W):strip(x)])):v;
  const same=r.snapshot()===ref.snapshot()?'byte for byte':JSON.stringify(strip(JSON.parse(r.snapshot())))===JSON.stringify(strip(JSON.parse(ref.snapshot())))?'apart from withdrawn':'NO';
  console.log(path.split('/').pop(),label,'restored; events',outs.join(','),'; continued',cont.join(','),'; same as a fresh run on the target:',same);
 }
}
