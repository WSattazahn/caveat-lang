#!/usr/bin/env node
// Finite profiling only. This driver never builds, changes runtime source or optimizes it.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { appendFileSync, copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const argv = process.argv.slice(2), action = argv.shift(), opts = {};
for (const arg of argv) {
  assert(arg.startsWith('--') && arg.includes('='), `Use --name=value: ${arg}`);
  const cut = arg.indexOf('='), name = arg.slice(2, cut);
  assert(!Object.hasOwn(opts, name), `Duplicate option: ${name}`);
  opts[name] = arg.slice(cut + 1);
}
assert(['register', 'freeze', 'run', 'summarize'].includes(action), 'Use register, freeze, run or summarize');
assert(opts.output, 'Pass --output=CAPTURE_DIRECTORY');
const root = path.resolve(opts.root ?? fileURLToPath(new URL('../../../', import.meta.url)));
const output = path.resolve(opts.output), script = fileURLToPath(import.meta.url);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const hash = file => sha(readFileSync(file));
const read = file => JSON.parse(readFileSync(file, 'utf8'));
const saved = file => read(path.join(output, file));
const save = (file, value) => writeFileSync(path.join(output, file), JSON.stringify(value, null, 2) + '\n', { encoding: 'utf8', flag: 'wx' });
const invoke = (executable, args, extra = {}) => spawnSync(executable, args, {
  cwd: root, encoding: 'utf8', timeout: 1800000, maxBuffer: 32 * 1024 * 1024, windowsHide: true, ...extra,
});
const command = (executable, args) => {
  const result = invoke(executable, args);
  assert(!result.error && result.status === 0, `${executable}: ${result.error ?? result.stderr}`);
  return result.stdout.trim();
};
const git = (...args) => command('git', ['-c', `safe.directory=${root.replaceAll('\\', '/')}`, ...args]);
const copy = (source, relative) => {
  const target = path.join(output, relative);
  assert(!existsSync(target), `Refuse overwrite: ${target}`);
  mkdirSync(path.dirname(target), { recursive: true });
  copyFileSync(source, target);
  return { file: relative, sha256: hash(target) };
};
const baseline = {
  revision: '055e2af0b411d8385941dd8522201e5afc73cb34',
  nativeBuildRevision: '8a161dcdfdc8a2a5b3ca43b4183552401002cca7',
  runtimeTree: 'aec511bf3ee147a79391f90674446eff9c7f393e',
  executableSha256: '9f5a0a5af2b4073849d30fd7070380eb1341ea4ae9206828aa7fd755cb207909',
  harnessSha256: '1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357',
};
const plainBuild = ['cargo', 'build', '--release', '--locked', '--manifest-path', 'runtime/Cargo.toml',
  '--example', 'collector_profile', '--no-default-features', '--features', 'collector-metrics'];
const diagnosticBuild = ['cargo', 'build', '--release', '--locked', '--manifest-path', 'runtime/Cargo.toml',
  '--example', 'renewal_removal_profile', '--no-default-features', '--features', 'renewal-removal-profile'];
const primary = Array.from({ length: 4 }, (_, i) => ({ name: 'release-mutual', cycles: 3000, trial: i + 1, primary: true }));
const supplements = [...[1, 2, 60, 300, 1000].map(cycles => ['release-mutual', cycles]),
  ['release-self', 3000], ['high-degree', 60], ['high-degree', 1000], ['reachable-chain', 3000]];
const matrix = [...primary, ...supplements.map(([name, cycles]) => ({ name, cycles, trial: 1, primary: false }))]
  .map((cell, i) => ({ ...cell, order: i % 2 ? ['diagnostic', 'plain'] : ['plain', 'diagnostic'] }));
const keyOf = cell => `${cell.name}-${cell.cycles}-t${cell.trial}`;
const phases = ['growth', 'growth_last_10_percent', 'steady', 'unrelated_changes', 'failed_release', 'release'];
const paths = ['renewal_removal', 'readings'];
const fields = ['blocks', 'searches', 'removes', 'matches', 'misses', 'probes', 'shifted_elements',
  'estimated_shifted_header_bytes', 'shared_cow_detaches', 'cow_cloned_histories', 'cow_cloned_occurrences',
  'block_ns', 'search_ns', 'cow_ns', 'remove_ns'];
const stateFields = ['save_sha256', 'serialized_save_bytes', 'retired_dynamic_records', 'undrained', 'withdrawals'];
const archiveFields = ['ndjson_sha256', 'ndjson_bytes', 'records', 'provenance_nodes', 'growth_ndjson_bytes',
  'growth_records', 'growth_nodes', 'max_undrained_items', 'items_at_growth_end'];
function equalSemantics(a, b) {
  for (const field of ['source_sha256', 'cycles', 'mode', 'drain_every']) assert.equal(a[field], b[field], field);
  for (const state of ['before_release', 'after_release'])
    for (const field of stateFields) assert.equal(a[state][field], b[state][field], `${state}.${field}`);
  for (const field of archiveFields) assert.equal(a.archive[field], b.archive[field], `archive.${field}`);
  for (const phase of phases) {
    assert.equal(a[phase].samples, b[phase].samples, `${phase}.samples`);
    if (a[phase].samples && phase !== 'failed_release')
      assert.deepEqual(a[phase].collector_work, b[phase].collector_work, `${phase}.successful_collector_work`);
  }
}
function diagnosticShape(data) {
  assert.equal(data.schema, 'caveat-renewal-removal-diagnostic/1');
  assert.equal(data.diagnostic_only, true);
  for (const phase of phases) {
    const row = data[phase];
    if (!row.samples) continue;
    for (const kind of paths) for (const field of fields)
      assert(Number.isSafeInteger(row.removal[kind][field]) && row.removal[kind][field] >= 0, `${phase}.${kind}.${field}`);
    assert(Number.isSafeInteger(row.removal.departure_ns) && row.removal.departure_ns >= 0);
    if (row.samples <= 3) {
      assert.equal(row.removal.small_phase_samples.length, row.samples);
      for (const sample of row.removal.small_phase_samples) {
        assert(Number.isSafeInteger(sample.ns) && sample.ns >= 0);
        assert(Number.isSafeInteger(sample.removal.departure_ns) && sample.removal.departure_ns >= 0);
        for (const kind of paths) for (const field of fields)
          assert(Number.isSafeInteger(sample.removal[kind][field]) && sample.removal[kind][field] >= 0);
      }
    }
  }
}

if (action === 'register') {
  assert(opts['accepted-run'], 'Pass --accepted-run=ORIGINAL_RUN_002');
  assert(!existsSync(output), 'Registration requires a wholly new directory, including after partial registration');
  const accepted = path.resolve(opts['accepted-run']), old = read(path.join(accepted, 'contract.json'));
  const identity = read(path.join(accepted, 'candidate.json'));
  assert.equal(identity.revision, baseline.nativeBuildRevision);
  assert.equal(identity.runtimeGitTree, baseline.runtimeTree);
  assert.equal(identity.sha256, baseline.executableSha256);
  assert.equal(sha(JSON.stringify(identity.inputs)), identity.sourceManifestSha256);
  const acceptedBuild = read(path.join(accepted, 'candidate-build.json'));
  assert.equal(acceptedBuild.revision, baseline.nativeBuildRevision);
  assert.equal(acceptedBuild.runtimeGitTree, baseline.runtimeTree);
  assert.equal(acceptedBuild.status, 0);
  assert.equal(acceptedBuild.error, null);
  assert.deepEqual(acceptedBuild.argv, plainBuild);
  assert.equal(acceptedBuild.executableSha256, baseline.executableSha256);
  for (const channel of ['stdout', 'stderr'])
    assert.equal(hash(path.join(accepted, `candidate-build.${channel}.txt`)), acceptedBuild[`${channel}Sha256`]);
  const acceptedSource = read(path.join(accepted, 'candidate-source/manifest.json'));
  assert.equal(acceptedSource.revision, identity.revision);
  assert.equal(acceptedSource.sourceManifestSha256, identity.sourceManifestSha256);
  assert.deepEqual(acceptedSource.files, identity.inputs);
  assert.equal(hash(path.join(accepted, identity.file)), baseline.executableSha256);
  for (const revision of [baseline.revision, baseline.nativeBuildRevision])
    assert.equal(git('rev-parse', `${revision}:runtime`), baseline.runtimeTree);
  assert.equal(git('diff', '--name-only', baseline.nativeBuildRevision, baseline.revision, '--', 'runtime', 'game', 'rust-toolchain.toml'), '');
  assert.equal(hash(path.join(root, 'runtime/examples/collector_profile.rs')), baseline.harnessSha256);
  mkdirSync(output, { recursive: true });
  const referenceManifest = [];
  const referenceCopy = (source, relative) => referenceManifest.push(copy(source, relative));
  referenceCopy(path.join(accepted, identity.file), 'accepted-reference/accepted.exe');
  for (const file of ['candidate.json', 'candidate-build.json', 'candidate-build.stdout.txt', 'candidate-build.stderr.txt', 'candidate-source/manifest.json'])
    referenceCopy(path.join(accepted, file), `accepted-reference/${file}`);
  copy(path.join(root, 'runtime/examples/collector_profile.rs'), 'collector_profile.rs');
  const used = new Set(matrix.map(cell => cell.name));
  const fixtures = old.fixtures.filter(f => used.has(f.name)).map(f => {
    assert.equal(hash(path.join(root, f.source)), f.sha256);
    assert.equal(hash(path.join(accepted, f.file)), f.sha256);
    copy(path.join(accepted, f.file), f.file);
    return f;
  });
  assert.equal(fixtures.length, 4);
  const references = matrix.map(cell => {
    const file = `target/${keyOf(cell)}-candidate.json`, row = read(path.join(accepted, file));
    assert.equal(row.revision, baseline.nativeBuildRevision);
    assert.equal(row.executableSha256, baseline.executableSha256);
    assert.equal(row.status, 0);
    assert.equal(row.signal, null);
    const fixture = fixtures.find(f => f.name === cell.name);
    assert.equal(row.fixtureSha256, fixture.sha256);
    for (const stream of ['stdout', 'stderr']) {
      assert.equal(hash(path.join(accepted, row[stream].file)), row[stream].sha256);
      referenceCopy(path.join(accepted, row[stream].file), `accepted-reference/${row[stream].file}`);
    }
    assert.deepEqual(read(path.join(accepted, row.stdout.file)), row.data);
    referenceCopy(path.join(accepted, file), `accepted-reference/${file}`);
    return { key: keyOf(cell), file: `accepted-reference/${file}`, originalFile: path.join(accepted, file), sha256: hash(path.join(accepted, file)) };
  });
  save('contract.json', {
    schema: 'caveat-renewal-removal-protocol/1', registeredAt: new Date().toISOString(), baseline,
    driverSha256: hash(script), fixtures, matrix, freshProcesses: 26, pairs: 13,
    references, referenceManifest, referenceScope: 'Historical accepted semantics and source identity only; never use old latency in current timing comparisons.',
    builds: { plain: plainBuild, diagnostic: diagnosticBuild },
    diagnostic: { schema: 'caveat-renewal-removal-diagnostic/1', paths, fields },
    settings: { drainEvery: 1, probes: 100, platform: process.platform, arch: process.arch,
      node: process.version, rustc: command('rustc', ['--version']), rustcVerbose: command('rustc', ['--version', '--verbose']),
      timeoutMs: 1800000, maxBufferBytes: 32 * 1024 * 1024,
      order: 'Serial, alternating fresh plain/diagnostic pairs; coordinator confirms no concurrent task builds/tests.' },
    purpose: 'Profile baseline renewal occurrence removal. No optimization, speedup requirement or memory acceptance threshold.',
    aggregation: 'Four primary matched pairs: show every value and min/median/max. Each rejected process contains three attempts; retain each diagnostic sample and its within-process median share. One-pair supplemental timings are descriptive only.',
    failures: 'Append attempt before launch; preserve all raw outputs/errors and partial results. Refuse reruns and overwrites. A failed validity check stops capture; do not reset or retune it.',
    limits: ['Plain control still includes allocator accounting and collector-metrics; only the new attribution feature is disabled.',
      'Branch timers exclude immutable history lookup and journal/reading classification; first shared-map COW is included.',
      'Nested search/COW/remove clocks are parts of block time, not additive independent totals.',
      'Post-block COW inventory scans affect apply/departure time; instrumentation overhead cannot simply be subtracted.',
      'Rejected collector_work is a zero placeholder; only nontransactional diagnostic counters measure attempted removal work.',
      'Requested heap excludes allocator metadata, stacks and RSS; native apply omits later serialization, drain, browser and WASM.',
      'No stable tails, universal workload claim or predicted optimization gain follows from this finite sample.'],
  });
  save('registration.json', { schema: 'caveat-renewal-removal-registration/1', registeredAt: new Date().toISOString(),
    contractSha256: hash(path.join(output, 'contract.json')), driverSha256: hash(script) });
  console.log('Registered 13 pairs / 26 fresh processes; no measurements or optimization target.');
} else {
  const contract = saved('contract.json');
  const registration = saved('registration.json'), contractSha256 = hash(path.join(output, 'contract.json'));
  const registrationSha256 = hash(path.join(output, 'registration.json'));
  assert.equal(registration.contractSha256, contractSha256, 'Registered contract changed');
  assert.equal(registration.driverSha256, hash(script), 'Registration driver changed');
  assert.equal(contract.driverSha256, hash(script), 'Registered driver changed');
  const verifyReference = () => {
    assert.equal(hash(path.join(output, 'contract.json')), contractSha256, 'Contract changed during phase');
    assert.equal(hash(path.join(output, 'registration.json')), registrationSha256, 'Registration changed during phase');
    assert.equal(hash(path.join(output, 'collector_profile.rs')), baseline.harnessSha256);
    for (const file of [...contract.fixtures, ...contract.referenceManifest])
      assert.equal(hash(path.join(output, file.file)), file.sha256, file.file);
  };
  verifyReference();
  const verifyFrozen = info => {
    assert.equal(info.contractSha256, contractSha256, 'Frozen identity belongs to another contract');
    assert.equal(hash(path.join(output, info.file)), info.sha256);
    for (const item of [info.buildObservation, ...info.buildStreams])
      assert.equal(hash(path.join(output, item.file)), item.sha256, item.file);
    for (const item of info.inputs) assert.equal(hash(path.join(root, item.file)), item.sha256, `Changed source: ${item.file}`);
  };
  if (action === 'freeze') {
    assert(['plain', 'diagnostic'].includes(opts.kind) && opts.exe && opts.revision && opts['build-observation'],
      'Pass --kind=plain|diagnostic --exe=FILE --revision=FULL_SHA --build-observation=JSON');
    assert.match(opts.revision, /^[0-9a-f]{40}$/);
    assert.equal(git('rev-parse', 'HEAD'), opts.revision);
    assert.equal(git('status', '--porcelain', '--untracked-files=no'), '', 'Require clean tracked profiling source');
    assert.equal(hash(path.join(root, 'runtime/examples/collector_profile.rs')), baseline.harnessSha256);
    const observation = read(opts['build-observation']);
    assert.equal(observation.revision, opts.revision);
    assert.equal(observation.status, 0);
    assert.equal(observation.error, null);
    assert.equal(observation.inputsStable, true);
    assert.equal(observation.headAfter, opts.revision);
    assert.equal(observation.trackedDiffAfter, '');
    assert.equal(observation.runtimeGitTree, git('rev-parse', 'HEAD:runtime'));
    assert.equal(observation.rustc, contract.settings.rustcVerbose);
    assert.deepEqual(observation.argv, contract.builds[opts.kind]);
    assert.equal(observation.executableSha256, hash(opts.exe));
    const names = new Set(git('ls-files', '--', 'runtime', 'game', 'web/the_door_round2.cav', 'rust-toolchain.toml',
      'scripts', 'package.json', 'package-lock.json', 'experiments/departure-gate/renewal-removal-profile').split('\n').filter(Boolean));
    for (const optional of ['runtime/build.rs', '.cargo/config', '.cargo/config.toml', 'runtime/.cargo/config', 'runtime/.cargo/config.toml'])
      if (existsSync(path.join(root, optional))) names.add(optional);
    const inputs = [...names].sort()
      .map(file => ({ file, sha256: hash(path.join(root, file)) }));
    assert.deepEqual(observation.inputs, inputs, 'Build and freeze source selections differ');
    const buildStreams = ['stdout', 'stderr'].map(channel => {
      const source = path.join(path.dirname(opts['build-observation']), `build.${channel}`);
      assert.equal(hash(source), observation[`${channel}Sha256`]);
      return copy(source, `${opts.kind}-build.${channel}`);
    });
    const frozen = copy(opts.exe, `${opts.kind}.exe`);
    const buildObservation = copy(opts['build-observation'], `${opts.kind}-build-observation.json`);
    save(`${opts.kind}.json`, { kind: opts.kind, revision: opts.revision, contractSha256, ...frozen,
      frozenAt: new Date().toISOString(), runtimeGitTree: git('rev-parse', 'HEAD:runtime'),
      inputs, sourceManifestSha256: sha(JSON.stringify(inputs)), buildArgv: contract.builds[opts.kind], buildObservation, buildStreams });
    console.log(`Frozen ${opts.kind}; no process launched.`);
  } else if (action === 'run') {
    const identity = { plain: saved('plain.json'), diagnostic: saved('diagnostic.json') };
    assert.equal(identity.plain.revision, identity.diagnostic.revision);
    assert.deepEqual(identity.plain.inputs, identity.diagnostic.inputs, 'Both builds require the same clean source');
    mkdirSync(path.join(output, 'capture'), { recursive: true });
    for (const cell of contract.matrix) {
      const key = keyOf(cell), rows = {}, comparison = `capture/${key}-comparison.json`;
      assert(!existsSync(path.join(output, comparison)), `Prior comparison preserved: ${key}`);
      try {
        for (const variant of cell.order) {
          const destination = `capture/${key}-${variant}.json`;
          const stdoutFile = `capture/${key}-${variant}.stdout.txt`, stderrFile = `capture/${key}-${variant}.stderr.txt`;
          for (const file of [destination, stdoutFile, stderrFile]) assert(!existsSync(path.join(output, file)), `Prior partial capture: ${file}`);
          const previous = existsSync(path.join(output, 'attempts.jsonl')) ? readFileSync(path.join(output, 'attempts.jsonl'), 'utf8').trim().split('\n').filter(Boolean).map(JSON.parse) : [];
          assert(!previous.some(a => a.key === key && a.variant === variant), `Prior attempted process preserved: ${key}/${variant}`);
          verifyReference(); verifyFrozen(identity.plain); verifyFrozen(identity.diagnostic);
          const info = identity[variant], fixture = contract.fixtures.find(f => f.name === cell.name);
          const args = [path.join(output, fixture.file), String(cell.cycles), fixture.mode, '1', '100'];
          const startedAt = new Date().toISOString(), executable = path.join(output, info.file);
          const attempt = { key, variant, startedAt, contractSha256, revision: info.revision, executableSha256: info.sha256, command: [executable, ...args] };
          appendFileSync(path.join(output, 'attempts.jsonl'), JSON.stringify(attempt) + '\n', 'utf8');
          const result = invoke(executable, args, { cwd: output });
          writeFileSync(path.join(output, stdoutFile), result.stdout ?? '', { encoding: 'utf8', flag: 'wx' });
          writeFileSync(path.join(output, stderrFile), result.stderr ?? '', { encoding: 'utf8', flag: 'wx' });
          const row = { ...cell, ...attempt, finishedAt: new Date().toISOString(), fixtureSha256: fixture.sha256,
            status: result.status, signal: result.signal, error: result.error?.message ?? null,
            stdout: { file: stdoutFile, sha256: hash(path.join(output, stdoutFile)) },
            stderr: { file: stderrFile, sha256: hash(path.join(output, stderrFile)) } };
          try { if (!result.error && result.status === 0) row.data = JSON.parse(result.stdout); }
          catch (error) { row.parseError = String(error); }
          save(destination, row);
          assert(!row.error && row.status === 0 && !row.parseError, `Process failed; raw attempt preserved: ${key}/${variant}`);
          assert.equal(row.data.instrumented, true);
          if (variant === 'diagnostic') diagnosticShape(row.data);
          else {
            assert.equal(row.data.schema, 'caveat-collector-native-profile/1');
            const reference = saved(contract.references.find(r => r.key === key).file);
            equalSemantics(row.data, reference.data);
          }
          rows[variant] = row.data;
          verifyReference(); verifyFrozen(info);
          console.log(`${key} ${variant}: release=${row.data.release.median_ns ?? 'none'}ns rejected=${row.data.failed_release.median_ns ?? 'none'}ns`);
        }
        equalSemantics(rows.plain, rows.diagnostic);
        save(comparison, { ...cell, valid: true, exactNativeSemanticEquality: true,
          acceptedReferenceSemanticEquality: true, scope: 'Finite save/archive digests/counts and successful collector work; no performance acceptance target.' });
      } catch (error) {
        save(comparison, { ...cell, valid: false, error: String(error.stack ?? error), completedVariants: Object.keys(rows) });
        throw error;
      }
    }
  } else {
    // Summarize only existing captures. New output is exclusive; native execution never occurs here.
    const median = values => { const a = [...values].sort((a, b) => a - b), n = a.length; return n ? (n % 2 ? a[(n - 1) / 2] : (a[n / 2 - 1] + a[n / 2]) / 2) : null; };
    const distribution = values => { const a = values.filter(x => x !== null); return { count: a.length, min: a.length ? Math.min(...a) : null, median: median(a), max: a.length ? Math.max(...a) : null }; };
    const ratio = (n, d) => d ? n / d : null;
    const rows = [], manifest = [];
    const recorded = relative => { const file = path.join(output, relative); manifest.push({ file: relative, sha256: hash(file) }); return read(file); };
    const attempts = readFileSync(path.join(output, 'attempts.jsonl'), 'utf8').trim().split('\n').filter(Boolean).map(JSON.parse);
    assert.equal(attempts.length, 26, 'Incomplete capture; preserve it and report the actual failure');
    manifest.push({ file: 'attempts.jsonl', sha256: hash(path.join(output, 'attempts.jsonl')) });
    for (const cell of contract.matrix) {
      const key = keyOf(cell), comparison = recorded(`capture/${key}-comparison.json`);
      assert.equal(comparison.valid, true, `Invalid comparison: ${key}`);
      const captured = Object.fromEntries(['plain', 'diagnostic'].map(variant => {
        const row = recorded(`capture/${key}-${variant}.json`);
        assert.equal(row.status, 0);
        for (const channel of ['stdout', 'stderr']) {
          assert.equal(hash(path.join(output, row[channel].file)), row[channel].sha256);
          manifest.push(row[channel]);
        }
        assert.deepEqual(read(path.join(output, row.stdout.file)), row.data);
        assert.equal(attempts.filter(a => a.key === key && a.variant === variant).length, 1);
        const attempt = attempts.find(a => a.key === key && a.variant === variant);
        for (const field of ['command', 'startedAt', 'revision', 'executableSha256', 'contractSha256'])
          assert.deepEqual(row[field], attempt[field], `Attempt ledger mismatch: ${key}/${variant}/${field}`);
        assert.equal(row.contractSha256, contractSha256);
        return [variant, row.data];
      }));
      const P = captured.plain, D = captured.diagnostic;
      equalSemantics(P, D); diagnosticShape(D);
      equalSemantics(P, saved(contract.references.find(r => r.key === key).file).data);
      const phaseRows = {};
      for (const phase of phases) {
        if (!P[phase].samples) { phaseRows[phase] = { samples: 0 }; continue; }
        const p = P[phase], d = D[phase];
        const samples = (d.removal.small_phase_samples ?? []).map(sample => ({
          apply_ns: sample.ns, departure_ns: sample.removal.departure_ns,
          ...Object.fromEntries(paths.map(kind => [kind, { ...sample.removal[kind],
            block_share_of_apply: ratio(sample.removal[kind].block_ns, sample.ns),
            block_share_of_departure: ratio(sample.removal[kind].block_ns, sample.removal.departure_ns),
            cow_share_of_block: ratio(sample.removal[kind].cow_ns, sample.removal[kind].block_ns) }])),
        }));
        phaseRows[phase] = { samples: p.samples, plain_median_ns: p.median_ns, diagnostic_median_ns: d.median_ns,
          diagnostic_to_plain_latency_ratio: ratio(d.median_ns, p.median_ns),
          plain_peak_bytes: p.max_dispatch_additional_heap_bytes, diagnostic_peak_bytes: d.max_dispatch_additional_heap_bytes,
          peak_delta_bytes: d.max_dispatch_additional_heap_bytes - p.max_dispatch_additional_heap_bytes,
          attempted_removal_totals: Object.fromEntries(paths.map(kind => [kind, d.removal[kind]])),
          departure_ns_total: d.removal.departure_ns, raw_small_phase_samples: samples,
          process_sample_medians: Object.fromEntries(paths.map(kind => [kind, {
            block_ns: median(samples.map(s => s[kind].block_ns)),
            block_share_of_apply: median(samples.map(s => s[kind].block_share_of_apply).filter(v => v !== null)),
            block_share_of_departure: median(samples.map(s => s[kind].block_share_of_departure).filter(v => v !== null)),
            cow_ns: median(samples.map(s => s[kind].cow_ns)),
          }])),
        };
      }
      rows.push({ ...cell, phases: phaseRows,
        retained_heap_bytes: Object.fromEntries(['before_release', 'after_release'].map(state => [state,
          { plain: P[state].retained_rust_heap_bytes, diagnostic: D[state].retained_rust_heap_bytes,
            delta: D[state].retained_rust_heap_bytes - P[state].retained_rust_heap_bytes }])),
        semantic_fields: { before_release: Object.fromEntries(stateFields.map(f => [f, P.before_release[f]])),
          after_release: Object.fromEntries(stateFields.map(f => [f, P.after_release[f]])),
          archive: Object.fromEntries(archiveFields.map(f => [f, P.archive[f]])) },
      });
    }
    const primaryRows = rows.filter(row => row.primary), primarySummary = {};
    for (const phase of ['release', 'failed_release']) {
      const values = primaryRows.map(row => row.phases[phase]);
      primarySummary[phase] = {
        plain_median_ns: distribution(values.map(v => v.plain_median_ns)),
        diagnostic_median_ns: distribution(values.map(v => v.diagnostic_median_ns)),
        diagnostic_to_plain_latency_ratio: distribution(values.map(v => v.diagnostic_to_plain_latency_ratio)),
        ...Object.fromEntries(paths.map(kind => [kind, Object.fromEntries(['block_ns', 'block_share_of_apply', 'block_share_of_departure', 'cow_ns']
          .map(field => [field, distribution(values.map(v => v.process_sample_medians[kind][field]))]))])),
      };
    }
    for (const file of ['contract.json', 'registration.json', 'plain.json', 'diagnostic.json']) manifest.push({ file, sha256: hash(path.join(output, file)) });
    for (const kind of ['plain', 'diagnostic']) {
      const identity = saved(`${kind}.json`);
      assert.equal(identity.contractSha256, contractSha256);
      for (const item of [{ file: identity.file, sha256: identity.sha256 }, identity.buildObservation, ...identity.buildStreams]) {
        assert.equal(hash(path.join(output, item.file)), item.sha256);
        manifest.push(item);
      }
    }
    const summary = { schema: 'caveat-renewal-removal-summary/1', recordedAt: new Date().toISOString(), contractSha256, complete: true,
      measurementValidity: 'All 26 processes captured, raw streams checked, finite semantic comparisons equal.',
      noPerformanceAcceptanceTarget: true, processes: 26, pairs: 13, primarySummary, rows, manifest,
      scope: 'Descriptive attribution and instrumentation overhead only. No speedup prediction, optimization approval or stable-tail estimate.' };
    const destination = `summary-${new Date().toISOString().replaceAll(':', '-')}.json`;
    save(destination, summary);
    console.log(JSON.stringify({ destination, complete: true, primarySummary }, null, 2));
  }
}
