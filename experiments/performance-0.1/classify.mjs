// Labels every event of every workload with its class (lib/classes.mjs), by
// replaying each workload once, untimed, and recording what the runtime
// reported. Run it before timing; analyze.mjs joins the labels to the
// per-event samples by position.
//
//   node experiments/performance-0.1/classify.mjs --runtime=DIR [--runtime=DIR2 ...] [--out=FILE]
//
// DIR holds caveat_runtime.js and caveat_runtime_bg.wasm with
// dispatch_outcome (0.1.0-rc.3 or later). The labels depend on the runtime's
// behaviour, so the file records each runtime's hash. With several runtimes,
// every one must label every event identically, or nothing is written.
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { harnessDirectory, loadManifest, loadWorkload, sha256 } from './lib/workloads.mjs';
import { CLASS_RULES, CLOCK_STATES, changedParts, classify, signature } from './lib/classes.mjs';

const runtimeDirs = process.argv.filter((value) => value.startsWith('--runtime=')).map((value) => path.resolve(value.slice(10)));
if (!runtimeDirs.length) throw new Error('give --runtime=DIR (a directory with caveat_runtime.js and caveat_runtime_bg.wasm)');
const outArg = process.argv.find((value) => value.startsWith('--out='))?.slice(6);
const out = path.resolve(outArg ?? path.join(harnessDirectory, 'inputs', 'event-classes.json'));

async function labelAll(runtimeDir, manifest) {
  // Each runtime is its own module instance: a query string makes Node load
  // the glue afresh instead of reusing the first runtime's.
  const glueUrl = `${pathToFileURL(path.join(runtimeDir, 'caveat_runtime.js')).href}?runtime=${encodeURIComponent(runtimeDir)}`;
  const glue = await import(glueUrl);
  const wasmBytes = await readFile(path.join(runtimeDir, 'caveat_runtime_bg.wasm'));
  await glue.default({ module_or_path: wasmBytes });
  const Session = glue.WebReactiveSession;
  if (typeof Session.prototype.dispatch_outcome !== 'function') throw new Error(`${runtimeDir}: this runtime has no dispatch_outcome`);
  const workloads = [];
  for (const spec of manifest.workloads) {
    const workload = await loadWorkload(manifest, spec.id);
    const clockStates = CLOCK_STATES[spec.id];
    if (!clockStates) throw new Error(`no clock states declared for ${spec.id} in lib/classes.mjs`);
    const episodes = [];
    for (const episode of workload.episodes) {
      const session = new Session(workload.program.source);
      let before = JSON.parse(session.save());
      const labels = [];
      for (const [event, payloadText] of episode) {
        const outcome = JSON.parse(session.dispatch_outcome(event, payloadText));
        let record;
        if (outcome.outcome === 'accepted') {
          const after = JSON.parse(session.save());
          record = { outcome: 'accepted', effects: outcome.snapshot.effects.map((effect) => effect.kind), changed: changedParts(before, after, clockStates) };
          before = after;
        } else {
          record = { outcome: outcome.outcome, code: outcome.code, effects: [], changed: [] };
        }
        const eventClass = classify(record);
        labels.push({
          event,
          class: eventClass,
          signature: signature(event, eventClass, record.effects, record.code),
          effects: [...new Set(record.effects)].sort(),
          changed: record.changed.slice(0, 8),
          ...(record.code ? { code: record.code } : {}),
        });
      }
      session.free();
      episodes.push(labels);
    }
    workloads.push({ spec, workload, clockStates, episodes });
  }
  return { runtimeDir, wasmSha256: sha256(wasmBytes), workloads };
}

const count = (values) => values.reduce((counts, value) => ({ ...counts, [value]: (counts[value] ?? 0) + 1 }), {});

const manifest = await loadManifest();
const labelled = [];
for (const runtimeDir of runtimeDirs) labelled.push(await labelAll(runtimeDir, manifest));
const [first, ...others] = labelled;

const workloads = first.workloads.map(({ spec, workload, clockStates, episodes }, index) => {
  const classes = episodes.map((labels) => labels.map((label) => label.class));
  const signatures = episodes.map((labels) => labels.map((label) => label.signature));
  const classesSha256 = sha256(JSON.stringify(classes));
  const signaturesSha256 = sha256(JSON.stringify(signatures));
  for (const other of others) {
    const theirs = other.workloads[index].episodes;
    const theirClasses = sha256(JSON.stringify(theirs.map((labels) => labels.map((label) => label.class))));
    const theirSignatures = sha256(JSON.stringify(theirs.map((labels) => labels.map((label) => label.signature))));
    if (theirClasses !== classesSha256 || theirSignatures !== signaturesSha256) {
      throw new Error(`${spec.id}: ${other.runtimeDir} labels events differently from ${first.runtimeDir}`);
    }
  }
  const flat = episodes.flat();
  const countsPerRepeat = count(flat.map((label) => label.class));
  console.log(`${spec.id}: ${JSON.stringify(countsPerRepeat)} per repeat, x${workload.repeats}`);
  return {
    id: spec.id,
    streamSha256: workload.streamSha256,
    programSha256: workload.program.sha256,
    clockStates,
    repeats: workload.repeats,
    eventsPerRepeat: flat.length,
    countsPerRepeat,
    countsPerPass: Object.fromEntries(Object.entries(countsPerRepeat).map(([name, n]) => [name, n * workload.repeats])),
    bySignature: count(flat.map((label) => label.signature)),
    classesSha256,
    signaturesSha256,
    classes,
    signatures,
    // Audit trail: what each assignment rested on, for each episode's first
    // 40 events and every event after them that reported an effect, was
    // refused, or changed state (runs of state-changing ticks keep their
    // first and last event only).
    audit: episodes.map((labels) => labels.map((label, position) => ({ position, ...label }))
      .filter((label, position, all) => position < 40 || label.effects.length || label.class === 'refused'
        || (label.class === 'state-changing' && (all[position - 1]?.class !== 'state-changing' || all[position + 1]?.class !== 'state-changing')))),
  };
});

await writeFile(out, `${JSON.stringify({
  schema: 'caveat-performance-classes/0.2',
  generatedBy: 'experiments/performance-0.1/classify.mjs',
  runtimes: labelled.map(({ runtimeDir, wasmSha256 }) => ({ dir: runtimeDir, wasmSha256 })),
  agreement: `${labelled.length} runtime(s) labelled every event of every workload identically`,
  rules: CLASS_RULES.map(([name, rule]) => ({ name, rule })),
  workloads,
})}\n`);
console.log(`wrote ${out}`);
