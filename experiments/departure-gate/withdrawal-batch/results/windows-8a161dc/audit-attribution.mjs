import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const dir=process.argv[2];assert(dir);
const read=file=>JSON.parse(readFileSync(`${dir}/${file}`,'utf8'));
const hash=file=>createHash('sha256').update(readFileSync(`${dir}/${file}`)).digest('hex');
const contract=read('contract.json'),diagnostic=read('diagnostic.json');
assert.equal(hash('baseline.exe'),contract.baseline.executable);assert.equal(hash('diagnostic.exe'),diagnostic.sha256);
const attempts=readFileSync(`${dir}/attempts.jsonl`,'utf8').trim().split('\n').map(line=>JSON.parse(line));assert.equal(attempts.length,9);assert(attempts.every(a=>a.stage==='diagnostic'));
const counters=['blocks','matches','misses','probes','shifted_elements','estimated_shifted_bytes','shared_cow_detaches','cow_cloned_elements'];
let rawStreams=0,workComparisons=0,rejectedComparisons=0;
for(const cell of contract.diagnosticMatrix){
 const key=`diagnostic/${cell.name}-${cell.cycles}-t${cell.trial}`;
 const rows={};
 for(const variant of cell.order){const receipt=read(`${key}-${variant}.json`);assert.equal(receipt.status,0);for(const stream of ['stdout','stderr']){assert.equal(hash(receipt[stream].file),receipt[stream].sha256);rawStreams++;}rows[variant]=receipt.data;}
 const d=rows.diagnostic;
 for(const sample of d.failed_release.extraction.small_phase_samples){for(const field of counters)assert.equal(sample.extraction[field],d.release.extraction.small_phase_samples[0].extraction[field],field);rejectedComparisons++;}
 if(rows.baseline)for(const phase of ['growth','growth_last_10_percent','steady','unrelated_changes','failed_release','release']){assert.deepEqual(rows.baseline[phase].collector_work,d[phase].collector_work);workComparisons++;}
}
const result={recordedAt:new Date().toISOString(),attempts:attempts.length,rawStreamsVerified:rawStreams,fullCollectorWorkComparisons:workComparisons,rejectedCounterComparisons:rejectedComparisons,baselineExeSha256:hash('baseline.exe'),diagnosticExeSha256:hash('diagnostic.exe'),reportSha256:hash('ATTRIBUTION.md'),summarySha256:hash('attribution-summary.json'),contractSha256:hash('contract.json'),passed:true};
writeFileSync(`${dir}/attribution-audit.json`,JSON.stringify(result,null,2)+'\n',{flag:'wx'});console.log(JSON.stringify(result,null,2));
