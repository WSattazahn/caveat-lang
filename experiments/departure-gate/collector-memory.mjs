#!/usr/bin/env node
// Supplemental native allocation-state evidence; no runtime behavior changes.
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {copyFileSync, mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {currentRuntimeFingerprint} from '../../scripts/runtime-build-fingerprint.mjs';

const root = fileURLToPath(new URL('../..', import.meta.url));
const args = process.argv.slice(2);
if (args.length && (args.length !== 2 || args[0] !== '--output')) {
  throw new Error('Usage: node experiments/departure-gate/collector-memory.mjs [--output DIRECTORY]');
}
const output = resolve(root, args[1] ?? 'test-results/collector-release-memory');
const runtimeRevision = 'c1fe15cfcb1cac6c069ba5b00f1c7238df7e9853';
const runtimeInputs = ['runtime/src', 'runtime/Cargo.toml', 'runtime/Cargo.lock', 'runtime/prelude.cav',
  'rust-toolchain.toml', 'game/the_door_round2.cav', 'web/the_door_round2.cav',
  'scripts/build-web.mjs', 'scripts/runtime-build-fingerprint.mjs', 'runtime/build.rs',
  '.cargo/config', '.cargo/config.toml', 'runtime/.cargo/config', 'runtime/.cargo/config.toml'];
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
function run(command, argv) {
  const result = spawnSync(command, argv, {cwd: root, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024});
  if (result.error) throw result.error;
  return result;
}
function requireSuccess(command, argv) {
  const result = run(command, argv);
  if (result.status !== 0) throw new Error(`${command} ${argv.join(' ')} failed (${result.status}):\n${result.stderr}\n${result.stdout}`);
  return result.stdout.trim();
}
function verifyRuntime() {
  requireSuccess('git', ['diff', '--exit-code', runtimeRevision, '--', ...runtimeInputs]);
  if (requireSuccess('git', ['ls-files', '--others', '--exclude-standard', '--', ...runtimeInputs])) {
    throw new Error('Untracked runtime/build inputs are outside the pinned source identity');
  }
}
verifyRuntime();
const runtimeBuildFingerprint = await currentRuntimeFingerprint(root);
mkdirSync(join(output, 'programs'), {recursive: true});
const inputNames = requireSuccess('git', ['ls-files', ...runtimeInputs]).split(/\r?\n/).filter(Boolean).sort();
const inputHashes = Object.fromEntries(inputNames.map(name => [name, digest(readFileSync(join(root, name)))]));
const sourceHash = digest(inputNames.map(name => `${name}\0${inputHashes[name]}\n`).join(''));
const harness = 'runtime/examples/collector_release_memory.rs';
const harnessHash = digest(readFileSync(join(root, harness)));
const buildCommand = ['cargo', 'build', '--locked', '--release', '--manifest-path', 'runtime/Cargo.toml', '--no-default-features', '--example', 'collector_release_memory'];
const built = run(buildCommand[0], buildCommand.slice(1));
writeFileSync(join(output, 'build.log'), built.stdout + built.stderr);
if (built.status !== 0) throw new Error(`Build failed; see ${join(output, 'build.log')}`);
const executable = join(root, 'runtime/target/release/examples', `collector_release_memory${process.platform === 'win32' ? '.exe' : ''}`);
const fixtures = {};
const commands = [];
const runs = [];
for (const name of ['release-self', 'release-mutual']) {
  const original = join(root, 'experiments/departure-gate/collector-fixtures', `${name}.cav`);
  const saved = join(output, 'programs', `${name}.cav`);
  copyFileSync(original, saved);
  fixtures[name] = digest(readFileSync(saved));
  for (const cycles of [60, 300, 1000, 3000]) {
    const command = [executable, saved, String(cycles)];
    commands.push(command);
    const result = run(command[0], command.slice(1));
    writeFileSync(join(output, `${name}-${cycles}.stderr.log`), result.stderr);
    writeFileSync(join(output, `${name}-${cycles}.json`), result.stdout);
    if (result.status !== 0) throw new Error(`${name}/${cycles} failed; see its stderr receipt`);
    const row = JSON.parse(result.stdout);
    if (row.harness_sha256 !== harnessHash) throw new Error('Compiled harness hash differs from current source');
    runs.push({...row, fixture: name});
    console.log(`${name}/${cycles}: ${row.after_next_accepted_noop_and_drain.requested_runtime_heap_bytes} B after no-op; ${row.after_same_save_restore_and_drop_original.requested_runtime_heap_bytes} B after identical-save restore`);
  }
}
verifyRuntime();
if (await currentRuntimeFingerprint(root) !== runtimeBuildFingerprint) throw new Error('Runtime build inputs changed during measurement');
for (const name of inputNames) {
  if (digest(readFileSync(join(root, name))) !== inputHashes[name]) throw new Error(`Runtime input changed during measurement: ${name}`);
}
if (digest(readFileSync(join(root, harness))) !== harnessHash) throw new Error('Harness changed during measurement');
const report = {
  schema: 'caveat-collector-release-memory-run/1',
  measured_at: new Date().toISOString(),
  runtime_revision: runtimeRevision,
  runtime_inputs_match_revision: true,
  runtime_source_sha256: sourceHash,
  runtime_build_fingerprint: runtimeBuildFingerprint,
  runtime_source_sha256_definition: 'SHA256 of sorted tracked runtime/src, Cargo, prelude, toolchain, embedded game, build-script and configuration entries encoded as path + NUL + file-SHA256 + LF (actual host bytes)',
  runtime_input_sha256: inputHashes,
  harness_sha256: harnessHash,
  driver_sha256: digest(readFileSync(fileURLToPath(import.meta.url))),
  executable_sha256: digest(readFileSync(executable)),
  rustc: requireSuccess('rustc', ['--version']),
  build_command: buildCommand,
  run_commands: commands,
  fixture_sha256: fixtures,
  runs,
  limits: [
    'One run per scenario measures allocation state, not latency distributions.',
    'Requested Rust heap excludes allocator internals, source/argument buffers, stacks and RSS; it is not exact collector-only allocation.',
    'The supplemental harness is additional to runtime commit c1fe15c. Runtime/build inputs match that revision before and after execution.',
    'Every archive buffer is dropped before each checkpoint; checkpoint structs own no heap. Save equality across restore is asserted.',
    'serialized_effects_bytes counts the serialized array value only, excluding key/punctuation, and is zero when omitted.',
  ],
};
writeFileSync(join(output, 'report.json'), `${JSON.stringify(report, null, 2)}\n`);
console.log(join(output, 'report.json'));
