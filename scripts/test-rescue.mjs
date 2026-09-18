import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';
import { chromium, webkit, devices } from 'playwright';

const root = fileURLToPath(new URL('../', import.meta.url));
const results = path.join(root, 'test-results');
const port = process.env.PORT || '4177';
const base = process.env.SITE_URL || `http://127.0.0.1:${port}/`;
const url = new URL('rescue.html', base.endsWith('/') ? base : `${base}/`).href;
const report = { routes: [], checks: [], browsers: {} };
const browsers = [];
let server;

async function startServer() {
  if (process.env.SITE_URL) return;
  server = spawn(process.execPath, ['scripts/serve.mjs'], {
    cwd: root, env: { ...process.env, PORT: port }, stdio: ['ignore', 'pipe', 'pipe'],
  });
  let output = '';
  await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error(`Server did not start: ${output}`)), 10000);
    const collect = chunk => {
      output += chunk;
      if (output.includes('CAVEAT games:')) { clearTimeout(timeout); resolve(); }
    };
    server.stdout.on('data', collect);
    server.stderr.on('data', collect);
    server.once('error', error => { clearTimeout(timeout); reject(error); });
    server.once('exit', code => { clearTimeout(timeout); reject(new Error(`Server exited ${code}: ${output}`)); });
  });
}

const read = page => page.evaluate(() => window.__rescue.snapshot());
const screenshot = async (page, name) => {
  const before = await read(page);
  const result = await page.screenshot({ path: path.join(results, `rescue-${name}.png`), fullPage: true });
  const after = await read(page);
  assert.equal(after.values.elapsed, before.values.elapsed, 'Capturing a screenshot must not advance the game between player inputs');
  return result;
};
const fits = page => page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth + 1);

async function fresh(browser, options = {}, renderFps = Number(process.env.RENDER_FPS || 10), preparePage) {
  const context = await browser.newContext({
    viewport: { width: 1200, height: 800 }, reducedMotion: 'reduce', ...options, deviceScaleFactor: 1,
  });
  const page = await context.newPage();
  page.setDefaultTimeout(20000);
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  if (preparePage) await preparePage(page);
  const response = await page.goto(url);
  assert(response?.ok(), `Game failed to load: HTTP ${response?.status()}`);
  await page.getByRole('button', { name: /^Start rescue\b/i }).waitFor();
  await page.waitForFunction(() => window.__rescue?.rendered);
  assert.equal(await page.evaluate(() => Object.isFrozen(window.__rescue)), true, 'Browser diagnostics must be read-only');
  assert.equal(await page.locator('#world').evaluate(canvas => canvas.width > 250 && canvas.height > 250), true);
  assert.equal(await fits(page), true, 'The game must fit without horizontal scrolling');
  const introductoryWords = (await page.locator('body').innerText()).trim().split(/\s+/).length;
  assert(introductoryWords < 110, `Opening asks the player to read ${introductoryWords} words`);
  await page.clock.install({ time: new Date('2026-01-01T00:00:00Z') });
  // install() starts a running virtual clock. Pause before starting the game so
  // software-GPU screenshots and input transport cannot add uncontrolled ticks.
  await page.clock.pauseAt(new Date('2026-01-01T00:00:01Z'));
  if (renderFps > 0) {
    // Keep every real 30 Hz Caveat simulation tick, but avoid rendering thousands
    // of expensive software-GPU frames during the complete-route checks. The
    // normal-cadence keyboard test below independently checks the ordinary RAF
    // path. No game event, state value, input, or outcome is replaced here.
    await page.evaluate(milliseconds => {
      window.requestAnimationFrame = callback => window.setTimeout(() => callback(performance.now()), milliseconds);
      window.cancelAnimationFrame = handle => window.clearTimeout(handle);
    }, 1000 / renderFps);
  }
  return { context, page, errors };
}

async function begin(page) {
  await page.getByRole('button', { name: /^Start rescue\b/i }).click();
  await page.clock.runFor(100);
  const state = await read(page);
  assert.equal(state.values.phase, 1);
  assert.equal(state.values.hull, 3);
  assert.equal(state.values.rescued, 0);
  assert.equal(await page.locator('[data-selection]').count(), 0, 'The action game must not be a sequence of reading choices');
  return state;
}

async function projected(page, x, z) {
  const point = await page.evaluate(({ x, z }) => window.__rescue.projected(x, z), { x, z });
  assert(point && Number.isFinite(point.x) && Number.isFinite(point.y), 'Sea target must have a finite screen projection');
  const viewport = page.viewportSize();
  assert(point.x > 0 && point.x < viewport.width && point.y > 0 && point.y < viewport.height,
    `Useful steering target is off screen at ${JSON.stringify(point)}`);
  return point;
}

async function pointerController(context, page, mobile, browser) {
  // CDP sends native held-touch gestures. Repeated touchscreen.tap would release
  // the lighthouse before the simulation tick and would not test dragging.
  const touch = mobile && browser.browserType().name() === 'chromium'
    ? await context.newCDPSession(page) : null;
  let pressed = false;
  return {
    async move(x, z) {
      const point = await projected(page, x, z);
      if (touch) {
        await touch.send('Input.dispatchTouchEvent', {
          type: pressed ? 'touchMove' : 'touchStart',
          touchPoints: [{ x: point.x, y: point.y, radiusX: 4, radiusY: 4, force: 1, id: 1 }],
        });
      } else {
        await page.mouse.move(point.x, point.y);
        if (!pressed) await page.mouse.down();
      }
      pressed = true;
    },
    async release() {
      if (!pressed) return;
      if (touch) await touch.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
      else await page.mouse.up();
      pressed = false;
    },
    kind: touch ? 'native touch drag' : 'mouse drag',
  };
}

async function pointerRoute(browser, options = {}, name = 'desktop') {
  const { context, page, errors } = await fresh(browser, options);
  const mobile = Boolean(options.isMobile);
  try {
    const pointer = await pointerController(context, page, mobile, browser);
    await screenshot(page, `${name}-ready`);
    const initial = await begin(page);
    await pointer.move(5.5, initial.values.boat_z - 8);
    let previousHitCount = 0;
    let midCaptured = false;
    let currentCaptured = false;
    let observedBasis;
    let end;
    for (let step = 0; step < 95; step++) {
      const before = await read(page);
      if (before.values.phase !== 1) { end = before; break; }
      const z = before.values.boat_z;
      await pointer.move(z > 41 ? 5.5 : z > 32 ? -4 : 5.3, z - 8);
      await page.clock.runFor(1000);
      const after = await read(page);
      assert(after.values.hit_count - previousHitCount <= 1, 'One frame caused repeated collision damage');
      previousHitCount = after.values.hit_count;
      if (!midCaptured && after.values.elapsed >= 20) {
        await screenshot(page, `${name}-steering`);
        assert(after.values.boat_x > 3, 'Pointer input did not steer the ferry');
        assert.equal(await fits(page), true);
        midCaptured = true;
      }
      if (!currentCaptured && after.values.current_seen === 1) {
        assert(after.relations.some(edge => edge.from === 'morning_forecast' && edge.relation === 'supports'));
        assert(after.relations.some(edge => edge.from === 'crosscurrent_reading' && edge.relation === 'opposes'));
        assert(after.commitments.some(entry => entry.action === 'trust_forecast' && entry.open));
        assert(after.commitments.some(entry => entry.action === 'counter_steer' && entry.retained.includes('surge_unmeasured')));
        for (const name of ['observed_foam', 'estimated_peak', 'steering_plan', 'compensation']) {
          assert(after.qualified_values[name].provenance.evidence.includes('crosscurrent_reading'), `${name} lost the observed sample`);
          assert(after.qualified_values[name].provenance.caveats.includes('surge_unmeasured'), `${name} lost the current caveat`);
        }
        observedBasis = after.commitment_bases.counter_steer;
        assert(Math.abs(observedBasis.value + 0.585) < 1e-10, 'The committed steering command must come from the source function');
        assert(after.relations.some(edge => edge.from === 'counter_steer' && edge.relation === 'relies_on' && edge.to === 'crosscurrent_reading'));
        await screenshot(page, `${name}-current-observed`);
        currentCaptured = true;
      }
    }
    await pointer.release();
    end ||= await read(page);
    assert.equal(end.values.phase, 2, `Normal ${name} steering did not reach harbor: ${JSON.stringify(end.values)}`);
    assert.equal(end.values.rescued, 32);
    assert.equal(end.values.hull, 3, 'The open-water route should be steerable without taking damage');
    assert.equal(end.values.current_seen, 1, 'This route should discover and account for the crosscurrent');
    assert.deepEqual(end.commitment_bases.counter_steer, observedBasis, 'Continued play rewrote the historical observation-based command');
    assert(end.values.elapsed >= 50 && end.values.elapsed <= 90, `Unexpected run duration ${end.values.elapsed}`);
    assert.equal(await fits(page), true);
    await page.getByRole('button', { name: /^Try again\b/i }).waitFor();
    await screenshot(page, `${name}-rescued`);
    const finished = end.values;
    await page.clock.runFor(1500);
    assert.deepEqual((await read(page)).values, finished, 'The simulation continued behind the result screen');
    await page.getByRole('button', { name: /^Try again\b/i }).click();
    await page.clock.runFor(100);
    const restarted = await read(page);
    assert.equal(restarted.values.phase, 1, 'Retry should start playing immediately');
    assert.equal(restarted.values.hull, 3);
    assert.equal(restarted.values.hit_count, 0);
    assert.equal(restarted.values.rescued, 0);
    assert(restarted.values.elapsed < 1);
    assert.deepEqual(errors, [], `Browser errors in ${name}`);
    report.routes.push({ name, input: pointer.kind, rescued: end.values.rescued, hull: end.values.hull, seconds: end.values.elapsed, hitCount: end.values.hit_count });
    console.log(`PASS ${name}: ${pointer.kind} rescued 32 in ${end.values.elapsed.toFixed(1)}s with ${end.values.hull} hull`);
  } catch (error) {
    await screenshot(page, `${name}-failure`).catch(() => {});
    throw error;
  } finally { await context.close(); }
}

async function keyboardAndFailure(browser) {
  const { context, page, errors } = await fresh(browser, { viewport: { width: 1000, height: 720 } }, 0);
  try {
    const initial = await begin(page);
    await page.keyboard.down('ArrowLeft');
    await page.clock.runFor(4000);
    await page.keyboard.up('ArrowLeft');
    const left = await read(page);
    assert(left.values.boat_x < initial.values.boat_x - 1, 'ArrowLeft did not steer the ferry');
    assert.equal(left.values.hull, 3, 'The opening must allow the player to learn safely');
    await page.keyboard.down('ArrowRight');
    await page.clock.runFor(4000);
    await page.keyboard.up('ArrowRight');
    const right = await read(page);
    assert(right.values.boat_x > left.values.boat_x + 1, 'ArrowRight did not turn the ferry back');
    await screenshot(page, 'keyboard-steering');
    report.checks.push('keyboard arrows steer the actual Caveat simulation');
    await page.getByRole('button', { name: /^Pause game$/i }).click();
    const paused = await read(page);
    assert.equal(paused.values.paused, 1);
    await page.clock.runFor(2000);
    const waiting = await read(page);
    for (const key of ['boat_x', 'boat_z', 'boat_vx', 'elapsed', 'hull']) {
      assert.equal(waiting.values[key], paused.values[key], `${key} changed while the source was paused`);
    }
    assert.deepEqual(waiting.relations, paused.relations);
    assert.deepEqual(waiting.commitments, paused.commitments);
    await page.getByRole('button', { name: /^Keep going\b/i }).click();
    await page.clock.runFor(500);
    const resumed = await read(page);
    assert.equal(resumed.values.paused, 0);
    assert(resumed.values.elapsed > paused.values.elapsed);
    assert(resumed.values.boat_z < paused.values.boat_z);
    report.checks.push('source-owned pause and resume preserve live knowledge');
    assert.deepEqual(errors, []);
  } finally { await context.close(); }

  const idle = await fresh(browser);
  try {
    await begin(idle.page);
    let previousHull = 3;
    let lastHitAt = -Infinity;
    let capturedImpact = false;
    let end;
    for (let step = 0; step < 190; step++) {
      await idle.page.clock.runFor(500);
      end = await read(idle.page);
      if (end.values.hull < previousHull) {
        assert.equal(previousHull - end.values.hull, 1);
        assert(end.values.elapsed - lastHitAt >= 1.8, 'A single obstacle removed lives too quickly');
        previousHull = end.values.hull;
        lastHitAt = end.values.elapsed;
        if (!capturedImpact) { await screenshot(idle.page, 'first-impact'); capturedImpact = true; }
      }
      if (end.values.phase !== 1) break;
    }
    assert.equal(end.values.phase, 3, 'Doing nothing must not win the rescue');
    assert.equal(end.values.rescued, 0);
    assert(capturedImpact, 'The idle path never produced collision feedback');
    await idle.page.getByRole('button', { name: /^Try again\b/i }).waitFor();
    await screenshot(idle.page, 'failure-screen');
    assert.deepEqual(idle.errors, []);
    report.checks.push('no-input loss', 'collision damage cooldown', 'visible retry after failure');
    console.log('PASS keyboard controls, no-input failure, and collision cooldown');
  } finally { await idle.context.close(); }
}

async function sourceOnlyPresentation(browser) {
  let sourceIntercepted = false;
  const addition = `
// This variant is delivered as Caveat source; the host JavaScript is untouched.
bind passengers_label.text = "SOURCE VARIANT CREW" when forecast_drift == 0;
bind crosscurrent_marker.scale = 1.5;
bind crosscurrent_marker.ring.color = "#ff00ff" when forecast_drift == 0;
bind crosscurrent_marker.visible = true;
cue qa_source_tone sound 523.25 0.07 0.02;
on start when forecast_drift == 0 emit qa_source_tone;
`;
  const { context, page, errors } = await fresh(browser, {}, 10, async page => {
    await page.route('**/light_the_way.cav', async route => {
      const response = await route.fetch();
      assert(response.ok(), 'The original Caveat source must load before applying the variant');
      sourceIntercepted = true;
      await route.fulfill({ response, body: `${await response.text()}\n${addition}` });
    });
  });
  try {
    assert(sourceIntercepted, 'The test must change the served Caveat source itself');
    const state = await begin(page);
    assert.equal(state.bindings.passengers_label.text, 'SOURCE VARIANT CREW');
    for (const [target, property] of [['passengers_label', 'text'], ['crosscurrent_marker', 'ring.color']]) {
      const basis = state.binding_qualifications[target][property];
      assert(basis.evidence.includes('morning_forecast'), 'The rendered primitive lost its source selection basis');
      assert(basis.caveats.includes('surge_unmeasured'), 'Presentation stripped an unresolved caveat');
    }
    assert.equal(await page.locator('[data-caveat="passengers_label"]').innerText(), 'SOURCE VARIANT CREW',
      'The HUD ignored the source-only text binding');
    const rendered = await page.evaluate(() => window.__rescue.presentation());
    assert.equal(rendered.bindings.crosscurrent_marker.scale, 1.5, 'The actual mesh ignored its source scale');
    assert.equal(rendered.bindings.crosscurrent_marker['ring.color'], 16711935, 'The actual material ignored its source color');
    assert.equal(rendered.bindings.crosscurrent_marker.visible, true);
    const consumed = await page.evaluate(() => window.__rescue.recentCues());
    const tone = consumed.find(cue => cue.id === 'qa_source_tone');
    assert(tone, 'The unchanged host did not consume the newly authored cue');
    assert.equal(tone.kind, 'sound');
    assert.equal(tone.frequency, 523.25);
    assert.equal(tone.duration, 0.07);
    assert.equal(tone.gain, 0.02);
    assert.equal(consumed.filter(cue => cue.id === 'qa_source_tone').length, 1, 'A cue replayed without a new source event');
    assert.equal(state.values.hull, 3);
    assert.equal(state.values.rescued, 0);
    await screenshot(page, 'source-only-presentation');
    assert.deepEqual(errors, []);
    report.checks.push('Caveat-only HTTP variant changes real HUD, mesh/material, and consumed sound cue without modifying JavaScript');
    console.log('PASS Caveat-only presentation and sound variant on the unchanged browser host');
  } catch (error) {
    await screenshot(page, 'source-only-presentation-failure').catch(() => {});
    throw error;
  } finally { await context.close(); }
}

try {
  await mkdir(results, { recursive: true });
  await startServer();
  const browserNames = (process.env.BROWSERS || 'chromium').split(',').map(name => name.trim());
  for (const name of browserNames) {
    const engine = { chromium, webkit }[name];
    assert(engine, `Unsupported browser ${name}`);
    const browser = await engine.launch({ headless: true, ...(name === 'chromium' ? { args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] } : {}) });
    browsers.push(browser);
    report.browsers[name] = browser.version();
    await pointerRoute(browser, {}, name);
    await pointerRoute(browser, { ...devices['iPhone 13'], viewport: { width: 390, height: 844 } }, `${name}-mobile`);
    if (name === 'chromium') {
      await keyboardAndFailure(browser);
      await sourceOnlyPresentation(browser);
    }
  }
  await writeFile(path.join(results, 'rescue-report.json'), `${JSON.stringify(report, null, 2)}\n`);
  console.log('Rescue browser checks passed.');
} catch (error) {
  console.error(error.stack || error.message);
  process.exitCode = 1;
} finally {
  for (const browser of browsers) await browser.close();
  if (server) server.kill('SIGTERM');
}
