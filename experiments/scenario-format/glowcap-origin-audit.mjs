// Supplementary audit, not a rescore: replay Glowcap's cr12 histories against
// caveat5/glowcap.cav through dispatch_outcome and record the structured
// origin/code of every expected rejection. The caveat5 adapter performs no
// validation, so its payload mapping is reproduced here. Run after
// `npm run build`; writes results-glowcap-origins.json beside this file.
import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';

const root = new URL('../../', import.meta.url);
const at = rel => new URL(rel, root);
const { SCENARIOS, applies } = await import(at('experiments/glowcap/scenarios.mjs').href);
const { default: init, WebReactiveSession } = await import(at('dist/pkg-reactive/caveat_runtime.js').href);
const wasm = await readFile(at('dist/pkg-reactive/caveat_runtime_bg.wasm'));
await init({ module_or_path: wasm });
const source = await readFile(at('experiments/glowcap/caveat5/glowcap.cav'), 'utf8');
const buildInfo = JSON.parse(await readFile(at('dist/build-info.json'), 'utf8'));
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');

const payloadOf = e => (e.type === 'tick' ? { dt: e.dt } : { target: e.id, sort: e.kind });
const toEvent = ([type, a, b]) => (type === 'tick' ? { type, dt: a } : { type, id: a, kind: b });

function send(session, event) {
  const beforeSave = session.save();
  const beforeView = session.view();
  let outcome;
  try { outcome = JSON.parse(session.dispatch_outcome(event.type, JSON.stringify(payloadOf(event)))); }
  catch (error) { return { outcome: 'fatal', message: String(error) }; }
  if (outcome.schema !== 'caveat-dispatch/0.1') return { outcome: 'fatal', message: `schema ${outcome.schema}` };
  if (outcome.outcome === 'rejected') {
    return { ...outcome, atomic: session.save() === beforeSave && session.view() === beforeView };
  }
  return outcome;
}

const tally = {};
const rows = [];
let accepted = 0;
let unexpected = 0;
for (const scenario of SCENARIOS.filter(s => applies(s, 'cr12'))) {
  let session = new WebReactiveSession(source);
  for (const step of scenario.steps) {
    if (step[0] === 'expect') continue;
    if (step[0] === 'resume') { const next = WebReactiveSession.restore(source, session.save()); session.free(); session = next; continue; }
    const expectReject = step[0] === 'reject';
    const event = expectReject ? step[1] : toEvent(step);
    const times = !expectReject && step[0] === 'tick' ? (step[2] ?? 1) : 1;
    for (let i = 0; i < times; i += 1) {
      const result = send(session, event);
      if (expectReject) {
        const key = result.outcome === 'rejected' ? `${result.origin}/${result.code}` : result.outcome;
        tally[key] = (tally[key] ?? 0) + 1;
        if (result.outcome !== 'rejected' || !result.atomic) unexpected += 1;
        rows.push({ scenario: scenario.id, event, outcome: key, atomic: result.atomic ?? null, message: result.message ?? null });
      } else if (result.outcome === 'accepted') accepted += 1;
      else { unexpected += 1; rows.push({ scenario: scenario.id, event, outcome: result.outcome, unexpected: true, message: result.message ?? null }); }
    }
  }
  session.free();
}
const report = {
  schema: 1,
  audit: 'glowcap-cr12-rejection-origins',
  runtime: { revision: buildInfo.revision, clean: buildInfo.clean, compiled: buildInfo.compiled, host: buildInfo.host, reactiveWasmSha256: sha256(wasm) },
  sourceSha256: sha256(source),
  expectedRejections: tally,
  acceptedDispatches: accepted,
  unexpected,
  rejections: rows,
};
await writeFile(new URL('./results-glowcap-origins.json', import.meta.url), `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({ expectedRejections: tally, acceptedDispatches: accepted, unexpected }));
if (unexpected) process.exitCode = 1;
