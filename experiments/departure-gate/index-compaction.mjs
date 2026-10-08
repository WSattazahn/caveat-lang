#!/usr/bin/env node
// Frozen, finite native allocation comparison. This driver never builds code.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { appendFileSync, copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const args = process.argv.slice(2);
const action = args.shift();
const options = Object.fromEntries(args.map(arg => {
  assert(arg.startsWith('--') && arg.includes('='), 'Options use --name=value');
  const i = arg.indexOf('='); return [arg.slice(2, i), arg.slice(i + 1)];
}));
const root = path.resolve(options.root ?? fileURLToPath(new URL('../..', import.meta.url)));
assert(options.output, 'Pass --output=NEW_OUTPUT_DIRECTORY');
const output = path.resolve(options.output);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const hash = file => sha(readFileSync(file));
const read = file => JSON.parse(readFileSync(file, 'utf8'));
const save = (file, value) => writeFileSync(file, JSON.stringify(value, null, 2) + '\n', { flag: 'wx' });
const command = (exe, argv, extra = {}) => {
  const result = spawnSync(exe, argv, { cwd: root, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024, timeout: 1800000, ...extra });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${exe} ${argv.join(' ')}\n${result.stderr}\n${result.stdout}`);
  return result.stdout;
};
const git = (...argv) => command('git', ['-c', `safe.directory=${root.replaceAll('\\', '/')}`, ...argv]).trim();
const expected = {
  harness: '1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357',
  f5: { revision: 'f5ec8294efe2be24705f234ef75e5f5459aa5e89', sha256: '27cd7b3e68a21a0633e2abe8effc7568529fca1ac3555e005df41fcc4453b8c4', metrics: false },
  before: { revision: '3d77aa27bcffa16f39818b2684e9afb3b102ed17', sha256: 'c0bf892c2cb618b377c38a14206bc56244d891c561f0684fceeb8b0aa8b0a9c5', metrics: true },
};
const definitions = [
  ['self', 'withdrawal-fixtures/self.cav', 'plain'],
  ['mutual', 'withdrawal-fixtures/mutual.cav', 'plain'],
  ['reachable-chain', 'withdrawal-fixtures/reachable-chain.cav', 'initialize'],
  ['release-self', 'collector-fixtures/release-self.cav', 'release'],
  ['release-mutual', 'collector-fixtures/release-mutual.cav', 'release'],
  ['high-degree', 'index-compaction/high-degree.cav', 'release'],
];
const matrix = definitions.flatMap(([name]) => {
  const sizes = name === 'high-degree' ? [60, 300, 1000] : [60, 300, 1000, 3000];
  return sizes.flatMap(cycles => {
    const repeated = name === 'high-degree' ? cycles === 1000 : cycles === 3000 && name !== 'release-self';
    return Array.from({ length: repeated ? 3 : 1 }, (_, i) => ({ name, cycles, trial: i + 1 }));
  });
});
const contractFile = path.join(output, 'contract.json');
const candidateFile = path.join(output, 'candidate.json');

if (action === 'prepare') {
  assert(options['f5-exe'] && options['before-exe'], 'prepare requires --f5-exe and --before-exe');
  mkdirSync(output, { recursive: true });
  assert(!existsSync(contractFile), 'Refuse to overwrite a registered contract');
  assert.equal(hash(path.join(root, 'runtime/examples/collector_profile.rs')), expected.harness);
  const original = read(path.join(root, 'experiments/departure-gate/collector-results/windows-c1fe15c/baseline.json'));
  const variants = {};
  for (const variant of ['f5', 'before']) {
    const source = path.resolve(options[`${variant}-exe`]);
    assert.equal(hash(source), expected[variant].sha256, `Frozen ${variant} executable mismatch`);
    const file = `${variant}-collector_profile.exe`;
    assert(!existsSync(path.join(output, file)), `Refuse overwrite ${file}`);
    copyFileSync(source, path.join(output, file));
    variants[variant] = { ...expected[variant], file, originalPath: source,
      buildArgv: ['cargo', 'build', '--release', '--locked', '--manifest-path', 'runtime/Cargo.toml', '--example', 'collector_profile', '--no-default-features', ...(expected[variant].metrics ? ['--features', 'collector-metrics'] : [])] };
  }
  copyFileSync(path.join(root, 'runtime/examples/collector_profile.rs'), path.join(output, 'collector_profile.rs'));
  mkdirSync(path.join(output, 'fixtures'));
  const workloads = definitions.map(([name, relative, mode]) => {
    const source = path.join(root, 'experiments/departure-gate', relative);
    const sha256 = hash(source);
    if (name !== 'high-degree') assert.equal(sha256, original.fixtureSha256[name]);
    const file = `fixtures/${name}.cav`;
    copyFileSync(source, path.join(output, file));
    return { name, source: path.relative(root, source).replaceAll('\\', '/'), file, sha256, mode };
  });
  save(contractFile, {
    schema: 'caveat-index-compaction-contract/1', preparedAt: new Date().toISOString(),
    harnessSha256: expected.harness, driverSha256: hash(fileURLToPath(import.meta.url)),
    variants, workloads, matrix, nativeRuns: matrix.length * 3,
    settings: { drainEvery: 1, unchangedProbes: 100, unrelatedProbes: 100, rustc: command('rustc', ['--version']).trim(), platform: process.platform, arch: process.arch,
      order: 'Trial 1 f5/before/candidate; trial 2 before/candidate/f5; trial 3 candidate/f5/before. All executions serial, quiet local verification required.' },
    targets: { population: 'reachable-chain, 3000 cycles, all three matched trials',
      retained: 'before_release.retained_rust_heap_bytes', growthPeak: 'growth.max_dispatch_additional_heap_bytes',
      formula: 'For each target and matched trial: F=fresh original f5; B=fresh repaired 3d; C=candidate. Require B>F, and (C-F)/(B-F)<=0.5. Candidate ceiling=floor(F+(B-F)/2). Both targets must pass in all three trials.',
      requiredReduction: 0.5, resetPolicy: 'Never overwrite runs, erase failures, change the denominator, or substitute saved bytes/timing. A revised implementation uses a new directory preserving the prior result.' },
    comparisons: 'Before/candidate exact native save and ordered archive hashes/counts. Required reachable-chain save also equals f5. Full snapshots checked by a separate WASM driver; native work counters are diagnostics except exact collected counts.',
    limits: ['Requested Rust heap excludes allocator internals, stacks and RSS.', 'Growth peak is whole-dispatch additional allocation, not collector-only bytes.', 'Native apply timings include allocator/counter overhead, omit snapshot serialization, and do not establish browser/WASM latency.', 'f5 lacks collector-metrics and uses the original no-feature build; its zero counters mean unavailable.', 'High-degree repeats a fixed 16-owner/16-subject shape; its cycle sweep is not a degree-scaling or universal bound claim.', 'Release-self has one sample at each size; repeated successful release-mutual timing has three samples per variant, not a stable population p99.'],
  });
  console.log(`Registered ${matrix.length} triples / ${matrix.length * 3} native runs; no execution or build.`);
} else {
  const contract = read(contractFile);
  assert.equal(contract.driverSha256, hash(fileURLToPath(import.meta.url)), 'Registered driver changed; preserve old registration and choose a new directory');
  const verifyFrozen = () => {
    assert.equal(hash(path.join(output, 'collector_profile.rs')), contract.harnessSha256);
    for (const variant of Object.values(contract.variants)) assert.equal(hash(path.join(output, variant.file)), variant.sha256);
    for (const workload of contract.workloads) assert.equal(hash(path.join(output, workload.file)), workload.sha256);
  };
  verifyFrozen();
  if (action === 'freeze') {
    assert(options['candidate-exe'] && options.revision, 'freeze requires --candidate-exe and --revision');
    assert.equal(git('rev-parse', 'HEAD'), options.revision);
    assert.equal(git('diff', 'HEAD', '--name-only'), '', 'Freeze only clean tracked source');
    assert.equal(hash(path.join(root, 'runtime/examples/collector_profile.rs')), contract.harnessSha256);
    const inputs = git('ls-files', '--', 'runtime', 'game', 'rust-toolchain.toml').split('\n').sort().map(file => ({ file, sha256: hash(path.join(root, file)) }));
    const executable = path.resolve(options['candidate-exe']);
    const file = 'candidate-collector_profile.exe';
    assert(!existsSync(candidateFile) && !existsSync(path.join(output, file)), 'Refuse replacement of frozen candidate');
    copyFileSync(executable, path.join(output, file));
    save(candidateFile, { revision: options.revision, file, sha256: hash(executable), metrics: true, frozenAt: new Date().toISOString(),
      runtimeGitTree: git('rev-parse', `${options.revision}:runtime`), inputs, sourceManifestSha256: sha(JSON.stringify(inputs)),
      buildArgv: contract.variants.before.buildArgv, harnessSha256: contract.harnessSha256 });
    console.log('Candidate frozen; no profile run.');
  } else if (action === 'run') {
    const candidate = read(candidateFile);
    const verifyCandidate = () => {
      assert.equal(hash(path.join(output, candidate.file)), candidate.sha256);
      for (const input of candidate.inputs) assert.equal(hash(path.join(root, input.file)), input.sha256, `Frozen input changed: ${input.file}`);
    };
    const cells = options.all === 'true' ? contract.matrix : contract.matrix.filter(cell => cell.name === options.workload && cell.cycles === Number(options.cycles) && cell.trial === Number(options.trial));
    assert(cells.length, 'Select --all=true or a registered --workload/--cycles/--trial');
    const variants = { ...contract.variants, candidate };
    mkdirSync(path.join(output, 'runs'), { recursive: true });
    for (const cell of cells) {
      verifyFrozen(); verifyCandidate();
      const workload = contract.workloads.find(value => value.name === cell.name);
      const key = `${cell.name}-${cell.cycles}-t${cell.trial}`;
      const resultFile = path.join(output, 'runs', `${key}-comparison.json`);
      assert(!existsSync(resultFile), `Refuse rerun ${key}`);
      const order = [['f5', 'before', 'candidate'], ['before', 'candidate', 'f5'], ['candidate', 'f5', 'before']][cell.trial - 1];
      const rows = {};
      try {
        for (const variant of order) {
          const destination = path.join(output, 'runs', `${key}-${variant}.json`);
          assert(!existsSync(destination), `Refuse overwrite ${destination}`);
          const info = variants[variant];
          const argv = [path.join(output, workload.file), String(cell.cycles), workload.mode, '1', '100'];
          const executable = path.join(output, info.file);
          const startedAt = new Date().toISOString();
          appendFileSync(path.join(output, 'attempts.jsonl'), JSON.stringify({ key, variant, startedAt, executableSha256: info.sha256, command: [executable, ...argv] }) + '\n');
          const data = JSON.parse(command(executable, argv, { cwd: output }));
          assert.equal(data.instrumented, info.metrics);
          verifyFrozen(); verifyCandidate();
          rows[variant] = data;
          save(destination, { ...cell, variant, revision: info.revision, executableSha256: info.sha256, fixtureSha256: workload.sha256, harnessSha256: contract.harnessSha256, startedAt, finishedAt: new Date().toISOString(), command: [executable, ...argv], data });
          console.log(`${key} ${variant}: ${data.before_release.retained_rust_heap_bytes} retained; ${data.growth.max_dispatch_additional_heap_bytes} growth peak`);
        }
        for (const variant of ['f5', 'candidate']) assert.equal(rows.before.source_sha256, rows[variant].source_sha256);
        for (const stage of ['before_release', 'after_release']) {
          for (const field of ['save_sha256', 'serialized_save_bytes', 'retired_dynamic_records', 'undrained', 'withdrawals']) assert.equal(rows.before[stage][field], rows.candidate[stage][field], `${stage}.${field}`);
          if (cell.name === 'reachable-chain') assert.equal(rows.f5[stage].save_sha256, rows.candidate[stage].save_sha256);
        }
        for (const field of ['ndjson_sha256', 'ndjson_bytes', 'records', 'provenance_nodes', 'growth_ndjson_bytes', 'growth_records', 'growth_nodes']) assert.equal(rows.before.archive[field], rows.candidate.archive[field], `archive.${field}`);
        for (const phase of ['growth', 'growth_last_10_percent', 'steady', 'unrelated_changes', 'release']) {
          assert.equal(rows.before[phase].samples, rows.candidate[phase].samples, `${phase}.samples`);
          if (rows.before[phase].samples) assert.equal(rows.before[phase].collector_work.collected, rows.candidate[phase].collector_work.collected, `${phase}.collected`);
        }
        if (workload.mode === 'release') for (const variant of ['before', 'candidate']) {
          assert.equal(rows[variant].failed_release.samples, 3);
          assert.equal(rows[variant].after_release.retired_dynamic_records, 0);
        }
        const targets = {};
        if (cell.name === 'reachable-chain' && cell.cycles === 3000) {
          for (const [name, get] of [ ['retained', row => row.before_release.retained_rust_heap_bytes], ['growthPeak', row => row.growth.max_dispatch_additional_heap_bytes] ]) {
            const F = get(rows.f5), B = get(rows.before), C = get(rows.candidate);
            const denominator = B - F;
            targets[name] = { F, B, C, denominator, candidateAdded: C - F, ceiling: Math.floor(F + denominator / 2), reduction: denominator > 0 ? 1 - (C - F) / denominator : null, passed: denominator > 0 && C - F <= denominator / 2 };
          }
        }
        const passed = Object.values(targets).every(target => target.passed);
        save(resultFile, { ...cell, order, exactNativeEquality: true, targets, passed,
          rows: Object.fromEntries(Object.entries(rows).map(([variant, data]) => [variant, { before_release: data.before_release, after_release: data.after_release, growth: data.growth, growth_last_10_percent: data.growth_last_10_percent, steady: data.steady, unrelated_changes: data.unrelated_changes, failed_release: data.failed_release, release: data.release }])) });
        if (!passed) process.exitCode = 2;
      } catch (error) {
        save(resultFile, { ...cell, order, passed: false, error: String(error.stack ?? error), completedVariants: Object.keys(rows) });
        throw error;
      }
    }
  } else if (action === 'summarize') {
    const results = contract.matrix.map(cell => {
      const file = `runs/${cell.name}-${cell.cycles}-t${cell.trial}-comparison.json`;
      return existsSync(path.join(output, file)) ? { file, sha256: hash(path.join(output, file)), result: read(path.join(output, file)) } : { file, missing: true };
    });
    const complete = results.every(row => !row.missing);
    const passed = complete && results.every(row => row.result.passed);
    const targets = results.filter(row => row.result?.name === 'reachable-chain' && row.result.cycles === 3000).map(row => ({ trial: row.result.trial, targets: row.result.targets, passed: row.result.passed }));
    const file = path.join(output, `summary-${new Date().toISOString().replaceAll(':', '-')}.json`);
    save(file, { recordedAt: new Date().toISOString(), complete, passed, expectedNativeRuns: contract.nativeRuns, targets, results, contractSha256: hash(contractFile), candidateSha256: existsSync(candidateFile) ? hash(candidateFile) : null });
    console.log(JSON.stringify({ file, complete, passed, targets }, null, 2));
    if (!passed) process.exitCode = 2;
  } else throw new Error('Use prepare, freeze, run, or summarize');
}
