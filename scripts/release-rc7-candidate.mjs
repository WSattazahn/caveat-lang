// One-time, explicitly authorized GitHub prerelease freeze. No npm publication.
// Runs only in the reviewed rc7 release-evidence workflow after live checks.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir, readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import path from 'node:path';

const REPO = 'WSattazahn/caveat-lang';
const REVISION = '1f3fc7a2208eec964399d6c14232690f411e48be';
const TAG = 'v0.1.0-rc.7';
const PACKAGE_HASH = '0e441f896c58ba41f2741a82af40a17e74247a0c765169abda10b9ddffad5bbc';
const REPORT_HASH = '7e39237a02ca1fe3e961a00d52e192a64483753b62c8cd9068672482af15fcf9';
const BUILD_HASH = '1102762defa0a3fb19545cc9cdb88b1a49e64c2f5da4247dc7b503dd96f6abb1';
const SECURITY_HASH = '77ff1e7bdc42360131678d7adf9399d3a2b58c6d24476362e21428dfb533125d';
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const token = process.env.GH_TOKEN;
assert(token, 'Platform GitHub token required');
assert.equal(process.env.GITHUB_REPOSITORY, REPO);
assert.equal(process.env.GITHUB_EVENT_NAME, 'pull_request');
const event = JSON.parse(await readFile(process.env.GITHUB_EVENT_PATH));
assert.equal(event.pull_request.head.repo.full_name, REPO);
assert.equal(event.pull_request.head.ref, 'codex/rc7-release-handoff');
assert.equal(event.pull_request.base.repo.full_name, REPO);
assert.equal(event.pull_request.base.ref, 'main');
const head = event.pull_request.head.sha;
const ownRun = Number(process.env.GITHUB_RUN_ID);
assert(Number.isSafeInteger(ownRun));
const directory = path.resolve('test-results/rc7-github-freeze');
await mkdir(directory, { recursive: true });
const recordFailure = async error => {
 const message=String(error?.message??error).replaceAll(token,'[REDACTED]');
 await writeFile(path.join(directory,'failure-receipt.json'),JSON.stringify({revision:REVISION,run:ownRun,head,error:message,failedAt:new Date().toISOString(),policy:'No bypass, force or asset overwrite; inspect partial tag/release before retry'},null,2)+'\n');
 console.error(message);process.exitCode=1;
};
process.on('uncaughtException',recordFailure);process.on('unhandledRejection',recordFailure);

async function request(endpoint, { method = 'GET', body, accept = 'application/vnd.github+json', absent = false } = {}) {
  const response = await fetch(`https://api.github.com/repos/${REPO}/${endpoint}`, {
    method, headers: { Authorization: `Bearer ${token}`, Accept: accept, 'X-GitHub-Api-Version': '2022-11-28', ...(body ? {'Content-Type':'application/json'} : {}) },
    ...(body ? { body: JSON.stringify(body) } : {}),
  });
  if (absent && response.status === 404) return null;
  if (!response.ok) throw new Error(`${method} ${endpoint}: HTTP ${response.status}: ${(await response.text()).slice(0,1000)}`);
  return response.json();
}
async function binary(url) {
  const response = await fetch(url, {headers: {Authorization:`Bearer ${token}`,Accept:'application/octet-stream','X-GitHub-Api-Version':'2022-11-28'}});
  if (!response.ok) throw new Error(`Binary download HTTP ${response.status}`);
  return Buffer.from(await response.arrayBuffer());
}
async function inspectGates() {
  const main = await request('git/ref/heads/main');
  assert.equal(main.object.sha, REVISION, 'Main moved; review concurrent work before release');
  const branch = await request('git/ref/heads/codex/rc7-release-handoff');
  assert.equal(branch.object.sha, head, 'Reviewed handoff head moved');
  const records=[];
  for (const [id,eventName,workflow,pathName] of [[36970516513,'push',360309222,'.github/workflows/runtime.yml'],[36972072386,'workflow_run',360395859,'.github/workflows/pages.yml']]) {
    const run = await request(`actions/runs/${id}`);
    assert.equal(run.repository.full_name,REPO);assert.equal(run.head_sha,REVISION);assert.equal(run.head_branch,'main');
    assert.equal(run.event,eventName);assert.equal(run.workflow_id,workflow);assert.equal(run.path,pathName);assert.equal(run.run_attempt,1);assert.equal(run.status,'completed');assert.equal(run.conclusion,'success');
    records.push({id,event:eventName,revision:run.head_sha,attempt:run.run_attempt,conclusion:run.conclusion});
  }
  const self=await request(`actions/runs/${ownRun}`);assert.equal(self.repository.full_name,REPO);assert.equal(self.event,'pull_request');assert.equal(self.head_sha,head);assert.equal(self.workflow_id,372844904);assert.equal(self.path,'.github/workflows/rc7-release-evidence.yml');assert.equal(self.run_attempt,1);
  const jobs=await request(`actions/runs/${ownRun}/jobs?per_page=100`);
  const live=jobs.jobs.filter(job=>job.name==='live-release');assert.equal(live.length,1);assert.equal(live[0].conclusion,'success');
  return records;
}
const gates=await inspectGates();
async function downloadArtifact(id, name, expectedHash, runId, expectedHead) {
  const artifact=await request(`actions/artifacts/${id}`);
  assert.equal(artifact.name,name);assert.equal(artifact.expired,false);assert.equal(artifact.workflow_run.id,runId);assert.equal(artifact.workflow_run.head_sha,expectedHead);
  const bytes=await binary(`https://api.github.com/repos/${REPO}/actions/artifacts/${id}/zip`);
  const digest=sha256(bytes);assert.equal(digest,expectedHash??artifact.digest.replace(/^sha256:/,''));
  const filename=path.join(directory,`${name}.zip`);await writeFile(filename,bytes);
  const extracted=path.join(directory,name);await mkdir(extracted,{recursive:true});
  // Only authenticated, hash-bound GitHub-generated ZIPs are extracted.
  const result=spawnSync('unzip',['-q',filename,'-d',extracted],{encoding:'utf8'});
  assert.equal(result.status,0,result.stderr);
  return {id,name,sha256:digest,filename,extracted};
}
const candidate=await downloadArtifact(11211387717,'kit-package-candidate','0ed4ed65f85ac3c5aec37357107ae1ba53172cfea302fa83169aa41b04e3f882',36970516513,REVISION);
const security=await downloadArtifact(11212126680,'kit-security-receipts-1','3f6b65348a51e6fe736f8e6cc98ba273ee282aa97bc8e2bfce12204464451684',36970516513,REVISION);
const browser=await downloadArtifact(11212140692,'browser-dist','0358c2046d01c368e93dabb98038900c0fe4fd1eef2fdda2c2f2a2855d08151e',36970516513,REVISION);
const ownArtifacts=await request(`actions/runs/${ownRun}/artifacts?per_page=100`);
const matches=ownArtifacts.artifacts.filter(a=>a.name==='rc7-live-release-evidence'&&!a.expired);assert.equal(matches.length,1);
const live=await downloadArtifact(matches[0].id,'rc7-live-release-evidence',null,ownRun,head);
async function files(root) {const output=[];for(const entry of await readdir(root,{withFileTypes:true})){const file=path.join(root,entry.name);if(entry.isDirectory())output.push(...await files(file));else if(entry.isFile())output.push(file);}return output;}
const candidateFiles=await files(candidate.extracted);
const tgz=candidateFiles.filter(f=>f.endsWith('.tgz'));assert.equal(tgz.length,1);assert.equal(sha256(await readFile(tgz[0])),PACKAGE_HASH);
const reports=candidateFiles.filter(f=>path.basename(f)==='report.json');assert.equal(reports.length,1);assert.equal(sha256(await readFile(reports[0])),REPORT_HASH);
const builds=candidateFiles.filter(f=>path.basename(f)==='build-info.json');assert.equal(builds.length,1);assert.equal(sha256(await readFile(builds[0])),BUILD_HASH);
const report=JSON.parse(await readFile(reports[0]));assert.equal(report.name,'caveat-lang');assert.equal(report.version,'0.1.0-rc.7');assert.equal(report.sha256,PACKAGE_HASH);
const build=JSON.parse(await readFile(builds[0]));assert.equal(build.revision,REVISION);assert(build.clean&&build.compiled);assert.equal(build.host,'x86_64-unknown-linux-gnu');
let securityFile;
for(const file of await files(security.extracted))if(path.basename(file)==='report.json'&&sha256(await readFile(file))===SECURITY_HASH){assert(!securityFile);securityFile=file;}
assert(securityFile);const scan=JSON.parse(await readFile(securityFile));assert.equal(scan.passed,true);assert.equal(scan.artifact.tarballSha256,PACKAGE_HASH);assert.equal(scan.artifact.buildRevision,REVISION);assert.equal(scan.artifact.packageReportSha256,REPORT_HASH);assert.equal(scan.artifact.buildInfoSha256,BUILD_HASH);
for(const command of scan.commands)for(const stream of [command.stdout,command.stderr]){assert.equal(path.basename(stream.file),stream.file);assert.equal(sha256(await readFile(path.join(path.dirname(securityFile),stream.file))),stream.sha256);}
const liveFiles=await files(live.extracted);const liveHashes=liveFiles.filter(f=>path.basename(f)==='runtime-hashes.json');assert.equal(liveHashes.length,1);const liveIdentity=JSON.parse(await readFile(liveHashes[0]));assert.equal(liveIdentity.revision,REVISION);assert.equal(liveIdentity.checks.length,4);
for(const check of liveIdentity.checks)assert.equal(sha256(await readFile(path.join(browser.extracted,check.name))),check.sha256);
const handoff=path.resolve('docs/releases/rc7-handoff');
const manifest=JSON.parse(await readFile(path.join(handoff,'release-manifest.json')));assert.equal(manifest.revision,REVISION);assert.equal(manifest.package.sha256,PACKAGE_HASH);
for(const [name,record]of Object.entries(manifest.handoffAssets)){assert.equal(path.basename(name),name);assert.equal(sha256(await readFile(path.join(handoff,name))),record.sha256,`Reviewed handoff changed: ${name}`);}
const proof=JSON.parse(await readFile(path.join(handoff,'cloud-review-summary.json')));assert.equal(proof.revision,REVISION);assert.equal(proof.pureControls,29);assert.equal(proof.seed6.divergences,0);assert.equal(proof.seed7.divergences,0);assert.equal(proof.seed6.sequences,2000);assert.equal(proof.seed7.sequences,2000);
const body=await readFile(path.join(handoff,'github-release-notes.md'),'utf8');assert(body.includes('npm publication pending.'));
const assets=new Map([
 ['caveat-lang-0.1.0-rc.7.tgz',await readFile(tgz[0])],['kit-package-report.json',await readFile(reports[0])],['build-info.json',await readFile(builds[0])],['kit-security-report.json',await readFile(securityFile)],
 ['kit-package-candidate.zip',await readFile(candidate.filename)],['kit-security-receipts-1.zip',await readFile(security.filename)],['browser-dist.zip',await readFile(browser.filename)],['rc7-live-release-evidence.zip',await readFile(live.filename)],
 ['release-manifest.json',await readFile(path.join(handoff,'release-manifest.json'))],['cloud-release-review.zip',await readFile(path.join(handoff,'cloud-release-review.zip'))],['Publish-Rc7.ps1',await readFile(path.join(handoff,'Publish-Rc7.ps1'))],
]);
const liveReceipt={revision:REVISION,run:ownRun,artifact:{id:live.id,name:live.name,sha256:live.sha256},runtime:liveIdentity,tagAndReleaseCreatedOnlyAfterAllGates:true};
assets.set('live-release-manifest.json',Buffer.from(JSON.stringify(liveReceipt,null,2)+'\n'));
const sums=[...assets].map(([name,bytes])=>`${sha256(bytes)}  ${name}`).join('\n')+'\n';assets.set('SHA256SUMS',Buffer.from(sums));
await inspectGates(); // Fresh verification immediately before any mutation.
let ref=await request(`git/ref/tags/${TAG}`,{absent:true});let tag;
if(ref){assert.equal(ref.object.type,'tag','Existing tag must be annotated');tag=await request(`git/tags/${ref.object.sha}`);assert.equal(tag.object.type,'commit');assert.equal(tag.object.sha,REVISION);}
else{tag=await request('git/tags',{method:'POST',body:{tag:TAG,message:'CAVEAT Language 0.1.0-rc.7 — verified Linux candidate; npm publication pending',object:REVISION,type:'commit'}});ref=await request('git/refs',{method:'POST',body:{ref:`refs/tags/${TAG}`,sha:tag.sha}});}
let release=await request(`releases/tags/${TAG}`,{absent:true});
if(release){assert.equal(release.prerelease,true);assert.equal(release.draft,false);assert.equal(release.tag_name,TAG);assert.equal(release.target_commitish,REVISION);assert.equal(release.name,'CAVEAT Language 0.1.0-rc.7');assert.equal(release.body,body,'Existing release notes changed; preserve and review, never overwrite');}
else release=await request('releases',{method:'POST',body:{tag_name:TAG,target_commitish:REVISION,name:'CAVEAT Language 0.1.0-rc.7',body,draft:false,prerelease:true,make_latest:'false'}});
const existing=await request(`releases/${release.id}/assets?per_page=100`);const receipts=[];
for(const [name,bytes]of assets){
 let asset=existing.find(item=>item.name===name);
 if(!asset){const url=release.upload_url.replace(/\{.*$/,'')+`?name=${encodeURIComponent(name)}`;assert.equal(new URL(url).origin,'https://uploads.github.com');const r=await fetch(url,{method:'POST',headers:{Authorization:`Bearer ${token}`,'Content-Type':'application/octet-stream','X-GitHub-Api-Version':'2022-11-28'},body:bytes});if(!r.ok)throw new Error(`Upload ${name}: HTTP ${r.status}: ${(await r.text()).slice(0,1000)}`);asset=await r.json();}
 const served=await binary(`https://api.github.com/repos/${REPO}/releases/assets/${asset.id}`);assert.equal(sha256(served),sha256(bytes),`Served release asset mismatch: ${name}`);
 receipts.push({id:asset.id,name,bytes:bytes.length,sha256:sha256(served),url:asset.browser_download_url});
}
const finalRef=await request(`git/ref/tags/${TAG}`);assert.equal(finalRef.object.sha,tag.sha);const finalRelease=await request(`releases/${release.id}`);assert.equal(finalRelease.prerelease,true);assert.equal(finalRelease.draft,false);assert.equal(finalRelease.tag_name,TAG);
const receipt={revision:REVISION,tag:TAG,annotatedTagObject:tag.sha,releaseId:release.id,releaseUrl:release.html_url,gates,liveRun:ownRun,assets:receipts,verifiedAt:new Date().toISOString(),npmPublished:false};
const receiptBytes=Buffer.from(JSON.stringify(receipt,null,2)+'\n');
await writeFile(path.join(directory,'github-freeze-verification.json'),receiptBytes);
const receiptUrl=release.upload_url.replace(/\{.*$/,'')+'?name=github-freeze-verification.json';
assert.equal(new URL(receiptUrl).origin,'https://uploads.github.com');
const receiptUpload=await fetch(receiptUrl,{method:'POST',headers:{Authorization:`Bearer ${token}`,'Content-Type':'application/json','X-GitHub-Api-Version':'2022-11-28'},body:receiptBytes});
assert(receiptUpload.ok,`Freeze receipt upload HTTP ${receiptUpload.status}`);
const receiptAsset=await receiptUpload.json();assert.equal(sha256(await binary(`https://api.github.com/repos/${REPO}/releases/assets/${receiptAsset.id}`)),sha256(receiptBytes));
console.log(`Verified annotated ${TAG} and ${receipts.length} same-byte prerelease assets at ${REVISION}; npm publication remains pending.`);
