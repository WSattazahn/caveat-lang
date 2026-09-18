import init, { WebReactiveSession } from './pkg/caveat_runtime.js';
import { createRescueWorld } from './rescue-world.js';

const $ = selector => document.querySelector(selector);
const app = $('#app'), canvas = $('#world');
const clamp = (value, min, max) => Math.max(min, Math.min(max, value));
const escape = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const keys = new Set(), bindingCache = new WeakMap(), flashTimers = new Map();
let source, session, snapshot, world, pointer = null, lastPointer = { x: 0, z: 0 };
let lastFrame = 0, accumulator = 0, toastTimer, audioContext, soundEnabled = true;
let cueSequence = null, focusTarget = null;
const recentCues = [];
const announce = message => { $('#announcer').textContent = message; };
const running = () => Boolean(snapshot?.bindings?.app?.running);
const time = seconds => { const total = Math.ceil(Math.max(0, Number(seconds) || 0)); return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`; };

function unlockSound() {
  if (!soundEnabled) return;
  try {
    if (!audioContext) {
      const Audio = window.AudioContext || window.webkitAudioContext;
      if (!Audio) return;
      audioContext = new Audio();
    }
    audioContext.resume().catch(() => {});
  } catch { soundEnabled = false; }
}

// Sound parameters and timing are emitted by the source; the host supplies
// only the audio device, its user preference, and a reusable oscillator.
function soundCue(cue) {
  if (!soundEnabled || !audioContext) return;
  const duration = Math.max(0.01, Number(cue.duration) || 0.01);
  const frequency = Number(cue.frequency), level = clamp(Number(cue.gain) || 0, 0, 1);
  if (!Number.isFinite(frequency) || frequency <= 0 || level <= 0) return;
  const now = audioContext.currentTime;
  const oscillator = audioContext.createOscillator(), gain = audioContext.createGain();
  oscillator.type = ['sine', 'triangle', 'square', 'sawtooth'].includes(cue.waveform) ? cue.waveform : 'sine';
  oscillator.frequency.setValueAtTime(frequency, now);
  gain.gain.setValueAtTime(0, now);
  gain.gain.linearRampToValueAtTime(level, now + Math.min(0.015, duration / 4));
  gain.gain.exponentialRampToValueAtTime(0.0001, now + duration);
  oscillator.connect(gain); gain.connect(audioContext.destination);
  oscillator.start(now); oscillator.stop(now + duration);
  oscillator.addEventListener('ended', () => { oscillator.disconnect(); gain.disconnect(); }, { once: true });
}

function showToast(text, duration) {
  clearTimeout(toastTimer);
  $('#toast').textContent = String(text ?? '');
  $('#toast').hidden = false;
  toastTimer = setTimeout(() => { $('#toast').hidden = true; }, Math.max(0, Number(duration) || 0) * 1000);
  announce(String(text ?? ''));
}

function playCues() {
  if (cueSequence === snapshot.sequence) return;
  cueSequence = snapshot.sequence;
  for (const cue of snapshot.cues || []) {
    recentCues.push({ ...cue, sequence: snapshot.sequence });
    if (recentCues.length > 32) recentCues.shift();
    if (cue.kind === 'sound') soundCue(cue);
    else if (cue.kind === 'toast') showToast(cue.text, cue.duration);
    else if (cue.kind === 'flash') {
      const node = document.getElementById(cue.target);
      if (!node) continue;
      clearTimeout(flashTimers.get(node));
      node.classList.add('cue-active');
      flashTimers.set(node, setTimeout(() => { node.classList.remove('cue-active'); flashTimers.delete(node); }, Math.max(0, Number(cue.duration) || 0) * 1000));
    }
    // World-space cues belong to the renderer, which receives the same snapshot.
  }
}

function formatValue(value, format) {
  if (format === 'time') return time(value);
  if (format === 'percent') return `${Math.round(Number(value) * 100)}%`;
  if (format === 'number') return Math.round(Number(value)).toLocaleString();
  return String(value ?? '');
}

// A property adapter, not a game-state interpreter. CAVEAT chooses the value,
// visibility, labels and screen; HTML chooses how a value is presented.
function renderBindings() {
  for (const node of document.querySelectorAll('[data-caveat]')) {
    const properties = snapshot.bindings?.[node.dataset.caveat];
    if (!properties) continue;
    const selected = node.dataset.caveatProperties?.split(',');
    const binding = selected ? Object.fromEntries(Object.entries(properties).filter(([key]) => selected.includes(key))) : properties;
    const signature = JSON.stringify(binding);
    if (bindingCache.get(node) === signature) continue;
    bindingCache.set(node, signature);
    if ('visible' in binding) node.hidden = !binding.visible;
    if ('screen' in binding) node.dataset.screen = String(binding.screen);
    if ('text' in binding) node.textContent = String(binding.text ?? '');
    if ('value' in binding) {
      const value = binding.value;
      if (node.dataset.format === 'pips') {
        const maximum = Math.max(0, Math.round(Number(binding.max) || 0));
        node.replaceChildren(...Array.from({ length: maximum }, (_, index) => {
          const pip = document.createElement('i');
          if (index >= Number(value)) pip.className = 'lost';
          return pip;
        }));
      } else if (node.dataset.valueStyle) {
        const property = node.dataset.valueStyle;
        if (['width', 'left'].includes(property)) node.style[property] = `${clamp(Number(value) || 0, 0, 1) * 100}%`;
      } else node.textContent = formatValue(value, node.dataset.format);
    }
    if ('progress' in binding) node.style.width = `${clamp(Number(binding.progress) || 0, 0, 1) * 100}%`;
    for (const [property, value] of Object.entries(binding)) {
      if (['visible', 'screen', 'text', 'value', 'max', 'progress'].includes(property)) continue;
      if (property === 'disabled') node.disabled = Boolean(value);
      else if (property === 'title' || property.startsWith('aria_')) node.setAttribute(property.replaceAll('_', '-'), String(value));
      else node.setAttribute(`data-${property.replaceAll('_', '-')}`, String(value));
    }
  }
  const nextFocus = snapshot.bindings?.focus?.target;
  if (nextFocus !== focusTarget) {
    focusTarget = nextFocus;
    const node = typeof nextFocus === 'string' ? document.getElementById(nextFocus) : null;
    if (node && !node.closest('[hidden]')) node.focus({ preventScroll: true });
  }
}

function clearInput() {
  keys.clear();
  const captured = pointer;
  pointer = null;
  if (captured !== null && canvas.hasPointerCapture(captured)) canvas.releasePointerCapture(captured);
}

function accept(next) {
  snapshot = next;
  world?.setRescueState(snapshot);
  renderBindings();
  if (!running()) { clearInput(); accumulator = 0; }
  playCues();
}

function dispatch(event, payload = {}) {
  if (!session || !event) return;
  accept(JSON.parse(session.dispatch(event, JSON.stringify(payload))));
}

function runControl(name, payload = {}) {
  const control = snapshot?.controls?.[name];
  if (!control) return;
  unlockSound();
  if (control.reset) {
    clearInput();
    session?.free(); session = new WebReactiveSession(source);
    cueSequence = null; focusTarget = null; accumulator = 0; lastFrame = performance.now();
    clearTimeout(toastTimer); $('#toast').hidden = true;
    for (const [node, timer] of flashTimers) { clearTimeout(timer); node.classList.remove('cue-active'); }
    flashTimers.clear(); recentCues.length = 0;
    accept(JSON.parse(session.snapshot()));
  }
  dispatch(control.event, payload);
}

function sendPointer(point, active) {
  if (!running() || !point || !Number.isFinite(point.x) || !Number.isFinite(point.z)) return;
  lastPointer = { x: point.x, z: point.z };
  const payload = { ...lastPointer, active: active ? 1 : 0 };
  const event = snapshot.events?.find(item => item.name === snapshot.controls?.pointer?.event);
  // Projection can extend beyond the playable world on wide screens. Honor
  // the declared input contract before dispatch; source owns gameplay bounds.
  for (const parameter of event?.parameters || []) {
    if (Number.isFinite(payload[parameter.name]) && Number.isFinite(parameter.min) && Number.isFinite(parameter.max)) {
      payload[parameter.name] = clamp(payload[parameter.name], parameter.min, parameter.max);
    }
  }
  runControl('pointer', payload);
}

function placeLabels() {
  if (!world) return;
  const rect = canvas.getBoundingClientRect();
  for (const node of document.querySelectorAll('[data-world-label]')) {
    const point = world.getScreenPosition(node.dataset.worldLabel);
    const visible = snapshot.bindings?.[node.dataset.caveat]?.visible;
    node.hidden = !point?.visible || visible === false || visible === 0;
    if (point) { node.style.left = `${point.x * rect.width}px`; node.style.top = `${point.y * rect.height - Number(node.dataset.labelLift || 0)}px`; }
  }
}

function frame(now) {
  const dt = lastFrame ? Math.min(.1, Math.max(0, (now - lastFrame) / 1000)) : 0;
  lastFrame = now;
  try {
    if (running() && !document.hidden) {
      const clock = snapshot.clock;
      const step = Number(clock?.step);
      if (clock?.event && Number.isFinite(step) && step > 0) {
        accumulator = Math.min(Math.max(.1, step), accumulator + dt);
        while (accumulator >= step && running()) {
          if (keys.size) {
            const dx = (keys.has('ArrowRight') || keys.has('d') ? 1 : 0) - (keys.has('ArrowLeft') || keys.has('a') ? 1 : 0);
            const dz = (keys.has('ArrowDown') || keys.has('s') ? 1 : 0) - (keys.has('ArrowUp') || keys.has('w') ? 1 : 0);
            runControl('keyboard', { dx, dz, active: 1, dt: step });
          }
          accumulator -= step;
          if (running()) dispatch(clock.event, { dt: step });
        }
      }
    }
    placeLabels();
  } catch (error) {
    console.error(error);
    try { runControl('pause'); } catch { clearInput(); }
    showToast('The game paused. Try starting over.', 3);
  }
  requestAnimationFrame(frame);
}

canvas.addEventListener('pointerdown', event => {
  if (event.button !== 0 || !running()) return;
  event.preventDefault(); pointer = event.pointerId; keys.clear();
  canvas.setPointerCapture(pointer); canvas.focus({ preventScroll: true }); unlockSound();
  sendPointer(world.pointFromScreen(event.clientX, event.clientY), true);
});
canvas.addEventListener('pointermove', event => {
  if (event.pointerId !== pointer) return;
  event.preventDefault();
  sendPointer(world.pointFromScreen(event.clientX, event.clientY), true);
});
for (const name of ['pointerup', 'pointercancel', 'lostpointercapture']) canvas.addEventListener(name, event => {
  if (event.pointerId !== pointer) return;
  pointer = null;
  sendPointer(lastPointer, false);
});
window.addEventListener('keydown', event => {
  if (event.key === 'Escape' && session) { event.preventDefault(); runControl(running() ? 'pause' : 'resume'); return; }
  const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
  if (running() && ['ArrowLeft','ArrowRight','ArrowUp','ArrowDown','a','s','d','w',' '].includes(key)) {
    event.preventDefault(); keys.add(key); pointer = null; unlockSound();
  }
});
window.addEventListener('keyup', event => {
  const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
  if (!keys.delete(key)) return;
  if (!keys.size && running()) runControl('keyboard', { dx: 0, dz: 0, active: 0, dt: 0 });
});
window.addEventListener('blur', () => { if (session) runControl('pause'); });
document.addEventListener('visibilitychange', () => { if (document.hidden && session) runControl('pause'); });
document.addEventListener('click', event => {
  const control = event.target.closest('[data-control]');
  if (control && !control.disabled) runControl(control.dataset.control);
});
$('#sound').addEventListener('click', () => {
  soundEnabled = !soundEnabled;
  $('#sound').setAttribute('aria-pressed', String(soundEnabled));
  $('#sound').setAttribute('aria-label', soundEnabled ? 'Turn sound off' : 'Turn sound on');
  if (soundEnabled) unlockSound(); else audioContext?.suspend().catch(() => {});
});

async function boot() {
  const response = await fetch('./light_the_way.cav');
  if (!response.ok) throw new Error('The game source could not load.');
  source = await response.text();
  await init();
  session = new WebReactiveSession(source); snapshot = JSON.parse(session.snapshot());
  world = createRescueWorld(canvas, { model: snapshot.world, labels: snapshot.labels });
  $('#start-actions').innerHTML = '<button class="primary" id="start-rescue" data-control="start"><span class="button-label" data-caveat="start_label">Start rescue</span><span aria-hidden="true">→</span></button>';
  $('#sound').setAttribute('aria-pressed', String(soundEnabled));
  $('#sound').setAttribute('aria-label', soundEnabled ? 'Turn sound off' : 'Turn sound on');
  accept(snapshot);
  Object.defineProperty(window, '__rescue', { value: Object.freeze({
    snapshot: () => JSON.parse(session.snapshot()),
    recentCues: () => structuredClone(recentCues),
    presentation: () => structuredClone(world.getBindingPresentation()),
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
