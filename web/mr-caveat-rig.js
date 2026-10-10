// Mr. Caveat as an articulated vector rig for The Missing Dumpling.
//
// Drawn by hand from the governing character sheet (MrCaveatConcept.png, see
// caveatism/character/PROVENANCE.md): ivory round head, red nose, small eyes,
// red-and-black two-point hat with gold bells, ruffled collar, red coat over the
// ivory fortune panel and ticket slot, ivory limbs with gold joints, black
// greaves, curled boots with red soles, and the wind-up key on his back.
//
// The rig is posed procedurally each frame. The walk cycle advances with the
// distance actually travelled, and the body drops until the lower boot touches
// the ground line, so his feet neither skate nor float.
//
// Artwork: this vector rendition of Mr. Caveat (Caveatism project) is a hand
// drawing adapted from the character sheet, licensed CC BY-SA 4.0
// (https://creativecommons.org/licenses/by-sa/4.0/); see caveatism/LICENSE and
// caveatism/character/PROVENANCE.md. The code that poses it is MIT, like the
// rest of the site. Mr. Caveat is fiction.

// Sheet palette (PROVENANCE.md, sampled swatches), lifted slightly for the
// night market's lighting.
const C = {
  red: '#b8332a', redDark: '#7f201b', ink: '#282626', inkDeep: '#161314',
  ivory: '#f4eadc', ivoryShade: '#d6c8b4', gold: '#d1a54f', goldDark: '#8f6a2a',
  gray: '#525051', line: '#1b1214',
};

// Leg geometry in viewBox units: hip to knee, knee to sole.
const HIP_Y = 126, THIGH = 34, SHIN = 40, GROUND = 200;
const HIPS = [53, 67];

let instances = 0;
let styled = false;

function injectStyle() {
  if (styled) return;
  styled = true;
  const style = document.createElement('style');
  style.textContent = `
.mc-svg{width:100%;height:100%;overflow:visible;display:block;filter:drop-shadow(0 0 1.1px rgba(255,230,186,.62))}
.mc-svg.left{transform:scaleX(-1)}
.mc-svg .mc-alt{display:none}
.mc-svg.f-suspicious .mc-mouth-base,.mc-svg.f-panicked .mc-mouth-base,.mc-svg.f-proud .mc-mouth-base,.mc-svg.f-thinking .mc-mouth-base,.mc-svg.f-shrug .mc-mouth-base{display:none}
.mc-svg.f-suspicious .mc-mouth-flat,.mc-svg.f-thinking .mc-mouth-flat{display:inline}
.mc-svg.f-panicked .mc-mouth-o{display:inline}
.mc-svg.f-proud .mc-mouth-grin,.mc-svg.f-shrug .mc-mouth-o{display:inline}
.mc-svg.f-proud .mc-eyes{display:none}.mc-svg.f-proud .mc-eyes-happy{display:inline}
.mc-svg.f-panicked .mc-bang{display:inline}
.mc-svg.f-thinking .mc-dots{display:inline}
`;
  document.head.appendChild(style);
}

function markup(p) {
  const leg = (cls, x) => `
  <g class="mc-leg ${cls}">
    <rect x="${x - 4}" y="${HIP_Y - 2}" width="8" height="${THIGH + 2}" rx="3.6" fill="url(#${p}iv)" stroke="${C.line}" stroke-width="1.2"/>
    <g class="mc-shin">
      <rect x="${x - 4.4}" y="${HIP_Y + THIGH}" width="8.8" height="29" rx="3" fill="${C.ink}" stroke="${C.line}" stroke-width="1.2"/>
      <path d="M${x - 4.4} ${HIP_Y + THIGH + 7}h8.8M${x - 4.4} ${HIP_Y + THIGH + 24}h8.8" stroke="${C.gold}" stroke-width="1.5"/>
      <path d="M${x - 5.6} ${GROUND - 11}q0-3 3-3h6q4 0 4.6 3l1.4 3.4q3.4 1.4 4.6-1.8q1.6 4.2-2 6.6l-1 2.8h-17.2z" fill="${C.ink}" stroke="${C.line}" stroke-width="1.2" stroke-linejoin="round"/>
      <path d="M${x - 5.6} ${GROUND - 2.2}h16.6l-.6 2.2h-16z" fill="${C.red}"/>
      <circle cx="${x + 14.6}" cy="${GROUND - 10.6}" r="2.4" fill="${C.gold}" stroke="${C.goldDark}" stroke-width=".8"/>
    </g>
    <circle cx="${x}" cy="${HIP_Y + THIGH}" r="4.4" fill="${C.gold}" stroke="${C.goldDark}" stroke-width="1"/>
    <circle cx="${x}" cy="${HIP_Y + THIGH}" r="1.6" fill="${C.goldDark}"/>
  </g>`;
  const arm = (cls, x) => `
  <g class="mc-arm ${cls}">
    <rect x="${x - 3.8}" y="74" width="7.6" height="23" rx="3.4" fill="url(#${p}iv)" stroke="${C.line}" stroke-width="1.2"/>
    <g class="mc-fore">
      <rect x="${x - 3.4}" y="96" width="6.8" height="18" rx="3" fill="url(#${p}iv)" stroke="${C.line}" stroke-width="1.2"/>
      <rect x="${x - 4.2}" y="110" width="8.4" height="4.6" rx="1.4" fill="${C.ink}" stroke="${C.line}" stroke-width=".9"/>
      <rect x="${x - 4.2}" y="111.6" width="8.4" height="1.5" fill="${C.red}"/>
      <path d="M${x - 4.4} 115.6q-1 6.2 4.4 7.4q5.6-.6 4.6-7.2q-.4-1.4-1.6-1.4h-6q-1.2 0-1.4 1.2z" fill="#fbf6ee" stroke="${C.line}" stroke-width="1.1"/>
      <path d="M${x - 1.4} 117.4v3.2M${x + 1.4} 117.4v3.2" stroke="${C.ivoryShade}" stroke-width=".8"/>
    </g>
    <circle cx="${x}" cy="96.5" r="3.5" fill="${C.gold}" stroke="${C.goldDark}" stroke-width=".9"/>
    <circle cx="${x}" cy="76" r="6.2" fill="${C.gold}" stroke="${C.goldDark}" stroke-width="1.1"/>
    <circle cx="${x}" cy="76" r="2.4" fill="${C.goldDark}" opacity=".55"/>
  </g>`;
  const bell = (cls, x, y) => `
  <g class="mc-bell ${cls}" data-x="${x}" data-y="${y}">
    <circle cx="${x}" cy="${y + 3.6}" r="3.9" fill="${C.gold}" stroke="${C.goldDark}" stroke-width="1"/>
    <path d="M${x - 3.8} ${y + 3.4}h7.6M${x} ${y + 5}v2.4" stroke="${C.goldDark}" stroke-width=".9"/>
    <circle cx="${x - 1.2}" cy="${y + 2.2}" r="1" fill="#fff2c4" opacity=".8"/>
  </g>`;
  // Collar: two rings of alternating red and black ruffle petals.
  let collar = '';
  for (let i = 0; i < 11; i++) {
    const t = i / 10, cx = 35 + t * 50, cy = 67.5 + Math.sin(t * Math.PI) * 3.4;
    collar += `<ellipse cx="${cx.toFixed(1)}" cy="${cy.toFixed(1)}" rx="4.4" ry="3.6" fill="${i % 2 ? C.ink : C.red}" stroke="${C.line}" stroke-width=".9"/>`;
  }
  for (let i = 0; i < 8; i++) {
    const t = i / 7, cx = 40 + t * 40, cy = 64.6 + Math.sin(t * Math.PI) * 2;
    collar += `<ellipse cx="${cx.toFixed(1)}" cy="${cy.toFixed(1)}" rx="3.8" ry="3" fill="${i % 2 ? C.red : C.ink}" stroke="${C.line}" stroke-width=".8"/>`;
  }
  return `
<svg class="mc-svg" viewBox="0 0 120 200" aria-hidden="true" focusable="false">
  <defs>
    <linearGradient id="${p}iv" x1="0" x2="1"><stop offset="0" stop-color="${C.ivory}"/><stop offset=".62" stop-color="${C.ivory}"/><stop offset="1" stop-color="${C.ivoryShade}"/></linearGradient>
    <radialGradient id="${p}head" cx=".38" cy=".34" r=".72"><stop offset="0" stop-color="#fffaf1"/><stop offset=".6" stop-color="${C.ivory}"/><stop offset="1" stop-color="${C.ivoryShade}"/></radialGradient>
    <linearGradient id="${p}coat" x1="0" x2="1"><stop offset="0" stop-color="${C.redDark}"/><stop offset=".3" stop-color="${C.red}"/><stop offset=".72" stop-color="${C.red}"/><stop offset="1" stop-color="${C.redDark}"/></linearGradient>
    <clipPath id="${p}skull"><circle cx="60" cy="41" r="18.4"/></clipPath>
  </defs>
  <g class="mc-root">
    <g class="mc-key">
      <rect x="28" y="88" width="12" height="4" rx="1" fill="${C.goldDark}" stroke="${C.line}" stroke-width=".6"/>
      <g transform="translate(-10.7 -18) scale(1.2)"><g class="mc-bow">
        <path d="M27.6 90c-1.6-3-2.6-6.8-1.4-10.4c1.6-4.4 8-4.4 9 .2c.8 3.6-3.4 6.8-7.6 10.2c4.2 3.4 8.4 6.6 7.6 10.2c-1 4.6-7.4 4.6-9 .2c-1.2-3.6-.2-7.4 1.4-10.4z" fill="${C.gold}" stroke="${C.goldDark}" stroke-width="1.2" stroke-linejoin="round"/>
        <circle cx="30.6" cy="81.4" r="1.7" fill="${C.goldDark}"/><circle cx="30.6" cy="98.6" r="1.7" fill="${C.goldDark}"/>
      </g></g>
    </g>
    ${arm('mc-arm-b', 39)}
    ${leg('mc-leg-b', HIPS[0])}
    ${leg('mc-leg-f', HIPS[1])}
    <g class="mc-body">
      <path d="M46 116h28v14q-14 3-28 0z" fill="${C.ink}" stroke="${C.line}" stroke-width="1.2"/>
      <path d="M41 70h38l5 46l3 14l-8-4l-5 3l-2-13h-24l-2 13l-5-3l-8 4l3-14z" fill="url(#${p}coat)" stroke="${C.line}" stroke-width="1.3" stroke-linejoin="round"/>
      <rect x="47.5" y="72" width="25" height="45" rx="2.4" fill="${C.ivory}" stroke="${C.gold}" stroke-width="1.8"/>
      <path d="M49.6 110h20.8" stroke="${C.gold}" stroke-width="1.1"/>
      <rect x="52.4" y="77" width="15.2" height="6.2" rx="1.2" fill="${C.gold}" stroke="${C.goldDark}" stroke-width="1"/>
      <rect x="54.6" y="79.4" width="10.8" height="1.6" rx=".8" fill="${C.inkDeep}"/>
      <g class="mc-ticket">
        <rect x="55.2" y="80" width="9.6" height="11" rx=".8" fill="#fffdf6" stroke="${C.ivoryShade}" stroke-width=".7"/>
        <path d="M56.8 83.6h6.4M57.4 85.8h5.2" stroke="${C.red}" stroke-width="1.1"/>
        <path d="M57 88.6h6" stroke="${C.red}" stroke-width=".6"/>
      </g>
      <circle cx="76" cy="98" r="1.4" fill="${C.gold}"/><circle cx="76" cy="105" r="1.4" fill="${C.gold}"/>
      <circle cx="44" cy="98" r="1.4" fill="${C.gold}"/>
      ${collar}
    </g>
    <g class="mc-head">
      <rect x="55.6" y="56" width="8.8" height="9" rx="1.6" fill="${C.gold}" stroke="${C.goldDark}" stroke-width="1"/>
      <path class="mc-horn-r" d="M66 25c7-9 18-13 26-9c5 2.6 7.4 8 6.8 16.6c-2.6-6-6.4-8.4-11-8.4c-6 0-11 2.6-16 5.6z" fill="${C.ink}" stroke="${C.line}" stroke-width="1.3" stroke-linejoin="round"/>
      <path class="mc-horn-l" d="M54 25c-7-9-18-13-26-9c-5 2.6-7.4 8-6.8 16.6c2.6-6 6.4-8.4 11-8.4c6 0 11 2.6 16 5.6z" fill="${C.red}" stroke="${C.line}" stroke-width="1.3" stroke-linejoin="round"/>
      ${bell('mc-bell-l', 21.4, 31.4)}
      ${bell('mc-bell-r', 98.6, 31.4)}
      <circle cx="41.6" cy="43" r="4.6" fill="${C.gold}" stroke="${C.goldDark}" stroke-width="1"/>
      <circle cx="78.4" cy="43" r="4.6" fill="${C.gold}" stroke="${C.goldDark}" stroke-width="1"/>
      <circle cx="60" cy="41" r="18.4" fill="url(#${p}head)" stroke="${C.line}" stroke-width="1.4"/>
      <g clip-path="url(#${p}skull)">
        <rect x="38" y="18" width="22" height="18" fill="${C.red}"/>
        <rect x="60" y="18" width="24" height="18" fill="${C.ink}"/>
        <path d="M44 52q-2 4 1 7" stroke="${C.ivoryShade}" stroke-width=".8" fill="none"/>
      </g>
      <circle cx="60" cy="41" r="18.4" fill="none" stroke="${C.line}" stroke-width="1.4"/>
      <path d="M40.6 32.6q19.4 5.4 38.8 0l.4 4.6q-19.8 5.6-39.6 0z" fill="${C.gold}" stroke="${C.goldDark}" stroke-width="1" stroke-linejoin="round"/>
      <g class="mc-face">
        <g class="mc-brows" stroke="${C.inkDeep}" stroke-width="1.3" stroke-linecap="round" fill="none">
          <path class="mc-brow-l" d="M51.6 37.4q2.6-1.8 5 -.4"/>
          <path class="mc-brow-r" d="M63.4 37q2.4-1.6 5 .2"/>
        </g>
        <g class="mc-eyes">
          <g class="mc-eye mc-eye-l"><ellipse cx="54.4" cy="41.6" rx="2.1" ry="2.7" fill="${C.inkDeep}"/><circle cx="55" cy="40.7" r=".75" fill="#fff"/></g>
          <g class="mc-eye mc-eye-r"><ellipse cx="66" cy="41.6" rx="2.1" ry="2.7" fill="${C.inkDeep}"/><circle cx="66.6" cy="40.7" r=".75" fill="#fff"/></g>
        </g>
        <path class="mc-alt mc-eyes-happy" d="M52.2 42q2.2-2.6 4.4 0M63.8 42q2.2-2.6 4.4 0" stroke="${C.inkDeep}" stroke-width="1.4" fill="none" stroke-linecap="round"/>
        <circle cx="60.8" cy="47.4" r="4.4" fill="#d6261f" stroke="${C.line}" stroke-width="1"/>
        <circle cx="59.4" cy="46" r="1.3" fill="#ff8f86"/>
        <path class="mc-mouth-base" d="M55 53.6q4.6 3 9.2-.6" stroke="${C.inkDeep}" stroke-width="1.2" fill="none" stroke-linecap="round"/>
        <path class="mc-alt mc-mouth-flat" d="M56 54.4q4-.8 8 .4" stroke="${C.inkDeep}" stroke-width="1.2" fill="none" stroke-linecap="round"/>
        <ellipse class="mc-alt mc-mouth-o" cx="60.4" cy="54.6" rx="1.7" ry="2" fill="${C.inkDeep}"/>
        <path class="mc-alt mc-mouth-grin" d="M54.6 53q5 4.4 10.4-.2" stroke="${C.inkDeep}" stroke-width="1.4" fill="none" stroke-linecap="round"/>
      </g>
      <path class="mc-alt mc-bang" d="M86 6l-2.2 13M83 23.4v.1" stroke="#ffd36e" stroke-width="3.2" stroke-linecap="round"/>
      <g class="mc-alt mc-dots" fill="#fff4db"><circle cx="82" cy="20" r="1.6"/><circle cx="87" cy="14" r="2.1"/><circle cx="93" cy="7" r="2.7"/></g>
    </g>
    ${arm('mc-arm-f', 81)}
  </g>
</svg>`;
}

const lerp = (a, b, t) => a + (b - a) * t;
const ease = (rate, dt) => 1 - Math.exp(-rate * dt);
const rad = d => d * Math.PI / 180;

// Gesture poses. Each arm names where its glove should go (viewBox units, as
// drawn facing right) and a two-bone solve finds the shoulder and elbow.
const UPPER = 20.5, FORE = 24;
const GESTURES = {
  none: null,
  thinking: { b: [37, 120], f: [64, 59], head: -7, lean: 0 },     // hand to chin, eyes up
  suspicious: { b: [46, 117], f: [66, 60], head: 8, lean: 4 },    // squint, lean in
  panicked: { b: [22, 70], f: [61, 57], head: -5, lean: -3 },     // "Oh...": hand to mouth
  proud: { b: [37, 120], f: [110, 64], head: -9, lean: -2 },      // open palm out
  shrug: { b: [16, 82], f: [104, 82], bend: [-1, 1], head: 6, lean: 0 },        // "Still a chance!"
};

// Shoulder and elbow angles (degrees, clockwise from hanging straight down)
// that put the glove at target (tx, ty) from shoulder (sx, sy). bend picks the
// elbow side: +1 bends the forearm forward, -1 backward.
function solveArm(sx, sy, tx, ty, bend) {
  const dx = tx - sx, dy = ty - sy;
  const d = Math.min(UPPER + FORE - 0.01, Math.max(Math.abs(UPPER - FORE) + 0.01, Math.hypot(dx, dy)));
  const toTarget = Math.atan2(-dx, dy); // angle of (dx, dy) from straight down
  const inner = Math.acos((UPPER * UPPER + d * d - FORE * FORE) / (2 * UPPER * d));
  const elbow = Math.PI - Math.acos((UPPER * UPPER + FORE * FORE - d * d) / (2 * UPPER * FORE));
  const shoulder = toTarget + bend * inner;
  return [shoulder * 180 / Math.PI, -bend * elbow * 180 / Math.PI];
}

export function mountMrCaveat(host) {
  injectStyle();
  const p = 'mc' + (++instances) + '-';
  host.insertAdjacentHTML('afterbegin', markup(p));
  const svg = host.querySelector('.mc-svg');
  const q = s => svg.querySelector(s);
  const parts = {
    root: q('.mc-root'), head: q('.mc-head'), body: q('.mc-body'), bow: q('.mc-bow'),
    ticket: q('.mc-ticket'), eyes: q('.mc-eyes'),
    legs: [q('.mc-leg-b'), q('.mc-leg-f')], shins: [q('.mc-leg-b .mc-shin'), q('.mc-leg-f .mc-shin')],
    arms: [q('.mc-arm-b'), q('.mc-arm-f')], fores: [q('.mc-arm-b .mc-fore'), q('.mc-arm-f .mc-fore')],
    bells: [q('.mc-bell-l'), q('.mc-bell-r')], browL: q('.mc-brow-l'), browR: q('.mc-brow-r'),
  };
  const armX = [39, 81];

  const s = {
    t: 0, phase: 0, amp: 0, run: 0, air: 0, faceLeft: false, mode: 'none', modeUntil: 0,
    g: 0, gPose: GESTURES.thinking, bell: [0, 0], bellV: [0, 0], key: 0, keyRest: 0, keyTick: 1.6,
    blinkAt: 2.4, blink: 0, ticket: 0, lookX: 0, idle: 0, prevV: 0,
  };

  function face(dir) {
    const left = dir < 0;
    if (left !== s.faceLeft) { s.faceLeft = left; svg.classList.toggle('left', left); }
  }

  function setMode(mode, seconds = 0) {
    s.mode = mode in GESTURES ? mode : 'none';
    s.modeUntil = seconds ? s.t + seconds : Infinity;
    if (s.mode !== 'none') s.gPose = GESTURES[s.mode];
    svg.classList.remove('f-thinking', 'f-suspicious', 'f-panicked', 'f-proud', 'f-shrug');
    if (s.mode !== 'none') svg.classList.add('f-' + s.mode);
  }

  // input: dt seconds, dist = |ground distance| this frame in viewBox units,
  // dir = -1 | 0 | 1 movement direction, air = height above ground (0 grounded),
  // running = true in the chase.
  function update(dt, input) {
    dt = Math.min(dt, 0.1);
    s.t += dt;
    if (s.mode !== 'none' && s.t >= s.modeUntil) setMode('none');
    const speed = input.dist / Math.max(dt, 1e-4); // units per second
    if (input.dir && !input.running) face(input.dir);
    s.run = lerp(s.run, input.running ? 1 : 0, ease(6, dt));
    const grounded = input.air <= 0.5;
    s.air = lerp(s.air, grounded ? 0 : 1, ease(grounded ? 22 : 12, dt));

    // Stride amplitude follows speed; cadence follows distance, so the planted
    // boot moves with the ground instead of sliding across it.
    const ampTarget = grounded ? Math.min(1, speed / (input.running ? 110 : 150)) : 0;
    s.amp = lerp(s.amp, ampTarget, ease(ampTarget > s.amp ? 10 : 7, dt));
    const swing = lerp(lerp(10, 30, s.amp), 40, s.run); // degrees
    const stride = 4 * (THIGH + SHIN) * Math.sin(rad(Math.max(swing * s.amp, 7))) * (1 + 0.5 * s.run);
    let cycles = input.dist / stride;
    if (input.running) cycles = Math.min(Math.max(cycles, 2.3 * dt), 3.4 * dt);
    if (grounded) s.phase = (s.phase + cycles) % 1;
    const th = s.phase * Math.PI * 2;
    const A = swing * s.amp;
    const kneeMax = lerp(40, 82, s.run) * s.amp;

    // Idle weight shift and breathing.
    s.idle = lerp(s.idle, s.amp < 0.08 && grounded ? 1 : 0, ease(3, dt));
    const breathe = Math.sin(s.t * 2.2) * 0.7 * s.idle;
    const shift = Math.sin(s.t * 0.9) * 2.4 * s.idle;

    const hip = [0, 0], knee = [0, 0];
    for (let i = 0; i < 2; i++) {
      const ph = th + i * Math.PI;
      hip[i] = -A * Math.sin(ph) + (i ? -shift : shift) * 0.6;
      knee[i] = kneeMax * Math.max(0, -Math.cos(ph)) + 4 * s.amp + (i ? 0 : 1.5 * s.idle);
      // Airborne tuck.
      hip[i] = lerp(hip[i], i ? -52 : 18, s.air);
      knee[i] = lerp(knee[i], i ? 78 : 46, s.air);
    }
    // Ground contact: drop the rig until the lower boot reaches the ground line.
    let lowest = 0;
    for (let i = 0; i < 2; i++) {
      const a = rad(hip[i]), b = rad(hip[i] + knee[i]);
      lowest = Math.max(lowest, HIP_Y + THIGH * Math.cos(a) + SHIN * Math.cos(b));
    }
    const drop = grounded ? GROUND - lowest : 0;

    // Gesture blend (thinking, suspicious, panicked, proud).
    s.g = lerp(s.g, s.mode === 'none' ? 0 : 1, ease(s.mode === 'none' ? 6 : 11, dt));
    const G = s.gPose;
    const armSwing = lerp(0.75, 1.1, s.run);
    const sh = [hip[1] * armSwing, hip[0] * armSwing];
    const el = [-(10 + 12 * s.amp + 70 * s.run), -(10 + 12 * s.amp + 70 * s.run)];
    for (let i = 0; i < 2; i++) {
      sh[i] = lerp(sh[i], i ? -40 : -30, s.air * 0.85) + (i ? -1 : 1) * 3 * s.idle;
      el[i] = lerp(el[i], -40, s.air * 0.6);
      const gp = i ? G.f : G.b;
      const [gs, ge] = solveArm(armX[i], 76, gp[0], gp[1], G.bend ? G.bend[i] : -1);
      sh[i] = lerp(sh[i], gs, s.g * (s.amp > 0.5 ? 0.45 : 1));
      el[i] = lerp(el[i], ge, s.g * (s.amp > 0.5 ? 0.45 : 1));
    }
    if (s.mode === 'panicked') { sh[0] += Math.sin(s.t * 38) * 3; sh[1] += Math.sin(s.t * 41) * 3; }

    const lean = lerp(0, 9, s.run) * s.amp + G.lean * s.g - 3 * s.air;
    const bob = drop + breathe;
    const headTilt = G.head * s.g + Math.sin(s.t * 0.7) * 2.2 * s.idle - 2 * s.run * Math.sin(th * 2);

    // Bells: damped springs pushed by bob and travel.
    const accel = (speed - s.prevV) / Math.max(dt, 1e-3);
    s.prevV = speed;
    for (let i = 0; i < 2; i++) {
      const target = -lean * 1.4 + Math.sin(th * 2 + i) * 9 * s.amp + (s.mode === 'panicked' ? Math.sin(s.t * 30 + i) * 14 : 0);
      s.bellV[i] += ((target - s.bell[i]) * 90 - s.bellV[i] * 7 - accel * 0.004) * dt;
      s.bell[i] += s.bellV[i] * dt;
    }

    // Wind-up key: spins with travel; at rest it settles face-on and
    // ratchets half a turn now and then.
    if (s.amp > 0.05 || !grounded) {
      s.key += dt * (5 + 9 * s.amp + 6 * s.run);
      s.keyRest = Math.ceil(s.key / Math.PI) * Math.PI;
      s.keyTick = 1.6;
    } else {
      s.keyTick -= dt;
      if (s.keyTick <= 0) { s.keyTick = 1.6; s.keyRest += Math.PI; }
      s.key = lerp(s.key, s.keyRest, ease(12, dt));
    }

    // Blink.
    s.blinkAt -= dt;
    if (s.blinkAt <= 0) { s.blink = 0.14; s.blinkAt = 2.2 + Math.random() * 3.2; }
    s.blink = Math.max(0, s.blink - dt);

    // Ticket peeks out of the fortune slot for thinking and announcements.
    s.ticket = lerp(s.ticket, s.mode === 'proud' ? 1 : s.mode === 'thinking' ? 0.55 : 0, ease(8, dt));

    // Apply.
    const f = n => n.toFixed(2);
    parts.root.setAttribute('transform', `translate(0 ${f(bob)}) rotate(${f(lean)} 60 ${GROUND})`);
    for (let i = 0; i < 2; i++) {
      const hx = HIPS[i];
      parts.legs[i].setAttribute('transform', `rotate(${f(hip[i])} ${hx} ${HIP_Y})`);
      parts.shins[i].setAttribute('transform', `rotate(${f(knee[i])} ${hx} ${HIP_Y + THIGH})`);
      parts.arms[i].setAttribute('transform', `rotate(${f(sh[i])} ${armX[i]} 76)`);
      parts.fores[i].setAttribute('transform', `rotate(${f(el[i])} ${armX[i]} 96.5)`);
      const b = parts.bells[i];
      b.setAttribute('transform', `rotate(${f(Math.max(-40, Math.min(40, s.bell[i])))} ${b.dataset.x} ${b.dataset.y})`);
    }
    parts.body.setAttribute('transform', `translate(0 ${f(breathe * 0.4)})`);
    parts.head.setAttribute('transform', `translate(0 ${f(breathe * 0.5 - s.amp * 0.8 * Math.abs(Math.sin(th * 2)))}) rotate(${f(headTilt)} 60 60)`);
    const k = Math.cos(s.key);
    parts.bow.setAttribute('transform', `translate(0 90) scale(1 ${f(Math.abs(k) < 0.08 ? 0.08 * Math.sign(k || 1) : k)}) translate(0 -90)`);
    parts.ticket.setAttribute('transform', `translate(0 ${f(s.ticket * 9)})`);
    const eyeY = s.blink > 0 ? 0.12 : 1;
    const squint = s.mode === 'suspicious' ? 0.42 : s.mode === 'panicked' ? 1.25 : 1;
    parts.eyes.children[0].setAttribute('transform', `translate(54.4 41.6) scale(1 ${f(eyeY * (s.mode === 'panicked' ? 1.25 : 1))}) translate(-54.4 -41.6)`);
    parts.eyes.children[1].setAttribute('transform', `translate(66 41.6) scale(1 ${f(eyeY * squint)}) translate(-66 -41.6)`);
    const browLift = s.mode === 'panicked' || s.mode === 'shrug' ? -2.4 : s.mode === 'thinking' ? -1.2 : 0;
    parts.browL.setAttribute('transform', `translate(0 ${f(browLift + (s.mode === 'suspicious' ? -1.6 : 0))})`);
    parts.browR.setAttribute('transform', s.mode === 'suspicious' ? 'translate(0 1.4) rotate(12 66 37)' : `translate(0 ${f(browLift)})`);
  }

  return { svg, update, setMode, face, get mode() { return s.mode; } };
}
