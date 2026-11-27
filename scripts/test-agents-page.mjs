// Plays web/agents.html in a real browser: three agents decide on one shared
// ledger, wrong-agent sends are refused, withdrawing security's stale mirror
// reopens only the decisions that rested on it while leaving comms in force,
// and live explain/dependents match. Run after `npm run build`.
// SITE_URL tests a deployed copy instead; PLAYWRIGHT_CHANNEL=chrome uses installed Chrome.
import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';
import { chromium } from 'playwright';

const root = fileURLToPath(new URL('../', import.meta.url));
const results = path.join(root, 'test-results');
const port = process.env.PORT || '4183';
const base = process.env.SITE_URL || `http://127.0.0.1:${port}/`;
const url = new URL('agents.html', base.endsWith('/') ? base : `${base}/`).href;
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

const cardStatus = (page, series) => page.locator(`[data-series="${series}"] [data-role="status"]`).innerText();
const cardGrounds = (page, series) => page.locator(`[data-series="${series}"] [data-role="grounds"]`).innerText();

await mkdir(results, { recursive: true });
await startServer();
const browser = await chromium.launch(process.env.PLAYWRIGHT_CHANNEL ? { channel: process.env.PLAYWRIGHT_CHANNEL } : {});
try {
  const page = await browser.newPage({ viewport: { width: 1180, height: 1000 } });
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
  page.on('response', (response) => {
    if (response.status() >= 400) errors.push(`HTTP ${response.status()} ${response.url()}`);
  });

  await page.goto(url);
  await page.waitForFunction(() => window.__agents?.ready === true);
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), true, 'no horizontal scroll on desktop');

  // Out-of-order guided actions must report the runtime refusal.
  await page.locator('#step-withdraw').click();
  assert.match(await page.locator('#status').innerText(), /Refused retract_advisory.*No advisory reading/);
  assert.equal(await page.evaluate(() => window.__agents.sequence()), 0);
  assert.equal(await cardStatus(page, 'sec_signoff'), 'not decided');

  // Step 1: all three agents add claims and commit.
  await page.locator('#step-decide').click();
  assert.equal(await cardStatus(page, 'sec_signoff'), 'in force');
  assert.equal(await cardStatus(page, 'canary_deploy'), 'in force');
  assert.equal(await cardStatus(page, 'release_notes'), 'in force');

  const explainBefore = await page.locator('#explain-out').innerText();
  assert.match(explainBefore, /by security/);
  assert.match(explainBefore, /by perf/);
  assert.match(explainBefore, /by comms/);
  assert.match(explainBefore, /cached_mirror/);

  // Live dependents pre-withdrawal: cve_mirror reaches sec_signoff and canary_deploy, not release_notes.
  const depBefore = await page.locator('#dependents-out').innerText();
  assert.match(depBefore, /sec_signoff@1/);
  assert.match(depBefore, /canary_deploy@1/);
  assert.doesNotMatch(depBefore, /release_notes@1/);

  // Step 2: wrong-agent refusal changes nothing.
  await page.locator('#step-wrong-agent').click();
  assert.match(await page.locator('#status').innerText(), /wrong_agent/);
  assert.equal(await cardStatus(page, 'sec_signoff'), 'in force');
  assert.match(await page.locator('#explain-out').innerText(), /approve_security by comms.*refused/);

  // Step 3: security withdraws stale advisory -> selective reopening.
  await page.locator('#step-withdraw').click();
  assert.equal(await cardStatus(page, 'sec_signoff'), 'reopened');
  assert.equal(await cardStatus(page, 'canary_deploy'), 'reopened');
  assert.equal(await cardStatus(page, 'release_notes'), 'in force');
  assert.match(await cardGrounds(page, 'sec_signoff'), /advisory@1.*withdrawn/);
  assert.match(await cardGrounds(page, 'canary_deploy'), /advisory@1/);
  assert.match(await cardGrounds(page, 'canary_deploy'), /bench@1/);
  assert.match(await cardGrounds(page, 'release_notes'), /changelog@1/);

  const explainAfter = await page.locator('#explain-out').innerText();
  assert.match(explainAfter, /advisory@1 has since been withdrawn at #7 because mirror_audit/);
  assert.match(explainAfter, /#7 retract_advisory by security: reopened because mirror_audit/);
  const depAfter = await page.locator('#dependents-out').innerText();
  assert.match(depAfter, /advisory@1 withdrawn at #7 during retract_advisory by security because mirror_audit/);

  await page.screenshot({ path: path.join(results, 'agents-desktop.png'), fullPage: true });

  // Switch dependents tab to git_log -> reaches release_notes only.
  await page.locator('[data-subject="git_log"]').click();
  const depGit = await page.locator('#dependents-out').innerText();
  assert.match(depGit, /release_notes@1/);
  assert.doesNotMatch(depGit, /sec_signoff@1/);

  // Step 4: recheck live mirror & recommit.
  await page.locator('#step-recommit').click();
  assert.equal(await cardStatus(page, 'sec_signoff'), 'in force');
  assert.equal(await cardStatus(page, 'canary_deploy'), 'in force');
  assert.equal(await cardStatus(page, 'release_notes'), 'in force');
  const explainFinal = await page.locator('#explain-out').innerText();
  assert.match(explainFinal, /sec_signoff@1 = 1\s+superseded/);
  assert.match(explainFinal, /sec_signoff@2 = 1\s+in force/);
  assert.match(explainFinal, /canary_deploy@2 = null\s+in force/);

  // The fresh recheck survives another attempt to withdraw the cached report.
  await page.locator('#step-withdraw').click();
  assert.match(await page.locator('#status').innerText(), /Refused retract_advisory.*already withdrawn/);
  assert.equal(await cardStatus(page, 'sec_signoff'), 'in force');

  // Mixing manual steps with the walkthrough yields both accepted and refused events.
  await page.locator('#step-reset').click();
  await page.locator('details.manual summary').click();
  await page.locator('[data-event="sec_check"]').first().click();
  await page.locator('[data-event="approve_security"]').click();
  await page.locator('#step-decide').click();
  const mixedStatus = await page.locator('#status').innerText();
  assert.match(mixedStatus, /Accepted 5 events/);
  assert.match(mixedStatus, /Refused approve_security.*already in force/);
  assert.match(mixedStatus, /canary_deploy@1 committed/);
  await page.locator('#step-withdraw').click();
  const correction = await page.locator('#status').innerText();
  assert.match(correction, /Withdrew advisory@2/);
  assert.match(correction, /canary_deploy@1 reopened/);
  assert.doesNotMatch(correction, /sec_signoff@1 reopened/);

  // Reset restores initial state.
  await page.locator('#step-reset').click();
  assert.equal(await cardStatus(page, 'sec_signoff'), 'not decided');
  assert.equal(await cardStatus(page, 'canary_deploy'), 'not decided');
  assert.equal(await cardStatus(page, 'release_notes'), 'not decided');

  // Mobile layout: fits at 390px with no sideways scroll.
  const phone = await browser.newPage({ viewport: { width: 390, height: 844 } });
  phone.on('pageerror', (error) => errors.push(`mobile: ${error.message}`));
  phone.on('console', (message) => { if (message.type() === 'error') errors.push(`mobile: ${message.text()}`); });
  await phone.goto(url);
  await phone.waitForFunction(() => window.__agents?.ready === true);
  await phone.locator('#step-decide').click();
  await phone.locator('#step-withdraw').click();
  assert.equal(await phone.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), true, 'no horizontal scroll on mobile');
  await phone.screenshot({ path: path.join(results, 'agents-mobile.png'), fullPage: true });
  await phone.close();

  // Trap recovery: one injected trap recovers on a fresh WebAssembly instance.
  const trapped = await browser.newPage({ viewport: { width: 1180, height: 1000 } });
  trapped.on('pageerror', (error) => errors.push(`trap: ${error.message}`));
  trapped.on('console', (message) => { if (message.type() === 'error') errors.push(`trap: ${message.text()}`); });
  await trapped.goto(url);
  await trapped.waitForFunction(() => window.__agents?.ready === true);
  await trapped.evaluate(async () => {
    const mod = await import(new URL('./pkg-reactive/caveat_runtime.js', location.href).href);
    const proto = mod.WebReactiveSession.prototype;
    const orig = proto.dispatch_outcome_as;
    let tripped = false;
    proto.dispatch_outcome_as = function (...args) {
      if (!tripped) {
        tripped = true;
        throw new WebAssembly.RuntimeError('injected trap');
      }
      return orig.apply(this, args);
    };
  });
  await trapped.locator('#step-decide').click();
  await trapped.waitForFunction(() => document.querySelector('[data-series="sec_signoff"] [data-role="status"]')?.textContent === 'in force');
  assert.equal(await trapped.evaluate(() => window.__agents.stopped()), false);
  await trapped.close();

  assert.deepEqual(errors, [], `browser errors:\n${errors.join('\n')}`);
  console.log('PASS agents.html desktop, mobile, and trap-recovery checks');
} finally {
  await browser.close();
  server?.kill();
}
