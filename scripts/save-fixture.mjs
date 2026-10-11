// Writes the per-release save fixtures (docs/COMPATIBILITY.md, "Saves,
// archives and source"): for each program in kit/test/fixtures/saves/programs,
// an installed release plays the program's `before` events, saves, restores
// that save and plays its `after` events. The fixture keeps the save, the
// archive drained before it, each later outcome and View 0.1, and the final
// View 0.2 and explanation, as that release gave them. A later runtime must
// restore the save and give the same outcomes and views
// (kit/test/save-fixtures.test.mjs).
//
//   node scripts/save-fixture.mjs --package=node_modules/caveat-lang
//
// Run it once per published release, from that release's installed package,
// and commit the output under kit/test/fixtures/saves/VERSION/.
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const fixtures = path.join(root, 'kit', 'test', 'fixtures', 'saves');
const packageDir = process.argv.find(arg => arg.startsWith('--package='))?.slice(10);
if (!packageDir) throw new Error('pass --package=DIR, an installed caveat-lang release');
const manifest = JSON.parse(readFileSync(path.join(packageDir, 'package.json'), 'utf8'));
assert.equal(manifest.name, 'caveat-lang');
const { loadRuntimeFromDirectory } = await import(pathToFileURL(path.resolve(packageDir, 'lib', 'node.mjs')).href);
const { explain } = await import(pathToFileURL(path.resolve(packageDir, 'lib', 'explain.mjs')).href);
const runtime = await loadRuntimeFromDirectory();

const outcomeOf = result => (result.outcome === 'accepted'
  ? { outcome: 'accepted' }
  : { outcome: result.outcome, origin: result.origin, code: result.code });

const out = path.join(fixtures, manifest.version);
mkdirSync(out, { recursive: true });
for (const file of readdirSync(path.join(fixtures, 'programs')).filter(name => name.endsWith('.cav')).sort()) {
  const name = file.slice(0, -4);
  const source = readFileSync(path.join(fixtures, 'programs', file), 'utf8');
  const { before, after } = JSON.parse(readFileSync(path.join(fixtures, 'events', `${name}.json`), 'utf8'));
  const first = runtime.open(source);
  const archive = [];
  const played = before.map(({ event, payload }) => {
    const result = outcomeOf(first.dispatch(event, payload));
    archive.push(...first.drainArchive());
    return result;
  });
  const save = first.save();
  first.close();
  const session = runtime.restore(source, save);
  const steps = after.map(({ event, payload }) => {
    const result = outcomeOf(session.dispatch(event, payload));
    archive.push(...session.drainArchive());
    return { ...result, view: JSON.stringify(session.view()) };
  });
  const snapshot = session.snapshot();
  const fixture = {
    note: 'Written by scripts/save-fixture.mjs from the installed release named here. Do not edit; regenerate from that release.',
    package: `${manifest.name}@${manifest.version}`,
    program: `programs/${file}`,
    events: `events/${name}.json`,
    before: played,
    save,
    after: steps,
    archive,
    view_v2: JSON.stringify(session.view({ schema: '0.2' })),
    explain: (({ decisions, evidence, displayed }) => ({ decisions, evidence, displayed }))(explain(snapshot, [], { archive })),
  };
  session.close();
  writeFileSync(path.join(out, `${name}.json`), `${JSON.stringify(fixture, null, 1)}\n`);
  console.log(`${manifest.version}/${name}.json: ${played.length} + ${steps.length} events, save ${Buffer.byteLength(save)} bytes, ${archive.length} archive items`);
}
