// What is being measured. A target is one of:
//   tree:DIR      a caveat-lang checkout. The native benchmark is built against
//                 DIR/runtime, the reactive WebAssembly is built from it with
//                 the flags scripts/build-web.mjs uses, and DIR/kit/lib is the
//                 kit. Everything built goes under DIR/runtime/target/, which
//                 is git-ignored build output; no tracked file changes.
//   package:DIR   an unpacked caveat-lang npm package: DIR/runtime holds the
//                 published WebAssembly and DIR/lib the kit. No native side.
//   runtime:DIR   a directory holding caveat_runtime.js and
//                 caveat_runtime_bg.wasm, with no kit (for example an older
//                 build). No native side.
import { spawnSync } from 'node:child_process';
import { copyFile, mkdir, readFile, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { homedir } from 'node:os';
import path from 'node:path';
import { harnessDirectory, sha256 } from './workloads.mjs';

const BINDGEN_VERSION = '0.2.104';

export function parseTarget(text) {
  const match = /^([A-Za-z0-9_.-]+)=(tree|package|runtime):(.+)$/.exec(text);
  if (!match) throw new Error(`--target=${text}: expected LABEL=tree:DIR, LABEL=package:DIR or LABEL=runtime:DIR`);
  return { label: match[1], kind: match[2], directory: path.resolve(match[3]) };
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, ...options });
  if (result.error) throw result.error;
  return result;
}

function output(command, args, cwd) {
  const result = spawnSync(command, args, { cwd, encoding: 'utf8' });
  return result.status === 0 ? result.stdout.trim() : null;
}

export function gitInfo(directory) {
  if (!existsSync(path.join(directory, '.git'))) return null;
  return {
    revision: output('git', ['rev-parse', 'HEAD'], directory),
    describe: output('git', ['describe', '--tags', '--always', '--dirty'], directory),
    branch: output('git', ['branch', '--show-current'], directory) || null,
    // Tracked files only: harness output under runtime/target is ignored.
    trackedClean: output('git', ['status', '--porcelain', '--untracked-files=no'], directory) === '',
    runtimeTreeSha: output('git', ['rev-parse', 'HEAD:runtime'], directory),
    kitLibTreeSha: output('git', ['rev-parse', 'HEAD:kit/lib'], directory),
  };
}

export async function fileHashes(directory, names) {
  const hashes = {};
  for (const name of names) {
    const file = path.join(directory, name);
    hashes[name] = existsSync(file) ? { sha256: sha256(await readFile(file)), bytes: (await readFile(file)).length } : null;
  }
  return hashes;
}

// The reactive-only WebAssembly exactly as scripts/build-web.mjs builds it:
// same cargo arguments, target directory and --remap-path-prefix flags, so
// the bytes match a `npm run build` of the same tree on this machine.
function remappedEnvironment(tree) {
  const cargoHome = process.env.CARGO_HOME || path.join(homedir(), '.cargo');
  const flags = (process.env.RUSTFLAGS || '').split(/\s+/).filter(Boolean);
  flags.push(`--remap-path-prefix=${cargoHome}=/cargo`, `--remap-path-prefix=${tree.replace(/[\\/]+$/, '')}=/caveat`);
  return { ...process.env, CARGO_ENCODED_RUSTFLAGS: flags.join('\x1f') };
}

export function runtimeOutputDirectory(tree) {
  return path.join(tree, 'runtime', 'target', 'perf-baseline', 'pkg-reactive');
}

export async function buildWasm(tree, log) {
  const bindgen = output('wasm-bindgen', ['--version'], tree);
  if (bindgen !== `wasm-bindgen ${BINDGEN_VERSION}`) throw new Error(`need wasm-bindgen ${BINDGEN_VERSION}; found ${bindgen}`);
  const targetDirectory = path.join(tree, 'runtime', 'target', 'reactive');
  log(`building reactive WebAssembly in ${tree}`);
  const cargo = run('cargo', ['build', '--locked', '--manifest-path', 'runtime/Cargo.toml', '--lib', '--target', 'wasm32-unknown-unknown',
    '--release', '--no-default-features', '--target-dir', targetDirectory], { cwd: tree, env: remappedEnvironment(tree), stdio: ['ignore', 'pipe', 'pipe'] });
  if (cargo.status !== 0) throw new Error(`WebAssembly build failed in ${tree}:\n${cargo.stderr}`);
  const out = runtimeOutputDirectory(tree);
  await mkdir(out, { recursive: true });
  const wasm = path.join(targetDirectory, 'wasm32-unknown-unknown', 'release', 'caveat_runtime.wasm');
  const generated = run('wasm-bindgen', [wasm, '--out-dir', out, '--target', 'web'], { cwd: tree });
  if (generated.status !== 0) throw new Error(`wasm-bindgen failed:\n${generated.stderr}`);
  // The kit's loader reads build-info.json beside the runtime, as it does in
  // a packed kit; the fields match scripts/build-web.mjs's.
  const rustc = output('rustc', ['-vV'], tree) ?? '';
  const field = (key) => rustc.split('\n').find((line) => line.startsWith(`${key}: `))?.slice(key.length + 2) ?? null;
  const git = gitInfo(tree);
  await writeFile(path.join(out, 'build-info.json'), `${JSON.stringify({
    revision: git?.revision ?? null,
    clean: git?.trackedClean ?? null,
    compiled: true,
    rustc: rustc.split('\n')[0] || null,
    host: field('host'),
    wasmBindgen: BINDGEN_VERSION,
    builtBy: 'experiments/performance-0.1 (same cargo arguments and remapping as scripts/build-web.mjs, reactive-only)',
  }, null, 2)}\n`);
  return out;
}

// A throwaway crate under TREE/runtime/target that depends on TREE/runtime by
// path and compiles native/perf_baseline.rs from this harness with the
// runtime's own [profile.release]. It starts from the runtime's Cargo.lock
// and builds offline, so no crate version can change; the check below
// proves every locked package is still locked at the same version.
export async function buildNative(tree, log) {
  const crate = path.join(tree, 'runtime', 'target', 'perf-baseline', 'native');
  await mkdir(crate, { recursive: true });
  const runtimeManifest = await readFile(path.join(tree, 'runtime', 'Cargo.toml'), 'utf8');
  const profile = /^\[profile\.release\][^[]*/m.exec(runtimeManifest)?.[0].trim() ?? '';
  const toml = (value) => `'${value.replace(/\\/g, '/')}'`;
  const manifest = [
    '# Generated by experiments/performance-0.1/lib/targets.mjs. Not part of any tree.',
    '[package]',
    'name = "caveat-perf-baseline"',
    'version = "0.0.0"',
    'edition = "2021"',
    'publish = false',
    '',
    '[[bin]]',
    'name = "perf_baseline"',
    `path = ${toml(path.join(harnessDirectory, 'native', 'perf_baseline.rs'))}`,
    '',
    '[dependencies]',
    `caveat-runtime = { path = ${toml(path.join(tree, 'runtime'))}, default-features = false }`,
    'serde_json = "1"',
    '',
    '[workspace]',
    '',
    '# Copied from the runtime\'s Cargo.toml: the build the WebAssembly uses.',
    profile,
    '',
  ].join('\n');
  await writeFile(path.join(crate, 'Cargo.toml'), manifest);
  await copyFile(path.join(tree, 'runtime', 'Cargo.lock'), path.join(crate, 'Cargo.lock'));
  log(`building native benchmark against ${path.join(tree, 'runtime')}`);
  const built = run('cargo', ['build', '--release', '--offline', '--manifest-path', path.join(crate, 'Cargo.toml'), '--target-dir', path.join(crate, 'target')],
    { cwd: crate, stdio: ['ignore', 'pipe', 'pipe'] });
  if (built.status !== 0) throw new Error(`native benchmark build failed:\n${built.stderr}`);
  const locked = await lockedVersions(path.join(tree, 'runtime', 'Cargo.lock'));
  const used = await lockedVersions(path.join(crate, 'Cargo.lock'));
  const changed = [...locked].filter((entry) => !used.has(entry));
  if (changed.length) throw new Error(`the benchmark crate changed locked packages: ${changed.join(', ')}`);
  const binary = path.join(crate, 'target', 'release', process.platform === 'win32' ? 'perf_baseline.exe' : 'perf_baseline');
  return {
    binary,
    crate,
    profile,
    features: 'caveat-runtime default-features = false (reactive-only, as the WebAssembly)',
    rustc: output('rustc', ['-vV'], crate),
    lockedPackagesUnchanged: locked.size,
  };
}

// Every locked package as "name version checksum".
async function lockedVersions(file) {
  const text = await readFile(file, 'utf8');
  const packages = new Set();
  for (const block of text.split('[[package]]').slice(1)) {
    const field = (key) => new RegExp(`^${key} = "([^"]+)"`, 'm').exec(block)?.[1] ?? '';
    packages.add(`${field('name')} ${field('version')} ${field('checksum')}`.trim());
  }
  return packages;
}

// Resolves a target to the files each engine needs, building what a tree
// target needs unless build is 'never' (then existing output is reused).
export async function prepareTarget(target, { build = 'auto', engines, log }) {
  const prepared = { ...target, git: null, runtimeDir: null, kitLib: null, native: null, adapterFile: null, build: {} };
  if (target.kind === 'tree') {
    prepared.git = gitInfo(target.directory);
    if (engines.includes('wasm')) {
      const out = runtimeOutputDirectory(target.directory);
      const present = existsSync(path.join(out, 'caveat_runtime_bg.wasm'));
      if (build === 'always' || (build === 'auto' && !present)) await buildWasm(target.directory, log);
      else if (!present) throw new Error(`${target.label}: no WebAssembly in ${out}; run without --build=never`);
      else log(`${target.label}: reusing the WebAssembly in ${out}`);
      prepared.runtimeDir = out;
      prepared.build.wasm = build === 'never' || (build === 'auto' && present) ? 'reused' : 'built';
    }
    prepared.kitLib = path.join(target.directory, 'kit', 'lib');
    prepared.adapterFile = path.join(target.directory, 'experiments', 'glowcap', 'caveat5', 'adapter.mjs');
    if (engines.includes('native')) {
      const binary = path.join(target.directory, 'runtime', 'target', 'perf-baseline', 'native', 'target', 'release', process.platform === 'win32' ? 'perf_baseline.exe' : 'perf_baseline');
      // The benchmark source lives in this harness, so an existing binary is
      // reused only on request.
      if (build === 'never' && existsSync(binary)) prepared.native = { binary, reused: true };
      else prepared.native = await buildNative(target.directory, log);
    }
  } else if (target.kind === 'package') {
    prepared.runtimeDir = path.join(target.directory, 'runtime');
    prepared.kitLib = path.join(target.directory, 'lib');
    const info = path.join(prepared.runtimeDir, 'build-info.json');
    prepared.packageInfo = {
      name: JSON.parse(await readFile(path.join(target.directory, 'package.json'), 'utf8')).name,
      version: JSON.parse(await readFile(path.join(target.directory, 'package.json'), 'utf8')).version,
      buildInfo: existsSync(info) ? JSON.parse(await readFile(info, 'utf8')) : null,
    };
  } else {
    prepared.runtimeDir = target.directory;
  }
  if (prepared.runtimeDir) {
    prepared.runtimeFiles = await fileHashes(prepared.runtimeDir, ['caveat_runtime_bg.wasm', 'caveat_runtime.js']);
    if (!prepared.runtimeFiles['caveat_runtime_bg.wasm']) throw new Error(`${target.label}: no caveat_runtime_bg.wasm in ${prepared.runtimeDir}`);
  }
  if (prepared.kitLib) prepared.kitFiles = await fileHashes(prepared.kitLib, ['session.mjs', 'node.mjs']);
  if (prepared.adapterFile && !existsSync(prepared.adapterFile)) prepared.adapterFile = null;
  return prepared;
}
