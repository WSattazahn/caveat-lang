// Security gate for one already-tested tarball. npm audit does not inspect the
// bundled WASM: the separate RustSec scan covers its build's complete lockfile.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { access, mkdir, readFile, realpath, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../', import.meta.url));
const REGISTRY = 'https://registry.npmjs.org';
const RUSTSEC = 'https://github.com/RustSec/advisory-db.git';
const AUDIT_VERSION = '0.22.2';
const sha256 = data => createHash('sha256').update(data).digest('hex');
const now = () => new Date().toISOString();
const json = bytes => JSON.parse(String(bytes).replace(/^\uFEFF/, ''));

export function validatePackageIdentity({ report, tarball, manifest, build, wasm }) {
  assert.equal(report.schema, 1, 'unsupported package report schema');
  assert.equal(report.name, 'caveat-lang', 'report must identify caveat-lang');
  assert.match(report.sha256 ?? '', /^[0-9a-f]{64}$/, 'report needs a tarball SHA256');
  assert.equal(sha256(tarball), report.sha256, 'tarball differs from the tested package');
  assert.equal(tarball.length, report.size, 'tarball size differs from the tested package');
  assert.equal(manifest.name, report.name, 'packed package name differs from report');
  assert.equal(manifest.version, report.version, 'packed package version differs from report');
  assert.equal(build.clean, true, 'release gate requires a clean source build');
  assert.equal(build.compiled, true, 'release gate requires compiled runtime evidence');
  assert.match(build.revision ?? '', /^[0-9a-f]{40}$/, 'build needs a complete Git revision');
  assert.equal(report.runtime?.revision, build.revision, 'reported and packed build revisions differ');
  assert.equal(report.runtime?.clean, true, 'package report must name a clean runtime');
  assert.equal(report.runtime?.compiled, true, 'package report must name a compiled runtime');
  assert.equal(report.runtime?.reactiveWasmSha256, sha256(wasm), 'packed WASM differs from tested runtime');
  for (const field of ['dependencies', 'optionalDependencies', 'peerDependencies', 'bundleDependencies', 'bundledDependencies']) {
    const value = manifest[field];
    assert.ok(value === undefined || value === false || (typeof value === 'object' && value !== null && Object.keys(value).length === 0),
      `packed kit has ${field}; review the dependency-free artifact contract before release`);
  }
  return { name: manifest.name, version: manifest.version, tarballSha256: report.sha256,
    wasmSha256: sha256(wasm), buildRevision: build.revision, npmRuntimeDependencies: [] };
}

export function evaluateNpmAudit(report, exitCode) {
  assert.ok(!report.error, `npm audit was unavailable: ${report.error?.code ?? ''} ${report.error?.summary ?? ''}`);
  assert.equal(report.auditReportVersion, 2, 'unsupported npm audit report');
  const counts = report.metadata?.vulnerabilities;
  assert.ok(counts && report.vulnerabilities && typeof report.vulnerabilities === 'object' && !Array.isArray(report.vulnerabilities), 'npm audit report lacks vulnerabilities');
  for (const severity of ['info', 'low', 'moderate', 'high', 'critical', 'total']) {
    assert.ok(Number.isInteger(counts[severity]) && counts[severity] >= 0, `invalid npm ${severity} count`);
  }
  const findings = Object.values(report.vulnerabilities);
  assert.equal(findings.length, counts.total, 'npm count disagrees with vulnerability records');
  assert.equal(['info', 'low', 'moderate', 'high', 'critical'].reduce((sum, severity) => sum + counts[severity], 0), counts.total, 'npm severity counts disagree');
  const severities = ['info', 'low', 'moderate', 'high', 'critical'];
  for (const finding of findings) assert.ok(severities.includes(finding.severity), 'npm finding has unknown severity');
  for (const severity of severities) assert.equal(findings.filter(finding => finding.severity === severity).length, counts[severity], `npm ${severity} records disagree`);
  const blocked = counts.high + counts.critical > 0;
  assert.equal(exitCode, blocked ? 1 : 0, 'npm audit exit disagrees with High/Critical threshold');
  return { passed: !blocked, counts, findings, policy: 'High/Critical block; lower-severity findings are retained for review' };
}

export function evaluateRustAudit(report, exitCode) {
  const settings = report.settings;
  assert.ok(settings && Array.isArray(settings.ignore) && Array.isArray(settings.target_arch) && Array.isArray(settings.target_os), 'RustSec report lacks audit settings');
  assert.deepEqual(settings.ignore, [], 'RustSec ignored advisories are forbidden');
  assert.equal(settings.severity, null, 'RustSec severity filtering is forbidden');
  assert.deepEqual(settings.target_arch, [], 'RustSec architecture filtering is forbidden');
  assert.deepEqual(settings.target_os, [], 'RustSec OS filtering is forbidden');
  assert.deepEqual([...settings.informational_warnings].sort(), ['notice', 'unmaintained', 'unsound'], 'RustSec informational warning filtering is forbidden');
  assert.ok(Number.isInteger(report.database?.['advisory-count']) && report.database['advisory-count'] > 0, 'RustSec advisory database was empty or unavailable');
  const vulnerabilities = report.vulnerabilities;
  assert.ok(vulnerabilities && Array.isArray(vulnerabilities.list), 'RustSec report lacks vulnerabilities');
  assert.equal(vulnerabilities.count, vulnerabilities.list.length, 'RustSec count disagrees with records');
  assert.equal(vulnerabilities.found, vulnerabilities.count > 0, 'RustSec found flag disagrees with count');
  assert.ok(report.warnings && typeof report.warnings === 'object' && !Array.isArray(report.warnings), 'RustSec report lacks warning records');
  // Informational records may contain unsoundness without a CVSS severity.
  // Keep the first gate conservative: any unresolved warning also blocks.
  const warnings = Object.values(report.warnings).flat();
  const blocked = vulnerabilities.count > 0 || warnings.length > 0;
  assert.ok(exitCode === 0 || exitCode === 1, 'RustSec did not complete an audit');
  if (vulnerabilities.count > 0) assert.equal(exitCode, 1, 'RustSec exit disagrees with vulnerability findings');
  if (!blocked) assert.equal(exitCode, 0, 'RustSec failed without reported findings');
  return { passed: !blocked, vulnerabilityCount: vulnerabilities.count, vulnerabilities: vulnerabilities.list, warnings,
    policy: 'Every vulnerability and unresolved informational warning blocks, including missing severity' };
}

export function parseSecurityArguments(args) {
  if (args.length === 1 && ['--help', '-h'].includes(args[0])) return { help: true };
  const result = {};
  const fields = { '--package-report': 'packageReport', '--audit-bin': 'auditBin', '--advisory-db': 'advisoryDb' };
  for (let index = 0; index < args.length; index += 2) {
    const field = fields[args[index]];
    if (!field || result[field] || !args[index + 1] || args[index + 1].startsWith('--')) throw new Error('Expected --package-report FILE [--audit-bin FILE] [--advisory-db DIRECTORY]');
    result[field] = path.resolve(args[index + 1]);
  }
  if (!result.packageReport) throw new Error('--package-report is required; no implicit latest artifact');
  return result;
}

async function npmCli() {
  const nodeDirectory = path.dirname(await realpath(process.execPath));
  const candidates = [process.env.npm_execpath,
    path.join(nodeDirectory, 'node_modules/npm/bin/npm-cli.js'),
    path.join(nodeDirectory, '../lib/node_modules/npm/bin/npm-cli.js'),
    '/usr/share/nodejs/npm/bin/npm-cli.js'].filter(Boolean);
  for (const candidate of candidates) {
    if (!candidate.endsWith('npm-cli.js')) continue;
    try { await access(candidate); return candidate; } catch { /* try the next known installation layout */ }
  }
  throw new Error('Cannot locate npm-cli.js; run this gate through npm run audit:kit-security');
}

export async function auditKitSecurity({ packageReport, auditBin, advisoryDb }) {
  packageReport = path.resolve(packageReport);
  const directory = path.join(ROOT, 'test-results', 'kit-security', `${now().replace(/[:.]/g, '-')}-${process.pid}`);
  await mkdir(directory, { recursive: true });
  const report = { schema: 'caveat-package-security/0.1', startedAt: now(), finishedAt: null, passed: false,
    artifact: null, tools: { node: process.version, platform: process.platform, architecture: process.arch },
    advisoryDatabase: null, scopes: {}, commands: [], failures: [], limitations: [
      'Known dependency advisory checks; not source-code auditing, malware detection or proof of absence of vulnerabilities.',
      'The Rust scan covers the complete build Cargo.lock, including build tools; linked crates are separately identified from Cargo metadata.',
      'RustSec yanked-crate checks are disabled; they are not vulnerability advisories and need a separate live registry check.',
      'npm returns no advisory database commit. Timestamps and hashed raw responses preserve this run, not a promise of identical future registry results.',
    ] };
  const write = (file, data) => writeFile(path.join(directory, file), data);
  let counter = 0;
  async function command(label, executable, args, { cwd = ROOT, allowFailure = false, timeout = 180000, env = process.env } = {}) {
    const id = `${String(++counter).padStart(2, '0')}-${label}`;
    const startedAt = now();
    const output = spawnSync(executable, args, { cwd, env, timeout, maxBuffer: 32 * 1024 * 1024, windowsHide: true });
    const stdout = output.stdout ?? Buffer.alloc(0);
    const stderr = output.stderr ?? Buffer.alloc(0);
    await write(`${id}.stdout`, stdout);
    await write(`${id}.stderr`, stderr);
    const receipt = { id, executable, args, cwd, startedAt, finishedAt: now(), exitCode: output.status,
      signal: output.signal, error: output.error?.message ?? null,
      stdout: { file: `${id}.stdout`, sha256: sha256(stdout) }, stderr: { file: `${id}.stderr`, sha256: sha256(stderr) } };
    report.commands.push(receipt);
    if (output.error || (!allowFailure && output.status !== 0)) throw new Error(`${label} failed (${output.status ?? output.error?.code}): ${String(stderr).slice(0, 800)}`);
    return { stdout, stderr, exitCode: output.status, receipt };
  }
  async function scope(name, body) {
    try { report.scopes[name] = await body(); }
    catch (error) { report.scopes[name] = { passed: false, error: error.message }; }
    if (!report.scopes[name].passed) report.failures.push(name);
  }
  try {
    const npm = await npmCli();
    const npmRun = (label, args, options) => command(label, process.execPath, [npm, ...args], options);
    report.tools.npm = String((await npmRun('npm-version', ['--version'])).stdout).trim();
    const packedReportBytes = await readFile(packageReport);
    const packedReport = json(packedReportBytes);
    await write('package-report.json', packedReportBytes);
    let artifact;
    await scope('artifact', async () => {
      assert.equal(typeof packedReport.tarball, 'string', 'report needs a tarball filename');
      assert.equal(path.basename(packedReport.tarball), packedReport.tarball, 'tarball must be beside its report');
      assert.match(packedReport.tarball, /^[A-Za-z0-9._-]+\.tgz$/, 'invalid tarball filename');
      const originalTarball = path.join(path.dirname(packageReport), packedReport.tarball);
      const tarball = await readFile(originalTarball);
      // Reject changed artifacts before asking npm to install anything.
      assert.equal(sha256(tarball), packedReport.sha256, 'tarball differs from the tested package');
      const tarballPath = path.join(directory, packedReport.tarball);
      await writeFile(tarballPath, tarball);
      const entries = String((await command('tar-list', 'tar', ['-tf', tarballPath])).stdout).trim().split(/\r?\n/);
      for (const entry of entries) assert.ok(entry.startsWith('package/') && !entry.split('/').includes('..') && !entry.includes('\\'), `unexpected archive path ${entry}`);
      const files = entries.filter(entry => !entry.endsWith('/')).map(entry => entry.slice('package/'.length)).sort();
      assert.equal(new Set(files).size, files.length, 'duplicate archive paths');
      assert.ok(!files.some(file => file.startsWith('node_modules/') || file.includes('/node_modules/')), 'unexpected bundled node_modules');
      assert.deepEqual(files, [...packedReport.files].sort(), 'actual tarball file inventory differs from tested report');
      const fromTar = async (name, label) => (await command(label, 'tar', ['-xOf', tarballPath, `package/${name}`])).stdout;
      const manifestBytes = await fromTar('package.json', 'packed-manifest');
      const buildBytes = await fromTar('runtime/build-info.json', 'packed-build');
      const wasm = await fromTar('runtime/caveat_runtime_bg.wasm', 'packed-wasm');
      const adapter = await fromTar('runtime/caveat_runtime.js', 'packed-adapter');
      const manifest = json(manifestBytes);
      const build = json(buildBytes);
      const identity = validatePackageIdentity({ report: packedReport, tarball, manifest, build, wasm });
      assert.equal(sha256(buildBytes), sha256(await readFile(path.join(path.dirname(packageReport), 'build-info.json'))), 'retained build-info differs from packed metadata');
      artifact = { tarballPath, manifest, build, wasm, adapter };
      report.artifact = { ...identity, packageReport: path.resolve(packageReport), packageReportSha256: sha256(packedReportBytes),
        tarball: tarballPath, originalTarball, manifestSha256: sha256(manifestBytes), adapterSha256: sha256(adapter), buildInfoSha256: sha256(buildBytes), buildInfo: build };
      return { passed: true, ...identity };
    });

    async function npmAudit(name, cwd, omitDev = false) {
      const inputFiles = [];
      for (const file of ['package.json', 'package-lock.json']) {
        const bytes = await readFile(path.join(cwd, file));
        await write(`${name}-${file}`, bytes);
        inputFiles.push({ file, sha256: sha256(bytes) });
      }
      const args = ['audit', '--json', '--audit-level=high', `--registry=${REGISTRY}`,
        ...(omitDev ? ['--omit=dev', '--include=prod', '--include=optional', '--include=peer'] : ['--include=dev', '--include=prod', '--include=optional', '--include=peer'])];
      const output = await npmRun(name, args, { cwd, allowFailure: true });
      const result = evaluateNpmAudit(json(output.stdout), output.exitCode);
      for (const input of inputFiles) assert.equal(sha256(await readFile(path.join(cwd, input.file))), input.sha256, `${name} ${input.file} changed during audit`);
      const manifest = json(await readFile(path.join(cwd, 'package.json')));
      return { ...result, registry: REGISTRY, inputFiles, dependencyOverrides: manifest.overrides ?? {}, responseSha256: sha256(output.stdout) };
    }

    if (artifact) await scope('packed-npm-runtime', async () => {
      const consumer = path.join(directory, 'consumer');
      await mkdir(consumer);
      await writeFile(path.join(consumer, 'package.json'), JSON.stringify({ name: 'caveat-security-consumer', version: '1.0.0', private: true }));
      await npmRun('install-packed', ['install', artifact.tarballPath, '--ignore-scripts', '--offline', '--no-audit', '--no-fund', '--omit=dev'], { cwd: consumer });
      const installed = path.join(consumer, 'node_modules', 'caveat-lang');
      assert.deepEqual(json(await readFile(path.join(installed, 'package.json'))), artifact.manifest, 'installed manifest differs from inspected tarball');
      assert.equal(sha256(await readFile(path.join(installed, 'runtime/caveat_runtime_bg.wasm'))), sha256(artifact.wasm), 'installed WASM differs from inspected tarball');
      assert.equal(sha256(await readFile(path.join(installed, 'runtime/caveat_runtime.js'))), sha256(artifact.adapter), 'installed adapter differs from inspected tarball');
      const lock = json(await readFile(path.join(consumer, 'package-lock.json')));
      assert.deepEqual(Object.keys(lock.packages).sort(), ['', 'node_modules/caveat-lang'], 'installed artifact unexpectedly resolves third-party npm packages');
      const result = await npmAudit('packed-npm-runtime', consumer, true);
      assert.equal(sha256(await readFile(artifact.tarballPath)), packedReport.sha256, 'retained tarball changed during audit');
      return result;
    });

    for (const [name, relative] of [['repository-development-and-site', '.'], ['editor-development', 'editors/vscode'],
      ['collision-fixture', 'scripts/fixtures/cli-collision'], ['mcp-client-fixture', 'scripts/fixtures/mcp-client']]) {
      await scope(name, () => npmAudit(name, path.join(ROOT, relative)));
    }

    if (artifact) await scope('rust-build-lockfile', async () => {
      const executable = path.resolve(auditBin ?? path.join(ROOT, 'test-results/security-tools/bin', process.platform === 'win32' ? 'cargo-audit.exe' : 'cargo-audit'));
      const version = String((await command('cargo-audit-version', executable, ['--version'])).stdout).trim();
      assert.equal(version, `cargo-audit ${AUDIT_VERSION}`, `install cargo-audit ${AUDIT_VERSION} with --locked into test-results/security-tools`);
      report.tools.cargoAudit = { version, executable, sha256: sha256(await readFile(executable)) };
      report.tools.cargo = String((await command('cargo-version', 'cargo', ['--version'])).stdout).trim();
      const db = await realpath(path.resolve(advisoryDb ?? path.join(ROOT, 'test-results/security-tools/advisory-db')));
      const git = async (label, args) => String((await command(label, 'git', ['-C', db, ...args])).stdout).trim();
      assert.equal(await realpath(await git('db-root', ['rev-parse', '--show-toplevel'])), db, 'advisory-db must be a Git repository root');
      assert.equal(await git('db-dirty', ['status', '--porcelain']), '', 'advisory database has local changes');
      const origin = await git('db-origin', ['remote', 'get-url', 'origin']);
      assert.equal(origin.toLowerCase().replace(/\.git$/, ''), RUSTSEC.toLowerCase().replace(/\.git$/, ''), 'advisory database origin is not RustSec');
      const fetched = await command('db-fetch', 'git', ['-C', db, 'fetch', '--depth=1', 'origin', 'HEAD']);
      const commit = await git('db-head', ['rev-parse', 'HEAD']);
      const remoteCommit = await git('db-current', ['rev-parse', 'FETCH_HEAD']);
      assert.equal(commit, remoteCommit, 'advisory database is behind its fetched HEAD; update this clean advisory clone and rerun');
      assert.match(commit, /^[0-9a-f]{40}$/);
      report.advisoryDatabase = { url: RUSTSEC, directory: db, commit, clean: true, fetchedAt: fetched.receipt.finishedAt,
        committedAt: await git('db-date', ['show', '-s', '--format=%cI', 'HEAD']) };
      const inputs = path.join(directory, 'rust-inputs');
      await mkdir(path.join(inputs, '.cargo'), { recursive: true });
      await mkdir(path.join(inputs, 'src'));
      // Project-local config takes precedence over the user's global audit.toml.
      const configuration = '[advisories]\nignore = []\n[output]\nquiet = false\n';
      await writeFile(path.join(inputs, '.cargo/audit.toml'), configuration);
      await writeFile(path.join(inputs, 'src/lib.rs'), '// Dependency inventory only; never compiled.\n');
      const inputFiles = [];
      for (const file of ['Cargo.toml', 'Cargo.lock']) {
        const bytes = (await command(`build-${file.replace('.', '-')}`, 'git', ['show', `${artifact.build.revision}:runtime/${file}`])).stdout;
        await writeFile(path.join(inputs, file), bytes);
        inputFiles.push({ file, sha256: sha256(bytes), buildRevision: artifact.build.revision });
      }
      // Fetch the exact recorded lock on fresh CI hosts before the offline tree.
      await command('fetch-build-crates', 'cargo', ['fetch', '--manifest-path', path.join(inputs, 'Cargo.toml'), '--locked', '--target', 'wasm32-unknown-unknown'], { cwd: inputs });
      // Resolves the recorded manifest/lock without changing either or compiling.
      const tree = await command('reactive-linked-tree', 'cargo', ['tree', '--manifest-path', path.join(inputs, 'Cargo.toml'), '--locked', '--offline',
        '--target', 'wasm32-unknown-unknown', '--no-default-features', '-e', 'normal,no-proc-macro', '--prefix', 'none', '--format', '{p}'], { cwd: inputs });
      const linkedCrates = [...new Set(String(tree.stdout).split(/\r?\n/).map(line => line.replace(/ \(\*\)$/, '')).filter(line => line && !line.startsWith('caveat-runtime ')))].sort();
      const output = await command('rustsec-audit', executable, ['audit', '--file', path.join(inputs, 'Cargo.lock'), '--db', db, '--no-fetch', '--no-yanked', '--json'], { cwd: inputs, allowFailure: true });
      const result = evaluateRustAudit(json(output.stdout), output.exitCode);
      for (const input of inputFiles) assert.equal(sha256(await readFile(path.join(inputs, input.file))), input.sha256, `${input.file} changed during the Rust scan`);
      assert.equal(await git('db-after', ['rev-parse', 'HEAD']), commit, 'advisory database changed during audit');
      assert.equal(await git('db-clean-after', ['status', '--porcelain']), '', 'advisory database changed during audit');
      return { ...result, inputFiles, configurationSha256: sha256(configuration), responseSha256: sha256(output.stdout),
        completeLockfile: true, linkedCrates, yankedCratesChecked: false };
    });
  } catch (error) { report.failures.push(`setup: ${error.message}`); }
  report.passed = report.failures.length === 0 && ['artifact', 'packed-npm-runtime', 'rust-build-lockfile',
    'repository-development-and-site', 'editor-development', 'collision-fixture', 'mcp-client-fixture'].every(name => report.scopes[name]?.passed);
  report.finishedAt = now();
  await write('report.json', `${JSON.stringify(report, null, 2)}\n`);
  await write('SHA256SUMS', `${sha256(await readFile(path.join(directory, 'report.json')))}  report.json\n`);
  return { directory, report };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const options = parseSecurityArguments(process.argv.slice(2));
    if (options.help) console.log('node scripts/audit-kit-security.mjs --package-report FILE [--audit-bin FILE] [--advisory-db DIRECTORY]');
    else {
      const result = await auditKitSecurity(options);
      console.log(`Package security checks ${result.report.passed ? 'pass' : 'FAIL'}: ${path.join(result.directory, 'report.json')}`);
      for (const failure of result.report.failures) console.error(`  ${failure}`);
      process.exitCode = result.report.passed ? 0 : 1;
    }
  } catch (error) { console.error(error.message); process.exitCode = 2; }
}
