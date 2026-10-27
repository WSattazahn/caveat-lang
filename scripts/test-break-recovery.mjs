#!/usr/bin/env node
// break.html recovers from a WebAssembly trap: the page's first session
// creation is made to trap (as WebKit 26.0's tier-up fault did, see
// bench/carmack/README.md), and the page must load a fresh runtime, run
// again, and report the example's rules as holding rather than "does not
// load".
//
//   node scripts/test-break-recovery.mjs --runtime DIR [--browsers webkit,chromium]
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createServer } from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(fileURLToPath(new URL('../', import.meta.url)));
const arg = (name, fallback) => { const i = process.argv.indexOf(`--${name}`); return i < 0 ? fallback : process.argv[i + 1]; };
const runtimeDir = path.resolve(arg('runtime', 'dist/pkg-reactive'));
const browserNames = arg('browsers', 'webkit,chromium').split(',').map(name => name.trim());
const CALL = 'const ret = wasm.webreactivesession_new(ptr0, len0);';
const INJECT = "if (!globalThis.__caveatTrapInjected) { globalThis.__caveatTrapInjected = true; throw new WebAssembly.RuntimeError('injected trap'); }";

const glue = readFileSync(path.join(runtimeDir, 'caveat_runtime.js'), 'utf8');
assert.ok(glue.includes(CALL), 'the runtime glue no longer creates sessions the way this test expects');
const files = {
  '/': ['text/html', readFileSync(path.join(root, 'web/break.html'))],
  '/break.js': ['text/javascript', readFileSync(path.join(root, 'web/break.js'))],
  '/break-check.js': ['text/javascript', readFileSync(path.join(root, 'web/break-check.js'))],
  '/break-apartment.cav': ['text/plain', readFileSync(path.join(root, 'web/break-apartment.cav'))],
  '/caveat-session.js': ['text/javascript', readFileSync(path.join(root, 'kit/lib/session.mjs'))],
  // Every import of the glue, the fresh one after the trap included, gets the
  // injection; it fires once per page.
  '/pkg-reactive/caveat_runtime.js': ['text/javascript', glue.replace(CALL, `${INJECT} ${CALL}`)],
  '/pkg-reactive/caveat_runtime_bg.wasm': ['application/wasm', readFileSync(path.join(runtimeDir, 'caveat_runtime_bg.wasm'))],
};
const server = createServer((request, response) => {
  const file = files[new URL(request.url, 'http://localhost').pathname];
  if (!file) { response.writeHead(404).end(); return; }
  response.writeHead(200, { 'content-type': file[0] }).end(file[1]);
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const base = `http://127.0.0.1:${server.address().port}/`;

const playwright = await import('playwright');
try {
  for (const name of browserNames) {
    const browser = await playwright[name].launch();
    try {
      const page = await browser.newPage();
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      await page.goto(base);
      await page.waitForFunction(() => /Every rule held|failure|does not load|did not/.test(document.getElementById('status').textContent), null, { timeout: 30000 });
      const status = await page.locator('#status').innerText();
      assert.equal(await page.evaluate(() => globalThis.__caveatTrapInjected === true), true, `${name}: the trap was never injected`);
      assert.match(status, /^Every rule held on 6 events\.$/, `${name}: the page did not recover from the trap: ${status}`);
      assert.deepEqual(errors, [], `${name}: page errors`);
      console.log(`PASS ${name} ${browser.version()}: a trap at session creation was retried on a fresh runtime`);
    } finally {
      await browser.close();
    }
  }
} finally {
  server.close();
}
