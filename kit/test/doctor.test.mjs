import assert from 'node:assert/strict';
import { chmod, copyFile, mkdir, mkdtemp, readFile, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import test from 'node:test';
import { doctor, formatDoctor, inspectExecutables } from '../lib/doctor.mjs';
import { defaultRuntimeDirectory } from '../lib/node.mjs';

const kit = fileURLToPath(new URL('../', import.meta.url));
const manifest = JSON.parse(await readFile(path.join(kit, 'package.json'), 'utf8'));
const status = (report, id) => report.checks.find(check => check.id === id)?.status;
async function inDirectory(body) {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-doctor-'));
  try { return await body(directory); } finally { await rm(directory, { recursive: true, force: true }); }
}
async function runtimeFixture(directory) {
  const runtime = path.join(directory, 'runtime');
  await mkdir(runtime);
  await writeFile(path.join(directory, 'package.json'), JSON.stringify({ type: 'module' }));
  for (const name of ['caveat_runtime.js', 'caveat_runtime_bg.wasm']) {
    await copyFile(path.join(defaultRuntimeDirectory(), name), path.join(runtime, name));
  }
  return runtime;
}

// The session is real: neutral observation, exact grounds, rejection and restore.
test('doctor measures the installed files and tests a real session; text reports every structured check', async () => {
  const report = await doctor({ env: {} });
  assert.equal(report.schema, 'caveat-doctor/0.1');
  assert.equal(report.ok, true, JSON.stringify(report.checks));
  assert.equal(report.package.name, manifest.name);
  assert.equal(report.package.version, manifest.version);
  assert.deepEqual(report.runtime.session, { accepted: true, rejectedWithoutChange: true, exactGrounds: true, restored: true });
  assert.equal(report.runtime.identity.reactiveWasmSha256, report.runtime.files.find(file => file.name.endsWith('.wasm')).sha256);
  for (const file of report.runtime.files) {
    const bytes = await readFile(path.join(report.runtime.directory, file.name));
    assert.equal(file.bytes, bytes.length);
    assert.equal(file.sha256, createHash('sha256').update(bytes).digest('hex'));
  }
  const text = formatDoctor(report);
  for (const check of report.checks) assert.ok(text.includes(`[${check.status.toUpperCase()}] ${check.id}: ${check.message}`));
  assert.match(text, /shell aliases, functions and caches are not inspected/);
  assert.equal(status(report, 'executable.caveat-lang'), 'warn');
});

test('unsupported or malformed Node versions fail and skip loading the runtime', async () => {
  for (const nodeVersion of ['18.20.0', 'not-a-version']) {
    const report = await doctor({ nodeVersion, env: {} });
    assert.equal(report.ok, false);
    assert.equal(status(report, 'node'), 'fail');
    assert.equal(report.runtime.identity, null);
    assert.equal(report.runtime.session, null);
    assert.match(formatDoctor(report), /checks failed/);
  }
});

test('missing and malformed package manifests fail with a structured diagnostic', async () => {
  await inDirectory(async directory => {
    for (const body of [null, '{', JSON.stringify({ ...manifest, name: 'another-product' })]) {
      if (body !== null) await writeFile(path.join(directory, 'package.json'), body);
      const report = await doctor({ packageDirectory: directory, env: {} });
      assert.equal(report.ok, false);
      assert.equal(status(report, 'package'), 'fail');
      assert.equal(report.package.name, null);
    }
  });
});

test('missing runtime files and corrupt WebAssembly fail before runtime execution', async () => {
  await inDirectory(async directory => {
    const absent = await doctor({ runtimeDirectory: directory, env: {} });
    assert.equal(absent.ok, false);
    assert.equal(status(absent, 'runtime.caveat_runtime.js'), 'fail');
    assert.equal(status(absent, 'runtime.caveat_runtime_bg.wasm'), 'fail');
    assert.equal(absent.runtime.session, null);
    const runtimeDirectory = await runtimeFixture(directory);
    await writeFile(path.join(runtimeDirectory, 'caveat_runtime_bg.wasm'), 'not wasm');
    const corrupt = await doctor({ runtimeDirectory, env: {} });
    assert.equal(corrupt.ok, false);
    assert.equal(status(corrupt, 'runtime.caveat_runtime_bg.wasm'), 'fail');
    assert.match(formatDoctor(corrupt), /invalid WebAssembly binary/);
    assert.equal(corrupt.runtime.identity, null);
  });
});

test('incompatible and malformed adapters fail clearly with otherwise valid WASM', async () => {
  for (const adapter of ['export default function () {}\nexport class WebReactiveSession {}', 'export this is invalid']) {
    await inDirectory(async directory => {
      const runtimeDirectory = await runtimeFixture(directory);
      await writeFile(path.join(runtimeDirectory, 'caveat_runtime.js'), adapter);
      const report = await doctor({ runtimeDirectory, env: {} });
      assert.equal(report.ok, false);
      assert.equal(status(report, 'runtime.caveat_runtime_bg.wasm'), 'pass');
      assert.equal(status(report, 'runtime.load'), 'fail');
      assert.equal(report.runtime.session, null);
    });
  }
});

test('missing build metadata warns, and malformed metadata fails without disguising a working session', async () => {
  await inDirectory(async directory => {
    const runtimeDirectory = await runtimeFixture(directory);
    const missing = await doctor({ runtimeDirectory, env: {} });
    assert.equal(missing.ok, true, JSON.stringify(missing.checks));
    assert.equal(status(missing, 'runtime.build'), 'warn');
    assert.equal(missing.runtime.buildInfo, null);
    for (const body of ['{', 'null', JSON.stringify({ revision: 'main', clean: true })]) {
      await writeFile(path.join(runtimeDirectory, 'build-info.json'), body);
      const malformed = await doctor({ runtimeDirectory, env: {} });
      assert.equal(malformed.ok, false);
      assert.equal(status(malformed, 'runtime.build'), 'fail');
      assert.equal(status(malformed, 'runtime.session'), 'pass');
    }
  });
});

async function executableFixture(directory, owner, name, relative = 'bin/cli.mjs') {
  const packageDirectory = path.join(directory, 'node_modules', owner);
  const target = path.join(packageDirectory, relative);
  const bin = path.join(directory, 'node_modules', '.bin');
  await mkdir(path.dirname(target), { recursive: true });
  await mkdir(bin, { recursive: true });
  await writeFile(path.join(packageDirectory, 'package.json'), JSON.stringify({ name: owner, version: manifest.version, bin: { [name]: relative } }));
  await writeFile(target, `throw new Error('doctor must never run an inspected executable');\n`);
  if (process.platform === 'win32') {
    await writeFile(path.join(bin, `${name}.cmd`), `@ECHO off\r\nendLocal & goto #_undefined_# 2>NUL || title %COMSPEC% & "%_prog%" "%dp0%\\..\\${owner}\\${relative.replaceAll('/', '\\')}" %*\r\n`);
  } else {
    await chmod(target, 0o755);
    await symlink(target, path.join(bin, name));
  }
  return { bin, target, packageDirectory };
}

test('PATH inspection identifies package bin targets, including another product, without executing them', async () => {
  await inDirectory(async directory => {
    const language = await executableFixture(directory, 'caveat-lang', 'caveat-lang');
    const competitor = await executableFixture(directory, 'caveat-cli', 'caveat');
    const env = { PATH: language.bin, PATHEXT: '.COM;.EXE;.BAT;.CMD' };
    const entries = await inspectExecutables({ env });
    assert.deepEqual(entries.map(entry => entry.owner.name), ['caveat-lang', 'caveat-cli']);
    assert.equal(entries[0].target, language.target);
    assert.equal(entries[1].target, competitor.target);
    const report = await doctor({ env });
    assert.equal(report.ok, true);
    assert.equal(status(report, 'executable.caveat-lang'), 'pass');
    assert.equal(status(report, 'executable.caveat'), 'warn');
    assert.match(formatDoctor(report), /belongs to caveat-cli/);
  });
});

test('an earlier PATH entry owning the unambiguous name fails, and later matches stay visible', async () => {
  await inDirectory(async directory => {
    const first = await executableFixture(path.join(directory, 'first'), 'another-product', 'caveat-lang');
    const second = await executableFixture(path.join(directory, 'second'), 'caveat-lang', 'caveat-lang');
    const env = { PATH: [first.bin, second.bin].join(path.delimiter), PATHEXT: '.CMD' };
    const report = await doctor({ env });
    assert.equal(report.ok, false);
    assert.equal(report.executables[0].owner.name, 'another-product');
    assert.equal(report.executables[0].candidates.length, 2);
    assert.equal(status(report, 'executable.caveat-lang'), 'fail');
    assert.match(formatDoctor(report), /other PATH candidates:/);
  });
});

test('a wrapper comment cannot establish ownership and unknown wrappers are never run', async () => {
  await inDirectory(async directory => {
    const { bin } = await executableFixture(directory, 'caveat-lang', 'caveat-lang');
    const file = path.join(bin, process.platform === 'win32' ? 'caveat-lang.cmd' : 'caveat-lang');
    await rm(file);
    await writeFile(file, process.platform === 'win32'
      ? 'REM "%_prog%" "%dp0%\\..\\caveat-lang\\bin\\cli.mjs" %*\r\nexit /b 99\r\n'
      : '#!/bin/sh\n# exec node "$basedir/../caveat-lang/bin/cli.mjs" "$@"\nexit 99\n');
    await chmod(file, 0o755);
    const [entry] = await inspectExecutables({ env: { PATH: bin, PATHEXT: '.CMD' } });
    assert.equal(entry.owner, null);
    assert.equal(entry.path, file);
  });
});


test('an empty PATH component searches cwd before later directories', async () => {
  await inDirectory(async directory => {
    const first = await executableFixture(path.join(directory, 'cwd'), 'another-product', 'caveat-lang');
    const second = await executableFixture(path.join(directory, 'later'), 'caveat-lang', 'caveat-lang');
    const [entry] = await inspectExecutables({
      env: { PATH: `${path.delimiter}${second.bin}`, PATHEXT: '.CMD' }, cwd: first.bin,
    });
    assert.equal(entry.owner.name, 'another-product');
    assert.equal(entry.candidates.length, 2);
    const [absent] = await inspectExecutables({ env: {}, cwd: first.bin });
    assert.equal(absent.path, null);
  });
});
