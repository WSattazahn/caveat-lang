#!/usr/bin/env node
// Exercise the real break page and session wrapper with faults injected at the
// generated glue's WebAssembly call boundary. Every case uses a fresh page.
//
// node scripts/test-break-recovery.mjs --runtime DIR [--browsers webkit,chromium]
// --page-script FILE can test an earlier page implementation as a negative control.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createServer } from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(fileURLToPath(new URL('../', import.meta.url)));
const arg = (name, fallback) => { const i = process.argv.indexOf(`--${name}`); return i < 0 ? fallback : process.argv[i + 1]; };
const runtimeDir = path.resolve(arg('runtime', 'dist/pkg-reactive'));
const browserNames = arg('browsers', 'webkit,chromium').split(',').map(name => name.trim());
const cases = [
  { name: 'normal', operation: null, expected: 'success' },
  { name: 'creation-once', operation: 'new', at: 1, expected: 'success' },
  { name: 'plain-snapshot-once', operation: 'snapshot', at: 1, expected: 'success' },
  { name: 'probed-snapshot-once', operation: 'snapshot', at: 2, expected: 'success' },
  { name: 'mid-dispatch-once', operation: 'dispatch_outcome', at: 2, expected: 'success' },
  { name: 'creation-repeated', operation: 'new', at: 1, repeat: true, expected: 'runtime' },
  { name: 'probed-snapshot-repeated', operation: 'snapshot', at: 2, repeat: true, expected: 'runtime' },
  { name: 'mid-dispatch-repeated', operation: 'dispatch_outcome', at: 2, repeat: true, expected: 'runtime' },
  { name: 'cleanup-once', operation: 'free', at: 2, expected: 'success' },
  { name: 'cleanup-repeated', operation: 'free', at: 2, repeat: true, expected: 'runtime' },
  { name: 'reload-failure', operation: 'new', at: 1, expected: 'reload' },
  { name: 'invalid-source', operation: null, expected: 'load' },
  { name: 'stale-reload-failure', race: 'failure', operation: 'new', at: 1 },
  { name: 'stale-reload-success', race: 'success', operation: 'new', at: 1 },
  { name: 'stale-example-fetch', race: 'example', operation: null },
  { name: 'edited-input-cancels-retry', race: 'edit', operation: 'new', at: 1 },
];
const requestedCases = arg('cases', null)?.split(',');
if (requestedCases) assert.ok(requestedCases.every(name => cases.some(test => test.name === name)), 'Unknown recovery case');
const selected = requestedCases ? cases.filter(test => requestedCases.includes(test.name)) : cases;

let glue = readFileSync(path.join(runtimeDir, 'caveat_runtime.js'), 'utf8');
for (const method of ['new', 'snapshot', 'snapshot_compact', 'dispatch_outcome']) {
  const operation = method === 'snapshot_compact' ? 'snapshot' : method;
  const call = new RegExp(`const ret = wasm\\.webreactivesession_${method}\\([^;]+;`, 'g');
  const matches = glue.match(call);
  assert.equal(matches?.length, 1, `Expected one generated ${method} call boundary`);
  glue = glue.replace(call, match => `__caveatRecoveryCall('${operation}'); ${match}`);
}
const freeCall = 'wasm.__wbg_webreactivesession_free(ptr, 0);';
assert.equal(glue.split(freeCall).length, 2, 'Expected one explicit generated free call boundary');
glue = glue.replace(freeCall, `__caveatRecoveryCall('free'); ${freeCall}`);
glue = `
const __caveatRecoveryState = globalThis.__caveatRecovery;
const __caveatRecoveryInstance = ++__caveatRecoveryState.instances;
const __caveatRecoveryCounts = {};
function __caveatRecoveryCall(operation) {
  const ordinal = __caveatRecoveryCounts[operation] = (__caveatRecoveryCounts[operation] || 0) + 1;
  const call = { instance: __caveatRecoveryInstance, operation, ordinal };
  __caveatRecoveryState.calls.push(call);
  const plan = __caveatRecoveryState.plan;
  if (operation === plan.operation && ordinal === plan.at && (plan.repeat || __caveatRecoveryInstance === 1)) {
    __caveatRecoveryState.traps.push(call);
    throw new WebAssembly.RuntimeError('injected ' + operation + ' trap');
  }
}
` + glue;
// Observe cleanup without changing the real session wrapper's behavior.
const session = readFileSync(path.join(root, 'kit/lib/session.mjs'), 'utf8') + `
const __caveatRecoveryClose = CaveatSession.prototype.close;
CaveatSession.prototype.close = function () {
  const row = { instance: globalThis.__caveatRecovery.instances, before: this.state };
  try { return __caveatRecoveryClose.call(this); }
  finally { row.after = this.state; globalThis.__caveatRecovery.closes.push(row); }
};
const __caveatRecoveryLoad = loadRuntime;
loadRuntime = async function (options) {
  const state = globalThis.__caveatRecovery, ordinal = ++state.loadCalls;
  try { return await __caveatRecoveryLoad(options); }
  finally { state.settledLoads.push(ordinal); }
};
`;
const files = {
  '/': ['text/html', readFileSync(path.join(root, 'web/break.html'))],
  '/break.js': ['text/javascript', readFileSync(path.resolve(arg('page-script', path.join(root, 'web/break.js'))))],
  '/break-check.js': ['text/javascript', readFileSync(path.join(root, 'web/break-check.js'))],
  '/break-guard.cav': ['text/plain', readFileSync(path.join(root, 'web/break-guard.cav'))],
  '/break-apartment.cav': ['text/plain', readFileSync(path.join(root, 'web/break-apartment.cav'))],
  '/caveat-session.js': ['text/javascript', session],
  '/pkg-reactive/caveat_runtime.js': ['text/javascript', glue],
  '/pkg-reactive/caveat_runtime_bg.wasm': ['application/wasm', readFileSync(path.join(runtimeDir, 'caveat_runtime_bg.wasm'))],
};
const playwright = await import('playwright');
const server = createServer((request, response) => {
  const file = files[new URL(request.url, 'http://localhost').pathname];
  if (!file) { response.writeHead(404).end(); return; }
  response.writeHead(200, { 'content-type': file[0] }).end(file[1]);
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const base = `http://127.0.0.1:${server.address().port}/`;

async function installPlan(page, plan) {
  await page.addInitScript(plan => {
    const state = globalThis.__caveatRecovery = { plan, instances: 0, calls: [], traps: [], closes: [], loadCalls: 0, settledLoads: [], sourceTexts: [] };
    const originalFetch = globalThis.fetch;
    globalThis.fetch = async (...args) => {
      const response = await originalFetch(...args);
      if (new URL(response.url).pathname.endsWith('.cav')) {
        const originalText = response.text.bind(response);
        response.text = async () => {
          try { return await originalText(); }
          finally { state.sourceTexts.push(new URL(response.url).pathname); }
        };
      }
      return response;
    };
  }, plan);
}
const waitStatus = (page, expected) => page.waitForFunction(expected => document.getElementById('status').textContent === expected, expected, { timeout: 30000 });
async function staleCompletionCase(page, test) {
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await installPlan(page, test);
  let nextRoute;
  const waitingRoute = () => new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('Expected delayed request did not arrive')), 30000);
    nextRoute = route => { clearTimeout(timer); resolve(route); };
  });
  const pattern = test.race === 'example' ? '**/break-guard.cav' : '**/caveat_runtime.js?caveat-instance=*';
  await page.route(pattern, route => { assert.ok(nextRoute, 'Unexpected delayed request'); const accept = nextRoute; nextRoute = null; accept(route); });
  let older;
  if (test.race === 'example') {
    await page.goto(base);
    await waitStatus(page, 'Every rule held on 6 events.');
    const held = waitingRoute();
    await page.locator('[data-example="guard"]').click();
    older = await held;
    await page.locator('[data-example="apartment"]').click();
    await page.waitForFunction(() => globalThis.__caveatRecovery.sourceTexts.filter(url => url.endsWith('/break-apartment.cav')).length === 2, null, { timeout: 30000 });
    await waitStatus(page, 'Every rule held on 6 events.');
  } else {
    const held = waitingRoute();
    await page.goto(base);
    older = await held;
    if (test.race === 'edit') {
      await page.locator('#events').fill('{"event":"gather"}');
      await waitStatus(page, 'Inputs changed; run the checks again.');
    } else {
      const heldNewer = waitingRoute();
      if (test.race === 'success') await page.locator('[data-example="guard"]').click();
      else await page.locator('#run').click();
      const newer = await heldNewer;
      await newer.continue();
      await waitStatus(page, test.race === 'success' ? 'Every rule held on 4 events.' : 'Every rule held on 6 events.');
    }
  }
  const current = await page.evaluate(() => ({ status: document.getElementById('status').textContent, source: document.getElementById('program').value, events: document.getElementById('events').value, calls: [...globalThis.__caveatRecovery.calls] }));
  if (test.race === 'failure') await older.fulfill({ status: 503, contentType: 'text/plain', body: 'Older reload fails after the newer run completed' });
  else await older.continue();
  if (test.race === 'example') await page.waitForFunction(() => globalThis.__caveatRecovery.sourceTexts.some(url => url.endsWith('/break-guard.cav')), null, { timeout: 30000 });
  else await page.waitForFunction(() => globalThis.__caveatRecovery.settledLoads.includes(2), null, { timeout: 30000 });
  // The observed promise settles before its production continuation. A browser
  // task barrier lets that continuation run before checking that it did nothing.
  await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 0)));
  const after = await page.evaluate(() => ({ status: document.getElementById('status').textContent, source: document.getElementById('program').value, events: document.getElementById('events').value, calls: [...globalThis.__caveatRecovery.calls] }));
  assert.deepEqual(after, current, test.name + ': an older completion changed the latest action or executed another run');
  assert.deepEqual(errors, [], test.name + ': an older completion escaped as a page error');
}

const failures = [];
try {
  for (const name of browserNames) {
    assert.ok(playwright[name]?.launch, `Unknown browser ${name}`);
    const browser = await playwright[name].launch();
    try {
      for (const test of selected) {
        const page = await browser.newPage();
        try {
          if (test.race) { await staleCompletionCase(page, test); console.log(`PASS ${name} ${browser.version()}: ${test.name}`); continue; }
          const errors = [], reloads = [];
          page.on('pageerror', error => errors.push(error.message));
          page.on('request', request => { if (new URL(request.url()).searchParams.has('caveat-instance')) reloads.push(request.url()); });
          await installPlan(page, test);
          if (test.expected === 'reload') await page.route('**/caveat_runtime.js?caveat-instance=*', route => route.fulfill({ status: 503, contentType: 'text/plain', body: 'Injected runtime reload failure' }));
          if (test.expected === 'load') await page.route('**/break-apartment.cav', route => route.fulfill({ status: 200, contentType: 'text/plain', body: 'not_a_caveat_statement;' }));
          await page.goto(base);
          await page.waitForFunction(() => /^(Every rule held|\d+ rule failures? found\.|The program does not load:|The runtime did not load:|The runtime did not reload:|The runtime failed after|The run did not complete:)/.test(document.getElementById('status').textContent), null, { timeout: 30000 });
          const status = await page.locator('#status').innerText();
          const observed = await page.evaluate(() => globalThis.__caveatRecovery);
          assert.equal(observed.traps.length, test.operation ? test.repeat ? 2 : 1 : 0, `${test.name}: wrong injected trap count`);
          assert.equal(reloads.length, test.operation ? 1 : 0, `${test.name}: retry must load exactly one fresh runtime`);
          assert.equal(observed.instances, test.operation && test.expected !== 'reload' ? 2 : 1, `${test.name}: wrong runtime instance count`);
          if (test.expected === 'success') {
            assert.equal(status, 'Every rule held on 6 events.', `${test.name}: complete replay required`);
            assert.equal(await page.locator('#results tr').count(), 7, `${test.name}: all six event rows required`);
            assert.equal((await page.locator('#results').innerText()).includes('fatal'), false, `${test.name}: successful retry retained a fatal row`);
          } else if (test.expected === 'runtime') {
            assert.match(status, /^The (run did not complete|runtime failed after a fresh-instance retry):/, `${test.name}: exhausted retry must remain a runtime failure`);
            assert.equal(status.includes('Every rule held'), false);
            if (test.operation === 'dispatch_outcome') assert.match(await page.locator('#results').innerText(), /fatal: injected dispatch_outcome trap/);
          } else if (test.expected === 'reload') assert.match(status, /^The runtime did not reload:/);
          else assert.match(status, /^The program does not load:/);
          assert.deepEqual(errors, [], `${test.name}: page errors escaped the recovery path`);
          assert.ok(observed.closes.every(close => close.after === 'closed'), `${test.name}: cleanup left a session open`);
          const expectedFirstCloses = test.operation === 'new' || test.expected === 'load' ? 0 : test.operation === 'snapshot' && test.at === 1 ? 1 : 2;
          assert.equal(observed.closes.filter(close => close.instance === 1).length, expectedFirstCloses, `${test.name}: every first-instance session must be closed once`);
          if (observed.instances === 2) assert.equal(observed.closes.filter(close => close.instance === 2).length, test.repeat ? expectedFirstCloses : 2, `${test.name}: every retry session must be closed once`);
          console.log(`PASS ${name} ${browser.version()}: ${test.name}`);
        } catch (error) {
          failures.push(`${name}/${test.name}: ${error.message}`);
          console.error(`FAIL ${name}/${test.name}: ${error.message}`);
        } finally { await page.close(); }
      }
    } finally { await browser.close(); }
  }
  assert.deepEqual(failures, [], 'Recovery cases failed');
} finally { server.close(); }
