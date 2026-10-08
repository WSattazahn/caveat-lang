import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,readdirSync,writeFileSync} from 'node:fs';
const dir=process.argv[2];assert(dir);
const read=file=>JSON.parse(readFileSync(`${dir}/${file}`,'utf8'));
const hash=file=>createHash('sha256').update(readFileSync(`${dir}/${file}`)).digest('hex');
const save=(file,value)=>writeFileSync(`${dir}/${file}`,value,{flag:'wx'});
const median=values=>{const s=[...values].sort((a,b)=>a-b),n=s.length;assert(n);return n%2?s[(n-1)/2]:(s[n/2-1]+s[n/2])/2;};
const range=values=>({values,median:median(values),min:Math.min(...values),max:Math.max(...values)});
const phases=['growth','growth_last_10_percent','steady','unrelated_changes','failed_release','release'];
const contract=read('contract.json'),candidate=read('candidate.json');
const rows=contract.targetMatrix.map(cell=>{
 const key=`target/${cell.name}-${cell.cycles}-t${cell.trial}`;
 const comparison=read(`${key}-comparison.json`);assert(!comparison.error,`Preserved failure ${key}`);
 const observed={};
 for(const variant of ['baseline','candidate']){const receipt=read(`${key}-${variant}.json`);assert.equal(receipt.status,0);for(const stream of ['stdout','stderr'])assert.equal(hash(receipt[stream].file),receipt[stream].sha256);observed[variant]=receipt.data;}
 return {...cell,comparison,observed};
});
assert.equal(rows.length*2,40);
const primary=rows.filter(r=>r.primary);assert.equal(primary.length,8);
const targets=Object.fromEntries(['release','failed_release'].map(phase=>{
 const pairs=primary.map(r=>{const B=r.observed.baseline[phase],C=r.observed.candidate[phase];return {trial:r.trial,BmedianNs:B.median_ns,CmedianNs:C.median_ns,ratio:C.median_ns/B.median_ns,Bpeak:B.max_dispatch_additional_heap_bytes,Cpeak:C.max_dispatch_additional_heap_bytes,peakPassed:C.max_dispatch_additional_heap_bytes<=B.max_dispatch_additional_heap_bytes};});
 const BmedianNs=median(pairs.map(p=>p.BmedianNs)),CmedianNs=median(pairs.map(p=>p.CmedianNs)),ratioOfMedians=CmedianNs/BmedianNs,pairedMedianRatio=median(pairs.map(p=>p.ratio));
 return [phase,{pairs,BmedianNs,CmedianNs,ratioOfMedians,pairedMedianRatio,latencyPassed:ratioOfMedians<=0.8&&pairedMedianRatio<=0.8,peakPassed:pairs.every(p=>p.peakPassed)}];
}));
const groups=[],increases=[];
for(const row of rows){
 if(groups.some(g=>g.name===row.name&&g.cycles===row.cycles))continue;
 const selected=rows.filter(r=>r.name===row.name&&r.cycles===row.cycles);
 const fields={retainedBefore:d=>d.before_release.retained_rust_heap_bytes,retainedAfter:d=>d.after_release.retained_rust_heap_bytes,saveBefore:d=>d.before_release.serialized_save_bytes,saveAfter:d=>d.after_release.serialized_save_bytes,retiredBefore:d=>d.before_release.retired_dynamic_records,retiredAfter:d=>d.after_release.retired_dynamic_records,archiveBytes:d=>d.archive.ndjson_bytes,archiveRecords:d=>d.archive.records,archiveNodes:d=>d.archive.provenance_nodes};
 const measurements=Object.fromEntries(Object.entries(fields).map(([name,get])=>[name,Object.fromEntries(['baseline','candidate'].map(v=>[v,range(selected.map(r=>get(r.observed[v])))]))]));
 const phaseMeasurements=Object.fromEntries(phases.map(phase=>[phase,Object.fromEntries(['baseline','candidate'].map(v=>{const values=selected.map(r=>r.observed[v][phase]);return [v,values[0].samples?{samplesPerProcess:values.map(x=>x.samples),medianNs:range(values.map(x=>x.median_ns)),p99Ns:range(values.map(x=>x.p99_ns)),maxNs:range(values.map(x=>x.max_ns)),peakBytes:range(values.map(x=>x.max_dispatch_additional_heap_bytes)),collectorWork:values.map(x=>x.collector_work)}:{samples:0}];}))]));
 for(const r of selected)for(const phase of phases){const B=r.observed.baseline[phase],C=r.observed.candidate[phase];if(B.samples&&C.max_dispatch_additional_heap_bytes>B.max_dispatch_additional_heap_bytes)increases.push({name:r.name,cycles:r.cycles,trial:r.trial,primary:r.primary,phase,B:B.max_dispatch_additional_heap_bytes,C:C.max_dispatch_additional_heap_bytes,delta:C.max_dispatch_additional_heap_bytes-B.max_dispatch_additional_heap_bytes});}
 groups.push({name:row.name,cycles:row.cycles,trials:selected.map(r=>r.trial),measurements,phases:phaseMeasurements});
}
const files=['contract.json','candidate.json','collector_profile.rs','attempts.jsonl',...readdirSync(`${dir}/target`).map(file=>`target/${file}`),...readdirSync(`${dir}/fixtures`).map(file=>`fixtures/${file}`)];
const result={recordedAt:new Date().toISOString(),baselineRevision:contract.baseline.revision,candidateRevision:candidate.revision,processes:40,exactNativeEquality:rows.every(r=>r.comparison.exactNativeEquality),targets,passed:rows.every(r=>r.comparison.exactNativeEquality)&&Object.values(targets).every(t=>t.latencyPassed&&t.peakPassed),anyPhasePeakIncreases:increases,groups,rows,manifest:files.sort().map(file=>({file,sha256:hash(file)})),limits:contract.limits};
save('target-supplement.json',JSON.stringify(result,null,2)+'\n');
const ms=n=>(n/1e6).toFixed(4),num=n=>n.toLocaleString('en-US');
const lines=['# Withdrawal batch target measurements','',`Frozen candidate ${candidate.revision}; 40 processes, exact native equality ${result.exactNativeEquality}, registered target pass ${result.passed}.`, '', '| Phase | Baseline median ms | Candidate median ms | Ratio of medians | Median pair ratio | Latency gate | Every pair peak gate |','|---|---:|---:|---:|---:|---|---|'];
for(const [phase,t] of Object.entries(targets))lines.push(`| ${phase} | ${ms(t.BmedianNs)} | ${ms(t.CmedianNs)} | ${t.ratioOfMedians.toFixed(6)} | ${t.pairedMedianRatio.toFixed(6)} | ${t.latencyPassed?'PASS':'MISS'} | ${t.peakPassed?'PASS':'MISS'} |`);
lines.push('', '| Phase / pair | Baseline ms | Candidate ms | C/B | Baseline peak bytes | Candidate peak bytes | Peak gate |','|---|---:|---:|---:|---:|---:|---|');
for(const [phase,t] of Object.entries(targets))for(const p of t.pairs)lines.push(`| ${phase} / ${p.trial} | ${ms(p.BmedianNs)} | ${ms(p.CmedianNs)} | ${p.ratio.toFixed(6)} | ${num(p.Bpeak)} | ${num(p.Cpeak)} | ${p.peakPassed?'PASS':'MISS'} |`);
lines.push('', '## Every observed phase peak increase','', increases.length?'The following increases are reported even where they are outside the scoped primary peak gate:':'No phase peak increase occurred in the registered matched processes. This is finite coverage, not a universal memory-regression proof.');
if(increases.length){lines.push('','| Workload / cycles / trial | Phase | Baseline bytes | Candidate bytes | Increase |','|---|---|---:|---:|---:|');for(const x of increases)lines.push(`| ${x.name}/${x.cycles}/${x.trial} | ${x.phase} | ${num(x.B)} | ${num(x.C)} | ${num(x.delta)} |`);}
lines.push('', '## All phase medians and absolute peaks','', 'Eight-process rows use medians of per-process medians; one-process supplementary rows are diagnostic. Complete individual values, p99/max, observed ranges and work are in JSON.', '', '| Workload / cycles | Phase | Trials | Median ms B → C | Peak bytes B → C |','|---|---|---:|---|---|');
for(const g of groups)for(const phase of phases){const B=g.phases[phase].baseline,C=g.phases[phase].candidate;if(!B.samplesPerProcess)continue;lines.push(`| ${g.name}/${g.cycles} | ${phase} | ${g.trials.length} | ${ms(B.medianNs.median)} → ${ms(C.medianNs.median)} | ${num(B.peakBytes.median)} → ${num(C.peakBytes.median)} |`);}
lines.push('', '## Retention, save and external archive','', '| Workload / cycles | Retained before B → C | Retained after B → C | Save before / after C | Archive bytes C | Archive records / nodes C |','|---|---|---|---|---:|---|');
for(const g of groups){const m=g.measurements;lines.push(`| ${g.name}/${g.cycles} | ${num(m.retainedBefore.baseline.median)} → ${num(m.retainedBefore.candidate.median)} | ${num(m.retainedAfter.baseline.median)} → ${num(m.retainedAfter.candidate.median)} | ${num(m.saveBefore.candidate.median)} / ${num(m.saveAfter.candidate.median)} | ${num(m.archiveBytes.candidate.median)} | ${num(m.archiveRecords.candidate.median)} / ${num(m.archiveNodes.candidate.median)} |`);}
lines.push('', ...contract.limits.map(s=>`- ${s}`),'');save('TARGET-SUPPLEMENT.md',lines.join('\n'));
console.log(JSON.stringify({passed:result.passed,exactNativeEquality:result.exactNativeEquality,targets,anyPhasePeakIncreases:increases,summarySha256:hash('target-supplement.json'),reportSha256:hash('TARGET-SUPPLEMENT.md')},null,2));
