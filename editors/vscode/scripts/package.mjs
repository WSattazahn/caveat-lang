// Packs the extension into test-results/vscode/ and checks the .vsix holds
// exactly the tested files: the grammar byte for byte, the manifest, the
// language configuration, this README and the repository's MIT license.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { inflateRawSync } from 'node:zlib';

const here = fileURLToPath(new URL('../', import.meta.url));
const root = join(here, '..', '..');
const out = join(root, 'test-results', 'vscode');
const manifest = JSON.parse(readFileSync(join(here, 'package.json'), 'utf8'));
const vsixName = `${manifest.name}-${manifest.version}.vsix`;
const vsix = join(out, vsixName);
const require = createRequire(import.meta.url);
const vsce = join(dirname(require.resolve('@vscode/vsce/package.json')), 'vsce');

// The files the extension ships, as `vsce ls` names them.
const shipped = ['LICENSE', 'README.md', 'language-configuration.json', 'package.json', 'syntaxes/caveat.tmLanguage.json'];

// The entries of a zip archive, by name.
function unzip(archive) {
  const end = archive.lastIndexOf(Buffer.from([0x50, 0x4b, 0x05, 0x06]));
  assert.ok(end >= 0, 'not a zip archive');
  const count = archive.readUInt16LE(end + 10);
  let cursor = archive.readUInt32LE(end + 16);
  const entries = new Map();
  for (let index = 0; index < count; index++) {
    assert.equal(archive.readUInt32LE(cursor), 0x02014b50, 'corrupt central directory');
    const method = archive.readUInt16LE(cursor + 10);
    const compressed = archive.readUInt32LE(cursor + 20);
    const size = archive.readUInt32LE(cursor + 24);
    const nameLength = archive.readUInt16LE(cursor + 28);
    const extraLength = archive.readUInt16LE(cursor + 30);
    const commentLength = archive.readUInt16LE(cursor + 32);
    const local = archive.readUInt32LE(cursor + 42);
    const name = archive.toString('utf8', cursor + 46, cursor + 46 + nameLength);
    const start = local + 30 + archive.readUInt16LE(local + 26) + archive.readUInt16LE(local + 28);
    const raw = archive.subarray(start, start + compressed);
    const data = method === 0 ? raw : inflateRawSync(raw);
    assert.equal(data.length, size, `${name}: wrong size`);
    entries.set(name, data);
    cursor += 46 + nameLength + extraLength + commentLength;
  }
  return entries;
}

const sha256 = data => createHash('sha256').update(data).digest('hex');
const run = args => execFileSync(process.execPath, [vsce, ...args], { cwd: here, encoding: 'utf8' });

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

// The license is the repository's; it is copied in only while packing.
const license = join(here, 'LICENSE');
copyFileSync(join(root, 'LICENSE'), license);
try {
  const listed = run(['ls', '--no-dependencies']).trim().split(/\r?\n/).sort();
  assert.deepEqual(listed, shipped, 'vsce would ship a different set of files');
  run(['package', '--no-dependencies', '--out', vsix]);
} finally {
  rmSync(license, { force: true });
}

const archive = readFileSync(vsix);
const entries = unzip(archive);
const expected = {
  'extension/LICENSE.txt': readFileSync(join(root, 'LICENSE')),
  'extension/readme.md': readFileSync(join(here, 'README.md')),
  'extension/language-configuration.json': readFileSync(join(here, 'language-configuration.json')),
  'extension/syntaxes/caveat.tmLanguage.json': readFileSync(join(here, 'syntaxes', 'caveat.tmLanguage.json')),
};
assert.deepEqual([...entries.keys()].sort(),
  ['[Content_Types].xml', 'extension.vsixmanifest', 'extension/package.json', ...Object.keys(expected)].sort());
for (const [name, data] of Object.entries(expected)) {
  assert.ok(entries.get(name).equals(data), `${name} differs from the tested source`);
}
// vsce may reorder the manifest's keys but must not change its content.
assert.deepEqual(JSON.parse(entries.get('extension/package.json')), manifest);
const identity = /<Identity\b[^>]*>/.exec(entries.get('extension.vsixmanifest').toString('utf8'))?.[0] ?? '';
for (const [attribute, value] of [['Id', manifest.name], ['Version', manifest.version], ['Publisher', manifest.publisher]]) {
  assert.ok(identity.includes(`${attribute}="${value}"`), `vsixmanifest identity lacks ${attribute}="${value}": ${identity}`);
}

const report = {
  vsix: vsixName,
  sha256: sha256(archive),
  bytes: archive.length,
  extension: `${manifest.publisher}.${manifest.name}@${manifest.version}`,
  files: Object.fromEntries([...entries].map(([name, data]) => [name, sha256(data)])),
  grammarSha256: sha256(expected['extension/syntaxes/caveat.tmLanguage.json']),
};
writeFileSync(join(out, 'report.json'), `${JSON.stringify(report, null, 2)}\n`);
writeFileSync(join(out, 'SHA256SUMS'), `${report.sha256}  ${vsixName}\n`);
console.log(`packed ${join('test-results', 'vscode', vsixName)} (${archive.length} bytes, sha256 ${report.sha256})`);
