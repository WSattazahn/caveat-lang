// Reuses the independently authored PR177 matrix; see pr177-matrix-provenance.json.
// Only artifact selection, identity checks, output isolation and reporting are adapted.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {loadRuntimeFromDirectory} from '../../../kit/lib/node.mjs';

const directory = path.dirname(fileURLToPath(import.meta.url));
const repository = fileURLToPath(new URL('../../../', import.meta.url));
const hash = value => createHash('sha256').update(value).digest('hex');
const provenancePath = path.join(directory, 'pr177-matrix-provenance.json');
const provenanceBytes = fs.readFileSync(provenancePath);
const provenance = JSON.parse(provenanceBytes);
const scriptPath = fileURLToPath(import.meta.url);
const scriptBytes = fs.readFileSync(scriptPath);
const marker = '\nfunction prepare(n,sentinels,style){';
const bodyOffset = scriptBytes.indexOf(Buffer.from(marker));
assert(bodyOffset >= 0, 'Missing preserved test body');
assert.equal(hash(scriptBytes.subarray(bodyOffset + 1)), provenance.substantiveTestBodySha256,
  'The external generator/assertion body changed; review and attribute any new matrix separately');
assert(Number(process.versions.node.split('.')[0]) >= 22, 'Use Node.js 22 or later');

const required = ['before', 'candidate', 'before-revision', 'candidate-revision', 'output'];
const options = {};
for (const arg of process.argv.slice(2)) {
  const match = /^--([^=]+)=(.+)$/.exec(arg);
  assert(match && required.includes(match[1]), `Unknown argument: ${arg}`);
  assert(!Object.hasOwn(options, match[1]), `Duplicate argument: ${match[1]}`);
  options[match[1]] = match[2];
}
for (const key of required) assert(options[key], `Required: --${key}=VALUE`);
for (const key of ['before-revision', 'candidate-revision']) {
  assert(/^[a-f0-9]{40}$/.test(options[key]), `${key} must be an exact full Git revision`);
}
const out = path.resolve(options.output);
assert(!fs.existsSync(out), 'Refuse to overwrite an existing output directory');
const sourceRelative = path.relative(repository, out);
assert(sourceRelative.startsWith('..' + path.sep) || sourceRelative === '..' || path.isAbsolute(sourceRelative),
  'Select an output directory outside the checkout so generated programs do not enter frozen inputs');

function freezeRuntime(value, revision) {
  const runtimeDirectory = path.resolve(value);
  const candidates = [path.join(runtimeDirectory, 'build-info.json'), path.join(runtimeDirectory, '..', 'build-info.json')];
  const buildInfoPath = candidates.find(file => fs.existsSync(file));
  assert(buildInfoPath, `Missing build-info.json for ${runtimeDirectory}`);
  const infoBytes = fs.readFileSync(buildInfoPath);
  const info = JSON.parse(infoBytes);
  assert.equal(info.revision, revision, 'Runtime build revision must match its explicit argument');
  assert.equal(info.clean, true, 'Use a clean runtime build');
  assert.equal(info.compiled, true, 'Use a compiled runtime build');
  const files = ['caveat_runtime.js', 'caveat_runtime_bg.wasm'].map(name => {
    const file = path.join(runtimeDirectory, name), sha256 = hash(fs.readFileSync(file));
    assert.equal(sha256, info.runtimeArtifacts?.[`pkg-reactive/${name}`], `Runtime bytes disagree with build-info: ${file}`);
    return {file, sha256};
  });
  files.push({file: buildInfoPath, sha256: hash(infoBytes)});
  return {directory: runtimeDirectory, revision, files};
}
const runtimeArtifacts = {
  before: freezeRuntime(options.before, options['before-revision']),
  candidate: freezeRuntime(options.candidate, options['candidate-revision']),
};
const loaderArtifacts = provenance.loaderFiles.map(({file, sha256}) => {
  const absolute = path.join(repository, file);
  assert.equal(hash(fs.readFileSync(absolute)), sha256, `The pinned C1 loader changed: ${file}`);
  return {file: absolute, sha256};
});
const B = await loadRuntimeFromDirectory(runtimeArtifacts.before.directory);
const C = await loadRuntimeFromDirectory(runtimeArtifacts.candidate.directory);
for (const [runtime, artifact] of [[B, runtimeArtifacts.before], [C, runtimeArtifacts.candidate]]) {
  assert.equal(runtime.identity.revision, artifact.revision);
  assert.equal(runtime.identity.reactiveWasmSha256, artifact.files[1].sha256);
}
fs.mkdirSync(path.dirname(out), {recursive: true});
fs.mkdirSync(out); // Exclusive directory creation: a concurrent existing run is never overwritten.
const stats={traces:0,logicalSteps:0,dispatches:0,rejectedSteps:0,archiveBatches:0,oracleSnapshots:0,restoredSessions:0,restoreDispatches:0},rows=[];
const startedAt = new Date().toISOString();
const report = (status, error) => {
  if (status === 'passed') {
    for (const {file, sha256} of [...loaderArtifacts, ...Object.values(runtimeArtifacts).flatMap(value => value.files)]) {
      assert.equal(hash(fs.readFileSync(file)), sha256, `Input changed during the matrix: ${file}`);
    }
    assert.equal(hash(fs.readFileSync(scriptPath)), hash(scriptBytes), 'Driver changed during the matrix');
    assert.equal(hash(fs.readFileSync(provenancePath)), hash(provenanceBytes), 'Provenance changed during the matrix');
  }
  fs.writeFileSync(out+'/result.json',JSON.stringify({status,node:process.version,baseline:B.identity,candidate:C.identity,stats,rows,error,
    scope:'The supplied independent PR177 matrix reused against the selected before/after artifacts; no added case coverage or performance evidence.',
    startedAt, recordedAt:new Date().toISOString(), driverSha256:hash(scriptBytes), provenanceSha256:hash(provenanceBytes),
    substantiveTestBodySha256:provenance.substantiveTestBodySha256, runtimeArtifacts, loaderArtifacts},null,2));
};
function prepare(n,sentinels,style){
 const families=['z','aa','a','q'];const items=Array.from({length:n},(_,j)=>({j,family:families[j%4],number:Math.floor(j/4)+2,reason:j%2?'r1':'r0',withdraw:style==='all'||(style==='mixed'&&j%4!==1)}));
 const src=['# PR177 independently generated extraction / ordering oracle','claim seen; evidence r0 from "original zero"; evidence r1 from "original one"; event begin; event flush; event noop; state divisor = 1; bind hud.guard = 1 / divisor;','on begin reveal r0 supports seen; on begin reveal r1 supports seen;',...families.map(f=>`evidence ${f} from "${f}"; renewable ${f} window 1; on flush renew ${f};`)];
 if(sentinels)for(let k=0;k<3;k++)src.push(`evidence stay${k} from "permanent"; event sentinel${k}; on sentinel${k} reveal stay${k} supports seen; on sentinel${k} withdraw stay${k} because r1;`);
 for(const x of items){src.push(`state h${x.j} = 0; event build${x.j}; on build${x.j} renew ${x.family}; on build${x.j} reveal ${x.family} supports seen; on build${x.j} set h${x.j} = qualified(${x.j+1}, ${x.family});`);if(x.withdraw)src.push(`on build${x.j} withdraw ${x.family} because ${x.reason}; on build${x.j} withdraw ${x.family} because ${x.reason==='r0'?'r1':'r0'};`);}
 // Different release orders expose prefix, suffix, middle and interleaved matches.
 const all=items.map(x=>x.j), groups=[[],all.filter(j=>j%3===0),all.filter(j=>j%3===1),all.filter(j=>j%3===2),all.slice(0,Math.min(3,n)),all.slice(-3),all.slice(2,-2),all];
 for(let g=0;g<groups.length;g++){
  const proc=groups[g].map(j=>`set h${j} = 0;`).join(' ');
  src.push(`event release${g}; event reject${g}; event fail${g}; proc clear${g}() { ${proc||'set divisor = 1;'} }; on release${g} call clear${g}(); on reject${g} call clear${g}(); on reject${g} reject "batch oracle refusal"; on fail${g} call clear${g}(); on fail${g} set divisor = 0;`);
 }
 return {source:src.join('\n'),items,groups};
}
try{
 for(const [n,sentinels,style] of [[1,false,'all'],[2,false,'none'],[8,false,'all'],[16,true,'all'],[48,false,'all'],[48,true,'mixed'],[48,true,'none'],[80,true,'all']]){
 for(const releaseOrder of [[4,5,1,2,3,7],[0,6,7]]){
 const {source,items,groups}=prepare(n,sentinels,style);const label=`n${n}-s${+sentinels}-${style}-${releaseOrder.join('')}`;fs.writeFileSync(out+'/'+label+'.cav',source);
 const sessions=[B.open(source),C.open(source),B.open(source),C.open(source)],archives=[[],[],[],[]];const expectedWithdrawals=[],expectedRecords=[],creation=new Map(),released=new Set(),trace=[];let seq=0;
 const drain=i=>{const a=sessions[i].drainArchive();archives[i].push(...a);return a;};
 const recordOrder=(a,b)=>a.history<b.history?-1:a.history>b.history?1:a.number-b.number;
 function checkRecords(actual,expected){assert.deepEqual(actual.map(x=>x.record),expected.map(x=>x.record),label+' archive history/numeric order');for(let i=0;i<actual.length;i++){assert.equal(actual[i].history,expected[i].history);assert.equal(actual[i].number,expected[i].number);assert.deepEqual((actual[i].withdrawal ?? null),expected[i].withdrawal,`exact archive ${actual[i].record}`);}}
 function flush(){assert.deepEqual(drain(2),drain(3));assert.deepEqual(archives[0],archives[2]);assert.deepEqual(archives[1],archives[3]);checkRecords(archives[0].filter(x=>x.kind!=='provenance'),expectedRecords);}
 function step(event,wanted='accepted',remove=[]){
  trace.push({event,wanted,remove});const before=sessions.map(s=>[s.save(),s.snapshotText(),s.viewText(),s.undrained]);const outcomes=sessions.map(s=>s.dispatch(event));stats.logicalSteps++;stats.dispatches+=4;
  outcomes.forEach(x=>assert.equal(x.outcome,wanted,`${label} ${event} ${JSON.stringify(x)}`));for(let i=1;i<4;i++){assert.deepEqual(outcomes[i],outcomes[0]);assert.equal(sessions[i].save(),sessions[0].save());assert.equal(sessions[i].snapshotText(),sessions[0].snapshotText());assert.equal(sessions[i].viewText(),sessions[0].viewText());}
  if(wanted==='rejected'){stats.rejectedSteps++;sessions.forEach((s,i)=>assert.deepEqual([s.save(),s.snapshotText(),s.viewText(),s.undrained],before[i]));assert.equal(outcomes[0].origin,event.startsWith('reject')?'policy':'evaluation');}
  else{
   seq++;
   if(event.startsWith('build')){const x=items[+event.slice(5)];const record=`${x.family}@${x.number}`;assert.equal(sessions[0].snapshot().renewals[x.family].occurrences.at(-1),record);const w=x.withdraw?{evidence:record,because:x.reason,sequence:seq,event}:null;creation.set(x.j,{record,history:x.family,number:x.number,withdrawal:w});if(w)expectedWithdrawals.push(w);}
   if(event.startsWith('sentinel'))expectedWithdrawals.push({evidence:`stay${event.slice(8)}`,because:'r1',sequence:seq,event});
  }
  let gone=[];
  if(wanted==='accepted'){
   gone=remove.filter(j=>!released.has(j)).map(j=>creation.get(j)).sort(recordOrder);remove.forEach(j=>released.add(j));const names=new Set(gone.map(x=>x.record));for(let i=expectedWithdrawals.length-1;i>=0;i--)if(names.has(expectedWithdrawals[i].evidence))expectedWithdrawals.splice(i,1);expectedRecords.push(...gone);
  }
  assert.deepEqual((sessions[0].snapshot().withdrawals ?? []),expectedWithdrawals,label+' standing survivor order and immutable reasons');stats.oracleSnapshots++;
  const a=drain(0),b=drain(1);assert.deepEqual(a,b);checkRecords(a.filter(x=>x.kind!=='provenance'),gone);if(gone.length)stats.archiveBatches++;
 }
 step('begin');if(sentinels)step('sentinel0');
 for(let j=0;j<n;j++){step('build'+j);if(sentinels&&j===Math.floor(n/3))step('sentinel1');if(sentinels&&j===Math.floor(2*n/3))step('sentinel2');}
 step('flush');
 for(const g of releaseOrder){
  step('reject'+g,'rejected');step('fail'+g,'rejected');
  // A restore branch must perform the actual next release, not merely load or
  // agree on a rejected no-op. Old undrained archives are not part of a save.
  const save=sessions[0].save(),restored=[B.restore(source,save),C.restore(source,save)];stats.restoredSessions+=2;for(const s of restored){assert.equal(s.undrained,0);assert.equal(s.save(),save);}
  step('release'+g,'accepted',groups[g]);
  const ro=restored.map(s=>s.dispatch('release'+g));stats.restoreDispatches+=2;for(const o of ro)assert.equal(o.outcome,'accepted');assert.deepEqual(ro[0],ro[1]);for(const s of restored){assert.equal(s.save(),sessions[0].save());assert.equal(s.snapshotText(),sessions[0].snapshotText());}
  assert.deepEqual(restored[0].drainArchive(),restored[1].drainArchive());restored.forEach(s=>s.close());
  step('release'+g); // repeated release: no additional departing withdrawal
 }
 step('noop');flush();assert.equal(released.size,n);assert.equal(expectedRecords.length,n);sessions.forEach(s=>s.close());stats.traces++;rows.push({label,passed:true,sourceSha256:hash(source),archiveSha256:hash(JSON.stringify(archives[0])),records:expectedRecords.length,steps:trace.length});fs.writeFileSync(out+'/'+label+'.json',JSON.stringify(trace,null,2));console.log(JSON.stringify(rows.at(-1)));report('running');
 }
 }
 report('passed');console.log('SUMMARY '+JSON.stringify(stats));
}catch(e){report('failed',{message:e.message,stack:e.stack});throw e;}
