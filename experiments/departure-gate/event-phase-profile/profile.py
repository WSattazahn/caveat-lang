"""Frozen finite event-phase profiling. Never builds or optimizes runtime code."""
from pathlib import Path
import argparse, datetime, hashlib, json, os, platform, shutil, subprocess
from validate_event_profile import validate_data

BASELINE='055e2af0b411d8385941dd8522201e5afc73cb34'
PRIOR='677ad1365398fe37bb24ff8fb6cfbae82d3ef5b5'
sha=lambda data: hashlib.sha256(data).hexdigest()
stamp=lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
read=lambda p: json.loads(p.read_text(encoding='utf-8'))
def save(p, data):
    with p.open('x',encoding='utf-8') as f: json.dump(data,f,indent=2); f.write('\n')
def copy(src,dst):
    assert not dst.exists(),dst
    dst.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(src,dst)
    return {'file':dst.relative_to(out).as_posix(),'sha256':sha(dst.read_bytes())}
def command(argv):
    return subprocess.run(argv,cwd=repo,capture_output=True,check=True).stdout.decode('utf-8').strip()
def git(*args): return command(['git','-c','safe.directory='+str(repo),*args])
def semantic(a,b):
    for f in ['source_sha256','cycles','mode','drain_every']: assert a[f]==b[f],f
    for state in ['before_release','after_release']:
        for f in ['save_sha256','serialized_save_bytes','retired_dynamic_records','withdrawals','undrained']:
            assert a[state][f]==b[state][f],(state,f,a[state][f],b[state][f])
    for f in ['ndjson_sha256','ndjson_bytes','records','provenance_nodes','growth_ndjson_bytes','growth_records','growth_nodes','max_undrained_items','items_at_growth_end']:
        assert a['archive'][f]==b['archive'][f],f
    for phase in ['growth','growth_last_10_percent','steady','unrelated_changes','failed_release','release']:
        assert a[phase]['samples']==b[phase]['samples'],phase
        if a[phase]['samples'] and phase!='failed_release':
            assert a[phase]['collector_work']==b[phase]['collector_work'],phase

def source_check(contract):
    assert git('rev-parse','HEAD')==contract['revision'] and not git('status','--porcelain','--untracked-files=no')
    for f in contract['builds']['plain']['inputs']:assert sha((repo/f['file']).read_bytes())==f['sha256'],f
    return {'at':stamp(),'head':contract['revision'],'inputs_match':True,'tracked_clean':True}

def expected_argv(contract,cell,mode,execution_root):
    fixture=next(f for f in contract['fixtures'] if f['name']==cell['name'])
    kind='diagnostic' if mode=='timing' else mode
    argv=[str(Path(execution_root)/contract['artifacts'][kind]['file']),str(Path(execution_root)/fixture['file']),str(cell['cycles']),fixture['mode'],'1','100']
    if mode in ['timing','origins']:argv.append(mode)
    return argv

p=argparse.ArgumentParser();p.add_argument('action',choices=['freeze','run','validate']);p.add_argument('--root',default='C:/Dev/caveat-lang');p.add_argument('--output',required=True);p.add_argument('--builds');p.add_argument('--accepted-input',required=True)
args=p.parse_args();repo=Path(args.root).resolve();out=Path(args.output).resolve();accepted=Path(args.accepted_input).resolve();script=Path(__file__).resolve()
if args.action=='freeze':
    assert not out.exists(),'Fresh capture directory required'
    assert not git('status','--porcelain','--untracked-files=no'),'Clean tracked source required'
    revision=git('rev-parse','HEAD');assert git('merge-base',PRIOR,revision)==PRIOR
    builds=Path(args.builds).resolve();identities={}
    for kind in ['plain','diagnostic','origins']:
        build=read(builds/kind/'build.json');exe=builds/kind/(kind+'.exe')
        assert build['revision']==revision and build['status']==0 and build['error'] is None and build['inputsStable']
        assert build['runtimeGitTree']==git('rev-parse','HEAD:runtime')
        feature={'plain':'collector-metrics','diagnostic':'event-phase-profile','origins':'event-phase-origins'}[kind]
        example='collector_profile' if kind=='plain' else 'event_phase_profile'
        assert build['argv']==['cargo','build','--release','--locked','--manifest-path','runtime/Cargo.toml','--example',example,'--no-default-features','--features',feature]
        assert build['headAfter']==revision and not build['trackedDiffAfter']
        assert sha(exe.read_bytes())==build['executableSha256']
        for channel in ['stdout','stderr']:assert sha((builds/kind/('build.'+channel)).read_bytes())==build[channel+'Sha256']
        for item in build['inputs']: assert sha((repo/item['file']).read_bytes())==item['sha256'],item
        identities[kind]=build
    assert identities['plain']['inputs']==identities['diagnostic']['inputs']==identities['origins']['inputs']
    old=read(accepted/'contract.json');old_id=read(accepted/'candidate.json')
    assert old_id['revision']=='8a161dcdfdc8a2a5b3ca43b4183552401002cca7'
    assert old_id['runtimeGitTree']==git('rev-parse',BASELINE+':runtime')
    accepted_exe=accepted/old_id['file'];assert sha(accepted_exe.read_bytes())==old_id['sha256']=='9f5a0a5af2b4073849d30fd7070380eb1341ea4ae9206828aa7fd755cb207909'
    old_build=read(accepted/'candidate-build.json');old_manifest=read(accepted/'candidate-source/manifest.json')
    assert old_build['revision']==old_id['revision'] and old_build['runtimeGitTree']==old_id['runtimeGitTree']
    assert old_build['status']==0 and old_build['error'] is None and old_build['executableSha256']==old_id['sha256']
    assert old_manifest['files']==old_id['inputs']==old_build['inputs']
    for channel in ['stdout','stderr']:assert sha((accepted/('candidate-build.'+channel+'.txt')).read_bytes())==old_build[channel+'Sha256']
    assert sha((repo/'runtime/examples/collector_profile.rs').read_bytes())=='1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357'
    fixtures=[]
    for f in old['fixtures']:
        assert sha((repo/f['source']).read_bytes())==f['sha256'];fixtures.append(f)
    cells=[dict(name='release-mutual',cycles=3000,trial=i+1,primary=True) for i in range(4)]
    cells += [dict(name=name,cycles=n,trial=1,primary=False) for name,n in [('release-mutual',60),('release-mutual',300),('release-mutual',1000),('release-self',3000),('high-degree',1000),('reachable-chain',300),('reachable-chain',1000),('reachable-chain',3000)]]
    orders=[['plain','timing','origins'],['timing','origins','plain'],['origins','plain','timing'],['origins','timing','plain'],['timing','plain','origins'],['plain','origins','timing']]
    for i,c in enumerate(cells):c['order']=orders[i%6];c['key']=f"{c['name']}-{c['cycles']}-t{c['trial']}"
    out.mkdir(parents=True)
    for f in fixtures:copy(repo/f['source'],out/f['file'])
    artifacts={kind:copy(builds/kind/(kind+'.exe'),out/(kind+'.exe')) for kind in identities}
    artifacts['accepted']=copy(accepted_exe,out/'accepted.exe')
    accepted_records=[copy(accepted/file,out/'accepted-source-identity'/file) for file in ['candidate.json','candidate-build.json','candidate-source/manifest.json','candidate-build.stdout.txt','candidate-build.stderr.txt']]
    copy(script,out/'frozen-profile.py')
    validator=copy(script.with_name('validate_event_profile.py'),out/'validate_event_profile.py')
    contract={'schema':'caveat-event-phase-protocol/1','registered_at':stamp(),'baseline':BASELINE,'parent_diagnostics':PRIOR,'revision':revision,'runtime_tree':git('rev-parse','HEAD:runtime'),'fixtures':fixtures,'matrix':cells,'artifacts':artifacts,'builds':identities,'driver_sha256':sha(script.read_bytes()),'settings':{'drain_every':1,'probes':100,'timeout_seconds':1800,'os':platform.platform(),'python':platform.python_version(),'rustc':command(['rustc','-vV'])},'methods':{'execution':'Serial fresh processes, balanced orders; no task-controlled builds/tests during measurements. A fresh process using the accepted source artifact precedes each cell for finite semantic comparison, not timing aggregation.','timing':'Four primary process triples. Rejections summarized within process before across-process median. Supplemental single triples descriptive. Report all raw samples. Original plain harness still contains collector counters and requested-heap accounting.','origins':'Separate allocation-origin mode; latency never substituted for timing-only samples. Peak location and surviving allocation origins differ. Account extra allocator metadata separately. Reallocation convention documented by harness.','population':'12 cells,36 measured processes and12 accepted-reference processes, three rejected releases per release case, one accepted release; chain mode has no final release.','validity':'Compare exact save strings via hash, archive ordered serialization digest, sizes/counts and successful collector work. Every attempted command/raw stream preserved. Never silently retry or delete failed attempts.','claims':'Profiling only. No optimization, no guaranteed saving, no merge/publish/release. Phase attribution is not proof of a particular source container; unexplained remainder reported.'}}
    contract['accepted_source_identity']={'native_build_revision':old_id['revision'],'runtime_tree':old_id['runtimeGitTree'],'records':accepted_records}
    contract['validator']=validator
    contract['capture_root']=str(out)
    contract['current_build_records']=[copy(builds/kind/file,out/'build-identity'/kind/file) for kind in identities for file in ['build.json','build.stdout','build.stderr']]
    save(out/'contract.json',contract)
    save(out/'registration.json',{'at':stamp(),'contract_sha256':sha((out/'contract.json').read_bytes()),'driver_sha256':sha(script.read_bytes())})
    print(json.dumps({'registered':len(cells),'processes':len(cells)*4,'revision':revision}))
    raise SystemExit(0)

contract=read(out/'contract.json');registration=read(out/'registration.json')
assert sha((out/'contract.json').read_bytes())==registration['contract_sha256']
assert sha(script.read_bytes())==contract['driver_sha256']
assert sha((out/'frozen-profile.py').read_bytes())==contract['driver_sha256']
assert sha((out/contract['validator']['file']).read_bytes())==contract['validator']['sha256']
assert sha(script.with_name('validate_event_profile.py').read_bytes())==contract['validator']['sha256']
for fixture in contract['fixtures']:assert sha((out/fixture['file']).read_bytes())==fixture['sha256']
source_check(contract)
for artifact in [*contract['artifacts'].values(),*contract['accepted_source_identity']['records'],*contract['current_build_records']]:assert sha((out/artifact['file']).read_bytes())==artifact['sha256']
if args.action=='run':
    assert not (out/'attempts.jsonl').exists(),'No silent restarts'
    (out/'capture').mkdir()
    env=dict(os.environ)
    for k in ['RUST_MIN_STACK','RUSTFLAGS','RUSTDOCFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_TARGET_DIR','RUSTUP_TOOLCHAIN']:env.pop(k,None)
    for cell in contract['matrix']:
        fixture=next(f for f in contract['fixtures'] if f['name']==cell['name']);values={}
        assert sha((out/fixture['file']).read_bytes())==fixture['sha256']
        for mode in ['accepted',*cell['order']]:
            kind=mode if mode in ['accepted','plain','origins'] else 'diagnostic';artifact=contract['artifacts'][kind]
            argv=expected_argv(contract,cell,mode,out)
            source_before=source_check(contract)
            assert sha((out/artifact['file']).read_bytes())==artifact['sha256']
            key=cell['key']+'-'+mode;start=stamp();print('BEGIN',key,start,flush=True)
            with (out/'attempts.jsonl').open('a',encoding='utf-8') as f:f.write(json.dumps({'key':key,'started_at':start,'argv':argv})+'\n')
            try:
                result=subprocess.run(argv,cwd=repo,env=env,capture_output=True,timeout=1800)
                stdout,stderr,status,error=result.stdout,result.stderr,result.returncode,None
            except subprocess.TimeoutExpired as exc:stdout,stderr,status,error=exc.stdout or b'',exc.stderr or b'',None,repr(exc)
            except OSError as exc:stdout,stderr,status,error=b'',b'',None,repr(exc)
            paths={}
            for channel,data in [('stdout',stdout),('stderr',stderr)]:
                name='capture/'+key+'.'+channel; (out/name).write_bytes(data);paths[channel]={'file':name,'sha256':sha(data)}
            row={'key':key,'cell':cell,'mode':mode,'revision':contract['accepted_source_identity']['native_build_revision'] if mode=='accepted' else contract['revision'],'accepted_runtime_baseline':contract['baseline'],'executable_sha256':artifact['sha256'],'argv':argv,'source_before':source_before,'started_at':start,'finished_at':stamp(),'status':status,'error':error,**paths}
            save(out/'capture'/(key+'.json'),row)
            assert status==0 and error is None,(key,status,error)
            save(out/'capture'/(key+'-source-after.json'),source_check(contract))
            assert sha((out/artifact['file']).read_bytes())==artifact['sha256']
            values[mode]=json.loads(stdout)
            if mode in ['timing','origins']:validate_data(values[mode],mode,100)
            if mode!='accepted':semantic(values['accepted'],values[mode])
            print('END',key,status,flush=True)
        save(out/'capture'/(cell['key']+'-comparison.json'),{'semantic_match':True,'modes':list(values),'scope':contract['methods']['validity']})

records=[]
attempts=[json.loads(line) for line in (out/'attempts.jsonl').read_text(encoding='utf-8').splitlines()]
expected_keys=[cell['key']+'-'+mode for cell in contract['matrix'] for mode in ['accepted',*cell['order']]]
assert [row['key'] for row in attempts]==expected_keys
last_finish=None
for cell in contract['matrix']:
    values={}
    for mode in ['accepted',*cell['order']]:
        key=cell['key']+'-'+mode;row=read(out/'capture'/(key+'.json'));assert row['status']==0 and row['error'] is None
        kind='diagnostic' if mode=='timing' else mode
        assert row['key']==key and row['cell']==cell and row['mode']==mode
        assert row['revision']==(contract['accepted_source_identity']['native_build_revision'] if mode=='accepted' else contract['revision'])
        assert row['executable_sha256']==contract['artifacts'][kind]['sha256']
        assert row['argv']==expected_argv(contract,cell,mode,contract['capture_root'])
        attempt=attempts[len(records)];assert attempt['argv']==row['argv'] and attempt['started_at']==row['started_at']
        assert row['started_at']<=row['finished_at'] and (last_finish is None or last_finish<=row['started_at']);last_finish=row['finished_at']
        source_after=read(out/'capture'/(key+'-source-after.json'))
        for source in [row['source_before'],source_after]:assert source['head']==contract['revision'] and source['inputs_match'] and source['tracked_clean']
        assert row['source_before']['at']<=row['started_at']<=row['finished_at']<=source_after['at']
        for channel in ['stdout','stderr']:assert sha((out/row[channel]['file']).read_bytes())==row[channel]['sha256']
        values[mode]=read(out/row['stdout']['file']);records.append(row)
        if mode in ['timing','origins']:validate_data(values[mode],mode,100)
    for mode in cell['order']:semantic(values['accepted'],values[mode])
    assert read(out/'capture'/(cell['key']+'-comparison.json'))['semantic_match']
name='validation-'+stamp().replace(':','-')+'.json'
save(out/name,{'complete':True,'cells':len(contract['matrix']),'processes':len(records),'contract_sha256':registration['contract_sha256'],'at':stamp(),'scope':contract['methods']['validity']})
print(json.dumps({'complete':True,'cells':len(contract['matrix']),'processes':len(records),'validation':name}))
