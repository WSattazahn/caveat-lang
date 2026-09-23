// Plays web/glowcap.html in a real browser: the "Change what you know" steps,
// then every label, the belief, the trust journal and the late-caveat feed must
// come through, with no console errors. Run after `npm run build`. SITE_URL
// tests a deployed copy instead; PLAYWRIGHT_CHANNEL=chrome uses an installed Chrome.
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
const browser = await chromium.launch(process.env.PLAYWRIGHT_CHANNEL ? { channel: process.env.PLAYWRIGHT_CHANNEL } : {});
try {
  const page = await browser.newPage({ viewport: { width: 1100, height: 1400 } });
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
  await page.goto(url);
  await page.waitForFunction(() => window.__glowcap !== undefined);

  // Change what you know: decide, learn something new, and see what changed,
  // why the decision reopened, and the original reasons kept.
  assert.equal(await page.locator('#change-learn').isDisabled(), true);
  assert.equal(await page.locator('#change-result').isVisible(), false);
  assert.equal(await page.locator('#change-question').isVisible(), false);
  await page.locator('#change-decide').click();
  assert.equal(await page.locator('#change-decision').innerText(), 'Decision: Started trusting glowing mushrooms because you absorbed the cave mushroom.');
  await page.locator('#change-learn').click();
  const changed = await page.locator('#change-diff').innerText();
  assert.match(changed, /Trust decision: trusting glowing mushrooms → reopened, not trusting/);
  assert.match(changed, /Belief: Probably safe → Uncertain/);
  assert.match(changed, /ruin mushroom: Probably a glowcap → Could be a duskcap — taste first/);
  assert.match(changed, /pool mushroom: Probably a glowcap → absorbed/);
  assert.equal(await page.locator('#change-why').innerText(), 'Stopped trusting glowing mushrooms because you absorbed the pool mushroom.');
  assert.match(await page.locator('#change-kept').innerText(), /^Started trusting glowing mushrooms because you absorbed the cave mushroom\.\s+Kept exactly as it was when the decision was made\.$/);
  assert.equal(await page.locator('#change-question').innerText(), 'What would your program need to learn to change its mind?');
  await page.locator('#change').screenshot({ path: path.join(results, 'glowcap-change.png') });
  // The guided steps use their own session: the garden below has not moved.
  assert.equal(await page.locator('#journal').innerText(), '');
  await page.locator('#change-again').click();
  assert.equal(await page.locator('#change-decide').isDisabled(), false);
  assert.equal(await page.locator('#change-result').isVisible(), false);
  assert.equal(await page.locator('#change-question').isVisible(), false);

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

  // On a phone the guided steps fit without sideways scrolling.
  const phone = await browser.newPage({ viewport: { width: 390, height: 844 } });
  phone.on('pageerror', (error) => errors.push(error.message));
  await phone.goto(url);
  await phone.waitForFunction(() => window.__glowcap !== undefined);
  await phone.locator('#change-decide').click();
  await phone.locator('#change-learn').click();
  assert.equal(await phone.locator('#change-question').isVisible(), true);
  assert.equal(await phone.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), true, 'no horizontal scroll on a phone');
  await phone.locator('#change').screenshot({ path: path.join(results, 'glowcap-change-phone.png') });
  assert.deepEqual(errors, []);
  console.log('Glowcap page plays through: "Change what you know", labels, belief, journal and late caveats render from the runtime.');
} finally {
  await browser.close();
  server?.kill();
}
