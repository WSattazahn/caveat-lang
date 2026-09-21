import { cp, mkdir, readdir, readFile, rm, writeFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = fileURLToPath(new URL('../', import.meta.url));
const dist = path.join(root, 'dist');
const wasm = path.join(root, 'runtime/target/wasm32-unknown-unknown/release/caveat_runtime.wasm');
const bindgenVersion = '0.2.104';

function tool(name) {
  const executable = process.platform === 'win32' ? `${name}.exe` : name;
  for (const candidate of [executable, path.join(homedir(), '.cargo/bin', executable)]) {
    const result = spawnSync(candidate, ['--version'], { encoding: 'utf8' });
    if (result.status === 0) return { command: candidate, version: result.stdout.trim() };
  }
  const install = name === 'wasm-bindgen'
    ? `cargo install wasm-bindgen-cli --version ${bindgenVersion} --locked`
    : 'Install stable Rust, then run rustup target add wasm32-unknown-unknown';
  throw new Error(`${name} was not found. ${install}`);
}

function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, stdio: 'inherit' });
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

try {
  const bindgen = tool('wasm-bindgen');
  if (bindgen.version !== `wasm-bindgen ${bindgenVersion}`) {
    throw new Error(`Expected wasm-bindgen ${bindgenVersion}; found ${bindgen.version}. Run cargo install wasm-bindgen-cli --version ${bindgenVersion} --locked --force`);
  }
  if (!process.argv.includes('--assemble-only')) {
    run(tool('cargo').command, ['build', '--locked', '--manifest-path', 'runtime/Cargo.toml', '--lib', '--target', 'wasm32-unknown-unknown', '--release']);
  }

  await rm(dist, { recursive: true, force: true });
  await mkdir(path.join(dist, 'pkg'), { recursive: true });
  run(bindgen.command, [wasm, '--out-dir', path.join(dist, 'pkg'), '--target', 'web']);
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
  await writeFile(path.join(dist, '.nojekyll'), '');
  console.log('Built dist/: CAVEAT WebAssembly, game sources, browser assets, and local Three.js.');
} catch (error) {
  console.error(`Build failed: ${error.message}`);
  process.exitCode = 1;
}
