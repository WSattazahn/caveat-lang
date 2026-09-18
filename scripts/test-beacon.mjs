import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';
import { chromium, webkit, devices } from 'playwright';

const root = fileURLToPath(new URL('../', import.meta.url));
const results = path.join(root, 'test-results');
const port = process.env.PORT || '4173';
const base = process.env.SITE_URL || `http://127.0.0.1:${port}/`;
const url = new URL('last-beacon.html', base.endsWith('/') ? base : `${base}/`).href;
const routes = [
  { name: 'beacon', choices: ['lens_salt', 'trust_beam', 'reserve_unknown', 'bridge_reserve', 'beam_heat', 'relight_beacon'], title: 'THE LAST BEACON', place: 'lantern_room', evidence: 'beacon_signal', claim: 'beacon_activated', outcome: 'beacon_guided', basis: 'measured_pulses' },
  { name: 'pilot', choices: ['chart_age', 'trust_chart', 'tide_lag', 'mark_channel', 'shoal_depth', 'launch_pilot'], title: 'A LIGHT IN HUMAN HANDS', place: 'harbor', evidence: 'pilot_dispatch', claim: 'pilot_dispatched', outcome: 'pilot_rescue', basis: 'leadline_clearance' },
  { name: 'offshore', choices: ['radio_echo', 'trust_radio', 'bearing_drift', 'establish_contact', 'anchor_hold', 'hold_offshore'], title: 'THE COURAGE TO WAIT', place: 'observatory', evidence: 'holding_order', claim: 'hold_transmitted', outcome: 'hold_verified', basis: 'anchor_bearing' },
  { name: 'beacon-limited', choices: ['radio_echo', 'compare_records', 'reserve_unknown', 'bridge_reserve', 'anchor_hold', 'relight_beacon'], title: 'THE HALF-LIT CROSSING', place: 'lantern_room', evidence: 'beacon_signal', claim: 'beacon_activated', outcome: 'beacon_limited', basis: 'hot_cable' },
  { name: 'beacon-unverified', choices: ['radio_echo', 'compare_records', 'reserve_unknown', 'preserve_options', 'anchor_hold', 'relight_beacon'], title: 'A BEAM WITHOUT A BEARING', place: 'lantern_room', evidence: 'beacon_signal', claim: 'beacon_activated', outcome: 'beacon_unverified', basis: null },
];
const outcomeEvidence = [...new Set(routes.map(route => route.evidence))];
const report = { routes: [], browsers: {}, checks: [] };
let server;
const browsers = [];

async function startServer() {
  if (process.env.SITE_URL) return;
  server = spawn(process.execPath, ['scripts/serve.mjs'], { cwd: root, env: { ...process.env, PORT: port }, stdio: ['ignore', 'pipe', 'pipe'] });
  let output = '';
  await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error(`Static server did not start: ${output}`)), 10000);
    server.stdout.on('data', chunk => { output += chunk; if (output.includes('CAVEAT games:')) { clearTimeout(timeout); resolve(); } });
    server.stderr.on('data', chunk => { output += chunk; });
    server.once('error', error => { clearTimeout(timeout); reject(error); });
    server.once('exit', code => { clearTimeout(timeout); reject(new Error(`Static server exited ${code}: ${output}`)); });
  });
}

const read = page => page.evaluate(() => window.__beacon.snapshot());
const idle = page => page.waitForFunction(() => window.__beacon && !window.__beacon.busy);
const screenshot = (page, name) => page.screenshot({ path: path.join(results, `${name}.png`), fullPage: true });

async function fresh(browser, options = {}) {
  // A CSS-sized render target keeps software GPU CI representative and fast;
  // high-DPI screenshot density is not needed to verify mobile layout.
  const context = await browser.newContext({ viewport: { width: 1100, height: 760 }, reducedMotion: 'reduce', ...options, deviceScaleFactor: 1 });
  const page = await context.newPage();
  page.setDefaultTimeout(20000);
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  const response = await page.goto(url);
  assert(response?.ok(), `Game HTTP failed: ${response?.status()}`);
  await page.getByRole('button', { name: /Begin the watch/ }).waitFor();
  await idle(page);
  assert.equal(await page.evaluate(() => window.__beacon.rendered), true, 'The 3D renderer must initialize, not silently fall back to text');
  assert.equal(await page.evaluate(() => Object.isFrozen(window.__beacon)), true, 'Smoke diagnostics must be read-only');
  assert.equal(await page.locator('#world').evaluate(canvas => canvas.width > 300 && canvas.height > 250), true, 'The island canvas needs a real viewport');
  return { context, page, errors };
}

async function begin(page) {
  await page.getByRole('button', { name: /Begin the watch/ }).click();
  await idle(page);
  await page.locator('[data-selection="lens_salt"]').waitFor();
  const state = await read(page);
  assert.equal(state.turn, 0);
  assert.equal(state.pending.kind, 'investigate');
  assert.equal(state.state.current_place, 'keeper_court');
  assert.equal(state.budget.remaining, 3);
  assert.deepEqual(state.selections, []);
  assert.deepEqual(state.commitments, []);
  assert.deepEqual(state.discoveries, []);
  assert.equal(state.last_execution, null);
  assert.equal(state.outcome, null);
  assert.deepEqual(state.blocked_actions, []);
  assert(!state.symbols.some(symbol => outcomeEvidence.includes(symbol.name)), 'Future ending evidence leaked into the live journal');
  assert(!state.relations.some(relation => outcomeEvidence.includes(relation.from)), 'A future outcome was evaluated before the player acted');
  assert.equal(await page.locator('[data-selection]').count(), 3);
}

async function verifyAvailablePlans(page, state, capture = false) {
  const actions = await page.locator('[data-selection]').evaluateAll(nodes => nodes.map(node => node.dataset.selection));
  assert.deepEqual(actions, state.pending.options, 'Only actions justified by current CAVEAT knowledge should be playable');
  assert.equal(await page.locator('[data-locked-action]').count(), state.blocked_actions.length);
  if (!state.blocked_actions.length) return;
  const details = page.locator('details.unavailable');
  await details.locator('summary').click();
  for (const blocked of state.blocked_actions) {
    assert.equal(await page.locator(`[data-selection="${blocked.action}"]`).count(), 0, 'A gated plan must not be presented as an executable action');
    const card = page.locator(`[data-locked-action="${blocked.action}"]`);
    assert.equal(await card.isVisible(), true, 'The player must be able to inspect why a plan is unavailable');
    const explanation = await card.innerText();
    assert(explanation.includes(state.labels[blocked.action]));
    for (const reason of blocked.reasons) {
      assert(explanation.includes(state.labels[reason.symbol]), 'A blocked plan should identify the missing source-authored observation');
      assert(explanation.includes(reason.kind === 'observed' ? 'Find the observation' : 'Examine the uncertainty'));
    }
  }
  assert.deepEqual(await read(page), state, 'Reading requirement explanations must not change knowledge or spend attention');
  if (capture) await screenshot(page, 'desktop-knowledge-gates');
  await details.locator('summary').click();
}

async function choose(page, selection, index, route) {
  const before = await read(page);
  await verifyAvailablePlans(page, before, route.name === 'beacon' && index === 1);
  await page.locator(`[data-selection="${selection}"]`).click();
  await idle(page);
  await page.locator('[data-continue]').waitFor();
  const state = await read(page);
  const turn = index + 1;
  assert.equal(state.turn, turn, `Selecting ${selection} must advance exactly once`);
  assert.deepEqual(state.selections, route.choices.slice(0, turn));
  assert.equal(state.last_execution.action, selection, 'Gameplay must execute the selected CAVEAT action');
  assert(state.last_execution.commands.length > 1, 'The source-authored world plan should execute');
  assert.equal(state.budget.remaining, 3 - Math.ceil(turn / 2));
  assert.equal(state.commitments.length, Math.floor(turn / 2));
  if (turn < 6) {
    assert.equal(state.outcome, null, 'No future outcome should exist before the final order');
    assert.equal(state.state.current_place, 'keeper_court', 'An investigation must physically return to the court');
    assert.equal(state.pending.kind, turn % 2 ? 'choice' : 'investigate');
  }
  if (turn % 2 === 0) {
    const commitment = state.commitments.find(item => item.action === selection);
    assert(commitment, 'A player decision must create a commitment');
    assert.equal(commitment.retained.length, turn / 2 * 3, 'Unresolved caveats must survive each decision');
    if (turn < 6) {
      assert.equal(commitment.reopened_by.length, 1, 'New evidence should reopen this commitment');
      assert.match(await page.locator('#phase-label').innerText(), /COMMITMENT REOPENED/);
    }
  }
  return state;
}

async function continueWatch(page) {
  await page.locator('[data-continue]').click();
  await idle(page);
}

async function verifyJournal(page) {
  const before = await read(page);
  await page.locator('#journal-toggle').click();
  assert.equal(await page.locator('#journal').isVisible(), true);
  assert.equal(await page.locator('#journal-close').evaluate(node => node === document.activeElement), true, 'Opening the journal must transfer keyboard focus');
  for (let i = 0; i < 7; i++) {
    await page.keyboard.press('Tab');
    assert.equal(await page.evaluate(() => document.querySelector('#journal').contains(document.activeElement)), true, 'Modal focus escaped the journal');
  }
  await page.getByRole('tab', { name: 'Evidence', exact: true }).focus();
  await page.keyboard.press('ArrowRight');
  assert.equal(await page.getByRole('tab', { name: 'Decisions', exact: true }).getAttribute('aria-selected'), 'true');
  assert.match(await page.locator('#page-decisions').innerText(), /REOPENED BY NEW EVIDENCE/);
  assert.match(await page.locator('#page-decisions').innerText(), /Uncertainty carried forward/);
  await page.keyboard.press('ArrowRight');
  assert.equal(await page.getByRole('tab', { name: 'Island chart', exact: true }).getAttribute('aria-selected'), 'true');
  await page.locator('[data-focus="observatory"]').click();
  assert.equal(await page.locator('#journal').isVisible(), false);
  assert.equal(await page.locator('#journal-toggle').evaluate(node => node === document.activeElement), true, 'Closing the chart should return focus to its opener');
  assert.deepEqual(await read(page), before, 'Looking at the chart must not spend attention or change the CAVEAT world');
  await page.locator('#journal-toggle').click();
  await page.keyboard.press('Escape');
  assert.equal(await page.locator('#journal').isVisible(), false);
}

async function restore(page, expected, complete = false) {
  await page.reload();
  await page.getByRole('button', { name: /Resume your watch/ }).waitFor();
  await page.getByRole('button', { name: /Resume your watch/ }).click();
  await idle(page);
  assert.deepEqual(await read(page), expected, 'Reload and resume must restore the complete CAVEAT state');
  assert.equal(await page.evaluate(() => window.__beacon.screen), complete ? 'ending' : 'playing');
  if (complete) assert.equal(await page.locator('[data-selection]').count(), 0);
}

async function playRoute(browser, route, options = {}, prefix = route.name) {
  const { context, page, errors } = await fresh(browser, options);
  try {
    const mobile = Boolean(options.isMobile);
    if (prefix === 'beacon') await screenshot(page, 'desktop-title');
    if (mobile) {
      assert.equal(await page.locator('#title').evaluate(node => node.scrollWidth <= node.clientWidth + 1), true, 'The mobile title is clipped inside its container');
      assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth + 1), true, 'Mobile title screen has horizontal overflow');
      await screenshot(page, `${prefix}-title`);
    }
    await begin(page);
    if (prefix === 'beacon') await screenshot(page, 'desktop-start');
    if (mobile) {
      assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth + 1), true, 'Mobile gameplay has horizontal overflow');
      await screenshot(page, `${prefix}-start`);
    }
    for (let index = 0; index < route.choices.length; index++) {
      const state = await choose(page, route.choices[index], index, route);
      if (index === 1 && prefix === 'beacon') {
        await screenshot(page, 'desktop-reopened');
        await verifyJournal(page);
      }
      await continueWatch(page);
      if (index === 1 && prefix === 'beacon') {
        await restore(page, state);
        report.checks.push('mid-watch save and resume', 'journal keyboard focus and chart isolation');
      }
    }
    const end = await read(page);
    assert.equal(end.pending.kind, 'complete');
    assert.equal(end.state.current_place, route.place);
    assert.equal(end.budget.remaining, 0);
    assert.equal(end.commitments.length, 3);
    assert.equal(end.commitments.filter(item => item.reopened_by.length).length, 2);
    assert.equal(end.outcome.action, route.choices.at(-1));
    assert.equal(end.outcome.id, route.outcome, 'Earlier knowledge must select the source-authored outcome for this same physical order');
    assert.equal(end.outcome.turn, 6);
    assert.deepEqual(end.outcome.basis, route.basis ? [{ kind: 'observed', symbol: route.basis }] : []);
    assert(end.relations.some(relation => relation.from === route.evidence && relation.to === route.claim && relation.relation === 'supports'), 'The chosen ending must create its own outcome evidence');
    assert(!end.relations.some(relation => outcomeEvidence.includes(relation.from) && relation.from !== route.evidence), 'An unchosen ending must never execute');
    assert.equal(await page.locator('#decision-title').innerText(), route.title);
    assert.equal(await page.locator('#decision-body').innerText(), end.labels[`${route.outcome}_result`]);
    assert.equal(await page.locator('.epilogue').innerText(), end.labels[`${route.outcome}_epilogue`]);
    assert.equal(await page.evaluate(() => window.__beacon.screen), 'ending');
    if (mobile) assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth + 1), true, 'Mobile ending has horizontal overflow');
    await screenshot(page, `${prefix}-ending`);
    if (prefix === 'beacon') {
      await restore(page, end, true);
      assert.equal(await page.locator('#decision-title').innerText(), route.title);
      report.checks.push('completed-watch save and resume');
    }
    assert.deepEqual(errors, [], `Browser errors on ${prefix}`);
    report.routes.push({ name: prefix, ending: route.title, outcome: end.outcome.id, basis: end.outcome.basis, action: end.outcome.action, turn: end.turn, place: end.state.current_place, retained: end.commitments.find(item => item.action === route.choices.at(-1)).retained.length });
    console.log(`PASS ${prefix}: six decisions, ${route.outcome}, correct world and retained uncertainty`);
  } catch (error) {
    await screenshot(page, `${prefix}-failure`).catch(() => {});
    throw error;
  } finally { await context.close(); }
}

async function animationLock(browser) {
  const { context, page, errors } = await fresh(browser, { reducedMotion: 'no-preference', viewport: { width: 900, height: 650 } });
  try {
    await begin(page);
    const started = Date.now();
    await page.locator('[data-selection="lens_salt"]').click();
    await page.waitForFunction(() => window.__beacon.busy);
    assert.equal(await page.locator('[data-selection]:disabled').count(), 3, 'All choices must lock while the world action is playing');
    assert.equal(await page.locator('#journal-toggle').isDisabled(), true);
    assert.equal(await page.locator('#restart').isDisabled(), true);
    await page.keyboard.press('2');
    await screenshot(page, 'desktop-travel');
    await idle(page);
    assert(Date.now() - started < 20000, 'A short world action must finish in real time even under software rendering');
    const state = await read(page);
    assert.deepEqual(state.selections, ['lens_salt'], 'A second input during travel must not consume another turn');
    assert.equal(state.turn, 1);
    assert.equal(await page.locator('[data-continue]').isEnabled(), true);
    assert.deepEqual(errors, []);
    report.checks.push('animated travel locks all actions and ignores duplicate input');
    console.log('PASS animated travel: one selection, locked controls, no duplicate turn');
  } catch (error) {
    await screenshot(page, 'travel-failure').catch(() => {});
    throw error;
  } finally { await context.close(); }
}

try {
  await mkdir(results, { recursive: true });
  await startServer();
  const chrome = await chromium.launch({ headless: true, args: ['--enable-unsafe-swiftshader'] });
  browsers.push(chrome);
  report.browsers.chromium = 'passed';
  for (const route of routes) await playRoute(chrome, route);
  const beaconOutcomes = report.routes.filter(route => route.action === 'relight_beacon').map(route => route.outcome);
  assert.deepEqual(new Set(beaconOutcomes), new Set(['beacon_guided', 'beacon_limited', 'beacon_unverified']));
  report.checks.push('knowledge-gated actions and explicit missing-evidence explanations', 'same final order resolves to three outcomes from earlier evidence');
  await animationLock(chrome);
  await playRoute(chrome, routes[1], { ...devices['iPhone 13'], defaultBrowserType: undefined }, 'mobile-chromium');
  let safari;
  try { safari = await webkit.launch({ headless: true }); }
  catch (error) {
    if (process.env.CI || !/missing dependencies|Host system|shared librar|cannot open shared object/i.test(error.message)) throw error;
    report.browsers.webkit = 'unavailable locally: missing host libraries; required in CI';
    console.warn('SKIP local WebKit: host libraries are unavailable. CI requires WebKit; Chromium mobile route passed.');
  }
  if (safari) {
    browsers.push(safari);
    await playRoute(safari, routes[2], devices['iPhone 13'], 'mobile-webkit');
    report.browsers.webkit = 'passed';
  }
  console.log('Last Beacon browser QA complete. Screenshots and report: test-results/');
} catch (error) {
  report.failure = error.stack || String(error);
  console.error(error);
  process.exitCode = 1;
} finally {
  await writeFile(path.join(results, 'beacon-report.json'), `${JSON.stringify(report, null, 2)}\n`).catch(() => {});
  for (const browser of browsers) await browser.close();
  server?.kill();
}
