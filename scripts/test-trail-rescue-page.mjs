// Exercises the shipped UI against the Caveat WASM policy, using real buttons.
// Run after npm run build; SITE_URL may target an already-built deployment.
import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';
import { chromium } from 'playwright';

const root = fileURLToPath(new URL('../', import.meta.url));
const results = path.join(root, 'test-results');
const port = process.env.PORT || '4182';
const base = process.env.SITE_URL || `http://127.0.0.1:${port}/`;
const url = new URL('trail-rescue.html', base.endsWith('/') ? base : `${base}/`).href;
const storageKey = 'caveat.trail-rescue.save.v1';
let server;
let browser;

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
    server.once('error', (failure) => { clearTimeout(timeout); reject(failure); });
    server.once('exit', (code) => { clearTimeout(timeout); reject(new Error(`Server exited ${code}: ${output}`)); });
  });
}

const view = (page) => page.evaluate(() => window.__trailRescue.view());
const act = (page, action, tunnel) => page.locator(`[data-action="${action}"][data-tunnel="${tunnel}"]`).click();
const wait = (page, dt) => page.locator(`[data-action="wait"][data-dt="${dt}"]`).click();
const fresh = (page) => page.locator('#reset').click();

async function play(viewport, name) {
  const context = await browser.newContext({ viewport });
  try {
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', (failure) => errors.push(failure.message));
    page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
    await page.goto(url);
    await page.waitForFunction(() => window.__trailRescue !== undefined);
    assert.equal(await page.locator('#stone-status').textContent(), 'Unknown');
    assert.equal(await page.locator('#rescue').isDisabled(), true);
    assert.equal(await page.locator('#scouts').innerText(), '3');
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), true, `${name}: no horizontal overflow`);

    // A free, qualified report is enough to plan and attempt a rescue.
    await act(page, 'report', 'stone');
    assert.equal(await page.locator('#stone-status').textContent(), 'Clear');
    assert.match(await page.locator('#stone-explanation').innerText(), /Secondhand/);
    await act(page, 'plan', 'stone');
    assert.equal(await page.locator('#rescue').isDisabled(), false);
    await page.locator('#rescue').click();
    assert.equal((await view(page)).outcome, 'rescued');
    assert.equal(await page.locator('#decision-title').innerText(), 'The firefly is home.');
    assert.equal(await page.locator('[data-action="scout"][data-tunnel="stone"]').isDisabled(), true);

    // The chosen basis ages even when newer support keeps the route clear.
    await fresh(page);
    await act(page, 'scout', 'stone');
    await act(page, 'plan', 'stone');
    await wait(page, 5);
    await act(page, 'report', 'stone');
    for (let i = 0; i < 5; i += 1) await wait(page, 5);
    assert.equal((await view(page)).now, 30);
    assert.equal(await page.locator('#stone-status').textContent(), 'Clear');
    assert.equal(await page.locator('#decision-title').innerText(), 'Time to reconsider');
    assert.equal(await page.locator('#rescue').isDisabled(), true);
    assert.match(await page.locator('#evidence').innerText(), /Stale/);
    assert.deepEqual((await view(page)).decision.reopenedBy, ['scout_stone_1']);
    await act(page, 'plan', 'stone');
    assert.deepEqual((await view(page)).decision.basis, ['report_stone']);
    assert.equal(await page.locator('#history > li').count(), 3);
    await page.screenshot({ path: path.join(results, `trail-rescue-${name}.png`), fullPage: true });

    // Save is a local checkpoint; reload + Resume retains full ordered history.
    const savedView = await view(page);
    await page.locator('#save').click();
    await wait(page, 30);
    await page.reload();
    await page.waitForFunction(() => window.__trailRescue !== undefined);
    await page.locator('#resume').click();
    assert.deepEqual(await view(page), savedView);
    assert.match(await page.locator('#notice').innerText(), /resumed/);
    await page.locator('#rescue').click();
    assert.equal((await view(page)).outcome, 'rescued');

    // A bad stored save must leave the live expedition intact and show an error.
    await fresh(page);
    await act(page, 'scout', 'stone');
    const beforeInvalidRestore = await view(page);
    for (const invalid of ['{bad json', '{}']) {
      await page.evaluate(({ key, value }) => localStorage.setItem(key, value), { key: storageKey, value: invalid });
      await page.locator('#resume').click();
      assert.equal(await page.locator('#error').isVisible(), true);
      assert.deepEqual(await view(page), beforeInvalidRestore);
    }
    await act(page, 'plan', 'stone');
    assert.equal(await page.locator('#error').isVisible(), false);

    // A mistaken visitor can support a reasonable plan which nevertheless fails.
    await fresh(page);
    await page.locator('#reed-report').selectOption('clear');
    await act(page, 'report', 'reed');
    await act(page, 'plan', 'reed');
    await page.locator('#rescue').click();
    assert.equal((await view(page)).outcome, 'failed');
    assert.equal(await page.locator('#decision-title').innerText(), 'The rescue failed.');
    assert.match(await page.locator('#decision-description').innerText(), /reed tunnel was blocked/);
    assert.deepEqual((await view(page)).decision.caveats, ['secondhand']);

    // Both report choices and a hidden world change go through the same policy.
    await fresh(page);
    await page.locator('#stone-report').selectOption('blocked');
    await act(page, 'report', 'stone');
    await act(page, 'scout', 'stone');
    assert.equal(await page.locator('#stone-status').textContent(), 'Disputed');
    await page.locator('#changing-conditions > summary').click();
    const beforeChange = await view(page);
    await page.locator('[data-action="change"][data-tunnel="reed"][data-condition="clear"]').click();
    assert.deepEqual(await view(page), beforeChange, 'Hidden conditions do not leak into the view');
    await act(page, 'scout', 'reed');
    assert.equal(await page.locator('#reed-status').textContent(), 'Clear');
    assert.deepEqual(errors, [], `${name}: no browser errors`);
    console.log(`Trail Rescue ${name}: rescue, staleness, replan, failed rescue, save/resume, bad saves and hidden changes pass.`);
  } finally { await context.close(); }
}

try {
  await mkdir(results, { recursive: true });
  await startServer();
  browser = await chromium.launch(process.env.PLAYWRIGHT_CHANNEL ? { channel: process.env.PLAYWRIGHT_CHANNEL } : {});
  await play({ width: 1180, height: 1000 }, 'desktop');
  await play({ width: 390, height: 844 }, 'mobile');
} finally {
  await browser?.close();
  if (server && server.exitCode === null) {
    await new Promise((resolve) => {
      const timeout = setTimeout(resolve, 5000);
      server.once('exit', () => { clearTimeout(timeout); resolve(); });
      server.kill();
    });
  }
}
