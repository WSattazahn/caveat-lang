// Restore the exact published rc.15 C3 save, then follow accepted source paths
// until every state has received a set. This is source-derived write coverage,
// checked against before/after guards, not a runtime statement tracer.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { deflateRawSync } from 'node:zlib';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
const here = path.dirname(fileURLToPath(import.meta.url));
const option = (name, fallback) => process.argv.find(arg => arg.startsWith(`--${name}=`))?.slice(name.length + 3) ?? fallback;
const packageDir = path.resolve(option('package', 'kit'));
const fixture = path.resolve(option('save', 'experiments/departure-gate/fixtures/rc15-c3-save.json'));
const source = readFileSync(path.join(here, 'c3-windows.cav'), 'utf8');
const oldSave = readFileSync(fixture, 'utf8');
const { loadRuntimeFromDirectory } = await import(pathToFileURL(path.join(packageDir, 'lib/node.mjs')));
const runtime = await loadRuntimeFromDirectory();
const session = runtime.restore(source, oldSave);
const value = (snapshot, name) => snapshot.values[name];
const stateNames = Object.keys(session.snapshot().value_grounds).sort();
const written = new Set();
const ids = ['cave', 'pool', 'ruin', 'grove', 'pit'];
let events = 0, archiveItems = 0, archiveBytes = 0;
const measure = () => {
  const save = session.save();
  const restored = runtime.restore(source, save);
  assert.equal(restored.save(), save, 'every checkpoint must restore to its exact save text');
  restored.close();
  return { events, raw: Buffer.byteLength(save), adapter: Buffer.byteLength(JSON.stringify(deflateRawSync(Buffer.from(save), { level: 9 }).toString('base64'))), states_written: [...written].sort() };
};
const checkpoints = { at_restore: measure() };
const send = (event, payload) => {
  const before = session.snapshot();
  const result = session.dispatchView(event, payload);
  assert.equal(result.outcome, 'accepted', `${event}: ${JSON.stringify(result)}`);
  events++;
  const after = session.snapshot();
  if (event === 'witness') {
    const target = payload.target;
    const slot = value(before, 'next_memory');
    assert.equal(value(before, `${target}_gone`), 0);
    assert.equal(value(after, `${target}_gone`), 1);
    for (const name of ['observation', 'observation_sort', 'next_memory', 'note_value',
      `${target}_gone`, `${target}_marked`, `${target}_eaten_by`, `${target}_regrow`,
      `m${slot}_support`, `m${slot}_contradiction`]) written.add(name);
    for (let i = 1; i <= 6; i++) { written.add(`j${i}_code`); written.add(`j${i}_cite`); }
    if (payload.sort === 'duskcap' || value(before, 'recovery') < 2) written.add('recovery');
  } else {
    written.add('glow'); written.add('heavy');
    for (const id of ids) if (value(before, `${id}_gone`) === 1) {
      written.add(`${id}_regrow`);
      if (value(after, `${id}_life`) === value(before, `${id}_life`) + 1) {
        assert.equal(value(after, `${id}_regrow`), 0);
        assert.equal(value(after, `${id}_gone`), 0);
        for (const suffix of ['taste', 'life', 'gone', 'eaten_by']) written.add(`${id}_${suffix}`);
      }
    }
  }
  if (events === 1) checkpoints.after_first_event = measure();
  if (!checkpoints.every_state_set && stateNames.every(name => written.has(name))) checkpoints.every_state_set = measure();
  const entries = session.drainArchive();
  archiveItems += entries.length;
  archiveBytes += Buffer.byteLength(JSON.stringify(entries));
};
try {
  for (let cycle = 0; cycle < 2; cycle++) {
    for (const [i, id] of ids.entries()) {
      send('witness', { target: id, sort: (cycle + i) % 2 ? 'glowcap' : 'duskcap' });
      if (checkpoints.every_state_set) break;
    }
    if (checkpoints.every_state_set) break;
    for (let tick = 0; tick < 46 * 16; tick++) send('tick', { dt: 0.0625 });
  }
  assert.ok(checkpoints.every_state_set, `states without covered set paths: ${stateNames.filter(name => !written.has(name))}`);
  assert.deepEqual([...written].sort(), stateNames, 'write-path coverage must match every declared state');
  console.log(JSON.stringify({ runtime: runtime.identity, source_sha256: createHash('sha256').update(source).digest('hex'),
    rc15_save_sha256: createHash('sha256').update(oldSave).digest('hex'), state_count: stateNames.length,
    coverage: 'Accepted witness/tick source paths checked against before/after state guards; not a runtime statement trace.',
    checkpoints, archive: { items: archiveItems, serialized_bytes: archiveBytes, undrained: session.undrained } }, null, 2));
} finally { session.close(); }
