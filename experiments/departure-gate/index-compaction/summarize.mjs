import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
const directory = process.argv[2];
assert(directory, 'Pass measured directory');
const read = file => JSON.parse(readFileSync(path.join(directory,file), 'utf8'));
const hash = file => createHash('sha256').update(readFileSync(path.join(directory,file))).digest('hex');
const contract = read('contract.json'), candidate = read('candidate.json');
const variants = ['f5','before','candidate'];
const phases = ['growth','growth_last_10_percent','steady','unrelated_changes','failed_release','release'];
const median = values => { const s=[...values].sort((a,b)=>a-b), n=s.length; return n%2?s[(n-1)/2]:(s[n/2-1]+s[n/2])/2; };
const distribution = values => ({values, median:median(values), min:Math.min(...values), max:Math.max(...values)});
const rows = contract.matrix.map(cell => {
  const key = `${cell.name}-${cell.cycles}-t${cell.trial}`;
  const comparison = read(`runs/${key}-comparison.json`);
  assert(!comparison.error, `Comparison failed: ${key}`);
  const observed = Object.fromEntries(variants.map(variant=>[variant,read(`runs/${key}-${variant}.json`).data]));
  return {...cell,key,comparison,observed};
});
const groups = [];
for (const row of rows) {
  if (groups.some(g=>g.name===row.name && g.cycles===row.cycles)) continue;
  const trials = rows.filter(r=>r.name===row.name && r.cycles===row.cycles);
  const fields = {
    retained_before:d=>d.before_release.retained_rust_heap_bytes,
    retained_after:d=>d.after_release.retained_rust_heap_bytes,
    save_before:d=>d.before_release.serialized_save_bytes,
    save_after:d=>d.after_release.serialized_save_bytes,
    retired_before:d=>d.before_release.retired_dynamic_records,
    retired_after:d=>d.after_release.retired_dynamic_records,
    archive_bytes:d=>d.archive.ndjson_bytes,
    archive_records:d=>d.archive.records,
    archive_nodes:d=>d.archive.provenance_nodes,
    archive_growth_bytes:d=>d.archive.growth_ndjson_bytes,
    archive_growth_records:d=>d.archive.growth_records,
    archive_growth_nodes:d=>d.archive.growth_nodes,
    max_heap_freed_by_drain:d=>d.archive.max_heap_freed_by_drain,
  };
  const measurements = Object.fromEntries(Object.entries(fields).map(([name,get])=>[name,Object.fromEntries(variants.map(v=>[v,distribution(trials.map(t=>get(t.observed[v])))]))]));
  const phaseMeasurements = Object.fromEntries(phases.map(phase=>[phase,Object.fromEntries(variants.map(v=> {
    const values=trials.map(t=>t.observed[v][phase]);
    if (!values[0].samples) return [v,{samples:0}];
    return [v,{samplesPerTrial:values.map(d=>d.samples), peakBytes:distribution(values.map(d=>d.max_dispatch_additional_heap_bytes)), medianNs:distribution(values.map(d=>d.median_ns)), p99Ns:distribution(values.map(d=>d.p99_ns)), maxNs:distribution(values.map(d=>d.max_ns)), work:values.map(d=>d.collector_work)}];
  }))]));
  groups.push({name:row.name,cycles:row.cycles,trials:trials.map(t=>t.trial), measurements, phases:phaseMeasurements});
}
const files = ['contract.json','candidate.json','collector_profile.rs','attempts.jsonl',...readdirSync(path.join(directory,'runs')).filter(f=>f.endsWith('.json')).map(f=>`runs/${f}`),...readdirSync(path.join(directory,'fixtures')).map(f=>`fixtures/${f}`),...variants.map(v=>`${v}-collector_profile.exe`)];
const manifest = files.sort().map(file=>({file,sha256:hash(file)}));
const targetRows = rows.filter(r=>r.name==='reachable-chain' && r.cycles===3000).map(r=>({trial:r.trial,passed:r.comparison.passed,targets:r.comparison.targets}));
const result = {recordedAt:new Date().toISOString(), candidateRevision:candidate.revision, nativeRuns:rows.length*3, expectedRuns:contract.nativeRuns, exactNativeEquality:rows.every(r=>r.comparison.exactNativeEquality), targets:targetRows, allTargetsPass:targetRows.length===3 && targetRows.every(r=>r.passed), groups, manifest, limits:contract.limits, definitions:{timings:'Median, minimum, and maximum of per-process phase medians across registered trials; per-process p99/max retained separately. Not pooled tail estimates.', allocation:'Requested Rust heap; report direct bytes and per-trial values. Collector-added gates subtract each fresh matched f5 process value from its paired repaired-3d/candidate values.', equality:'Before/candidate save and ordered archive hashes and counts plus required reachable-chain f5 save equality; complete snapshots are a separate WASM result.'}};
const save = (file,value) => writeFileSync(path.join(directory,file),value,{flag:'wx'});
save('supplement-summary.json',JSON.stringify(result,null,2)+'\n');
const fmt = n=>n.toLocaleString('en-US');
const pct = (c,b)=>b?`${((c/b-1)*100).toFixed(2)}%`:'n/a';
const ms = ns=>(ns/1e6).toFixed(4);
const lines = ['# Frozen native comparison', '',`Candidate ${candidate.revision}. ${rows.length*3} native runs; exact native equality ${result.exactNativeEquality}; both primary targets pass in every trial: ${result.allTargetsPass}.`, '', 'The after-release field equals before-release for modes without a release event. Archive drain results concern runtime requested heap; emitted archive bytes remain external output.', '', '| Trial | Target | f5 bytes | 3d bytes | Candidate bytes | Added denominator | Candidate ceiling | Added reduction | Result |','|---|---|---:|---:|---:|---:|---:|---:|---|'];
for (const row of targetRows) for(const [name,t] of Object.entries(row.targets)) lines.push(`| ${row.trial} | ${name} | ${fmt(t.F)} | ${fmt(t.B)} | ${fmt(t.C)} | ${fmt(t.denominator)} | ${fmt(t.ceiling)} | ${(t.reduction*100).toFixed(4)}% | ${t.passed?'PASS':'MISS'} |`);
lines.push('', 'Allocation values below are medians across registered trial processes; full literal values and ranges are in JSON. All sizes have one trial except the registered three-trial stress rows.', '', '| Workload | Cycles | Trials | Retained 3d → candidate | Growth peak 3d → candidate | Unrelated peak 3d → candidate | After release 3d → candidate |','|---|---:|---:|---|---|---|---|');
for(const g of groups){const m=g.measurements,p=g.phases; lines.push(`| ${g.name} | ${g.cycles} | ${g.trials.length} | ${fmt(m.retained_before.before.median)} → ${fmt(m.retained_before.candidate.median)} | ${fmt(p.growth.before.peakBytes.median)} → ${fmt(p.growth.candidate.peakBytes.median)} | ${fmt(p.unrelated_changes.before.peakBytes.median)} → ${fmt(p.unrelated_changes.candidate.peakBytes.median)} | ${fmt(m.retained_after.before.median)} → ${fmt(m.retained_after.candidate.median)} |`);}
lines.push('', 'Stress timings: median of the per-process phase medians, milliseconds (observed min–max). Allocator accounting and work instrumentation are enabled on 3d and candidate; these are native apply timings, excluding snapshot serialization and browser/WASM work.', '', '| Workload/size | Phase | Trials | 3d ms (range) | Candidate ms (range) | Change | Peak 3d → candidate |','|---|---|---:|---|---|---|---|');
for(const g of groups.filter(g=>g.cycles===(g.name==='high-degree'?1000:3000))) for(const phase of phases){const b=g.phases[phase].before,c=g.phases[phase].candidate;if(!b.samplesPerTrial)continue; lines.push(`| ${g.name}/${g.cycles} | ${phase} | ${g.trials.length} | ${ms(b.medianNs.median)} (${ms(b.medianNs.min)}–${ms(b.medianNs.max)}) | ${ms(c.medianNs.median)} (${ms(c.medianNs.min)}–${ms(c.medianNs.max)}) | ${pct(c.medianNs.median,b.medianNs.median)} | ${fmt(b.peakBytes.median)} → ${fmt(c.peakBytes.median)} |`);}
lines.push('', 'Save/archive values at the stress sizes are equal between 3d and candidate; f5 differences are retained in JSON. Archive totals include growth, unrelated departures, and successful release where applicable.', '', '| Workload/size | Retired before → after | Save before → after | Archive bytes | Archive records / nodes |','|---|---|---|---:|---|');
for(const g of groups.filter(g=>g.cycles===(g.name==='high-degree'?1000:3000))){const m=g.measurements; lines.push(`| ${g.name}/${g.cycles} | ${fmt(m.retired_before.candidate.median)} → ${fmt(m.retired_after.candidate.median)} | ${fmt(m.save_before.candidate.median)} → ${fmt(m.save_after.candidate.median)} | ${fmt(m.archive_bytes.candidate.median)} | ${fmt(m.archive_records.candidate.median)} / ${fmt(m.archive_nodes.candidate.median)} |`);}
lines.push('', ...contract.limits.map(s=>`- ${s}`),'');
save('supplement-summary.md',lines.join('\n'));
console.log(JSON.stringify({nativeRuns:result.nativeRuns,exactNativeEquality:result.exactNativeEquality,allTargetsPass:result.allTargetsPass,targets:result.targets,summarySha256:hash('supplement-summary.json'),markdownSha256:hash('supplement-summary.md')},null,2));
