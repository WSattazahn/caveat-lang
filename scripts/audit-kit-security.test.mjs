import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { auditKitSecurity, evaluateNpmAudit, evaluateRustAudit, parseSecurityArguments, validatePackageIdentity } from './audit-kit-security.mjs';

const digest = bytes => createHash('sha256').update(bytes).digest('hex');
function artifact() {
  const tarball = Buffer.from('synthetic packed bytes');
  const wasm = Buffer.from('synthetic wasm bytes');
  const revision = 'a'.repeat(40);
  const build = { revision, clean: true, compiled: true };
  const manifest = { name: 'caveat-lang', version: '0.1.0-rc.7' };
  const report = { schema: 1, name: manifest.name, version: manifest.version, size: tarball.length, sha256: digest(tarball),
    runtime: { ...build, reactiveWasmSha256: digest(wasm) } };
  return { report, tarball, manifest, build, wasm };
}
function npmReport(severities = []) {
  const counts = { info: 0, low: 0, moderate: 0, high: 0, critical: 0, total: severities.length };
  const vulnerabilities = {};
  severities.forEach((severity, index) => { counts[severity]++; vulnerabilities[`package-${index}`] = { name: `package-${index}`, severity }; });
  return { auditReportVersion: 2, vulnerabilities, metadata: { vulnerabilities: counts } };
}
function rustReport() {
  return { database: { 'advisory-count': 1000 }, settings: { target_arch: [], target_os: [], severity: null, ignore: [],
    informational_warnings: ['unmaintained', 'unsound', 'notice'] }, vulnerabilities: { found: false, count: 0, list: [] }, warnings: {} };
}

test('artifact identity requires the tested bytes, clean compiled build and exact package/runtime identity', () => {
  assert.equal(validatePackageIdentity(artifact()).npmRuntimeDependencies.length, 0);
  const mutations = [
    [input => { input.tarball = Buffer.from('changed'); }, /tarball differs/],
    [input => { input.report.size++; }, /size differs/],
    [input => { input.manifest.version = '0.1.0-rc.6'; }, /version differs/],
    [input => { input.build.clean = false; }, /clean source build/],
    [input => { input.build.compiled = false; }, /compiled runtime/],
    [input => { input.report.runtime.revision = 'b'.repeat(40); }, /revisions differ/],
    [input => { input.wasm = Buffer.from('different runtime'); }, /WASM differs/],
  ];
  for (const [mutate, message] of mutations) { const input = artifact(); mutate(input); assert.throws(() => validatePackageIdentity(input), message); }
});

test('adding runtime, optional, peer or bundled npm dependencies requires explicit artifact-contract review', () => {
  for (const field of ['dependencies', 'optionalDependencies', 'peerDependencies', 'bundledDependencies', 'bundleDependencies']) {
    const input = artifact();
    input.manifest[field] = field.includes('bundle') ? ['third-party'] : { 'third-party': '1.0.0' };
    assert.throws(() => validatePackageIdentity(input), new RegExp(field));
  }
});

test('npm blocks High/Critical and retains all lower-severity findings', () => {
  assert.equal(evaluateNpmAudit(npmReport(), 0).passed, true);
  assert.equal(evaluateNpmAudit(npmReport(['low', 'moderate']), 0).findings.length, 2);
  assert.equal(evaluateNpmAudit(npmReport(['high']), 1).passed, false);
  assert.equal(evaluateNpmAudit(npmReport(['critical', 'moderate']), 1).passed, false);
});

test('registry failures, omitted findings and contradictory audit exits never appear clean', () => {
  assert.throws(() => evaluateNpmAudit({ error: { code: 'ENETUNREACH' } }, 1), /unavailable/);
  assert.throws(() => evaluateNpmAudit({}, 0), /unsupported/);
  assert.throws(() => evaluateNpmAudit(npmReport(), 2), /exit disagrees/);
  assert.throws(() => evaluateNpmAudit(npmReport(['high']), 0), /exit disagrees/);
  const missing = npmReport(['high']); delete missing.vulnerabilities['package-0'];
  assert.throws(() => evaluateNpmAudit(missing, 0), /count disagrees/);
  const renamed = npmReport(['high']); renamed.vulnerabilities['package-0'].severity = 'low';
  assert.throws(() => evaluateNpmAudit(renamed, 0), /records disagree/);
});

test('RustSec blocks every vulnerability, including one with no severity, and unresolved warnings', () => {
  assert.equal(evaluateRustAudit(rustReport(), 0).passed, true);
  const vulnerable = rustReport();
  vulnerable.vulnerabilities = { found: true, count: 1, list: [{ advisory: { id: 'RUSTSEC-TEST', cvss: null } }] };
  assert.equal(evaluateRustAudit(vulnerable, 1).passed, false);
  const warning = rustReport(); warning.warnings = { unsound: [{ advisory: { id: 'RUSTSEC-WARNING' } }] };
  assert.equal(evaluateRustAudit(warning, 0).passed, false);
});

test('RustSec suppressions, filtered targets, empty databases and incomplete scans are rejected', () => {
  for (const [name, value] of [['ignore', ['RUSTSEC-TEST']], ['severity', 'high'], ['target_arch', ['wasm32']], ['target_os', ['linux']], ['informational_warnings', []]]) {
    const report = rustReport(); report.settings[name] = value;
    assert.throws(() => evaluateRustAudit(report, 0), /forbidden/);
  }
  const empty = rustReport(); empty.database['advisory-count'] = 0;
  assert.throws(() => evaluateRustAudit(empty, 0), /empty or unavailable/);
  assert.throws(() => evaluateRustAudit(rustReport(), 2), /did not complete/);
  const missing = rustReport(); missing.vulnerabilities.count = 1;
  assert.throws(() => evaluateRustAudit(missing, 0), /count disagrees/);
});

test('the CLI requires an explicit artifact and refuses unknown/duplicate or missing options', () => {
  for (const args of [[], ['--package-report'], ['--latest'], ['--package-report', 'x', '--package-report', 'y']]) {
    assert.throws(() => parseSecurityArguments(args));
  }
  assert.equal(parseSecurityArguments(['--package-report', 'example/report.json']).packageReport, path.resolve('example/report.json'));
  assert.deepEqual(parseSecurityArguments(['--help']), { help: true });
});

test('an unavailable artifact produces a retained failure receipt instead of a clean result', async () => {
  const result = await auditKitSecurity({ packageReport: fileURLToPath(new URL('./does-not-exist-security-report.json', import.meta.url)) });
  assert.equal(result.report.passed, false);
  assert.ok(result.report.failures.some(failure => failure.startsWith('setup:')));
  const retained = JSON.parse(await readFile(path.join(result.directory, 'report.json'), 'utf8'));
  assert.deepEqual(retained, result.report);
});
