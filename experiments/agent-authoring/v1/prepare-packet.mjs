// Recreate the public reference packet from the pinned commit, never HEAD.
import { readFile, mkdir, writeFile, copyFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const here = fileURLToPath(new URL('./', import.meta.url));
const base = '04f72dc';
const documents = [
  'docs/AI_AUTHORING.md', 'docs/CAVEAT_ESSENCE.md', 'runtime/prelude.cav',
  ...['0.1', '0.2', '0.3', '0.4', '0.5', '0.6', '0.7'].map(version => `spec/caveat-reactive-${version}.md`),
  'spec/caveat-0.1.md', 'spec/caveat-define-0.1.md', 'spec/caveat-repetition-0.1.md',
  'spec/caveat-procedure-symbols-0.1.md', 'spec/caveat-typed-parameters-0.1.md',
  'spec/caveat-text-0.1.md', 'spec/caveat-reject-0.1.md',
  'spec/caveat-explanations-0.1.md', 'spec/caveat-explanations-0.2.md',
  'spec/caveat-decision-journal-0.1.md', 'spec/caveat-late-qualification-0.1.md',
  'spec/caveat-renewal-0.1.md', 'spec/caveat-state-caveats-0.1.md',
  'spec/caveat-observation-order-0.1.md', 'spec/caveat-save-0.1.md', 'spec/caveat-view-0.1.md',
];
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const manifest = { schema: 1, sourceCommit: base, references: {}, runtime: {} };
for (const name of documents) {
  const contents = execFileSync('git', ['show', `${base}:${name}`], { cwd: root });
  const destination = path.join(here, 'packet/reference', name);
  await mkdir(path.dirname(destination), { recursive: true });
  await writeFile(destination, contents);
  manifest.references[name] = hash(contents);
}
await mkdir(path.join(here, 'packet/runtime'), { recursive: true });
for (const name of ['caveat_runtime.js', 'caveat_runtime_bg.wasm']) {
  const original = path.join(root, 'dist/pkg-reactive', name);
  manifest.runtime[name] = hash(await readFile(original));
  await copyFile(original, path.join(here, 'packet/runtime', name));
}
// Check rather than silently replace the frozen manifest on subsequent runs.
const manifestPath = path.join(here, 'packet/manifest.json');
let previous;
try { previous = JSON.parse(await readFile(manifestPath, 'utf8')); } catch (error) { if (error.code !== 'ENOENT') throw error; }
if (previous && JSON.stringify(previous) !== JSON.stringify(manifest)) throw new Error('Packet differs from the frozen manifest; build the pinned runtime before preparing it.');
if (!previous) await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`Prepared ${documents.length} pinned references and the WASM runtime (${base}).`);
