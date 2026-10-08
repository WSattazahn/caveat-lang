// Reproducible native measurements; no publication and no Git worktrees.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = fileURLToPath(new URL('../../', import.meta.url));
const argument = (name, fallback) => process.argv.find(value => value.startsWith(`--${name}=`))?.slice(name.length + 3) ?? fallback;
const only = argument('only', 'both');
assert(['both', 'baseline', 'candidate'].includes(only));
const sizes = argument('sizes', '60,300,1000,3000').split(',').map(Number);
const repeats = Number(argument('repeats', '3'));
const probes = Number(argument('probes', '100'));
assert(sizes.every(n => Number.isInteger(n) && n >= 10 && n <= 4000));
assert(Number.isInteger(repeats) && repeats >= 1 && repeats <= 30);
const output = path.resolve(root, argument('output', 'test-results/collector-profile'));
const baselineRevision = 'f5ec8294efe2be24705f234ef75e5f5459aa5e89';
mkdirSync(output, { recursive: true });
const hash = file => createHash('sha256').update(readFileSync(file)).digest('hex');
function run(executable, argv, cwd, log) {
  const result = spawnSync(executable, argv, { cwd, encoding:'utf8', maxBuffer:32*1024*1024, timeout:1800000 });
  if (log) writeFileSync(log, (result.stdout ?? '') + (result.stderr ?? ''));
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${executable} ${argv.join(' ')}\n${result.stderr}\n${result.stdout}`);
  return result.stdout;
}
const git = (...args) => run('git', ['-c', `safe.directory=${root.replaceAll('\\','/')}`, ...args], root);
const native = name => process.platform === 'win32' ? `${name}.exe` : name;
const harness = path.join(root, 'runtime/examples/collector_profile.rs');
const baselineRoot = path.resolve(root, argument('baseline-root', path.join(output, 'baseline-source')));
if (!existsSync(path.join(baselineRoot, 'runtime/Cargo.toml'))) {
  assert(!existsSync(baselineRoot), 'Refuse to mix a partial baseline source directory');
  const archive = path.join(output, 'baseline-source.tar');
  git('archive', '--format=tar', `--output=${archive}`, baselineRevision, 'runtime', 'game', 'rust-toolchain.toml');
  mkdirSync(baselineRoot);
  run('tar', ['-xf', archive, '-C', baselineRoot], root);
}
// Verify extracted baseline source bytes against the pinned Git tree, including
// when reusing its compiled target directory across measurement batches.
const sourceTree = git('ls-tree','-r','--format=%(objectname) %(path)',baselineRevision,'--','runtime','game','rust-toolchain.toml');
for (const line of sourceTree.trim().split('\n')) {
  const split=line.indexOf(' '), expected=line.slice(0,split), name=line.slice(split+1).trimEnd();
  const bytes=readFileSync(path.join(baselineRoot,name));
  const actual=createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex');
  assert.equal(actual,expected,`Frozen baseline source changed: ${name}`);
}
copyFileSync(harness, path.join(baselineRoot, 'runtime/examples/collector_profile.rs'));
const programs = [
  ['self', 'withdrawal-fixtures/self.cav', 'plain'],
  ['mutual', 'withdrawal-fixtures/mutual.cav', 'plain'],
  ['reachable-chain', 'withdrawal-fixtures/reachable-chain.cav', 'initialize'],
  ['release-self', 'collector-fixtures/release-self.cav', 'release'],
  ['release-mutual', 'collector-fixtures/release-mutual.cav', 'release'],
];
const programDir = path.join(output, 'programs'); mkdirSync(programDir, { recursive:true });
for (const [name, file] of programs) {
  const source = path.join(root, 'experiments/departure-gate', file);
  const saved = path.join(programDir, `${name}.cav`);
  if (existsSync(saved)) assert.equal(hash(saved), hash(source), 'Frozen fixture changed; choose a new output directory');
  else copyFileSync(source, saved);
}

const variants = only === 'both' ? ['baseline','candidate'] : [only];
for (const variant of variants) {
  const cwd = variant === 'baseline' ? baselineRoot : root;
  const args = ['build','--release','--locked','--manifest-path','runtime/Cargo.toml','--example','collector_profile','--no-default-features'];
  if (variant === 'candidate') args.push('--features','collector-metrics');
  run('cargo', args, cwd, path.join(output, `${variant}-build.log`));
  const binary = path.join(cwd, 'runtime/target/release/examples', native('collector_profile'));
  const measuredBinary = path.join(output, `${variant}-${native('collector_profile')}`);
  copyFileSync(binary, measuredBinary);
  const report = {schema:'caveat-collector-profile-run/1', measuredAt:new Date().toISOString(),
    variant, baselineRevision, revision:variant==='baseline'?baselineRevision:git('rev-parse','HEAD').trim(),
    clean:variant==='baseline'||git('diff','HEAD','--name-only').trim()==='',
    platform:process.platform, architecture:process.arch, rustc:run('rustc',['--version'],root).trim(),
    harnessSha256:hash(harness), executableSha256:hash(measuredBinary),
    fixtureSha256:Object.fromEntries(programs.map(([name])=>[name,hash(path.join(programDir,`${name}.cav`))])),
    limits:['Counter and counting-allocator overhead is included. Native apply is not a browser/WASM latency claim.',
      'Requested allocator bytes exclude allocator internals, stacks and RSS. Dispatch peak is an upper bound, not isolated collector-only temporary memory.',
      'Release p99 has only one sample per trial. Compare all trial samples and report their count; do not claim a stable population tail.',
      'Host archives are serialized to a counting/hash sink and discarded after drain; ndjson bytes represent external storage.'],runs:[]};
  const destination = path.join(output, `${variant}.json`);
  assert(!existsSync(destination), 'Refuse to overwrite measured results; choose a new output directory');
  for (const [name,,mode] of programs) for (const size of sizes) for (let trial=1;trial<=repeats;trial++) {
    const result = JSON.parse(run(measuredBinary,[path.join(programDir,`${name}.cav`),String(size),mode,'1',String(probes)],root));
    report.runs.push({name,trial,...result});
    writeFileSync(destination, JSON.stringify(report,null,2)+'\n');
    console.error(`${variant} ${name} ${size} trial ${trial}: ${result.after_release.serialized_save_bytes} saved bytes`);
  }
  // Actual queue retention and draining cost: same language run, different drain schedule.
  for (const name of ['self','mutual']) {
    const size = Math.min(1000,Math.max(...sizes));
    const result=JSON.parse(run(measuredBinary,[path.join(programDir,`${name}.cav`),String(size),'plain',String(size),String(probes)],root));
    report.runs.push({name,trial:1,...result});
  }
  writeFileSync(destination, JSON.stringify(report,null,2)+'\n');
  console.error(`Saved ${variant} measurements: ${destination}`);
}

const baselineFile=path.join(output,'baseline.json'), candidateFile=path.join(output,'candidate.json');
if (existsSync(baselineFile)&&existsSync(candidateFile)) {
  const before=JSON.parse(readFileSync(baselineFile,'utf8')),after=JSON.parse(readFileSync(candidateFile,'utf8'));
  assert.equal(before.harnessSha256,after.harnessSha256,'Use the same harness for both builds');
  assert.deepEqual(before.fixtureSha256,after.fixtureSha256);
  const paired=[];
  for (const candidate of after.runs) {
    const baseline=before.runs.find(row=>row.name===candidate.name&&row.cycles===candidate.cycles&&row.trial===candidate.trial&&row.drain_every===candidate.drain_every);
    assert(baseline,'Missing baseline trial'); assert.equal(baseline.source_sha256,candidate.source_sha256);
    if (candidate.name==='reachable-chain') {
      assert.equal(candidate.after_release.save_sha256,baseline.after_release.save_sha256,'Required reachable chain changed');
    } else {
      assert.equal(candidate.after_release.retired_dynamic_records,0,'An isolated group remained after collection/root release');
    }
    assert.equal(candidate.after_release.undrained,0);
    paired.push({name:candidate.name,cycles:candidate.cycles,trial:candidate.trial,drain_every:candidate.drain_every,
      baseline_save:baseline.after_release.serialized_save_bytes,candidate_save:candidate.after_release.serialized_save_bytes,
      baseline_heap:baseline.after_release.retained_rust_heap_bytes,candidate_heap:candidate.after_release.retained_rust_heap_bytes,
      baseline_growth_p99_ns:baseline.growth_last_10_percent.p99_ns,candidate_growth_p99_ns:candidate.growth_last_10_percent.p99_ns,
      candidate_collector_work:candidate.growth.collector_work,candidate_steady_work:candidate.steady.collector_work,candidate_unrelated_changes:candidate.unrelated_changes,
      candidate_release:candidate.release,candidate_archive:candidate.archive});
  }
  writeFileSync(path.join(output,'comparison.json'),JSON.stringify({passed:true,baselineRevision,
    candidateRevision:after.revision,candidateClean:after.clean,scope:'These fixtures and observed costs only; no universal bound or collector-only allocation isolation',paired},null,2)+'\n');
}
