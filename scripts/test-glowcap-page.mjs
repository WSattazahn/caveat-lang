// Plays web/glowcap.html in a real browser: every label, the belief, the trust
// journal and the late-caveat feed must come through, with no console errors.
// Run after `npm run build`. SITE_URL tests a deployed copy instead.
import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';
import { chromium } from 'playwright';

const root = fileURLToPath(new URL('../', import.meta.url));
const results = path.join(root, 'test-results');
const port = process.env.PORT || '4178';
const base = process.env.SITE_URL || `http://127.0.0.1:${port}/`;
const url = new URL('glowcap.html', base.endsWith('/') ? base : `${base}/`).href;
let server;

async function startServer() {
  if (process.env.SITE_URL) return;
  server = spawn(process.execPath, ['scripts/serve.mjs'], {
    cwd: root, env: { ...process.env, PORT: port }, stdio: ['ignore', 'pipe', 'pipe'],
  });
  let output = '';
  await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error(`Server did not start: ${output}`)), 10000);
    const collect = (chunk) => {
      output += chunk;
      if (output.includes('CAVEAT games:')) { clearTimeout(timeout); resolve(); }
    };
    server.stdout.on('data', collect);
    server.stderr.on('data', collect);
    server.once('exit', (code) => { clearTimeout(timeout); reject(new Error(`Server exited ${code}: ${output}`)); });
  });
}

const label = (page, id) => page.locator(`[data-id="${id}"] .label`).innerText();
const why = (page, id) => page.locator(`[data-id="${id}"] .why`).innerText();
const click = (page, act, id) => page.locator(`button[data-act="${act}"][data-id="${id}"]`).click();

await mkdir(results, { recursive: true });
await startServer();
const browser = await chromium.launch();
try {
  const page = await browser.newPage({ viewport: { width: 1100, height: 1400 } });
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
  await page.goto(url);
  await page.waitForFunction(() => window.__glowcap !== undefined);

  assert.equal(await label(page, 'pool'), 'Glowing mushroom');
  assert.match(await why(page, 'pool'), /nothing is claimed/);

  await click(page, 'absorb', 'cave');
  assert.equal(await label(page, 'pool'), 'Probably a glowcap');
  assert.match(await why(page, 'pool'), /You absorbed the cave mushroom\./);
  assert.match(await page.locator('#journal').innerText(), /Started trusting glowing mushrooms/);
  assert.equal(await page.locator('button[data-act="absorb"][data-id="cave"]').isDisabled(), true);

  await click(page, 'absorb', 'pool');
  assert.equal(await label(page, 'ruin'), 'Could be a duskcap — taste first');
  assert.match(await why(page, 'ruin'), /You absorbed the pool mushroom\./);
  assert.match(await page.locator('#journal').innerText(), /Stopped trusting glowing mushrooms/);
  assert.equal(await page.locator('#belief-title').innerText(), 'Uncertain');

  await page.locator('#wait60').click();
  await click(page, 'taste', 'ruin');
  assert.equal(await label(page, 'ruin'), 'Probably a glowcap (tasted in the dark)');
  assert.match(await why(page, 'ruin'), /Tasted in the dark/);

  await page.locator('#wait60').click();
  assert.equal(await label(page, 'ruin'), 'Probably a glowcap (taste has faded)');
  assert.match(await why(page, 'ruin'), /The taste has faded from memory\./);
  assert.match(await page.locator('#feed').innerText(), /A caveat arrived late/);

  await click(page, 'absorb', 'ruin');
  assert.match(await page.locator('#journal').innerText(), /Trusted glowing mushrooms again/);

  await page.screenshot({ path: path.join(results, 'glowcap-page.png'), fullPage: true });
  assert.deepEqual(errors, []);
  console.log('Glowcap page plays through: labels, belief, journal and late caveats render from the runtime.');
} finally {
  await browser.close();
  server?.kill();
}
