import { existsSync } from 'node:fs';
import { cp, mkdir, readdir, readFile, rm, writeFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = fileURLToPath(new URL('../', import.meta.url));
const dist = path.join(root, 'dist');
const wasm = path.join(root, 'runtime/target/wasm32-unknown-unknown/release/caveat_runtime.wasm');
// The reactive-only runtime has its own target directory, so switching
// features does not invalidate the full build's cache.
const reactiveTarget = path.join(root, 'runtime/target/reactive');
const reactiveWasm = path.join(reactiveTarget, 'wasm32-unknown-unknown/release/caveat_runtime.wasm');
const bindgenVersion = '0.2.104';

function tool(name) {
  const executable = process.platform === 'win32' ? `${name}.exe` : name;
  for (const candidate of [executable, path.join(homedir(), '.cargo/bin', executable)]) {
    const result = spawnSync(candidate, ['--version'], { cwd: root, encoding: 'utf8' });
    if (result.status === 0) return { command: candidate, version: result.stdout.trim() };
  }
  const install = name === 'wasm-bindgen'
    ? `cargo install wasm-bindgen-cli --version ${bindgenVersion} --locked`
    : 'Install rustup; rust-toolchain.toml then installs the pinned compiler and wasm target on first use';
  throw new Error(`${name} was not found. ${install}`);
}

function run(command, args, env = process.env) {
  const result = spawnSync(command, args, { cwd: root, stdio: 'inherit', env });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} exited with status ${result.status}`);
}

// Resolve a program's imports with the runtime's own linker.
function linkBundle(file) {
  const result = spawnSync(
    'cargo',
    ['run', '--quiet', '--manifest-path', 'runtime/Cargo.toml', '--bin', 'caveat', '--', '--link', file],
    { cwd: root, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`cannot link ${path.basename(file)}: ${(result.stderr || '').trim()}`);
  }
  return result.stdout;
}

// Whether a .cav file is a draft 0.5 module, a program that imports modules,
// or an ordinary single-file program. Reads only the statement heads, so it
// does not reimplement the linker.
async function caveatFileKind(file) {
  const source = await readFile(file, 'utf8');
  const statements = source
    .replace(/"(?:[^"\\]|\\.)*"/g, '""')
    .replace(/(#|\/\/)[^\n]*/g, '')
    .split(';');
  for (const statement of statements) {
    const words = statement.trim().split(/\s+/);
    if (words[0] === 'module' && words.length === 2) return 'module';
    if (words[0] === 'use' && words.length === 2) return 'imports';
  }
  return 'program';
}

function output(command, args) {
  const result = spawnSync(command, args, { cwd: root, encoding: 'utf8' });
  return result.status === 0 ? result.stdout.trim() : null;
}

// A release build keeps file paths for panic locations, so left alone the WASM
// records its builder: C:\Users\<name>\.cargo\registry\... on one machine,
// /home/runner/.cargo/registry/... on CI. Remapping the cargo home and the
// checkout makes the bytes independent of who built them and where. Windows
// still writes `\` inside the rest of each path, so only a Linux build can be
// reproduced elsewhere; build-info.json says which kind this is.
function remappedRustflags() {
  const cargoHome = process.env.CARGO_HOME || path.join(homedir(), '.cargo');
  const flags = (process.env.RUSTFLAGS || '').split(/\s+/).filter(Boolean);
  flags.push(`--remap-path-prefix=${cargoHome}=/cargo`, `--remap-path-prefix=${root.replace(/[\\/]+$/, '')}=/caveat`);
  // The encoded form keeps a path with spaces as one argument.
  return { ...process.env, CARGO_ENCODED_RUSTFLAGS: flags.join('\x1f') };
}

// What produced dist/pkg: the revision, whether tracked files matched it, and
// the compiler and host. `compiled` is false for --assemble-only, which reuses
// whatever runtime/target holds and so cannot vouch for how it was built.
function buildInfo(compiled) {
  const rustc = output('rustc', ['-vV']) ?? '';
  const field = key => rustc.split('\n').find(line => line.startsWith(`${key}: `))?.slice(key.length + 2) ?? null;
  return {
    revision: output('git', ['rev-parse', 'HEAD']),
    clean: output('git', ['status', '--porcelain', '--untracked-files=no']) === '',
    compiled,
    rustc: rustc.split('\n')[0] || null,
    host: field('host'),
    wasmBindgen: bindgenVersion,
  };
}

try {
  const bindgen = tool('wasm-bindgen');
  if (bindgen.version !== `wasm-bindgen ${bindgenVersion}`) {
    throw new Error(`Expected wasm-bindgen ${bindgenVersion}; found ${bindgen.version}. Run cargo install wasm-bindgen-cli --version ${bindgenVersion} --locked --force`);
  }
  const compile = !process.argv.includes('--assemble-only');
  if (compile) {
    run(tool('cargo').command, ['build', '--locked', '--manifest-path', 'runtime/Cargo.toml', '--lib', '--target', 'wasm32-unknown-unknown', '--release'], remappedRustflags());
  }

  await rm(dist, { recursive: true, force: true });
  await mkdir(path.join(dist, 'pkg'), { recursive: true });
  run(bindgen.command, [wasm, '--out-dir', path.join(dist, 'pkg'), '--target', 'web']);
  // For hosts that run only reactive programs: no sequential, graphics or 3D
  // sessions. See spec/caveat-view-0.1.md. --assemble-only reuses the last
  // lean build if there is one, as it reuses the full one.
  if (compile) {
    run(tool('cargo').command, ['build', '--locked', '--manifest-path', 'runtime/Cargo.toml', '--lib', '--target', 'wasm32-unknown-unknown', '--release', '--no-default-features', '--target-dir', reactiveTarget], remappedRustflags());
  }
  if (compile || existsSync(reactiveWasm)) {
    run(bindgen.command, [reactiveWasm, '--out-dir', path.join(dist, 'pkg-reactive'), '--target', 'web']);
  }
  await writeFile(path.join(dist, 'build-info.json'), `${JSON.stringify(buildInfo(compile), null, 2)}\n`);
  await cp(path.join(root, 'web'), dist, { recursive: true });
  for (const entry of await readdir(path.join(root, 'game'), { withFileTypes: true })) {
    if (!entry.isFile() || !entry.name.endsWith('.cav')) continue;
    const source = path.join(root, 'game', entry.name);
    const kind = await caveatFileKind(source);
    // A draft 0.5 module is a library, not a playable program. Copying one
    // into dist/ would publish something the runtime refuses to start.
    if (kind === 'module') continue;
    if (kind === 'imports') {
      // Linked by the runtime's own linker rather than a second
      // implementation here, so dist/ can never disagree with what the
      // sessions do. The bundle keeps its .cav name: a single-file program and
      // a bundle are both just program text to the runtime.
      await writeFile(path.join(dist, entry.name), linkBundle(source));
      continue;
    }
    await cp(source, path.join(dist, entry.name));
  }
  await cp(path.join(root, 'examples/slime_glow_ability.cav'), path.join(dist, 'slime_glow_ability.cav'));
  await mkdir(path.join(dist, 'vendor'), { recursive: true });
  for (const name of ['three.module.js', 'three.core.js']) {
    await cp(path.join(root, 'node_modules/three/build', name), path.join(dist, 'vendor', name));
  }
  await cp(path.join(root, 'node_modules/three/LICENSE'), path.join(dist, 'vendor/THREE-LICENSE.txt'));
  // The site ships the runtime, so it carries Caveat's license and the notices
  // of the crates compiled into it.
  for (const name of ['LICENSE', 'THIRD_PARTY_NOTICES.md']) await cp(path.join(root, name), path.join(dist, name));
  // Pages load the runtime through the kit's session library: typed outcomes,
  // and no call into an instance after it traps. It is copied as .js, which
  // every server sends as JavaScript.
  await cp(path.join(root, 'kit/lib/session.mjs'), path.join(dist, 'caveat-session.js'));
  await writeFile(path.join(dist, '.nojekyll'), '');
  console.log('Built dist/: CAVEAT WebAssembly, game sources, browser assets, and local Three.js.');
} catch (error) {
  console.error(`Build failed: ${error.message}`);
  process.exitCode = 1;
}
