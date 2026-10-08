import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
const dir=process.argv[2];assert(dir);
const read=file=>JSON.parse(readFileSync(`${dir}/${file}`,'utf8'));
const hash=file=>createHash('sha256').update(readFileSync(`${dir}/${file}`)).digest('hex');
const contract=read('contract.json'),candidate=read('candidate.json'),supplement=read('target-supplement.json');
assert.equal(hash('baseline.exe'),contract.baseline.executable);assert.equal(hash('candidate.exe'),candidate.sha256);
const registeredFile=readdirSync(dir).find(file=>file==='summary-2026-10-08T07-00-12.480Z.json');assert(registeredFile);
const registered=read(registeredFile);assert(registered.complete&&registered.passed&&supplement.passed);
for(const phase of ['release','failed_release'])for(const field of ['ratioOfMedians','pairedMedianRatio'])assert.equal(registered.targets[phase][field],supplement.targets[phase][field]);
const attempts=readFileSync(`${dir}/attempts.jsonl`,'utf8').trim().split('\n').map(line=>JSON.parse(line));
assert.equal(attempts.filter(a=>a.stage==='target').length,40);assert.equal(attempts.filter(a=>a.stage==='diagnostic').length,9);assert.equal(attempts.length,49);
let rawStreams=0,phaseComparisons=0;
for(const cell of contract.targetMatrix){
 const key=`target/${cell.name}-${cell.cycles}-t${cell.trial}`,rows={};
 for(const variant of ['baseline','candidate']){const receipt=read(`${key}-${variant}.json`);assert.equal(receipt.status,0);assert.equal(receipt.data.schema,'caveat-collector-native-profile/1');assert.equal(receipt.data.instrumented,true);assert.equal(receipt.data.diagnostic_only,undefined);for(const stream of ['stdout','stderr']){assert.equal(hash(receipt[stream].file),receipt[stream].sha256);rawStreams++;}rows[variant]=receipt.data;}
 for(const stage of ['before_release','after_release'])assert.deepEqual(rows.baseline[stage],rows.candidate[stage]);
 assert.deepEqual(rows.baseline.archive,rows.candidate.archive);
 for(const phase of ['growth','growth_last_10_percent','steady','unrelated_changes','failed_release','release']){const B=rows.baseline[phase],C=rows.candidate[phase];assert.equal(B.samples,C.samples);if(!B.samples)continue;assert.equal(B.max_dispatch_additional_heap_bytes,C.max_dispatch_additional_heap_bytes);assert.deepEqual(B.collector_work,C.collector_work);phaseComparisons++;}
}
for(const input of candidate.inputs)assert.equal(hash(`candidate-source/${input.file}`),input.sha256);
const result={recordedAt:new Date().toISOString(),registeredSummary:registeredFile,totalAttempts:49,targetAttempts:40,diagnosticAttempts:9,targetRawStreamsVerified:rawStreams,allSampledPhasePeaksAndFullCollectorWorkExactlyEqual:phaseComparisons,allBeforeAfterRetainedAndSaveFieldsExactlyEqual:true,allArchiveFieldsExactlyEqual:true,candidateSourceFilesVerified:candidate.inputs.length,baselineExeSha256:hash('baseline.exe'),candidateExeSha256:hash('candidate.exe'),reportSha256:hash('REPORT.md'),supplementSha256:hash('target-supplement.json'),supplementMarkdownSha256:hash('TARGET-SUPPLEMENT.md'),registeredSummarySha256:hash(registeredFile),contractSha256:hash('contract.json'),passed:true};
writeFileSync(`${dir}/target-audit.json`,JSON.stringify(result,null,2)+'\n',{flag:'wx'});console.log(JSON.stringify(result,null,2));
