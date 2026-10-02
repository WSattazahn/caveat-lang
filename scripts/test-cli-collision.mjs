// Real, pinned co-installation regression for npm's ambiguous caveat shim.
// This fixture needs npm registry access (or a primed cache); npm ci verifies
// the lockfile's integrity hashes. Lifecycle scripts and the competing CLI
// are never executed. The fixture overrides only smol-toml to its patched
// 1.7.1 release; this verifies the real CLI package with a patched dependency
// tree, not the competitor's original dependency tree. All writes stay in the caller's fresh result directory.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { copyFile, mkdir, readFile, realpath, unlink } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const fixture = fileURLToPath(new URL('./fixtures/cli-collision/', import.meta.url));
const normalized = value => value.replaceAll('\\', '/');

function npm(args, cwd) {
  const result = process.platform === 'win32'
    ? spawnSync(`npm ${args.map(arg => `"${arg}"`).join(' ')}`, { cwd, encoding: 'utf8', shell: true })
    : spawnSync('npm', args, { cwd, encoding: 'utf8' });
  assert.equal(result.status, 0, `npm ${args.join(' ')}\n${result.stdout}${result.stderr}`);
  return result.stdout;
}

export function installedCommand(directory, name, args) {
  // Invoke the actual npm-created executable, not the package's .mjs file.
  const binary = path.join(directory, 'node_modules', '.bin', name);
  return process.platform === 'win32'
    ? spawnSync(`"${binary}.cmd" ${args.map(arg => `"${arg}"`).join(' ')}`, { cwd: directory, encoding: 'utf8', shell: true })
    : spawnSync(binary, args, { cwd: directory, encoding: 'utf8' });
}

async function assertShim(directory, name, owner, target) {
  const binary = path.join(directory, 'node_modules', '.bin', name);
  if (process.platform === 'win32') {
    const shim = normalized(await readFile(`${binary}.cmd`, 'utf8'));
    assert.ok(shim.includes(`../${owner}/${target}`), `${name}.cmd does not point to ${owner}/${target}`);
  } else {
    assert.equal(await realpath(binary), await realpath(path.join(directory, 'node_modules', owner, target)));
  }
}

export async function checkCliCollision({ tarball, directory, manifest, help, version }) {
  const fixtureManifest = JSON.parse(await readFile(path.join(fixture, 'package.json'), 'utf8'));
  const lock = JSON.parse(await readFile(path.join(fixture, 'package-lock.json'), 'utf8'));
  const pinned = lock.packages['node_modules/caveat-cli'];
  const cases = [];
  for (const owner of ['caveat-lang', 'caveat-cli']) {
    const consumer = path.join(directory, owner);
    await mkdir(consumer, { recursive: true });
    for (const file of ['package.json', 'package-lock.json']) await copyFile(path.join(fixture, file), path.join(consumer, file));
    npm(['ci', '--ignore-scripts', '--no-audit', '--no-fund', '--registry', 'https://registry.npmjs.org'], consumer);
    npm(['install', tarball, '--offline', '--ignore-scripts', '--no-audit', '--no-fund'], consumer);
    const installedLock = JSON.parse(await readFile(path.join(consumer, 'package-lock.json'), 'utf8'));
    for (const [name, entry] of Object.entries(lock.packages)) {
      if (!name) continue;
      assert.equal(installedLock.packages[name].version, entry.version, `${name} keeps its pinned version`);
      assert.equal(installedLock.packages[name].integrity, entry.integrity, `${name} keeps its pinned integrity`);
    }
    const competing = JSON.parse(await readFile(path.join(consumer, 'node_modules', 'caveat-cli', 'package.json'), 'utf8'));
    const language = JSON.parse(await readFile(path.join(consumer, 'node_modules', manifest.name, 'package.json'), 'utf8'));
    assert.equal(competing.version, pinned.version);
    assert.deepEqual(competing.bin, { caveat: 'dist/caveat.js' });
    assert.equal(language.version, manifest.version);
    // npm versions differ in which package wins on install. Rebuild the chosen
    // owner with scripts disabled to exercise both real npm shim ownerships.
    // Remove only these known shims in this new fixture directory, no tree.
    for (const suffix of ['', '.cmd', '.ps1']) {
      try { await unlink(path.join(consumer, 'node_modules', '.bin', `caveat${suffix}`)); }
      catch (error) { if (error.code !== 'ENOENT') throw error; }
    }
    npm(['rebuild', owner, '--ignore-scripts', '--offline', '--no-audit', '--no-fund'], consumer);
    const target = owner === manifest.name ? language.bin.caveat : competing.bin.caveat;
    await assertShim(consumer, 'caveat', owner, target);
    await assertShim(consumer, 'caveat-lang', manifest.name, language.bin['caveat-lang']);
    for (const [argument, expected] of [['help', help], ['--version', version]]) {
      const result = installedCommand(consumer, 'caveat-lang', [argument]);
      assert.equal(result.status, 0, result.stdout + result.stderr);
      assert.equal(result.stdout, expected, `${argument} still launches CAVEAT Language with ${owner} owning caveat`);
    }
    assert.equal(npm(['exec', '--no', '--', 'caveat-lang', 'help'], consumer), help);
    const health = JSON.parse(npm(['exec', '--no', '--', 'caveat-lang', 'doctor', '--json'], consumer));
    assert.equal(health.ok, true, JSON.stringify(health.checks));
    for (const [name, expected] of [['caveat', owner], ['caveat-lang', manifest.name]]) {
      assert.equal(health.executables.find(item => item.name === name).owner?.name, expected,
        `doctor identifies the actual ${name} PATH owner`);
    }
    cases.push({ doctor: true, caveatOwner: owner, caveatLangOwner: manifest.name, help: true, version: true, npmExec: true });
  }
  return { package: `caveat-cli@${pinned.version}`, integrity: pinned.integrity, dependencyOverrides: fixtureManifest.overrides ?? {}, lifecycleScripts: false, competingCliExecuted: false, cases };
}
