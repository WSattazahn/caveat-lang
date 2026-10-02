import assert from 'node:assert/strict';
import {readFile,writeFile,mkdir,readdir} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
const repo='WSattazahn/caveat-lang',revision=process.env.FINAL_MAIN;
const token=process.env.GH_TOKEN;
const hash=b=>createHash('sha256').update(b).digest('hex');
async function api(p){const r=await fetch('https://api.github.com/repos/'+repo+'/'+p,{headers:{Authorization:'Bearer '+token,Accept:'application/vnd.github+json'}});assert(r.ok,p+' HTTP'+r.status);return r.json();}
if(process.argv.includes('--recheck-release-body')){
 const before=JSON.parse(await readFile('test-results/postpublication/github-release-body-before.json'));
 const current=await api('releases/tags/v0.1.0-rc.7');
 assert.equal(current.id,before.id);assert(current.prerelease&&!current.draft);
 assert.equal(current.body,before.body,'Release body changed; preserve concurrent evidence and inspect');
 console.log('Release body remains unchanged; append verified completion without discarding owner evidence');process.exit(0);
}
assert.equal((await api('git/ref/heads/main')).object.sha,revision,'Main moved');
const runs=(await api('actions/runs?head_sha='+revision+'&per_page=50')).workflow_runs;
const runtime=runs.find(r=>r.name==='runtime'&&r.event==='push'&&r.head_branch==='main');
const pages=runs.find(r=>r.name==='deploy-pages'&&r.event==='workflow_run');
assert(runtime&&runtime.conclusion==='success','Final main Runtime must pass');
assert(pages&&pages.conclusion==='success','Final Pages including live QA must pass');
for(const [run,file,event] of [[runtime,'runtime.yml','push'],[pages,'pages.yml','workflow_run']]){
 const workflow=await api('actions/workflows/'+file);
 assert.equal(run.workflow_id,workflow.id);assert.equal(run.path,'.github/workflows/'+file);
 assert.equal(run.repository.full_name,repo);assert.equal(run.head_repository.full_name,repo);
 assert.equal(run.head_sha,revision);assert.equal(run.head_branch,'main');
 assert.equal(run.event,event);assert.equal(run.status,'completed');assert.equal(run.conclusion,'success');
}
function bindArtifact(artifact){assert.equal(artifact.workflow_run.id,runtime.id);assert.equal(artifact.workflow_run.head_sha,revision);assert.equal(artifact.workflow_run.head_branch,'main');}
const artifacts=(await api('actions/runs/'+runtime.id+'/artifacts')).artifacts;
const dist=artifacts.find(a=>a.name==='browser-dist');assert(dist&&!dist.expired);bindArtifact(dist);
const zipResponse=await fetch('https://api.github.com/repos/'+repo+'/actions/artifacts/'+dist.id+'/zip',{headers:{Authorization:'Bearer '+token,Accept:'application/vnd.github+json'}});assert(zipResponse.ok);
const zip=Buffer.from(await zipResponse.arrayBuffer());assert.equal('sha256:'+hash(zip),dist.digest);
await mkdir('test-results/postpublication',{recursive:true});
await writeFile('test-results/postpublication/browser-dist.zip',zip);
execFileSync('unzip',['-q','test-results/postpublication/browser-dist.zip','-d','test-results/postpublication/dist']);
const root='test-results/postpublication/dist/';
const files={};
for(const file of ['build-info.json','pkg/caveat_runtime.js','pkg/caveat_runtime_bg.wasm','pkg-reactive/caveat_runtime.js','pkg-reactive/caveat_runtime_bg.wasm','about.html']){
 const expected=await readFile(root+file);
 const response=await fetch('https://wsattazahn.github.io/caveat-lang/'+file+'?verified='+revision);assert(response.ok,file+' live HTTP'+response.status);
 const live=Buffer.from(await response.arrayBuffer());assert.equal(hash(live),hash(expected),file+' deployed bytes mismatch');
 files[file]={sha256:hash(live),bytes:live.length};
 if(file==='build-info.json'){const build=JSON.parse(live);assert.equal(build.revision,revision);assert(build.clean&&build.compiled);assert.equal(build.host,'x86_64-unknown-linux-gnu');}
 if(file==='about.html'){const text=live.toString();assert(text.includes('npm install caveat-lang@0.1.0-rc.7'));assert(!text.includes('npm publication is pending'));assert(text.includes('rc.8'));}
}
const release=await api('releases/tags/v0.1.0-rc.7');assert(release.prerelease&&!release.draft);
const tagref=await api('git/ref/tags/v0.1.0-rc.7');assert.equal(tagref.object.type,'tag');assert.equal(tagref.object.sha,'77ce114a89f4eec7030c3da1f34d78613cc67223');
assert.equal((await api('git/tags/'+tagref.object.sha)).object.sha,'1f3fc7a2208eec964399d6c14232690f411e48be');
const frozen={};
const frozenExpected=[{"name":"browser-dist.zip","id":605023818,"digest":"sha256:0358c2046d01c368e93dabb98038900c0fe4fd1eef2fdda2c2f2a2855d08151e"},{"name":"build-info.json","id":605023699,"digest":"sha256:1102762defa0a3fb19545cc9cdb88b1a49e64c2f5da4247dc7b503dd96f6abb1"},{"name":"caveat-lang-0.1.0-rc.7.tgz","id":605023640,"digest":"sha256:0e441f896c58ba41f2741a82af40a17e74247a0c765169abda10b9ddffad5bbc"},{"name":"cloud-release-review.zip","id":605023959,"digest":"sha256:a5cd5696e4ebf88602e3a921e1f5087519e6fe26c195468bfa8a5fee79a525ab"},{"name":"github-freeze-verification.json","id":605024137,"digest":"sha256:f30220b477f2eec509e6601a41aa659de0312374674767ebdb8db7d3fde5b51e"},{"name":"kit-package-candidate.zip","id":605023763,"digest":"sha256:0ed4ed65f85ac3c5aec37357107ae1ba53172cfea302fa83169aa41b04e3f882"},{"name":"kit-package-report.json","id":605023671,"digest":"sha256:7e39237a02ca1fe3e961a00d52e192a64483753b62c8cd9068672482af15fcf9"},{"name":"kit-security-receipts-1.zip","id":605023786,"digest":"sha256:3f6b65348a51e6fe736f8e6cc98ba273ee282aa97bc8e2bfce12204464451684"},{"name":"kit-security-report.json","id":605023722,"digest":"sha256:77ff1e7bdc42360131678d7adf9399d3a2b58c6d24476362e21428dfb533125d"},{"name":"live-release-manifest.json","id":605024042,"digest":"sha256:b28e64fc18927cc0e805c243573a2b19754cd9f8814b0e99b7156c91a262a939"},{"name":"npm-publication-verification-20261002T145851184.json","id":605839559,"digest":"sha256:bcd81d52a6b90ac917a5527cfb2138453c7c346280431a5bb9e6a459c975ce45"},{"name":"Publish-Rc7.ps1","id":605023994,"digest":"sha256:860d75b726097e481b7e9d0206ffa4f401de9bfc89e63162a106f2e4a1f4e88e"},{"name":"rc7-live-release-evidence.zip","id":605023857,"digest":"sha256:9e476148d14ee3e1991d1ba28d4406d4927fd9e6ad1741af7a320e3cc538cdbf"},{"name":"release-manifest.json","id":605023929,"digest":"sha256:f3821c5b29e845587165c104047c7a589ef797121219434b72da631f38b56844"},{"name":"SHA256SUMS","id":605024097,"digest":"sha256:8c598f9e2f74cf473ae1bb805c753a4b2ed28353c125fe365fae3700152d8331"}];
for(const expected of frozenExpected){
 const asset=release.assets.find(a=>a.name===expected.name);assert(asset);assert.equal(asset.id,expected.id);assert.equal(asset.digest,expected.digest);
 const r=await fetch(asset.browser_download_url);assert(r.ok);const b=Buffer.from(await r.arrayBuffer());assert.equal('sha256:'+hash(b),expected.digest);frozen[asset.name]={id:asset.id,sha256:hash(b),bytes:b.length};
}
const helperAtMain=await api('contents/docs/releases/rc7-handoff/Publish-Rc7.ps1?ref='+revision);
assert.equal(helperAtMain.encoding,'base64');const amendedSourceHelperSha256=hash(Buffer.from(helperAtMain.content,'base64'));

async function retainedArtifact(artifact,destination){
 assert(artifact&&!artifact.expired);
 const response=await fetch('https://api.github.com/repos/'+repo+'/actions/artifacts/'+artifact.id+'/zip',{headers:{Authorization:'Bearer '+token,Accept:'application/vnd.github+json'}});assert(response.ok);
 const zip=Buffer.from(await response.arrayBuffer());assert.equal('sha256:'+hash(zip),artifact.digest);
 await writeFile(destination,zip);return {id:artifact.id,sha256:hash(zip),bytes:zip.length};
}
const securityArtifact=artifacts.find(a=>a.name==='kit-security-receipts-'+runtime.run_attempt);
bindArtifact(securityArtifact);
const securityZip=await retainedArtifact(securityArtifact,'test-results/postpublication/final-main-security.zip');
execFileSync('unzip',['-q','test-results/postpublication/final-main-security.zip','-d','test-results/postpublication/security']);
async function findReports(directory){
 const found=[];for(const entry of await readdir(directory,{withFileTypes:true})){
  const p=directory+'/'+entry.name;if(entry.isDirectory())found.push(...await findReports(p));else if(entry.name==='report.json')found.push(p);
 }return found;
}
const reports=await findReports('test-results/postpublication/security');assert.equal(reports.length,1);
const securityBytes=await readFile(reports[0]);const security=JSON.parse(securityBytes);assert(security.passed);assert.equal(security.artifact.version,'0.1.0-rc.8');assert.equal(security.artifact.buildInfo.revision,revision);
for(const command of security.commands){for(const stream of ['stdout','stderr']){const r=command[stream];assert.equal(hash(await readFile(reports[0].slice(0,reports[0].lastIndexOf('/')+1)+r.file)),r.sha256);}}
const retainedRecordRun=37026797517;
const recordArtifacts=(await api('actions/runs/'+retainedRecordRun+'/artifacts')).artifacts;
const recordZip=await retainedArtifact(recordArtifacts.find(a=>a.id===11234894115),'test-results/postpublication/publication-record-37026797517.zip');
assert.equal(recordZip.sha256,'d643906f7f2e0f822011acd9404f26cfdded8bd1fe87c88902195cc029b6fca3');

const registryProof=JSON.parse(await readFile('test-results/rc7-publication-record/verification.json'));
assert.equal(registryProof.version,'0.1.0-rc.7');assert.equal(registryProof.revision,'1f3fc7a2208eec964399d6c14232690f411e48be');
assert.equal(registryProof.sha256,'0e441f896c58ba41f2741a82af40a17e74247a0c765169abda10b9ddffad5bbc');
const integrity='sha512-CyxtunAuNyo1k7KLQhovEvMwrS+uWP2U4VLTukZTQwNRcKJbvI8E5AfYXGgd8C4tOC1oSixH3YD2J5El7Lim8g==';
assert.equal(registryProof.integrity,integrity);assert.equal(registryProof.registry,'https://registry.npmjs.org');
assert.equal(registryProof.channels.next,'0.1.0-rc.7');assert.equal(registryProof.channels.latest,'0.1.0-rc.5');
assert(registryProof.freshInstall&&registryProof.aliases);assert.equal(registryProof.run,process.env.GITHUB_RUN_ID);
assert.equal(registryProof.lock.version,'0.1.0-rc.7');assert.equal(registryProof.lock.integrity,integrity);assert.equal(new URL(registryProof.lock.resolved).origin,'https://registry.npmjs.org');
assert.equal(registryProof.registryMetadata.name,'caveat-lang');assert.equal(registryProof.registryMetadata.version,'0.1.0-rc.7');assert.equal(registryProof.registryMetadata.dist.integrity,integrity);
assert.equal(new URL(registryProof.registryMetadata.dist.tarball).origin,'https://registry.npmjs.org');
assert(registryProof.doctor.ok);assert.equal(registryProof.doctor.package.version,'0.1.0-rc.7');
const installedBuild=registryProof.doctor.runtime.buildInfo;assert.equal(installedBuild.revision,registryProof.revision);assert(installedBuild.clean&&installedBuild.compiled);assert.equal(installedBuild.host,'x86_64-unknown-linux-gnu');
assert.equal(registryProof.wasmSha256,'32139e7b59306e4da799f6977ec9356f68add27c7f7159a443d7a1f56cae8977');assert(registryProof.demo.preservation.unchanged);assert.equal(registryProof.demo.steps.length,5);
assert(Number.isFinite(Date.parse(registryProof.verifiedAt)));assert(Math.abs(Date.now()-Date.parse(registryProof.verifiedAt))<15*60*1000,'Registry receipt is stale');
for(const command of registryProof.commands){assert.equal(command.exitCode,0);assert.equal(hash(await readFile('test-results/rc7-publication-record/'+command.file)),command.sha256);}
for(const run of [runtime,pages]){const latest=await api('actions/runs/'+run.id);assert.equal(latest.run_attempt,run.run_attempt,'CI rerun during verification');assert.equal(latest.status,'completed');assert.equal(latest.conclusion,'success');assert.equal(latest.head_sha,revision);}
const latestArtifacts=(await api('actions/runs/'+runtime.id+'/artifacts')).artifacts;
for(const original of [dist,securityArtifact]){const latest=latestArtifacts.find(a=>a.name===original.name);assert(latest);assert.equal(latest.id,original.id);assert.equal(latest.digest,original.digest);}
const receipt={verifiedAt:new Date().toISOString(),main:revision,publicationReleaseRevision:'1f3fc7a2208eec964399d6c14232690f411e48be',runtime:{id:runtime.id,attempt:runtime.run_attempt},pages:{id:pages.id,attempt:pages.run_attempt},browserArtifact:{id:dist.id,sha256:hash(zip)},liveFiles:files,frozenAssets:frozen,amendedSourceHelperSha256,registryProof,finalMainDevelopmentSecurity:{artifact:securityZip,reportSha256:hash(securityBytes),report:security},retainedPublicationRecord:recordZip,cloudVerificationRun:process.env.GITHUB_RUN_ID,limitations:['Checks do not prove absence of all unknown defects or vulnerabilities.','Frozen initial WebKit attempt was interrupted; retry passed.','Historical study qualifications and name-audit limits remain.']};
assert.equal((await api('git/ref/heads/main')).object.sha,revision,'Main moved during final verification');
await writeFile('test-results/postpublication/verification.json',JSON.stringify(receipt,null,2)+'\n');
await writeFile('test-results/postpublication/github-release-body-before.json',JSON.stringify({id:release.id,body:release.body},null,2));
const marker='## Cloud publication documentation completion';
assert(!release.body.includes(marker),'Completion section already exists; inspect instead of duplicating');
const notes=release.body.replace('Documentation publication records are the explicit cloud follow-up; rc.8 development is already open.','Publication documentation records and final-main Pages verification are complete below; rc.8 development remains unpublished.');
const addition='\n\n'+marker+'\n\nPublication documentation [PR86](https://github.com/'+repo+'/pull/86) is integrated at main '+revision+'. Runtime '+runtime.id+' attempt '+runtime.run_attempt+' and Pages '+pages.id+' attempt '+pages.run_attempt+' pass; deployed build-info, both runtime adapters/WASM and About bytes match the exact tested artifact. Official npm rc.7 bytes and another fresh exact-version consumer pass. next remains rc.7; latest remains rc.5. No further npm publication or owner dispatch is required. rc.8 remains unpublished development.\n\nThe additional rc7-postpublication-cloud-verification-'+revision+'.json asset records final-main hashes, registry/consumer proof and served hashes for every original frozen asset. postpublication-pages-main-* and postpublication-development-security-main-* are documentation/rc.8-development evidence, distinct from the immutable rc.7 Linux release candidate and its security report. The failed Actions-token PR creation receipt is retained in publication-record-37026797517.zip; it was recovered through the authorized GitHub connection. Original interrupted/failed receipts and study/name-audit qualifications remain preserved. These checks do not guarantee absence of all unknown defects or vulnerabilities.\n';
await writeFile('test-results/postpublication/github-final-notes.md',notes+addition);
console.log(JSON.stringify(receipt,null,2));
