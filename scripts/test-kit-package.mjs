// Packs the developer kit with the reactive runtime inside it, installs the
// tarball into a fresh consumer directory, and uses it only through the
// installed package: the caveat command, the library by package name, and the
// session library in a browser. Run after `npm run build`.
//
//   node scripts/test-kit-package.mjs            PLAYWRIGHT_CHANNEL=chrome uses an installed Chrome
//   node scripts/test-kit-package.mjs --no-browser
//
// In CI the runtime comes from the Linux build, and the job keeps the tested
// tarball with this report, build-info.json and SHA256SUMS. Publishing it is
// a separate, authorized step (docs/CONSOLIDATION_PLAN.md).
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync } from 'node:fs';
import { copyFile, mkdir, readFile, readdir, rmdir, unlink, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { CLOCK_SOURCE, assertBrowserResults, checkKitInBrowser } from './test-kit-browser.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const kit = path.join(root, 'kit');
const bundled = path.join(kit, 'runtime');
const dist = path.join(root, 'dist');
const run = path.join(root, 'test-results', 'kit-package', new Date().toISOString().replace(/[:.]/g, '-'));
const consumer = path.join(run, 'consumer');
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const RUNTIME_FILES = ['caveat_runtime.js', 'caveat_runtime_bg.wasm'];
const manifest = JSON.parse(await readFile(path.join(kit, 'package.json'), 'utf8'));
// Caveat's license and the notices of the crates compiled into the runtime.
const LEGAL_FILES = ['LICENSE', 'THIRD_PARTY_NOTICES.md'];

function npm(args, cwd) {
  // npm is a .cmd on Windows, which Node only starts through a shell, so the
  // command is one quoted string there. Arguments here never contain quotes.
  const result = process.platform === 'win32'
    ? spawnSync(`npm ${args.map(arg => `"${arg}"`).join(' ')}`, { cwd, encoding: 'utf8', shell: true })
    : spawnSync('npm', args, { cwd, encoding: 'utf8' });
  if (result.status !== 0) throw new Error(`npm ${args.join(' ')} failed:\n${result.stdout}\n${result.stderr}`);
  return result.stdout;
}

function node(args, cwd) {
  return spawnSync(process.execPath, args, { cwd, encoding: 'utf8' });
}

// The bundled runtime and the legal files exist in kit/ only while packing, so
// development never runs a stale runtime and the repository keeps one copy of
// each. They are regular files, removed one by one.
async function stageRuntime() {
  for (const target of [bundled, ...LEGAL_FILES.map(file => path.join(kit, file))]) {
    if (existsSync(target)) throw new Error(`${target} already exists; remove it before packing`);
  }
  await mkdir(bundled);
  for (const file of RUNTIME_FILES) await copyFile(path.join(dist, 'pkg-reactive', file), path.join(bundled, file));
  await copyFile(path.join(dist, 'build-info.json'), path.join(bundled, 'build-info.json'));
  for (const file of LEGAL_FILES) await copyFile(path.join(root, file), path.join(kit, file));
}
async function removeRuntime() {
  for (const file of LEGAL_FILES) if (existsSync(path.join(kit, file))) await unlink(path.join(kit, file));
  if (!existsSync(bundled)) return;
  for (const file of await readdir(bundled)) await unlink(path.join(bundled, file));
  await rmdir(bundled);
}

const report = { schema: 1, checks: {} };
await mkdir(consumer, { recursive: true });
await stageRuntime();
let tarball;
try {
  const dry = JSON.parse(npm(['pack', '--dry-run', '--json'], kit))[0];
  const files = dry.files.map(file => file.path).sort();
  assert.deepEqual(files, [
    'LICENSE', 'README.md', 'THIRD_PARTY_NOTICES.md', 'bin/caveat.mjs', 'lib/node.mjs', 'lib/scenarios.mjs', 'lib/session.mjs', 'package.json',
    'runtime/build-info.json', 'runtime/caveat_runtime.js', 'runtime/caveat_runtime_bg.wasm',
  ], 'the tarball holds exactly the library, command, runtime, README, license and notices');
  const packed = JSON.parse(npm(['pack', '--json', '--pack-destination', run], kit))[0];
  assert.equal(packed.filename, `${manifest.name}-${manifest.version}.tgz`);
  tarball = path.join(run, packed.filename);
  const bytes = await readFile(tarball);
  Object.assign(report, {
    name: manifest.name, version: manifest.version, distTag: manifest.publishConfig?.tag ?? null,
    private: manifest.private === true,
    tarball: packed.filename, size: bytes.length, sha256: sha256(bytes), files,
  });
  // The build that produced the packed runtime, kept beside the tarball.
  await copyFile(path.join(dist, 'build-info.json'), path.join(run, 'build-info.json'));
} finally {
  await removeRuntime();
}

// A fresh consumer: nothing from the repository but the tarball and two
// example files. The kit has no dependencies, so the install is offline.
await writeFile(path.join(consumer, 'package.json'), `${JSON.stringify({ name: 'kit-consumer', private: true, type: 'module' }, null, 2)}\n`);
npm(['install', tarball, '--offline', '--ignore-scripts', '--no-audit', '--no-fund'], consumer);
for (const file of ['thermostat_history.cav', 'thermostat_history.scenarios.json']) {
  await copyFile(path.join(root, 'examples', file), path.join(consumer, file));
}
const installed = path.join(consumer, 'node_modules', manifest.name);
const installedManifest = JSON.parse(await readFile(path.join(installed, 'package.json'), 'utf8'));
for (const key of ['name', 'version', 'private', 'publishConfig', 'bin', 'exports']) {
  assert.deepEqual(installedManifest[key], manifest[key], `installed package preserves ${key}`);
}
for (const file of RUNTIME_FILES) {
  assert.equal(sha256(await readFile(path.join(installed, 'runtime', file))), sha256(await readFile(path.join(dist, 'pkg-reactive', file))), `installed ${file} matches the build`);
}
assert.equal(await readFile(path.join(installed, 'runtime', 'build-info.json'), 'utf8'),
  await readFile(path.join(run, 'build-info.json'), 'utf8'), 'retained build metadata matches the installed runtime');
assert.ok(['caveat', 'caveat.cmd'].some(name => existsSync(path.join(consumer, 'node_modules', '.bin', name))), 'npm linked the caveat command');
report.checks.install = true;

// The license ships unchanged, and the metadata says what it is.
for (const file of LEGAL_FILES) {
  assert.equal(await readFile(path.join(installed, file), 'utf8'), await readFile(path.join(root, file), 'utf8'), `installed ${file} matches the repository`);
}
assert.match(await readFile(path.join(installed, 'LICENSE'), 'utf8'), /^MIT License\n/);
assert.equal(installedManifest.license, 'MIT');
report.checks.license = 'MIT';

// The command, by its installed path.
const cli = path.join(installed, 'bin', 'caveat.mjs');
const passing = node([cli, 'test', 'thermostat_history.scenarios.json'], consumer);
assert.equal(passing.status, 0, passing.stdout + passing.stderr);
const doc = JSON.parse(await readFile(path.join(consumer, 'thermostat_history.scenarios.json'), 'utf8'));
doc.scenarios[0].steps[4].expect['/bindings/heating/text'] = '50%';
await writeFile(path.join(consumer, 'altered.scenarios.json'), JSON.stringify(doc));
const failing = node([cli, 'test', 'altered.scenarios.json'], consumer);
assert.equal(failing.status, 1);
assert.match(failing.stdout, /FAIL T01 step 5 expect \[primary\]: \/bindings\/heating\/text expected "50%", actual "0%"/);
await writeFile(path.join(consumer, 'invalid.scenarios.json'), JSON.stringify({ ...doc, extra: true }));
assert.equal(node([cli, 'test', 'invalid.scenarios.json'], consumer).status, 2);
const json = JSON.parse(node([cli, 'test', '--json', 'thermostat_history.scenarios.json'], consumer).stdout);
const buildInfo = JSON.parse(await readFile(path.join(dist, 'build-info.json'), 'utf8'));
assert.equal(json.runtime.revision, buildInfo.revision, 'the report names the bundled runtime');
report.runtime = json.runtime;
report.checks.command = true;

// The library, imported by package name from the consumer.
await writeFile(path.join(consumer, 'clock.cav'), CLOCK_SOURCE);
await writeFile(path.join(consumer, 'use.mjs'), `
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from '${manifest.name}/node';
import { CaveatError } from '${manifest.name}/session';
import { parseScenarioFile } from '${manifest.name}/scenarios';
const runtime = await loadRuntimeFromDirectory();
const source = await readFile('thermostat_history.cav', 'utf8');
const session = runtime.open(source);
assert.equal(session.dispatch('read', { value: 17 }).snapshot.bindings.heating.text, '100%');
assert.equal(session.view().bindings.heating.text, '100%');
const before = session.save();
assert.deepEqual([session.dispatch('read', { value: 'x' }).code, session.save() === before], ['payload_invalid', true]);
assert.deepEqual([session.dispatch('read', { value: 41 }).code, session.save() === before], ['bound_exceeded', true]);
assert.equal(session.dispatch('warm').code, 'unknown_event');
assert.throws(() => session.dispatch('read', { value: NaN }), error => error instanceof CaveatError && error.kind === 'payload');
const resumed = runtime.restore(source, session.save());
assert.deepEqual(resumed.snapshot(), session.snapshot());
assert.deepEqual(resumed.dispatch('read', { value: 25 }), session.dispatch('read', { value: 25 }));
const clockSource = await readFile('clock.cav', 'utf8');
const clock = runtime.open(clockSource);
clock.dispatch('advance', { dt: 2.5 });
assert.equal(runtime.restore(clockSource, clock.save()).view().bindings.hud.elapsed, 2.5);
parseScenarioFile(await readFile('thermostat_history.scenarios.json', 'utf8'));
console.log(JSON.stringify(runtime.identity));
`);
const used = node(['use.mjs'], consumer);
assert.equal(used.status, 0, used.stdout + used.stderr);
assert.equal(JSON.parse(used.stdout).revision, buildInfo.revision);
report.checks.library = true;

// The session library and runner in a browser, loaded from the installed package.
if (!process.argv.includes('--no-browser')) {
  const outcome = await checkKitInBrowser({ root: consumer, kit: `/node_modules/${manifest.name}/lib/`, runtime: `/node_modules/${manifest.name}/runtime/`, examples: '/' });
  assertBrowserResults(outcome);
  report.checks.browser = outcome.browser;
}

await writeFile(path.join(run, 'report.json'), `${JSON.stringify(report, null, 2)}\n`);
// These are the exact files CI retains. Write the checksum manifest only once
// every package check has passed; do not pack a second tarball for uploading.
const checksums = [];
for (const file of [report.tarball, 'report.json', 'build-info.json']) {
  const digest = sha256(await readFile(path.join(run, file)));
  if (file === report.tarball) assert.equal(digest, report.sha256, 'the tested tarball is unchanged');
  checksums.push(`${digest}  ${file}`);
}
await writeFile(path.join(run, 'SHA256SUMS'), `${checksums.join('\n')}\n`);
console.log(`Packed kit checks pass: ${report.tarball}, ${report.size} bytes, sha256 ${report.sha256}`);
console.log(`Installed from the tarball: command, library${report.checks.browser ? ` and browser (${report.checks.browser})` : ''}. Report: ${path.relative(root, path.join(run, 'report.json'))}`);
