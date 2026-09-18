// Regression coverage for failures before the application's module executes.
// Run after npm run assemble && npm run package:game.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { chromium } from 'playwright';

const dist = path.resolve(fileURLToPath(new URL('../dist/', import.meta.url)));
const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.cav': 'text/plain' };
const server = createServer(async (request, response) => {
  const name = new URL(request.url, 'http://localhost').pathname;
  try {
    const file = path.resolve(dist, `.${name}`);
    if (!file.startsWith(`${dist}${path.sep}`)) throw new Error('Invalid path');
    response.setHeader('Content-Type', types[path.extname(file)] || 'application/octet-stream');
    if (name === '/The-Last-Beacon.html') {
      // A file preview can allow classic JavaScript but reject data: imports.
      response.setHeader('Content-Security-Policy', "script-src 'self' 'unsafe-inline'; object-src 'none'");
    }
    response.end(await readFile(file));
  } catch { response.writeHead(404).end('Not found'); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const base = `http://127.0.0.1:${server.address().port}`;
let browser;
try {
  browser = await chromium.launch({ headless: true, args: ['--enable-unsafe-swiftshader'] });
  async function check(name, run, options = {}) {
    const context = await browser.newContext(options);
    const page = await context.newPage();
    page.setDefaultTimeout(30000);
    try { await run(page); console.log(`PASS ${name}`); }
    finally { await context.close(); }
  }

  await check('healthy launch is not replaced by the watchdog', async page => {
    await page.clock.install();
    await page.goto(`${base}/last-beacon.html`);
    await page.getByRole('button', { name: /Begin the watch/ }).waitFor();
    await page.clock.fastForward(25000);
    assert.equal(await page.getByRole('button', { name: /Begin the watch/ }).isVisible(), true);
    assert.equal(await page.locator('#app').getAttribute('data-launch-status'), null);
  });

  await check('failed static dependency produces a recovery action', async page => {
    await page.route('**/beacon-world.js', route => route.abort('failed'));
    await page.goto(`${base}/last-beacon.html`);
    await page.getByRole('button', { name: 'Try again', exact: true }).waitFor();
    assert.match(await page.locator('#loading-status').innerText(), /could not load|could not start/);
    assert.equal(await page.locator('#app').getAttribute('data-launch-status'), 'blocked');
  });

  await check('stalled module stops showing an indefinite preparation message', async page => {
    await page.route('**/last-beacon.js', () => {});
    await page.goto(`${base}/last-beacon.html`, { waitUntil: 'commit' });
    await page.getByRole('button', { name: 'Try again', exact: true }).waitFor();
    assert.match(await page.locator('#loading-status').innerText(), /has not started/);
  });

  await check('restricted standalone preview explains the block', async page => {
    await page.goto(`${base}/The-Last-Beacon.html`);
    await page.getByRole('button', { name: 'Try again', exact: true }).waitFor();
    assert.match(await page.locator('#loading-status').innerText(), /blocked|could not load|could not start/);
  });

  await check('disabled JavaScript leaves a readable browser instruction', async page => {
    await page.goto(`${base}/last-beacon.html`);
    assert.equal(await page.locator('noscript .error-box').isVisible(), true);
    assert.match(await page.locator('noscript .error-box').innerText(), /JavaScript is disabled/);
    assert.equal(await page.locator('.launch-help').isVisible(), true);
  }, { javaScriptEnabled: false });
} finally {
  await browser?.close();
  await new Promise(resolve => server.close(resolve));
}
