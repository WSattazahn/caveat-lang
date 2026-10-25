import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { buildSbom, main, parseLockChecksums, parseNotices, validateSbom } from './release-sbom.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const revision = 'a'.repeat(40);
const digest = 'b'.repeat(64);
const timestamp = '2026-10-25T04:00:00Z';
const manifest = { name: 'caveat-lang', version: '1.4.0', license: 'MIT' };

test('the checked-in notices and lockfile give every listed crate a checksum', () => {
  const crates = parseNotices(readFileSync(path.join(root, 'THIRD_PARTY_NOTICES.md'), 'utf8').replace(/\r\n/g, '\n'));
  const checksums = parseLockChecksums(readFileSync(path.join(root, 'runtime', 'Cargo.lock'), 'utf8'));
  assert.ok(crates.some(crate => crate.name === 'sha2'));
  assert.ok(!crates.some(crate => crate.name === 'serde_derive'), 'proc macros are not compiled in');
  const sbom = validateSbom(buildSbom({ manifest, sha256: digest, crates, checksums, revision, timestamp }));
  assert.equal(sbom.components.length, crates.length);
  for (const component of sbom.components) {
    assert.match(component.hashes[0].content, /^[0-9a-f]{64}$/);
    assert.equal(component.purl, `pkg:cargo/${component.name}@${component.version}`);
  }
});

test('the document names the tarball, its revision and each crate it depends on', () => {
  const crates = [{ name: 'sha2', version: '0.10.9', license: 'MIT OR Apache-2.0' }];
  const checksums = new Map([['sha2 0.10.9', 'c'.repeat(64)]]);
  const sbom = buildSbom({ manifest, sha256: digest, crates, checksums, revision, timestamp });
  assert.equal(sbom.bomFormat, 'CycloneDX');
  assert.equal(sbom.specVersion, '1.6');
  assert.match(sbom.serialNumber, /^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-5[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
  assert.equal(sbom.metadata.component.purl, 'pkg:npm/caveat-lang@1.4.0');
  assert.deepEqual(sbom.metadata.component.hashes, [{ alg: 'SHA-256', content: digest }]);
  assert.deepEqual(sbom.metadata.properties, [{ name: 'caveat-lang:revision', value: revision }]);
  assert.deepEqual(sbom.components[0].licenses, [{ expression: 'MIT OR Apache-2.0', acknowledgement: 'declared' }]);
  assert.deepEqual(sbom.dependencies[0], { ref: 'pkg:npm/caveat-lang@1.4.0', dependsOn: ['pkg:cargo/sha2@0.10.9'] });
  assert.deepEqual(buildSbom({ manifest, sha256: digest, crates, checksums, revision, timestamp }), sbom, 'the same release gives the same document');
});

test('refuses a crate without a checksum, another package, npm dependencies and malformed inputs', () => {
  const crates = [{ name: 'sha2', version: '0.10.9', license: 'MIT' }];
  const checksums = new Map([['sha2 0.10.9', 'c'.repeat(64)]]);
  const base = { manifest, sha256: digest, crates, checksums, revision, timestamp };
  const refusals = [
    [{ checksums: new Map() }, /no registry checksum/],
    [{ manifest: { ...manifest, name: 'other' } }, /not caveat-lang/],
    [{ manifest: { ...manifest, dependencies: { left: '1' } } }, /declares dependencies/],
    [{ sha256: digest.toUpperCase() }, /64 lowercase/],
    [{ revision: 'abc' }, /complete Git SHA/],
    [{ timestamp: 'soon' }, /not a date/],
  ];
  for (const [change, expected] of refusals) assert.throws(() => buildSbom({ ...base, ...change }), expected);
  assert.throws(() => parseNotices('# Third-party notices\n'), /no crates/);
});

test('the validator refuses documents a consumer could not use', () => {
  const crates = [{ name: 'sha2', version: '0.10.9', license: 'MIT' }, { name: 'itoa', version: '1.0.18', license: 'MIT OR Apache-2.0' }];
  const checksums = new Map([['sha2 0.10.9', 'c'.repeat(64)], ['itoa 1.0.18', 'e'.repeat(64)]]);
  const good = () => buildSbom({ manifest, sha256: digest, crates, checksums, revision, timestamp });
  assert.doesNotThrow(() => validateSbom(good()));
  const breaks = [
    [sbom => { sbom.specVersion = '1.5'; }, /not a CycloneDX 1\.6/],
    [sbom => { sbom.serialNumber = 'abc'; }, /urn:uuid/],
    [sbom => { sbom.components[0].hashes[0].alg = 'SHA256'; }, /sha2 0\.10\.9 lacks a SHA-256/],
    [sbom => { sbom.components[1].purl = 'itoa'; }, /itoa 1\.0\.18 lacks a cargo purl/],
    [sbom => { sbom.components[0].licenses = []; }, /lacks a license expression/],
    [sbom => { sbom.components[1] = { ...sbom.components[0] }; }, /listed twice/],
    [sbom => { sbom.dependencies[0].dependsOn.pop(); }, /every component/],
    [sbom => { sbom.dependencies.push({ ref: 'pkg:cargo/x@1', dependsOn: [] }); }, /names no component/],
    [sbom => { sbom.metadata.component.hashes = []; }, /metadata\.component/],
  ];
  for (const [change, expected] of breaks) {
    const sbom = good();
    change(sbom);
    assert.throws(() => validateSbom(sbom), expected);
  }
});

test('the command reads the notices, lockfile and time from the revision and the manifest from the tarball', () => {
  const directory = mkdtempSync(path.join(tmpdir(), 'release-sbom-'));
  const git = (...args) => execFileSync('git', args, { cwd: directory, encoding: 'utf8' }).trim();
  git('init', '-q');
  mkdirSync(path.join(directory, 'runtime'));
  writeFileSync(path.join(directory, 'THIRD_PARTY_NOTICES.md'), '# Third-party notices\r\n\r\n## sha2 0.10.9\r\n\r\nDeclared license: `MIT OR Apache-2.0`. Used under MIT.\r\n');
  writeFileSync(path.join(directory, 'runtime', 'Cargo.lock'), `version = 4\n\n[[package]]\nname = "sha2"\nversion = "0.10.9"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "${'d'.repeat(64)}"\n`);
  git('add', '.');
  git('-c', 'user.name=test', '-c', 'user.email=test@example.invalid', 'commit', '-q', '-m', 'fixture', '--date', '2026-10-25T04:00:00Z');
  git('-c', 'user.name=test', '-c', 'user.email=test@example.invalid', 'tag', '-a', 'v1.4.0', '-m', 'v1.4.0');
  // A later commit must not leak into the tag's document.
  writeFileSync(path.join(directory, 'THIRD_PARTY_NOTICES.md'), '# Third-party notices\n');
  git('-c', 'user.name=test', '-c', 'user.email=test@example.invalid', 'commit', '-q', '-am', 'later');
  const package_ = path.join(directory, 'pack');
  mkdirSync(path.join(package_, 'package'), { recursive: true });
  writeFileSync(path.join(package_, 'package', 'package.json'), JSON.stringify(manifest));
  const tarball = path.join(directory, 'caveat-lang-1.4.0.tgz');
  execFileSync('tar', ['-czf', tarball, '-C', package_, 'package']);
  const out = path.join(directory, 'caveat-lang-1.4.0.cdx.json');
  main(['--tarball', tarball, '--revision', 'refs/tags/v1.4.0', '--out', out], directory);
  const sbom = JSON.parse(readFileSync(out, 'utf8'));
  assert.equal(sbom.metadata.component.hashes[0].content, createHash('sha256').update(readFileSync(tarball)).digest('hex'));
  assert.deepEqual(sbom.metadata.properties, [{ name: 'caveat-lang:revision', value: git('rev-parse', 'v1.4.0^{commit}') }]);
  assert.equal(sbom.metadata.timestamp, new Date(git('show', '-s', '--format=%cI', 'v1.4.0^{commit}')).toISOString());
  assert.equal(sbom.components[0].hashes[0].content, 'd'.repeat(64));
  assert.throws(() => main(['--tarball', tarball, '--revision', 'HEAD', '--out', out], directory), /no crates/);
  assert.throws(() => main(['--tarball', tarball], directory), /--revision is required/);
});
