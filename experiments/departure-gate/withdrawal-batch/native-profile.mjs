#!/usr/bin/env node
// Frozen finite dispatch measurement; never builds or modifies runtime source.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { appendFileSync, copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const args=process.argv.slice(2), action=args.shift();
const opts=Object.fromEntries(args.map(arg=>{assert(arg.startsWith('--')&&arg.includes('='));const i=arg.indexOf('=');return [arg.slice(2,i),arg.slice(i+1)];}));
assert(opts.output,'Pass --output=FRESH_DIRECTORY');
const root=path.resolve(opts.root??fileURLToPath(new URL('../../../',import.meta.url))), output=path.resolve(opts.output);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex'), hash=file=>sha(readFileSync(file));
const read=file=>JSON.parse(readFileSync(path.join(output,file),'utf8'));
const save=(file,value)=>writeFileSync(path.join(output,file),JSON.stringify(value,null,2)+'\n',{flag:'wx'});
const run=(exe,argv,extra={})=>spawnSync(exe,argv,{cwd:root,encoding:'utf8',timeout:1800000,maxBuffer:32*1024*1024,...extra});
const command=(exe,argv)=>{const r=run(exe,argv);if(r.error)throw r.error;assert.equal(r.status,0,`${exe}: ${r.stderr}`);return r.stdout.trim();};
const git=(...argv)=>command('git',['-c',`safe.directory=${root.replaceAll('\\','/')}`,...argv]);
const expected={revision:'c1d8fea1164e9d89edc8ff33af5a3bb608bc37d3', executable:'c206c83292dc10e908246a3bace2d6d0dc3a1dc9b4a8b6c0c9126bc1edfe32fd', harness:'1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357', runtimeTree:'907b6ae90b73bf536bc78c9346d4fb2e8f8dcdd3'};
const definitions=[['release-mutual','collector-fixtures/release-mutual.cav','release'],['release-self','collector-fixtures/release-self.cav','release'],['high-degree','index-compaction/high-degree.cav','release'],['reachable-chain','withdrawal-fixtures/reachable-chain.cav','initialize'],['self','withdrawal-fixtures/self.cav','plain'],['mutual','withdrawal-fixtures/mutual.cav','plain']];
const primary=Array.from({length:8},(_,i)=>({name:'release-mutual',cycles:3000,trial:i+1,primary:true}));
const supplemental=[...[1,2,60,300,1000].map(cycles=>['release-mutual',cycles]),['release-self',60],['release-self',3000],['high-degree',60],['high-degree',1000],['reachable-chain',3000],['self',60],['mutual',60]].map(([name,cycles])=>({name,cycles,trial:1,primary:false}));
const targetMatrix=[...primary,...supplemental];
const diagnosticMatrix=[...[60,300,1000].map(cycles=>({name:'release-mutual',cycles,trial:1,order:['diagnostic']})),...Array.from({length:3},(_,i)=>({name:'release-mutual',cycles:3000,trial:i+1,order:i%2?['diagnostic','baseline']:['baseline','diagnostic']}))];
const baseBuild=['cargo','build','--release','--locked','--manifest-path','runtime/Cargo.toml','--example','collector_profile','--no-default-features','--features','collector-metrics'];
const median=values=>{const v=[...values].sort((a,b)=>a-b),n=v.length;assert(n);return n%2?v[(n-1)/2]:(v[n/2-1]+v[n/2])/2;};
if(action==='register'){
  assert(opts['baseline-exe']);assert(!existsSync(output),'Registration requires a new directory, including after partial setup');mkdirSync(output,{recursive:true});
  assert.equal(hash(opts['baseline-exe']),expected.executable);assert.equal(hash(path.join(root,'runtime/examples/collector_profile.rs')),expected.harness);
  assert.equal(git('rev-parse',`${expected.revision}:runtime`),expected.runtimeTree);
  copyFileSync(opts['baseline-exe'],path.join(output,'baseline.exe'));
  copyFileSync(path.join(root,'runtime/examples/collector_profile.rs'),path.join(output,'collector_profile.rs'));
  mkdirSync(path.join(output,'fixtures'));
  const original=JSON.parse(readFileSync(path.join(root,'experiments/departure-gate/collector-results/windows-c1fe15c/baseline.json'),'utf8'));
  const fixtures=definitions.map(([name,relative,mode])=>{const source=`experiments/departure-gate/${relative}`,sha256=hash(path.join(root,source));assert.equal(sha256,name==='high-degree'?'f3fd8fe258e926dab76a7793187215915037f67bc95317ec50556b1344eace4f':original.fixtureSha256[name]);const file=`fixtures/${name}.cav`;copyFileSync(path.join(root,source),path.join(output,file));return {name,source,file,sha256,mode};});
  save('contract.json',{schema:'withdrawal-batch-measurement/1',registeredAt:new Date().toISOString(),baseline:{...expected,file:'baseline.exe',buildArgv:baseBuild},driverSha256:hash(fileURLToPath(import.meta.url)),fixtures,targetMatrix,diagnosticMatrix,targetProcesses:40,diagnosticProcesses:9,settings:{drainEvery:1,probes:100,platform:process.platform,arch:process.arch,rustc:command('rustc',['--version']),order:'Primary odd pair baseline/candidate; even pair candidate/baseline; all executions serial on quiet machine.'},targets:{population:'release-mutual3000, eight fresh matched pairs',success:'release.median_ns (one successful release per process)',rejected:'failed_release.median_ns (median of three rejected attempts per process)',latency:'For success and rejected separately, require median(C)/median(B)<=0.8 AND median of eight paired C/B ratios<=0.8. No pooling reconstructed samples.',peak:'In every primary pair, successful and rejected max_dispatch_additional_heap_bytes C<=B, separately. No tolerance, averaging, or aggregate maximum substitution.',supplemental:'No extra acceptance threshold; report every phase absolute peak and any growth/small-case increase prominently.',failurePolicy:'Preserve every attempt and failure. No reruns, changed thresholds, reset baseline, or favorable subset. New implementation uses a new directory.'},limits:['Requested heap excludes allocator internals, stacks and RSS.','Dispatch peaks include copies, collector work, departure and archive; not isolated collector scratch.','Native apply omits snapshot serialization/browser/WASM work.','Diagnostic per-block clocks/TLS stores perturb timing; diagnostic executable is never used for target acceptance.','Position result implies exact predicate-call count; shifted bytes estimate moved Withdrawal headers, not String payloads or measured traffic.','N16 high-degree cycles repeat a fixed shape, not increasing degree.']});
  console.log('Registered 40 target processes and 9 separate diagnostic/control processes; no measurements.');
}else{
  const contract=read('contract.json');assert.equal(contract.driverSha256,hash(fileURLToPath(import.meta.url)),'Registered driver changed');
  const verifyBase=()=>{assert.equal(hash(path.join(output,'baseline.exe')),expected.executable);assert.equal(hash(path.join(output,'collector_profile.rs')),expected.harness);for(const f of contract.fixtures)assert.equal(hash(path.join(output,f.file)),f.sha256);};verifyBase();
  const identity=kind=>read(`${kind}.json`);
  const verifyIdentity=info=>{assert.equal(hash(path.join(output,info.file)),info.sha256);for(const item of info.inputs)assert.equal(hash(path.join(root,item.file)),item.sha256,`Changed frozen input ${item.file}`);};
  if(action==='freeze'){
    assert(['diagnostic','candidate'].includes(opts.kind)&&opts.exe&&opts.revision);assert.equal(git('rev-parse','HEAD'),opts.revision);assert.equal(git('diff','HEAD','--name-only'),'','Freeze clean tracked source');assert.equal(hash(path.join(root,'runtime/examples/collector_profile.rs')),expected.harness);
    const inputs=git('ls-files','--','runtime','game','rust-toolchain.toml').split('\n').sort().map(file=>({file,sha256:hash(path.join(root,file))}));
    const file=`${opts.kind}.exe`;assert(!existsSync(path.join(output,file)));copyFileSync(opts.exe,path.join(output,file));
    save(`${opts.kind}.json`,{kind:opts.kind,revision:opts.revision,file,sha256:hash(opts.exe),frozenAt:new Date().toISOString(),runtimeGitTree:git('rev-parse',`${opts.revision}:runtime`),inputs,sourceManifestSha256:sha(JSON.stringify(inputs)),buildArgv:opts.kind==='candidate'?baseBuild:['cargo','build','--release','--locked','--manifest-path','runtime/Cargo.toml','--example','withdrawal_extraction_profile','--no-default-features','--features','withdrawal-extraction-profile']});
    console.log(`${opts.kind} frozen; no measurements.`);
  }else if(action==='run-diagnostic'||action==='run-target'){
    const stage=action==='run-diagnostic'?'diagnostic':'target',info=identity(stage==='diagnostic'?'diagnostic':'candidate');
    const matrix=stage==='diagnostic'?contract.diagnosticMatrix:contract.targetMatrix;
    mkdirSync(path.join(output,stage),{recursive:true});
    for(const cell of matrix){
      const key=`${cell.name}-${cell.cycles}-t${cell.trial}`,order=cell.order??(cell.trial%2?['baseline','candidate']:['candidate','baseline']);
      const comparisonFile=`${stage}/${key}-comparison.json`;assert(!existsSync(path.join(output,comparisonFile)),`Refuse rerun ${key}`);const rows={};
      try{
        for(const variant of order){
          const destination=`${stage}/${key}-${variant}.json`,stdoutFile=`${stage}/${key}-${variant}.stdout.txt`,stderrFile=`${stage}/${key}-${variant}.stderr.txt`;for(const file of [destination,stdoutFile,stderrFile])assert(!existsSync(path.join(output,file)),`Partial attempt preserved ${file}`);const previousAttempts=existsSync(path.join(output,'attempts.jsonl'))?readFileSync(path.join(output,'attempts.jsonl'),'utf8').trim().split('\n').filter(Boolean).map(JSON.parse):[];assert(!previousAttempts.some(a=>a.stage===stage&&a.key===key&&a.variant===variant),`Prior attempt preserved ${key}/${variant}`);verifyBase();verifyIdentity(info);
          const fixture=contract.fixtures.find(f=>f.name===cell.name),executable=variant==='baseline'?{...contract.baseline,sha256:expected.executable}:info;
          const argv=[path.join(output,fixture.file),String(cell.cycles),fixture.mode,'1','100'],startedAt=new Date().toISOString();
          appendFileSync(path.join(output,'attempts.jsonl'),JSON.stringify({stage,key,variant,startedAt,executableSha256:executable.sha256,command:[path.join(output,executable.file),...argv]})+'\n');
          const result=run(path.join(output,executable.file),argv,{cwd:output});
          writeFileSync(path.join(output,stdoutFile),result.stdout??'',{flag:'wx'});writeFileSync(path.join(output,stderrFile),result.stderr??'',{flag:'wx'});
          const raw={status:result.status,signal:result.signal,stdout:{file:stdoutFile,sha256:hash(path.join(output,stdoutFile))},stderr:{file:stderrFile,sha256:hash(path.join(output,stderrFile))}};
          if(result.error||result.status!==0){save(destination,{...cell,stage,variant,startedAt,finishedAt:new Date().toISOString(),command:[path.join(output,executable.file),...argv],...raw,error:String(result.error??'')});throw new Error(`Failed ${key}/${variant}; raw output retained`);}
          let data;try{data=JSON.parse(result.stdout);}catch(error){save(destination,{...cell,stage,variant,startedAt,...raw,error:String(error)});throw error;}
          save(destination,{...cell,stage,variant,revision:executable.revision,executableSha256:executable.sha256,fixtureSha256:fixture.sha256,startedAt,finishedAt:new Date().toISOString(),command:[path.join(output,executable.file),...argv],...raw,data});rows[variant]=data;verifyBase();verifyIdentity(info);
          console.log(`${stage} ${key} ${variant}: release=${data.release.median_ns??0}ns rejected=${data.failed_release.median_ns??0}ns`);
        }
        const counterpart=rows.candidate??rows.diagnostic;
        if(rows.baseline&&counterpart){assert.equal(rows.baseline.source_sha256,counterpart.source_sha256);for(const stageName of ['before_release','after_release'])for(const field of ['save_sha256','serialized_save_bytes','retired_dynamic_records','undrained','withdrawals'])assert.equal(rows.baseline[stageName][field],counterpart[stageName][field],`${stageName}.${field}`);for(const field of ['ndjson_sha256','ndjson_bytes','records','provenance_nodes','growth_ndjson_bytes','growth_records','growth_nodes'])assert.equal(rows.baseline.archive[field],counterpart.archive[field],`archive.${field}`);for(const phase of ['growth','growth_last_10_percent','steady','unrelated_changes','failed_release','release']){assert.equal(rows.baseline[phase].samples,counterpart[phase].samples,`${phase}.samples`);if(rows.baseline[phase].samples)assert.equal(rows.baseline[phase].collector_work.collected,counterpart[phase].collector_work.collected,`${phase}.collected`);}}
        const peaks=cell.primary?Object.fromEntries(['release','failed_release'].map(phase=>[phase,{B:rows.baseline[phase].max_dispatch_additional_heap_bytes,C:rows.candidate[phase].max_dispatch_additional_heap_bytes,passed:rows.candidate[phase].max_dispatch_additional_heap_bytes<=rows.baseline[phase].max_dispatch_additional_heap_bytes}])):{};
        const passed=Object.values(peaks).every(p=>p.passed);save(comparisonFile,{...cell,order,exactNativeEquality:!!rows.baseline,peaks,passed});if(!passed)process.exitCode=2;
      }catch(error){save(comparisonFile,{...cell,order,passed:false,error:String(error.stack??error),completedVariants:Object.keys(rows)});throw error;}
    }
  }else if(action==='summarize'){
    const comparisons=contract.targetMatrix.map(cell=>{const file=`target/${cell.name}-${cell.cycles}-t${cell.trial}-comparison.json`;return existsSync(path.join(output,file))?{file,sha256:hash(path.join(output,file)),data:read(file)}:{file,missing:true};});
    const complete=comparisons.every(r=>!r.missing),targets={};
    if(complete&&!comparisons.some(r=>r.data.error))for(const phase of ['release','failed_release']){const values=contract.targetMatrix.filter(c=>c.primary).map(c=>{const key=`target/${c.name}-${c.cycles}-t${c.trial}`;const B=read(`${key}-baseline.json`).data[phase].median_ns,C=read(`${key}-candidate.json`).data[phase].median_ns;return {trial:c.trial,B,C,ratio:C/B};});const Bmedian=median(values.map(v=>v.B)),Cmedian=median(values.map(v=>v.C)),pairedMedianRatio=median(values.map(v=>v.ratio));targets[phase]={values,Bmedian,Cmedian,ratioOfMedians:Cmedian/Bmedian,pairedMedianRatio,passed:Cmedian/Bmedian<=0.8&&pairedMedianRatio<=0.8};}
    const passed=complete&&Object.keys(targets).length===2&&Object.values(targets).every(t=>t.passed)&&comparisons.every(r=>r.data.passed);
    const file=`summary-${new Date().toISOString().replaceAll(':','-')}.json`;save(file,{recordedAt:new Date().toISOString(),complete,passed,targets,comparisons,contractSha256:hash(path.join(output,'contract.json'))});console.log(JSON.stringify({file,complete,passed,targets},null,2));if(!passed)process.exitCode=2;
  }else throw new Error('Use register, freeze, run-diagnostic, run-target or summarize');
}
