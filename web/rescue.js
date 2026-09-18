import init, { WebReactiveSession } from './pkg/caveat_runtime.js';
import { createRescueWorld } from './rescue-world.js';

const $ = selector => document.querySelector(selector);
const app = $('#app'), canvas = $('#world');
const step = 1 / 30;
const clamp = (value, min, max) => Math.max(min, Math.min(max, value));
const escape = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const keys = new Set();
let source, session, snapshot, world, paused = false, pointer = null;
let lastFrame = 0, accumulator = 0, maxHull = 3, toastTimer, impactTimer;
let audioContext, hum, humGain, soundEnabled = true, best = 0, lastUi = '';
const label = (key, fallback = '') => snapshot?.labels?.[key] || fallback;
const announce = message => { $('#announcer').textContent = message; };
const time = seconds => { const total = Math.ceil(Math.max(0, seconds)); return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`; };

function unlockSound() {
  if (!soundEnabled) return;
  try {
    if (!audioContext) {
      const Audio = window.AudioContext || window.webkitAudioContext;
      if (!Audio) return;
      audioContext = new Audio();
      hum = audioContext.createOscillator(); hum.type = 'sine'; hum.frequency.value = 82;
      humGain = audioContext.createGain(); humGain.gain.value = 0;
      hum.connect(humGain); humGain.connect(audioContext.destination); hum.start();
    }
    audioContext.resume().catch(() => {});
  } catch { soundEnabled = false; }
}

function tone(kind) {
  if (!soundEnabled || !audioContext) return;
  const now = audioContext.currentTime;
  const notes = kind === 'win' ? [330, 440, 554, 660] : kind === 'hit' ? [95] : kind === 'reef' ? [440, 550] : [220, 330];
  notes.forEach((frequency, i) => {
    const oscillator = audioContext.createOscillator(), gain = audioContext.createGain();
    oscillator.type = kind === 'hit' ? 'triangle' : 'sine';
    oscillator.frequency.setValueAtTime(frequency, now + i * 0.11);
    gain.gain.setValueAtTime(0, now + i * 0.11);
    gain.gain.linearRampToValueAtTime(0.07, now + i * 0.11 + 0.015);
    gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.11 + 0.22);
    oscillator.connect(gain); gain.connect(audioContext.destination);
    oscillator.start(now + i * 0.11); oscillator.stop(now + i * 0.11 + 0.25);
  });
}

function toast(message, toneName = 'reef') {
  clearTimeout(toastTimer);
  $('#toast').textContent = message;
  $('#toast').dataset.tone = toneName;
  $('#toast').hidden = false;
  toastTimer = setTimeout(() => { $('#toast').hidden = true; }, toneName === 'hit' ? 1500 : 950);
  announce(message);
}

function accept(next, feedback = true) {
  const before = snapshot?.values;
  snapshot = next;
  world?.setRescueState(snapshot);
  if (feedback && before && next.values.phase === 1) {
    if (next.values.hull < before.hull) {
      toast('Hull hit!', 'hit'); tone('hit');
      clearTimeout(impactTimer); $('#impact').classList.add('hit');
      impactTimer = setTimeout(() => $('#impact').classList.remove('hit'), 220);
    } else if ((next.values.reefs_seen || 0) > (before.reefs_seen || 0)) {
      toast('Reef spotted'); tone('reef');
    }
  }
  if (before?.phase === 1 && [2, 3].includes(next.values.phase)) finish();
  renderHud();
}

function dispatch(event, payload = {}) {
  accept(JSON.parse(session.dispatch(event, JSON.stringify(payload))));
}

function aim(x, z, active) {
  if (!session || snapshot.values.phase !== 1 || paused) return;
  const values = snapshot.values;
  const definition = snapshot.events?.find(event => event.name === 'aim');
  const bounds = name => definition?.parameters?.find(parameter => parameter.name === name);
  const bx = bounds('x'), bz = bounds('z');
  const a = clamp(x, bx?.min ?? values.aim_x_min, bx?.max ?? values.aim_x_max);
  const b = clamp(z, bz?.min ?? values.aim_z_min, bz?.max ?? values.aim_z_max);
  if (!Number.isFinite(a) || !Number.isFinite(b)) return;
  dispatch('aim', { x: a, z: b, active: active ? 1 : 0 });
}

function releaseControls() {
  keys.clear(); pointer = null;
  if (snapshot?.values.phase === 1 && !paused) aim(snapshot.values.aim_x, snapshot.values.aim_z, false);
}

function start() {
  releaseControls();
  session?.free(); session = new WebReactiveSession(source);
  snapshot = JSON.parse(session.snapshot());
  maxHull = snapshot.values.hull;
  paused = false; accumulator = 0; lastFrame = performance.now(); lastUi = '';
  $('#intro').hidden = true; $('#demo-cue').hidden = true; $('#ending').hidden = true;
  $('#pause-panel').hidden = true; $('#hud').hidden = false; $('#play-footer').hidden = false;
  $('#pause').hidden = false; $('#toast').hidden = true; $('#impact').classList.remove('hit');
  clearTimeout(toastTimer); clearTimeout(impactTimer);
  app.dataset.screen = 'playing';
  dispatch('start');
  unlockSound(); tone('start');
  canvas.focus({ preventScroll: true });
  announce('Guide the ferry to the green harbor. Hold and drag the light, or use the arrow keys.');
}

function setPaused(value) {
  if (!snapshot || snapshot.values.phase !== 1 || paused === value) return;
  releaseControls(); paused = value; accumulator = 0; lastFrame = performance.now();
  app.dataset.screen = value ? 'paused' : 'playing';
  $('#pause-panel').hidden = !value;
  $('#pause').setAttribute('aria-label', value ? 'Resume game' : 'Pause game');
  if (value) $('#resume').focus(); else canvas.focus({ preventScroll: true });
  if (humGain) humGain.gain.setTargetAtTime(0, audioContext.currentTime, .1);
}

function finish() {
  keys.clear(); pointer = null; paused = false;
  const values = snapshot.values, won = values.phase === 2;
  app.dataset.screen = won ? 'won' : 'lost';
  $('#ending').hidden = false; $('#pause').hidden = true; $('#play-footer').hidden = true;
  $('#toast').hidden = true; clearTimeout(toastTimer);
  $('#ending-eyebrow').textContent = won ? 'HOME AT LAST' : 'ONE MORE TRY';
  $('#ending-title').textContent = won ? `${Math.round(values.rescued)} safe.` : label('loss_title', 'Ferry lost.');
  $('#ending-note').textContent = won ? label('win_body', 'You brought them home.') : label(`loss_${values.failure_reason}`, 'The ferry hit the reefs. Try a different line.');
  const score = Math.round(values.score || 0);
  if (won && score > best) {
    best = score;
    try { localStorage.setItem('caveat:light-the-way:best:v1', String(best)); } catch { /* A run remains playable without storage. */ }
  }
  $('#result-stats').innerHTML = won
    ? `<div><strong>${score.toLocaleString()}</strong><span>KEEPER SCORE</span></div><div><strong>${time(values.elapsed)}</strong><span>CROSSING</span></div><div><strong>${best.toLocaleString()}</strong><span>BEST</span></div>`
    : `<div><strong>${Math.round((values.progress || 0) * 100)}%</strong><span>OF THE WAY</span></div><div><strong>${time(values.elapsed)}</strong><span>AT SEA</span></div>`;
  tone(won ? 'win' : 'hit');
  $('#retry').focus({ preventScroll: true });
  announce(`${$('#ending-title').textContent} ${$('#ending-note').textContent}`);
}

function renderHud() {
  if (!snapshot) return;
  const values = snapshot.values;
  const seconds = Math.ceil(Math.max(0, values.time_limit - values.elapsed));
  const signature = `${values.hull}:${seconds}:${values.phase}:${values.light_on}:${values.rescued}`;
  if (signature !== lastUi) {
    lastUi = signature;
    $('#passengers').textContent = Math.round(values.phase === 2 ? values.rescued : values.total_passengers);
    $('.passengers span').textContent = values.phase === 2 ? 'PEOPLE SAFE' : 'PEOPLE ABOARD';
    $('#time').textContent = time(seconds);
    $('.time-status span').textContent = values.phase > 1 ? 'TIME LEFT' : 'TO GET HOME';
    $('#hull').innerHTML = Array.from({ length: maxHull }, (_, i) => `<i class="${i >= values.hull ? 'lost' : ''}"></i>`).join('');
    $('#hull').setAttribute('aria-label', `${values.hull} hull remaining`);
    $('#hull').dataset.low = String(values.hull === 1);
    $('#control-hint').textContent = values.light_on ? 'GUIDE THEM HOME' : 'HOLD + DRAG THE LIGHT';
  }
  const progress = clamp(values.progress || 0, 0, 1);
  $('#progress-fill').style.width = `${progress * 100}%`;
  $('#progress-ship').style.left = `${progress * 100}%`;
  if (humGain) humGain.gain.setTargetAtTime(soundEnabled && values.phase === 1 && values.light_on && !paused ? .014 : 0, audioContext.currentTime, .07);
}

function placeLabels() {
  if (!world) return;
  const rect = canvas.getBoundingClientRect();
  for (const [id, selector, lift] of [['harbor_goal', '#goal-label', 14], ['ferry', '#boat-label', 20]]) {
    const point = world.getScreenPosition(id), node = $(selector);
    node.hidden = !point?.visible || ['paused', 'won', 'lost'].includes(app.dataset.screen);
    if (point) { node.style.left = `${point.x * rect.width}px`; node.style.top = `${point.y * rect.height - lift}px`; }
  }
}

function frame(now) {
  const dt = lastFrame ? Math.min(.1, Math.max(0, (now - lastFrame) / 1000)) : 0;
  lastFrame = now;
  try {
    if (snapshot?.values.phase === 1 && !paused && !document.hidden) {
      if (keys.size) {
        const values = snapshot.values, speed = values.aim_speed || 12;
        const dx = (keys.has('ArrowRight') || keys.has('d') ? 1 : 0) - (keys.has('ArrowLeft') || keys.has('a') ? 1 : 0);
        const dz = (keys.has('ArrowDown') || keys.has('s') ? 1 : 0) - (keys.has('ArrowUp') || keys.has('w') ? 1 : 0);
        aim(values.aim_x + dx * speed * dt, values.aim_z + dz * speed * dt, true);
      }
      accumulator = Math.min(.1, accumulator + dt);
      while (accumulator >= step && snapshot.values.phase === 1) { dispatch('tick', { dt: step }); accumulator -= step; }
    }
    placeLabels();
  } catch (error) {
    console.error(error); setPaused(true); toast('The rescue paused. Try starting over.', 'hit');
  }
  requestAnimationFrame(frame);
}

canvas.addEventListener('pointerdown', event => {
  if (event.button !== 0 || snapshot?.values.phase !== 1 || paused) return;
  event.preventDefault(); pointer = event.pointerId; keys.clear();
  canvas.setPointerCapture(pointer); canvas.focus({ preventScroll: true }); unlockSound();
  const point = world.pointFromScreen(event.clientX, event.clientY);
  if (point) aim(point.x, point.z, true);
});
canvas.addEventListener('pointermove', event => {
  if (event.pointerId !== pointer) return;
  event.preventDefault();
  const point = world.pointFromScreen(event.clientX, event.clientY);
  if (point) aim(point.x, point.z, true);
});
for (const name of ['pointerup', 'pointercancel', 'lostpointercapture']) canvas.addEventListener(name, event => {
  if (event.pointerId !== pointer) return;
  pointer = null;
  if (snapshot?.values.phase === 1 && !paused) aim(snapshot.values.aim_x, snapshot.values.aim_z, false);
});
window.addEventListener('keydown', event => {
  if (event.key === 'Escape' && snapshot?.values.phase === 1) { event.preventDefault(); setPaused(!paused); return; }
  const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
  if (snapshot?.values.phase === 1 && !paused && ['ArrowLeft','ArrowRight','ArrowUp','ArrowDown','a','s','d','w',' '].includes(key)) {
    event.preventDefault(); keys.add(key); pointer = null; unlockSound();
  }
});
window.addEventListener('keyup', event => {
  const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
  if (!keys.delete(key)) return;
  if (!keys.size && snapshot?.values.phase === 1 && !paused) aim(snapshot.values.aim_x, snapshot.values.aim_z, false);
});
window.addEventListener('blur', () => { if (snapshot?.values.phase === 1) setPaused(true); });
document.addEventListener('visibilitychange', () => { if (document.hidden && snapshot?.values.phase === 1) setPaused(true); });
$('#pause').addEventListener('click', () => setPaused(!paused));
$('#resume').addEventListener('click', () => setPaused(false));
$('#restart-paused').addEventListener('click', start); $('#retry').addEventListener('click', start);
$('#sound').addEventListener('click', () => {
  soundEnabled = !soundEnabled;
  $('#sound').setAttribute('aria-pressed', String(soundEnabled));
  $('#sound').setAttribute('aria-label', soundEnabled ? 'Turn sound off' : 'Turn sound on');
  if (soundEnabled) unlockSound(); else if (humGain) humGain.gain.setTargetAtTime(0, audioContext.currentTime, .03);
});

async function boot() {
  const response = await fetch('./light_the_way.cav');
  if (!response.ok) throw new Error('The game source could not load.');
  source = await response.text();
  await init();
  session = new WebReactiveSession(source); snapshot = JSON.parse(session.snapshot());
  world = createRescueWorld(canvas, { model: snapshot.world, labels: snapshot.labels });
  world.setRescueState(snapshot); maxHull = snapshot.values.hull;
  try { best = Math.max(0, Number(localStorage.getItem('caveat:light-the-way:best:v1')) || 0); } catch { /* Optional local score. */ }
  $('#instruction').textContent = label('instruction', 'Drag the light left or right. Reach the green harbor.');
  $('#start-actions').innerHTML = '<button class="primary" id="start-rescue">Start rescue <span aria-hidden="true">→</span></button>';
  $('#start-rescue').addEventListener('click', start);
  $('#sound').setAttribute('aria-pressed', String(soundEnabled));
  $('#sound').setAttribute('aria-label', soundEnabled ? 'Turn sound off' : 'Turn sound on');
  app.dataset.screen = 'ready';
  Object.defineProperty(window, '__rescue', { value: Object.freeze({
    snapshot: () => JSON.parse(session.snapshot()),
    get rendered() { return Boolean(world); }, get screen() { return app.dataset.screen; },
    projected(x, z) { const p = world.projected(x, z), r = canvas.getBoundingClientRect(); return p ? { x: r.left + p.x * r.width, y: r.top + p.y * r.height, visible: p.visible } : null; },
  }) });
  requestAnimationFrame(frame);
}

boot().catch(error => {
  console.error(error); app.dataset.screen = 'error';
  $('#start-actions').innerHTML = `<div class="error-box">The rescue could not start. ${escape(error.message || error)}<br><button class="secondary" id="reload">Try again</button></div>`;
  $('#reload').addEventListener('click', () => location.reload());
});
