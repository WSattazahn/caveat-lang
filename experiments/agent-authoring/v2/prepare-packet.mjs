// Recreate only from the committed runtime/docs revision in study.json.
import assert from 'node:assert/strict';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { GUIDE_TRANSFORM, transformReference } from './packet-transform.mjs';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const here = fileURLToPath(new URL('./', import.meta.url));
const study = JSON.parse(await readFile(path.join(here, 'study.json'), 'utf8'));
const base = study.runtimeCommit;
assert.match(base, /^[0-9a-f]{40}$/, 'Set study.runtimeCommit to the full committed runtime revision first');
const git = args => execFileSync('git', args, { cwd: root });
assert.equal(git(['rev-parse', `${base}^{commit}`]).toString().trim(), base);
const provenance = JSON.parse(await readFile(path.join(root, 'dist/build-info.json'), 'utf8'));
assert.equal(provenance.revision, base, 'Build the pinned runtime revision first');
assert.equal(provenance.compiled, true, 'An assemble-only build cannot establish runtime provenance');
assert.equal(provenance.clean, true, 'The runtime build must come from a clean tracked checkout');
assert.equal(git(['diff', base, '--', 'runtime', 'rust-toolchain.toml', 'scripts/build-web.mjs']).length, 0,
  'Runtime/build inputs differ from the pinned revision');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const manifest = { schema: 2, sourceCommit: base, build: provenance, transformations: [], references: {}, runtime: {} };
const files = [];
for (const name of study.references) {
  const original = git(['show', `${base}:${name}`]);
  const contents = transformReference(name, original);
  if (name === GUIDE_TRANSFORM.file) manifest.transformations.push({ ...GUIDE_TRANSFORM,
    sourceSha256: hash(original), packetSha256: hash(contents) });
  manifest.references[name] = hash(contents);
  files.push([path.join(here, 'packet/reference', name), contents]);
}
for (const name of ['caveat_runtime.js', 'caveat_runtime_bg.wasm']) {
  const contents = await readFile(path.join(root, 'dist/pkg-reactive', name));
  manifest.runtime[name] = hash(contents);
  files.push([path.join(here, 'packet/runtime', name), contents]);
}
const manifestPath = path.join(here, 'packet/manifest.json');
let previous;
try { previous = JSON.parse(await readFile(manifestPath, 'utf8')); }
catch (error) { if (error.code !== 'ENOENT') throw error; }
if (previous) assert.deepEqual(manifest, previous, 'Packet differs from frozen manifest; no packet bytes were replaced');
// Validate all inputs before replacing any generated packet file.
for (const [file, contents] of files) {
  await mkdir(path.dirname(file), { recursive: true });
  await writeFile(file, contents);
}
if (!previous) await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, { flag: 'wx' });
console.log(`Prepared ${study.references.length} references and the frozen runtime (${base}; ${provenance.host}).`);
