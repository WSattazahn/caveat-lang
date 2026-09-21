import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
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

async function assertSamplingProgress(page, before, name) {
  const partial = await read(page);
  assert(partial.values.sample_charge > 0 && partial.values.sample_charge < partial.values.sample_duration,
    'A short held beam should show progress, not instantly produce a reading');
  assert(partial.values.boat_z < before.values.boat_z, 'The ferry froze while attention was spent sampling');
  assert.equal(await page.locator('#sample-panel').isVisible(), true);
  assert.match(await page.locator('#sample-status').innerText(), /READING CURRENT/i);
  const progress = Number(await page.locator('#sample-progress').getAttribute('aria-valuenow'));
  assert(Math.abs(progress - partial.values.sample_charge / partial.values.sample_duration) < 1e-8);
  assert.equal(await fits(page), true);
  await screenshot(page, name);
  return partial;
}

const flowReadings = state => state.reading_streams.flow.occurrences;
const navigationBasis = state => state.commitment_bases[state.decision_series.navigation.current];
function assertHistoryPrefix(earlier, later) {
  const occurrences = flowReadings(earlier);
  assert.deepEqual(flowReadings(later).slice(0, occurrences.length), occurrences, 'A later reading rewrote an earlier occurrence');
  const revisions = earlier.decision_series.navigation.revisions;
  assert.deepEqual(later.decision_series.navigation.revisions.slice(0, revisions.length), revisions, 'A later decision rewrote its history');
  for (const revision of revisions) {
    assert.deepEqual(later.commitment_bases[revision.id], earlier.commitment_bases[revision.id], 'A historical numeric basis was recomputed');
  }
}

async function pointerRoute(browser, options = {}, name = 'desktop') {
  const { context, page, errors } = await fresh(browser, options);
  const mobile = Boolean(options.isMobile);
  try {
    const pointer = await pointerController(context, page, mobile, browser);
    await screenshot(page, `${name}-ready`);
    const initial = await begin(page);
    await pointer.move(-3.8, 33.5);
    await page.clock.runFor(500);
    await assertSamplingProgress(page, initial, `${name}-first-reading-progress`);
    await page.clock.runFor(600);
    let lastObserved = await read(page);
    assert.equal(flowReadings(lastObserved).length, 1, 'One completed hold must produce exactly one occurrence');
    assert.equal(lastObserved.decision_series.navigation.revisions.length, 2, 'Forecast plus first observed decision');
    assert(Math.abs(navigationBasis(lastObserved).value + 0.585) < 1e-10);
    const reading = flowReadings(lastObserved)[0];
    assert(navigationBasis(lastObserved).provenance.evidence.includes(reading.id));
    assert(reading.provenance.caveats.includes('reading_may_age'));
    assert.match(await page.locator('#reading-age').innerText(), /^READ 0s AGO$/);
    assert.match(await page.locator('#flow-legend').innerText(), /BRIGHT.*FLOW.*FAINT.*LAST READING/i);
    let previousHitCount = 0;
    let midCaptured = false;
    let handledWeather = 0;
    let end;
    for (let step = 0; step < 95; step++) {
      const before = await read(page);
      if (before.values.phase !== 1) { end = before; break; }
      if (before.values.weather_phase > handledWeather) {
        const weather = before.values.weather_phase;
        assert.equal(weather, handledWeather + 1);
        assert.equal(before.values.current_strength, weather === 1 ? -1.2 : 1.2);
        assert.equal(flowReadings(before).length, mobile ? weather : 1, 'Weather must not silently sample exact force');
        assert.deepEqual(navigationBasis(before), navigationBasis(lastObserved));
        assertHistoryPrefix(lastObserved, before);
        assert(before.commitments.some(entry => entry.action === before.decision_series.navigation.current && entry.open));
        assert.match(await page.locator('#reading-age').innerText(), /^READ [0-9]+s AGO$/);
        const presentation = await page.evaluate(() => window.__rescue.presentation());
        const flowDirection = presentation.bindings.crosscurrent_marker['arrow.rotation_y'];
        const heldDirection = presentation.bindings.last_reading_marker['arrow.rotation_y'];
        if (mobile || weather === 1) assert(Math.abs(flowDirection - heldDirection) > 3, 'Retained and actual directions should visibly disagree');
        assert.equal(presentation.bindings.last_reading_marker['arrow.opacity'], 0.42);
        await screenshot(page, `${name}-shift-${weather}-stale-reading`);
        handledWeather = weather;
        if (mobile) {
          await pointer.move(-3.8, 33.5);
          await page.clock.runFor(500);
          await assertSamplingProgress(page, before, `${name}-shift-${weather}-reading-progress`);
          await page.clock.runFor(600);
          const revised = await read(page);
          assert.equal(flowReadings(revised).length, weather + 1);
          assert.equal(revised.decision_series.navigation.revisions.length, weather + 2);
          assertHistoryPrefix(lastObserved, revised);
          const newest = flowReadings(revised).at(-1);
          assert.equal(newest.ordinal, weather + 1);
          assert.equal(newest.relation, weather === 1 ? 'opposes' : 'supports');
          assert.equal(newest.claim, 'current_eastward');
          assert(Math.abs(navigationBasis(revised).value - (weather === 1 ? 0.78 : -0.78)) < 1e-10);
          assert(navigationBasis(revised).provenance.evidence.includes(newest.id));
          assert(navigationBasis(revised).provenance.caveats.includes('reading_may_age'));
          assert.equal(await page.locator('#reading-age').innerText(), 'READ 0s AGO');
          assert.equal(revised.bindings.toast.text, 'Flow reversed. Steering revised.');
          assert.equal(await page.locator('[data-caveat="toast"]').innerText(), revised.bindings.toast.text);
          for (const reading of flowReadings(revised).slice(-2)) {
            assert(revised.binding_qualifications.toast.text.evidence.includes(reading.id));
          }
          const actual = await page.evaluate(() => window.__rescue.presentation());
          assert.equal(actual.bindings.crosscurrent_marker['arrow.rotation_y'], actual.bindings.last_reading_marker['arrow.rotation_y']);
          lastObserved = revised;
          await screenshot(page, `${name}-shift-${weather}-reading-revised`);
          continue;
        }
      }
      const z = before.values.boat_z;
      const lane = z > 41 ? 5.5 : z > 32 ? -4 : 5.3;
      // Adaptive manual steering responds only to the visible ferry position.
      const target = mobile ? lane : lane + (lane - before.values.boat_x) * 1.2;
      await pointer.move(Math.max(-9.5, Math.min(13, target)), Math.max(19, z - 12));
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
    }
    await pointer.release();
    end ||= await read(page);
    assert.equal(end.values.phase, 2, `Normal ${name} steering did not reach harbor: ${JSON.stringify(end.values)}`);
    assert.equal(end.values.rescued, 32);
    assert.equal(end.values.hull, 3, 'The route should be steerable without damage');
    assert.equal(handledWeather, 2);
    assert.equal(flowReadings(end).length, mobile ? 3 : 1);
    assert.equal(end.decision_series.navigation.revisions.length, mobile ? 4 : 2);
    assert.equal(end.reading_streams.weather.occurrences.length, 2);
    assertHistoryPrefix(lastObserved, end);
    assert(end.values.elapsed >= 50 && end.values.elapsed <= 90);
    assert.equal(await fits(page), true);
    await page.getByRole('button', { name: /^Try again\b/i }).waitFor();
    await screenshot(page, `${name}-rescued`);
    const finished = end.values;
    await page.clock.runFor(1500);
    assert.deepEqual((await read(page)).values, finished, 'The simulation continued behind the result screen');
    await page.getByRole('button', { name: /^Try again\b/i }).click();
    await page.clock.runFor(100);
    const restarted = await read(page);
    assert.equal(restarted.values.phase, 1);
    assert.equal(restarted.values.hull, 3);
    assert.equal(restarted.values.hit_count, 0);
    assert.equal(restarted.values.rescued, 0);
    assert.equal(flowReadings(restarted).length, 0, 'Retry leaked the previous crossing history');
    assert.equal(restarted.decision_series.navigation.revisions.length, 1);
    assert(restarted.values.elapsed < 1);
    assert.deepEqual(errors, [], `Browser errors in ${name}`);
    report.routes.push({ name, input: pointer.kind, currentPolicy: mobile ? 'three deliberate readings' : 'adaptive manual steering with first reading', readings: flowReadings(end).length, revisions: end.decision_series.navigation.revisions.length, rescued: end.values.rescued, hull: end.values.hull, seconds: end.values.elapsed, hitCount: end.values.hit_count });
    console.log(`PASS ${name}: ${pointer.kind}, ${flowReadings(end).length} readings / ${end.decision_series.navigation.revisions.length} revisions, rescued 32 in ${end.values.elapsed.toFixed(1)}s with ${end.values.hull} hull`);
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
    await page.keyboard.down(' ');
    for (const [field, target, negative, positive] of [
      ['aim_x', -3.8, 'ArrowLeft', 'ArrowRight'], ['aim_z', 33.5, 'ArrowUp', 'ArrowDown'],
    ]) {
      const current = await read(page);
      const difference = target - current.values[field];
      const key = difference < 0 ? negative : positive;
      await page.keyboard.down(key);
      await page.clock.runFor(Math.round(Math.abs(difference) / current.values.aim_speed * 1000));
      await page.keyboard.up(key);
    }
    const holding = await read(page);
    assert(Math.hypot(holding.values.aim_x + 3.8, holding.values.aim_z - 33.5) < holding.values.sample_radius,
      'Keyboard arrows could not aim at the current marker');
    await page.clock.runFor(500);
    await assertSamplingProgress(page, holding, 'keyboard-reading-progress');
    await page.clock.runFor(600);
    await page.keyboard.up(' ');
    assert.equal(flowReadings(await read(page)).length, 1, 'Space did not keep the beam steady for a real sample');
    report.checks.push('keyboard arrows plus held Space complete a moving-ferry current measurement');
    await page.getByRole('button', { name: /^Pause game$/i }).click();
    const paused = await read(page);
    const pausedPresentation = await page.evaluate(() => window.__rescue.presentation());
    assert.equal(paused.values.paused, 1);
    await page.clock.runFor(2000);
    const waiting = await read(page);
    for (const key of ['boat_x', 'boat_z', 'boat_vx', 'elapsed', 'hull', 'sample_charge', 'notice_remaining', 'rain_travel']) {
      assert.equal(waiting.values[key], paused.values[key], `${key} changed while the source was paused`);
    }
    assert.deepEqual(waiting.relations, paused.relations);
    assert.deepEqual(waiting.commitments, paused.commitments);
    const waitingPresentation = await page.evaluate(() => window.__rescue.presentation());
    for (const property of ['water_time', 'rain_travel']) {
      assert.equal(waitingPresentation.bindings.atmosphere[property], pausedPresentation.bindings.atmosphere[property],
        `Actual rendered ${property} moved while source time was paused`);
    }
    await page.getByRole('button', { name: /^Keep going\b/i }).click();
    await page.clock.runFor(500);
    const resumed = await read(page);
    assert.equal(resumed.values.paused, 0);
    assert(resumed.values.elapsed > paused.values.elapsed);
    assert(resumed.values.boat_z < paused.values.boat_z);
    const resumedPresentation = await page.evaluate(() => window.__rescue.presentation());
    assert(resumedPresentation.bindings.atmosphere.water_time > pausedPresentation.bindings.atmosphere.water_time);
    assert(Math.abs(resumedPresentation.bindings.atmosphere.rain_travel - pausedPresentation.bindings.atmosphere.rain_travel) > 0.01);
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

async function keyboardSourcePolicy(browser) {
  const heldFields = ['key_left', 'key_right', 'key_up', 'key_down', 'key_a', 'key_d', 'key_w', 'key_s', 'key_space', 'keyboard_active'];
  const released = (state, label) => {
    for (const field of heldFields) assert.equal(state.values[field], 0, `${label}: ${field} stayed held`);
  };
  const { context, page, errors } = await fresh(browser);
  try {
    const initial = await begin(page);
    await page.keyboard.down('a');
    await page.keyboard.down('ArrowLeft');
    await page.clock.runFor(100);
    const aliases = await read(page);
    assert(aliases.values.aim_x < initial.values.aim_x);
    assert.equal(aliases.values.key_a, 1);
    assert.equal(aliases.values.key_left, 1);
    await page.keyboard.up('a');
    const oneHeld = await read(page);
    assert.equal(oneHeld.values.key_a, 0);
    assert.equal(oneHeld.values.key_left, 1);
    assert.equal(oneHeld.values.light_on, 1, 'Releasing one alias released the other held key');
    await page.clock.runFor(100);
    const left = await read(page);
    assert(left.values.aim_x < oneHeld.values.aim_x);

    await page.keyboard.down('ArrowRight');
    await page.clock.runFor(100);
    const opposed = await read(page);
    assert.equal(opposed.values.aim_x, left.values.aim_x, 'Opposite directions did not cancel in Caveat');
    assert.equal(opposed.values.light_on, 1, 'Opposing movement keys should still hold the light');
    await page.keyboard.up('ArrowLeft');
    await page.clock.runFor(100);
    const right = await read(page);
    assert(right.values.aim_x > opposed.values.aim_x);
    await page.keyboard.down('ArrowRight');
    assert.equal((await read(page)).sequence, right.sequence, 'A repeated keydown was published as a new source input');
    await page.keyboard.up('ArrowRight');
    const noKeys = await read(page);
    released(noKeys, 'last key release');
    assert.equal(noKeys.values.light_on, 0, 'Last key release did not stop the light immediately');

    // Pointer takeover clears source and transport state. A repeat/release from
    // the still-physically-held key must not steal or switch off the beam.
    await page.keyboard.down('ArrowLeft');
    const target = await projected(page, 4, 50);
    await page.mouse.move(target.x, target.y);
    await page.mouse.down();
    const pointer = await read(page);
    released(pointer, 'pointer takeover');
    assert.equal(pointer.values.light_on, 1);
    await page.keyboard.down('ArrowLeft');
    await page.clock.runFor(100);
    const repeated = await read(page);
    released(repeated, 'repeat after pointer takeover');
    assert.equal(repeated.values.aim_x, pointer.values.aim_x, 'A held-key repeat reclaimed pointer steering');
    await page.keyboard.up('ArrowLeft');
    assert.equal((await read(page)).values.light_on, 1, 'An old key release switched off the pointer beam');
    await page.mouse.up();
    assert.equal((await read(page)).values.light_on, 0);

    await page.mouse.down();
    await page.keyboard.down('ArrowRight');
    const keyboard = await read(page);
    assert.equal(keyboard.values.key_right, 1);
    await page.mouse.up();
    assert.equal((await read(page)).values.light_on, 1, 'An old pointer release switched off keyboard input');
    await page.clock.runFor(100);
    assert((await read(page)).values.aim_x > keyboard.values.aim_x);
    await page.keyboard.up('ArrowRight');

    await page.keyboard.down('a');
    await page.getByRole('button', { name: /^Pause game$/i }).click();
    const paused = await read(page);
    released(paused, 'pause');
    assert.equal(paused.values.light_on, 0);
    await page.getByRole('button', { name: /^Keep going\b/i }).click();
    await page.keyboard.down('a');
    await page.clock.runFor(100);
    const resumed = await read(page);
    released(resumed, 'repeat after resume');
    assert.equal(resumed.values.aim_x, paused.values.aim_x);
    assert.equal(resumed.values.light_on, 0, 'Resume reused a physically held key without a fresh press');
    await page.keyboard.up('a');

    await page.keyboard.down('a');
    await page.keyboard.down('Escape');
    const escaped = await read(page);
    released(escaped, 'source Escape pause');
    assert.equal(escaped.values.paused, 1);
    await page.keyboard.down('Escape');
    assert.equal((await read(page)).values.paused, 1, 'An Escape repeat resumed the paused game');
    await page.keyboard.up('Escape');
    await page.keyboard.down('ArrowRight');
    await page.keyboard.up('ArrowRight');
    released(await read(page), 'movement input while paused');
    await page.keyboard.down('Escape');
    await page.keyboard.up('Escape');
    assert.equal((await read(page)).values.paused, 0, 'A new source Escape press did not resume');
    await page.keyboard.down('a');
    released(await read(page), 'held key after source Escape resume');
    await page.keyboard.up('a');

    await page.keyboard.down('w');
    // Exercise the browser lifecycle notification; no Caveat state is mutated.
    await page.evaluate(() => window.dispatchEvent(new Event('blur')));
    const blurred = await read(page);
    released(blurred, 'window blur');
    assert.equal(blurred.values.paused, 1);
    assert.equal(blurred.values.light_on, 0);
    await page.getByRole('button', { name: /^Start over$/i }).click();
    await page.keyboard.down('w');
    await page.clock.runFor(100);
    const restarted = await read(page);
    released(restarted, 'repeat after retry');
    assert.equal(restarted.values.phase, 1);
    assert.equal(restarted.values.aim_x, initial.values.aim_x);
    assert.equal(restarted.values.aim_z, initial.values.aim_z);
    assert.equal(restarted.values.light_on, 0);
    await page.keyboard.up('w');
    await screenshot(page, 'source-keyboard-release');
    assert.deepEqual(errors, []);
    report.checks.push('source keyboard aliases, opposing directions, immediate releases, pointer takeover, Escape/pause, blur, retry, and repeated-key suppression');
  } finally { await context.close(); }

  let sourceIntercepted = false;
  const mapped = await fresh(browser, {}, 10, async page => {
    await page.route('**/light_the_way.cav', async route => {
      const response = await route.fetch();
      assert(response.ok());
      const original = await response.text();
      const remapped = original.replace('control key_KeyA = key_a_input;', 'control key_KeyJ = key_a_input;')
        .replace('control key_Escape = toggle_pause;', 'control key_KeyP = toggle_pause;')
        .replace('max(positive, positive_alias) - max(negative, negative_alias)', 'max(negative, negative_alias) - max(positive, positive_alias)');
      assert.notEqual(remapped, original, 'The source-only control policy must change');
      sourceIntercepted = true;
      await route.fulfill({ response, body: remapped });
    });
  });
  try {
    assert(sourceIntercepted);
    const initial = await begin(mapped.page);
    assert.equal(initial.controls.key_KeyA, undefined);
    assert.equal(initial.controls.key_KeyJ.event, 'key_a_input');
    await mapped.page.keyboard.down('a');
    await mapped.page.clock.runFor(100);
    await mapped.page.keyboard.up('a');
    const removed = await read(mapped.page);
    released(removed, 'source-removed key');
    assert.equal(removed.values.aim_x, initial.values.aim_x, 'The host retained its former A-key behavior');
    assert.equal(removed.values.light_on, 0);
    await mapped.page.keyboard.down('j');
    await mapped.page.clock.runFor(300);
    const changed = await read(mapped.page);
    assert.equal(changed.values.key_a, 1, 'The host did not deliver the newly source-declared physical key');
    assert(changed.values.aim_x > removed.values.aim_x, 'The unchanged host overrode source-reversed steering');
    await mapped.page.keyboard.up('j');
    const end = await read(mapped.page);
    released(end, 'source-remapped key release');
    assert.equal(end.values.light_on, 0);
    assertHistoryPrefix(initial, end);
    await mapped.page.keyboard.press('Escape');
    assert.equal((await read(mapped.page)).values.paused, 0, 'The host retained an undeclared Escape shortcut');
    await mapped.page.keyboard.press('p');
    assert.equal((await read(mapped.page)).values.paused, 1, 'The new source pause key was ignored');
    await mapped.page.keyboard.press('p');
    assert.equal((await read(mapped.page)).values.paused, 0, 'The source pause key could not resume');
    await screenshot(mapped.page, 'source-keyboard-remap');
    assert.deepEqual(mapped.errors, []);
    report.checks.push('Caveat-only HTTP variant remaps A to J and Escape to P, and reverses steering on the unchanged browser host');
    console.log('PASS source-authored keyboard policy, physical-key lifecycle, and Caveat-only remapping');
  } finally { await mapped.context.close(); }
}

async function sourceOnlyPresentation(browser) {
  let sourceIntercepted = false;
  const addition = `
// This variant is delivered as Caveat source; the host JavaScript is untouched.
bind passengers_label.text = "SOURCE VARIANT CREW" when forecast_drift == 0;
bind time.text = "SOURCE CLOCK " + number_text(floor(elapsed));
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
    assert.equal(await page.locator('#time').innerText(), 'SOURCE CLOCK 0', 'The host replaced source-authored formatted text');
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
    await page.clock.runFor(1200);
    assert.equal(await page.locator('#time').innerText(), 'SOURCE CLOCK 1', 'The source text function did not update the real HUD');
    await screenshot(page, 'source-only-presentation');
    assert.deepEqual(errors, []);
    report.checks.push('Caveat-only HTTP variant changes real HUD, mesh/material, and consumed sound cue without modifying JavaScript');
    console.log('PASS Caveat-only presentation and sound variant on the unchanged browser host');
  } catch (error) {
    await screenshot(page, 'source-only-presentation-failure').catch(() => {});
    throw error;
  } finally { await context.close(); }
}

async function historyComputation(browser, name) {
  const source = await readFile(path.join(root, 'examples/thermostat_history.cav'), 'utf8');
  const context = await browser.newContext();
  const page = await context.newPage();
  try {
    // Load on the local build's origin, then exercise its real WebAssembly
    // API with a separate program, without a game-specific calculation host.
    await page.goto(new URL('rescue.css', base).href);
    const result = await page.evaluate(async source => {
      const { default: init, WebReactiveSession } = await import('./pkg/caveat_runtime.js');
      await init();
      const session = new WebReactiveSession(source + `
        event invalid_index;
        on invalid_index sample temperature = 30 supports warm_enough;
        on invalid_index set recorded_mean = history_at(temperature, 99);
      `);
      try {
        for (const value of [17, 25, 17]) session.dispatch('read', JSON.stringify({ value }));
        const before = session.snapshot();
        let rejected = false;
        try { session.dispatch('invalid_index', '{}'); } catch { rejected = true; }
        return { snapshot: JSON.parse(before), rejected, unchanged: before === session.snapshot() };
      } finally { session.free(); }
    }, source);
    assert(result.rejected && result.unchanged, 'Failed indexed access published partial WASM state');
    assert.equal(result.snapshot.values.recorded_mean, 59 / 3);
    assert.equal(result.snapshot.values.recorded_low, 17);
    assert.equal(result.snapshot.values.recorded_high, 25);
    assert.equal(result.snapshot.bindings.change.text, 'Falling');
    assert.deepEqual(result.snapshot.qualified_values.recorded_mean.provenance.evidence,
      ['temperature@1', 'temperature@2', 'temperature@3']);
    assert(result.snapshot.qualified_values.recorded_mean.provenance.caveats.includes('calibration_offset'));
    report.checks.push(`${name}: source-defined history folds, qualified indexed reads, and atomic rejection in WebAssembly`);
    console.log(`PASS ${name}: Caveat history computation in WebAssembly`);
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
    await historyComputation(browser, name);
    await pointerRoute(browser, {}, name);
    await pointerRoute(browser, { ...devices['iPhone 13'], viewport: { width: 390, height: 844 } }, `${name}-mobile`);
    if (name === 'chromium') {
      await keyboardSourcePolicy(browser);
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
