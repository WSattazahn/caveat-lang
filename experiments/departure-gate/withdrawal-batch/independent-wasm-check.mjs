// Reuses the independently authored PR176 matrix; see independent-review-provenance.json.
// Only artifact selection, identity checks, output isolation and reporting are adapted.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {loadRuntimeFromDirectory} from '../../../kit/lib/node.mjs';

const directory = path.dirname(fileURLToPath(import.meta.url));
const repository = path.resolve(directory, '../../..');
const hash = value => createHash('sha256').update(value).digest('hex');
const provenancePath = path.join(directory, 'independent-review-provenance.json');
const provenanceBytes = fs.readFileSync(provenancePath);
const provenance = JSON.parse(provenanceBytes);
const scriptPath = fileURLToPath(import.meta.url);
const scriptBytes = fs.readFileSync(scriptPath);
const marker = '\nfunction make(n,r){';
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
const stats={traces:0,steps:0,dispatches:0,rejectedSteps:0,restoredSessions:0,restoreDispatches:0,expectedClosureChecks:0};const rows=[];
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
    scope:'The supplied independent PR176 matrix reused against the selected before/after artifacts; no added case coverage or performance evidence.',
    startedAt, recordedAt:new Date().toISOString(), driverSha256:hash(scriptBytes), provenanceSha256:hash(provenanceBytes),
    substantiveTestBodySha256:provenance.substantiveTestBodySha256, runtimeArtifacts, loaderArtifacts},null,2));
};
function make(n,r){
 const names=Array.from({length:n},(_,i)=>`s${String(i).padStart(3,'0')}`),patterns=[];
 for(const size of new Set([0,1,2,7,8,9,n].filter(x=>x<=n)))for(const start of [0,Math.floor(n/2)])patterns.push(Array.from({length:size},(_,i)=>(i+start)%n));
 const sum=a=>a.length?a.map(i=>`bank${i}`).join(' + '):'0';
 const all=names.map((_,i)=>i),clear=all.map(i=>`set bank${i} = 0;`).concat(Array.from({length:r},(_,i)=>`set hold${i} = 0;`)).join('\n');
 let s=`# Independently authored PR176 boundary test n=${n} r=${r}\nclaim seen; caveat later consequence material; evidence zreason from "shared"; renewable zreason window 1;\n`;
 s+=names.map((x,i)=>`evidence ${x} from "${x}"; renewable ${x} window 1; state bank${i} = 0;`).join('\n')+'\n';
 s+=Array.from({length:r},(_,i)=>`state hold${i} = 0;`).join('\n')+'\nstate divisor = 1;\n';
 s+=`event build; event duplicate; event retire; event pin; event tick dt min 0 max 0.1; event noop; event clearbanks; event release; event fail; event refuse; event assign target min 0 max ${r-1}, pattern min 0 max ${patterns.length-1}; event clear target min 0 max ${r-1};\nproc clear_all() { ${clear} };\n`;
 s+='on build renew zreason; on build reveal zreason supports seen;\n'+names.map((x,i)=>`on build renew ${x}; on build reveal ${x} supports seen; on build set bank${i} = qualified(1, ${x});`).join('\n')+'\n';
 s+=Array.from({length:r},(_,i)=>`on build set hold${i} = ${sum(all)};`).join('\n')+'\n';
 s+='on duplicate withdraw zreason because zreason;\n'+names.map(x=>`on duplicate withdraw ${x} because zreason; on retire renew ${x};`).join('\n')+'\non retire renew zreason; on pin qualify zreason with later after 0.1;\n';
 for(let i=0;i<r;i++){for(let p=0;p<patterns.length;p++)s+=`on assign when target == ${i} and pattern == ${p} set hold${i} = ${sum(patterns[p])};\n`;s+=`on clear when target == ${i} set hold${i} = 0;\n`;}
 s+=all.map(i=>`on clearbanks set bank${i} = 0;`).join('\n')+'\non release call clear_all(); on refuse call clear_all(); on refuse reject "independent refusal"; on fail call clear_all(); on fail set divisor = 0; bind hud.guard = 1 / divisor;\n';
 return {source:s,names,patterns};
}
try{
for(const [n,r] of [[1,1],[1,8],[1,9],[1,33],[2,2],[7,7],[8,8],[9,9],[16,16],[33,9],[64,16]]){
 const {source,names,patterns}=make(n,r);fs.writeFileSync(`${out}/subjects-${n}-holders-${r}.cav`,source);
 for(const seed of [101,997]){
 let x=seed+n*97+r;const rand=()=>{x^=x<<13;x^=x>>>17;x^=x<<5;return x>>>0;};
 const ss=[B.open(source),C.open(source),B.open(source),C.open(source)],aa=[[],[],[],[]],trace=[];let steps=0;
 const drain=i=>{const a=ss[i].drainArchive();aa[i].push(...a);return a;};
 const flush=()=>{assert.deepEqual(drain(2),drain(3));assert.deepEqual(aa[0],aa[2]);assert.deepEqual(aa[1],aa[3]);};
 function step(event,payload={},wanted='accepted'){
  trace.push({event,payload,wanted});const before=ss.map(s=>[s.save(),s.snapshotText(),s.viewText(),s.undrained]);
  const oo=ss.map(s=>s.dispatch(event,payload));stats.steps++;stats.dispatches+=4;steps++;
  for(let i=0;i<4;i++)assert.equal(oo[i].outcome,wanted,`n=${n} r=${r} seed=${seed} ${event}: ${JSON.stringify(oo[i])}`);
  for(let i=1;i<4;i++){assert.deepEqual(oo[i],oo[0]);assert.equal(ss[i].save(),ss[0].save());assert.equal(ss[i].snapshotText(),ss[0].snapshotText());assert.equal(ss[i].viewText(),ss[0].viewText());}
  if(wanted==='rejected'){stats.rejectedSteps++;for(let i=0;i<4;i++)assert.deepEqual([ss[i].save(),ss[i].snapshotText(),ss[i].viewText(),ss[i].undrained],before[i]);assert.equal(oo[0].origin,event==='fail'?'evaluation':'policy');}
  assert.deepEqual(drain(0),drain(1));if(steps%17===0)flush();
  if(steps%13===0){const saved=ss[0].save(),bb=[B.restore(source,saved),C.restore(source,saved)];stats.restoredSessions+=2;for(const b of bb){assert.equal(b.save(),saved);assert.equal(b.snapshotText(),ss[0].snapshotText());assert.equal(b.undrained,0);}const bo=bb.map(b=>b.dispatch('noop'));stats.restoreDispatches+=2;for(const o of bo)assert.equal(o.outcome,'accepted');assert.deepEqual(bo[0],bo[1]);assert.equal(bb[0].save(),bb[1].save());assert.deepEqual(bb[0].drainArchive(),bb[1].drainArchive());bb.forEach(b=>b.close());}
 }
 for(let cycle=0;cycle<3;cycle++){
  step('build');step('duplicate');step('duplicate');step('duplicate');const pin=cycle%2===0;if(pin)step('pin');
  const snap=ss[0].snapshot(),old=names.map(z=>snap.renewals[z].occurrences.at(-1)),reason=snap.renewals.zreason.occurrences.at(-1);assert.equal(snap.withdrawals.length,n+1);
  step('retire');let holds=Array.from({length:r},()=>new Set(names.map((_,i)=>i)));
  for(let j=0;j<36;j++){const target=j<patterns.length?0:rand()%r,p=j<patterns.length?j:rand()%patterns.length;step('assign',{target,pattern:p});holds[target]=new Set(patterns[p]);if(j%11===0)step('fail',{},'rejected');}
  step('clearbanks');
  function expected(){const keep=new Set(holds.flatMap(a=>[...a])),want=[...keep].map(i=>old[i]);if(keep.size||pin)want.push(reason);const actual=Object.keys(ss[0].snapshot().retired||{}).filter(z=>z.includes('@'));assert.deepEqual(actual.sort(),want.sort(),`closure n=${n} r=${r} cycle=${cycle}`);stats.expectedClosureChecks++;}
  expected();step('refuse',{},'rejected');step('fail',{},'rejected');expected();
  const order=Array.from({length:r},(_,i)=>i);for(let i=r-1;i>0;i--){const j=rand()%(i+1);[order[i],order[j]]=[order[j],order[i]];}
  for(const target of order){step('clear',{target});holds[target].clear();expected();}
  if(pin)step('tick',{dt:0.1});assert.deepEqual(Object.keys(ss[0].snapshot().retired||{}).filter(z=>z.includes('@')),[]);stats.expectedClosureChecks++;step('release');step('noop');flush();
 }
 flush();assert.deepEqual(aa[0],aa[1]);const records=aa[0].filter(z=>z.kind!=='provenance');assert.equal(records.filter(z=>z.withdrawal).length,(n+1)*3);for(const rec of records.filter(z=>z.withdrawal)){assert.equal(rec.withdrawal.evidence,rec.record);assert.equal(rec.withdrawal.event,'duplicate');assert.ok(rec.withdrawal.because.startsWith('zreason@'));}
 ss.forEach(s=>s.close());stats.traces++;const row={n,r,seed,steps,sourceSha256:hash(source),archiveSha256:hash(JSON.stringify(aa[0])),archivedRecords:records.length,passed:true};rows.push(row);fs.writeFileSync(`${out}/trace-${n}-${r}-${seed}.json`,JSON.stringify(trace));console.log(JSON.stringify(row));report('running');
 }
}
report('passed');console.log('SUMMARY '+JSON.stringify(stats));
}catch(e){report('failed',{message:e.message,stack:e.stack});throw e;}
