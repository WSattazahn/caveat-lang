// Runs the developer kit's session library and scenario runner inside a real
// browser: load the WebAssembly runtime from URLs, dispatch, refuse bad input
// without changing state, save and restore, read elapsed(), and run scenario
// files with fetch. Run after `npm run build`.
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
  for (const check of ['accepted', 'inputRefusalKeepsState', 'malformedKeepsState', 'payloadRefused', 'restoreMatches', 'resumedAgrees', 'elapsed', 'scenariosPass', 'scenarioFailureReported', 'sharedTrap', 'freshAfterTrap']) {
    assert.equal(results[check], true, check);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const outcome = await checkKitInBrowser({ root, kit: '/kit/lib/', runtime: '/dist/pkg-reactive/', examples: '/examples/' });
  assertBrowserResults(outcome);
  console.log(`Kit browser checks pass in ${outcome.browser}: session library, runtime from URLs, elapsed(), and scenario files run with fetch.`);
}
