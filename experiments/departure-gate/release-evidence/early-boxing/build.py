"""Fresh ordinary native build at an exact clean revision; no dispatch runs."""
from pathlib import Path
import datetime,hashlib,json,os,shutil,subprocess,sys
repo=Path('C:/Dev/caveat-lang');out=Path(__file__).resolve().parent
kind,expected=sys.argv[1:];assert kind in {'before','after'} and len(expected)==40
dest=out/'builds'/kind;target=out/'private-build-target'/kind
assert not dest.exists() and not target.exists(),'Preserve every build attempt'
dest.mkdir(parents=True)
sha=lambda b:hashlib.sha256(b).hexdigest()
stamp=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
env=dict(os.environ,GIT_CONFIG_COUNT='1',GIT_CONFIG_KEY_0='safe.directory',GIT_CONFIG_VALUE_0=str(repo),PYTHONDONTWRITEBYTECODE='1')
cleared={k:env.pop(k,None) is not None for k in ['RUSTFLAGS','RUSTDOCFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_BUILD_TARGET','CARGO_TARGET_DIR','RUSTUP_TOOLCHAIN','RUSTC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','CARGO_BUILD_RUSTFLAGS','RUST_MIN_STACK']}
def cmd(argv):return subprocess.run(argv,cwd=repo,env=env,capture_output=True,check=True).stdout.decode('utf-8').strip()
def git(*args):return cmd(['git','-c','safe.directory='+str(repo),*args])
head=git('rev-parse','HEAD');assert head==expected
assert not git('status','--porcelain','--untracked-files=no'),'Clean exact source required'
files=set(git('ls-files','--','runtime','rust-toolchain.toml').splitlines())
for name in ['.cargo/config','.cargo/config.toml','runtime/.cargo/config','runtime/.cargo/config.toml']:
    if(repo/name).is_file():files.add(name)
inputs=[dict(file=f,sha256=sha((repo/f).read_bytes())) for f in sorted(files)]
for row in inputs:
    p=dest/'source'/row['file'];p.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(repo/row['file'],p)
argv=['cargo','build','--release','--locked','--manifest-path','runtime/Cargo.toml','--example','collector_profile','--no-default-features','--features','collector-metrics','--target-dir',str(target)]
record=dict(kind=kind,revision=head,runtimeGitTree=git('rev-parse','HEAD:runtime'),argv=argv,cwd=str(repo),targetDir=str(target),startedAt=stamp(),rustc=cmd(['rustc','-vV']),cargo=cmd(['cargo','--version']),cleared_environment=cleared,inputs=inputs,featureTree=cmd(['cargo','tree','--locked','--manifest-path','runtime/Cargo.toml','--no-default-features','--features','collector-metrics','-e','features']))
error=None
try:
    p=subprocess.run(argv,cwd=repo,env=env,capture_output=True,timeout=1800);status=p.returncode;stdout=p.stdout;stderr=p.stderr
except subprocess.TimeoutExpired as e:
    status=None;stdout=e.stdout or b'';stderr=e.stderr or b'';error=str(e)
(dest/'build.stdout').write_bytes(stdout);(dest/'build.stderr').write_bytes(stderr)
after=[dict(file=x['file'],sha256=sha((repo/x['file']).read_bytes())) for x in inputs]
record.update(finishedAt=stamp(),status=status,error=error,inputsStable=inputs==after,headAfter=git('rev-parse','HEAD'),trackedDiffAfter=git('status','--porcelain','--untracked-files=no'),stdoutSha256=sha(stdout),stderrSha256=sha(stderr))
if status==0:
    source=target/'release/examples/collector_profile.exe';shutil.copyfile(source,dest/(kind+'.exe'))
    record.update(executable=str(source),frozen_executable=str(dest/(kind+'.exe')),executableSha256=sha(source.read_bytes()))
(dest/'build.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
assert status==0 and record['inputsStable'] and record['headAfter']==head and not record['trackedDiffAfter'],record
if kind=='before':
    info=json.loads((repo/'dist/build-info.json').read_text(encoding='utf-8'))
    assert info['revision']==head and info['clean'] and info['compiled']
    dst=out/'before-wasm';dst.mkdir()
    for name in ['pkg','pkg-reactive']:shutil.copytree(repo/'dist'/name,dst/name)
    shutil.copyfile(repo/'dist/build-info.json',dst/'build-info.json')
    for name,expected_hash in info['runtimeArtifacts'].items():assert sha((dst/name).read_bytes())==expected_hash,name
    (dst/'COPY-PROVENANCE.json').write_text(json.dumps({'scope':'Copied previously verified ordinary WASM build at exact baseline; native comparison artifacts are fresh builds in separate target directories.','copiedAt':stamp(),'revision':head,'files':[{'file':p.relative_to(dst).as_posix(),'sha256':sha(p.read_bytes())} for p in sorted(dst.rglob('*')) if p.is_file()]},indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:record.get(k) for k in ['kind','revision','runtimeGitTree','status','inputsStable','executableSha256']},indent=2))
