// Runs the developer kit's session library and scenario runner inside a real
// browser: load the WebAssembly runtime from URLs, dispatch, and dispatch for a
// view, refuse bad input without changing state, save and restore, read
// elapsed(), and run scenario files with fetch. Run after `npm run build`.
//
//   node scripts/test-kit-browser.mjs      PLAYWRIGHT_CHANNEL=chrome uses an installed Chrome
//
// checkKitInBrowser() is reused by test-kit-package.mjs against the packed kit.
import assert from 'node:assert/strict';
import { createReadStream } from 'node:fs';
import { stat } from 'node:fs/promises';
import { createServer } from 'node:http';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { chromium } from 'playwright';

const types = {
  '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.mjs': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8', '.cav': 'text/plain; charset=utf-8', '.wasm': 'application/wasm',
};

export const CLOCK_SOURCE = 'scene "Clock probe.";\nevent advance dt min 0 max 3600;\nclock advance every 1;\nbind hud.elapsed = elapsed();\n';

const NEUTRAL_SOURCE = `evidence memory from "lookup"; caveat stale consequence material;
  state score = 0; event consult; event age;
  on consult reveal memory; on consult set score = qualified(80, memory);
  on age qualify memory with stale;
  bind hud.seen = observed(memory); bind hud.score = score because score;`;

const GROUNDS_SOURCE = `evidence chart from "chart"; evidence other from "independent observation";
  caveat age consequence low; caveat unrelated consequence material; age qualifies chart;
  state basis = 0; state narrow = 0; event consult;
  on consult reveal chart; on consult reveal other;
  on consult set basis = qualified(1, chart);
  on consult set narrow = basis because nothing;
  on consult commit act because enough using basis;`;

const ARCHIVE_SOURCE = `claim seen; evidence glimpse from "glimpse";
  readings s from glimpse window 1;
  state left = 0; state right = 0; state chosen = 0;
  event look v min 1 max 5; event pick;
  on look sample s = v supports seen;
  on look when v < 5 and v != 3 and latest(s) > 0 set left = left;
  on look when v < 5 and v != 2 and latest(s) > 0 set right = right;
  on pick set chosen = right;
  bind hud.chosen = chosen;
  bind hud.union = left + right;`;

// Executed in the page. The paths are URL prefixes on the test server.
function pageCheck({ kit, runtime: runtimeBase, examples, clockSource }) {
  return `
const results = {};
try {
  const { loadRuntime } = await import(${JSON.stringify(`${kit}session.mjs`)});
  const { parseScenarioFile, runScenarioFile } = await import(${JSON.stringify(`${kit}scenarios.mjs`)});
  const moduleUrl = new URL(${JSON.stringify(`${runtimeBase}caveat_runtime.js`)}, location.href).href;
  const wasmUrl = new URL(${JSON.stringify(`${runtimeBase}caveat_runtime_bg.wasm`)}, location.href);
  const runtime = await loadRuntime({ module: moduleUrl, wasm: wasmUrl });
  const text = async url => {
    const response = await fetch(url);
    if (!response.ok) throw new Error(url + ' ' + response.status);
    return response.text();
  };
  const exampleUrl = new URL(${JSON.stringify(`${examples}thermostat_history.cav`)}, location.href);
  const source = await text(exampleUrl);

  const session = runtime.open(source);
  const accepted = session.dispatch('read', { value: 17 });
  results.accepted = accepted.outcome === 'accepted' && accepted.snapshot.bindings.heating.text === '100%';
  const before = session.save();
  const refused = session.dispatch('read', { value: 41 });
  results.inputRefusalKeepsState = refused.origin === 'input' && refused.code === 'bound_exceeded' && session.save() === before;
  const malformed = session.dispatch('read', { value: 'x' });
  results.malformedKeepsState = malformed.code === 'payload_invalid' && session.save() === before;
  try { session.dispatch('read', { value: NaN }); results.payloadRefused = false; } catch (error) { results.payloadRefused = error.kind === 'payload'; }
  // dispatchView: the same events, the view in place of the snapshot.
  const viewed = runtime.open(source);
  const shown = viewed.dispatchView('read', { value: 17 });
  results.dispatchViewAccepted = shown.outcome === 'accepted' && !Object.hasOwn(shown, 'snapshot')
    && JSON.stringify(shown.view) === JSON.stringify(viewed.view()) && JSON.stringify(shown.view) === JSON.stringify(session.view())
    && viewed.save() === before;
  results.dispatchViewRefusal = JSON.stringify(viewed.dispatchView('read', { value: 41 })) === JSON.stringify(refused) && viewed.save() === before;
  viewed.close();
  const resumed = runtime.restore(source, session.save());
  results.restoreMatches = JSON.stringify(resumed.snapshot()) === JSON.stringify(session.snapshot());
  results.resumedAgrees = JSON.stringify(resumed.dispatch('read', { value: 25 })) === JSON.stringify(session.dispatch('read', { value: 25 }));
  session.close();
  resumed.close();

  const clock = runtime.open(${JSON.stringify(clockSource)});
  clock.dispatch('advance', { dt: 2.5 });
  const clockRestored = runtime.restore(${JSON.stringify(clockSource)}, clock.save());
  results.elapsed = clock.view().bindings.hud.elapsed === 2.5 && clockRestored.snapshot().elapsed === 2.5;
  clock.close();
  clockRestored.close();

  const neutral = runtime.open(${JSON.stringify(NEUTRAL_SOURCE)});
  const neutralView = neutral.dispatchView('consult');
  neutral.dispatch('age');
  const neutralRestored = runtime.restore(${JSON.stringify(NEUTRAL_SOURCE)}, neutral.save());
  const { explain, dependents } = await import(${JSON.stringify(`${kit}explain.mjs`)});
  const neutralEvidence = explain(neutralRestored.snapshot()).evidence;
  results.neutralObservation = neutralView.outcome === 'accepted' && neutralView.view.bindings.hud.seen
    && JSON.stringify(neutralRestored.snapshot()) === JSON.stringify(neutral.snapshot())
    && neutralEvidence.length === 1 && neutralEvidence[0].relation === null
    && neutralEvidence[0].claim === null && neutralEvidence[0].caveats.includes('stale');
  neutral.close();
  neutralRestored.close();

  // Runtime-produced sparse roots, transferred after departure and consumed
  // by the pure browser archive verifier (no Node crypto/runtime fallback).
  const archived = runtime.open(${JSON.stringify(ARCHIVE_SOURCE)});
  for (let v = 1; v <= 5; v++) archived.dispatch('look', { v });
  const archive = archived.drainArchive();
  archived.dispatch('pick');
  archive.push(...archived.drainArchive());
  const archiveRestored = runtime.restore(${JSON.stringify(ARCHIVE_SOURCE)}, archived.save());
  const archiveSnapshot = archiveRestored.snapshot();
  const display = explain(archiveSnapshot, [], { archive }).displayed;
  const chosen = display.find(item => item.name === 'hud.chosen').lineage.departed[0];
  const joined = display.find(item => item.name === 'hud.union').lineage.departed[0];
  results.archiveTransfer = archive.some(item => item.kind === 'provenance')
    && chosen.archive_status === 'complete' && JSON.stringify(chosen.records) === JSON.stringify(['s@1', 's@3', 's@4'])
    && JSON.stringify(joined.records) === JSON.stringify(['s@1', 's@2', 's@3', 's@4'])
    && dependents(archiveSnapshot, 's@3', { archive }).values.some(item => item.name === 'chosen' && item.basis === 'lineage')
    && !dependents(archiveSnapshot, 's@2', { archive }).values.some(item => item.name === 'chosen')
    && explain(archiveSnapshot).displayed.find(item => item.name === 'hud.chosen').lineage.departed[0].archive_status === 'unavailable'
    && archiveRestored.undrained === 0;
  archived.close(); archiveRestored.close();

  // Exercise the restore trust boundary through the public kit and actual WASM.
  // Every added name is valid and observed/declared; only subset inclusion fails.
  const groundsSource = ${JSON.stringify(GROUNDS_SOURCE)};
  const grounded = runtime.open(groundsSource);
  grounded.dispatch('consult');
  const groundedSave = grounded.save();
  const validGrounds = runtime.restore(groundsSource, groundedSave);
  results.restoreNarrowedGrounds = JSON.stringify(validGrounds.snapshot()) === JSON.stringify(grounded.snapshot())
    && validGrounds.snapshot().value_grounds.narrow.evidence.length === 0;
  validGrounds.close();
  const alterations = [
    ['state basis grounds include evidence other outside its lineage', saved => {
      saved.states.basis.grounds = { evidence: ['chart', 'other'], caveats: ['age'] };
    }],
    ['state basis grounds include caveat unrelated outside its lineage', saved => {
      saved.states.basis.grounds = { evidence: ['chart'], caveats: ['age', 'unrelated'] };
    }],
    ['commitment act grounds include evidence other outside its lineage', saved => {
      saved.commitment_grounds.act.evidence = ['chart', 'other'];
      saved.decision_journal[0].because = ['chart', 'other'];
    }],
    ['commitment act grounds include caveat unrelated outside its lineage', saved => {
      saved.commitment_grounds.act.caveats = ['age', 'unrelated'];
      saved.decision_journal[0].caveats = ['age', 'unrelated'];
    }],
  ];
  results.restoreGroundsRejected = true;
  for (const [expected, alter] of alterations) {
    const saved = JSON.parse(groundedSave);
    alter(saved);
    let refused = false;
    try {
      const unexpected = runtime.restore(groundsSource, JSON.stringify(saved));
      unexpected.close();
    } catch (error) {
      refused = error.kind === 'restore' && error.message.includes(expected);
    }
    if (!refused) throw new Error('restore failed to refuse: ' + expected);
  }
  grounded.close();

  const fileUrl = new URL(${JSON.stringify(`${examples}thermostat_history.scenarios.json`)}, location.href);
  const doc = parseScenarioFile(await text(fileUrl));
  const readSource = relative => text(new URL(relative, fileUrl));
  const passing = await runScenarioFile(doc, { runtime, readSource });
  results.scenariosPass = passing.failed === 0 && passing.passed === doc.scenarios.length;
  const altered = JSON.parse(JSON.stringify(doc));
  altered.scenarios[0].steps[4].expect['/bindings/heating/text'] = '50%';
  const failing = await runScenarioFile(altered, { runtime, readSource });
  const failure = failing.scenarios[0].failure;
  results.scenarioFailureReported = failing.failed === 1 && failure.kind === 'expect' && failure.path === '/bindings/heating/text' && failure.actual === '0%';

  // Two loads of one module share its instance, so a trap reached through one
  // stops the other; loading again afterwards gives a fresh, working instance.
  const again = await loadRuntime({ module: moduleUrl, wasm: wasmUrl });
  const one = runtime.open(source);
  const two = again.open(source);
  const { prototype } = (await import(moduleUrl)).WebReactiveSession;
  const original = prototype.dispatch_outcome;
  prototype.dispatch_outcome = () => { throw new WebAssembly.RuntimeError('unreachable'); };
  try { one.dispatch('read', { value: 17 }); } catch { /* the trap */ }
  prototype.dispatch_outcome = original;
  let siblingRefused = false;
  try { two.snapshot(); } catch (error) { siblingRefused = error.kind === 'fatal'; }
  results.sharedTrap = runtime.trapped && again.trapped && siblingRefused;
  const fresh = await loadRuntime({ module: moduleUrl, wasm: wasmUrl });
  const three = fresh.open(source);
  results.freshAfterTrap = !fresh.trapped && three.dispatch('read', { value: 17 }).outcome === 'accepted' && runtime.trapped;
  three.close();
  results.userAgent = navigator.userAgent;
} catch (error) {
  results.error = String(error && error.stack || error);
}
window.__kitResult = results;
`;
}

function serve(root, check) {
  const base = path.resolve(root);
  const server = createServer(async (request, response) => {
    try {
      const pathname = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
      if (request.method !== 'GET') { response.writeHead(405).end(); return; }
      if (pathname === '/') { response.writeHead(200, { 'Content-Type': types['.html'] }).end('<!doctype html><title>kit</title><link rel="icon" href="data:,"><script type="module" src="/__kit-check.mjs"></script>'); return; }
      if (pathname === '/__kit-check.mjs') { response.writeHead(200, { 'Content-Type': types['.mjs'] }).end(check); return; }
      const file = path.resolve(base, `.${pathname}`);
      if (!file.startsWith(base + path.sep) || !(await stat(file)).isFile()) { response.writeHead(404).end(); return; }
      response.writeHead(200, { 'Content-Type': types[path.extname(file)] ?? 'application/octet-stream' });
      createReadStream(file).pipe(response);
    } catch {
      response.writeHead(404).end();
    }
  });
  return new Promise(resolve => server.listen(0, '127.0.0.1', () => resolve(server)));
}

// root: directory served at /. kit, runtime, examples: URL prefixes under it.
export async function checkKitInBrowser({ root, kit, runtime, examples, channel = process.env.PLAYWRIGHT_CHANNEL || undefined }) {
  const server = await serve(root, pageCheck({ kit, runtime, examples, clockSource: CLOCK_SOURCE }));
  const browser = await chromium.launch(channel ? { channel } : {});
  const problems = [];
  try {
    const page = await browser.newPage();
    page.on('console', message => { if (message.type() === 'error') problems.push(`console: ${message.text()}`); });
    page.on('pageerror', error => problems.push(`page: ${error.message}`));
    page.on('response', response => { if (response.status() >= 400) problems.push(`${response.status()} ${new URL(response.url()).pathname}`); });
    // A page error (such as a syntax error in the check) fails at once rather
    // than waiting for the timeout.
    const pageFailed = new Promise((_, reject) => page.on('pageerror', error => reject(new Error(`page error: ${error.message}`))));
    pageFailed.catch(() => {});
    await page.goto(`http://127.0.0.1:${server.address().port}/`);
    await Promise.race([page.waitForFunction(() => window.__kitResult, null, { timeout: 60_000 }), pageFailed]);
    const results = await page.evaluate(() => window.__kitResult);
    return { results, problems, browser: browser.version() };
  } finally {
    await browser.close();
    server.close();
  }
}

export function assertBrowserResults({ results, problems }) {
  assert.equal(results.error, undefined, results.error);
  assert.deepEqual(problems, []);
  for (const check of ['accepted', 'inputRefusalKeepsState', 'malformedKeepsState', 'payloadRefused', 'dispatchViewAccepted', 'dispatchViewRefusal', 'restoreMatches', 'resumedAgrees', 'elapsed', 'neutralObservation', 'archiveTransfer', 'restoreNarrowedGrounds', 'restoreGroundsRejected', 'scenariosPass', 'scenarioFailureReported', 'sharedTrap', 'freshAfterTrap']) {
    assert.equal(results[check], true, check);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const outcome = await checkKitInBrowser({ root, kit: '/kit/lib/', runtime: '/dist/pkg-reactive/', examples: '/examples/' });
  assertBrowserResults(outcome);
  console.log(`Kit browser checks pass in ${outcome.browser}: session library, runtime from URLs, elapsed(), and scenario files run with fetch.`);
}
