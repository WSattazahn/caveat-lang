/**
 * A procedural coastal presentation kit for CAVEAT worlds.
 *
 * The .cav program owns the graph, availability, evidence and consequences.
 * This module only gives declared place/entity kinds a visual representation
 * and animates the generic commands emitted by the action runtime. In
 * particular, it never selects an action or derives an ending from an ID.
 */
import * as THREE from './vendor/three.module.js';

const TAU = Math.PI * 2;
const clamp = THREE.MathUtils.clamp;
const smooth = t => t * t * (3 - 2 * t);
const v = (x = 0, y = 0, z = 0) => new THREE.Vector3(x, y, z);
const UP = v(0, 1, 0);


function seeded(seed = 81733) {
  return () => {
    seed = (Math.imul(seed, 1664525) + 1013904223) | 0;
    return (seed >>> 0) / 4294967296;
  };
}

function radialTexture() {
  const el = document.createElement('canvas');
  el.width = el.height = 128;
  const ctx = el.getContext('2d');
  const gradient = ctx.createRadialGradient(64, 64, 0, 64, 64, 64);
  gradient.addColorStop(0, 'rgba(255,255,255,1)');
  gradient.addColorStop(0.08, 'rgba(255,255,255,.85)');
  gradient.addColorStop(0.25, 'rgba(255,255,255,.18)');
  gradient.addColorStop(1, 'rgba(255,255,255,0)');
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, 128, 128);
  return new THREE.CanvasTexture(el);
}

export function createBeaconWorld(canvas, options = {}) {
  const rescueMode = options.mode === 'rescue';
  const random = seeded();
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x071c2b);
  scene.fog = new THREE.FogExp2(0x092736, rescueMode ? 0.0032 : 0.0078);
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: false, powerPreference: 'high-performance' });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 1.7));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = rescueMode ? 1.4 : 1.22;
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  const camera = new THREE.PerspectiveCamera(43, 1, 0.1, 750);
  const baseEye = v(30, 25, 35);
  const baseTarget = v(0, 0, 0);
  const cameraEye = baseEye.clone();
  const cameraTarget = baseTarget.clone();
  camera.position.copy(baseEye);
  camera.lookAt(baseTarget);
  const world = new THREE.Group();
  const graph = new THREE.Group();
  const landscape = new THREE.Group();
  scene.add(world, graph, landscape);
  const glowTexture = radialTexture();
  const materialCache = new Map();
  const geometryCache = new Map();
  const animated = [];
  const places = new Map();
  const entities = new Map();
  const routes = new Map();
  const lamps = [];
  const clicks = [];
  const tweens = new Set();
  let model;
  let currentPlace;
  let alive = true;
  let busy = false;
  let raf = 0;
  let lastTime = performance.now();
  let elapsed = 0;
  let yaw = 0;
  let pitch = 0;
  let zoom = 1;
  let drag = null;
  let resetGeneration = 0;
  let reducedMotion = window.matchMedia?.('(prefers-reduced-motion: reduce)')?.matches || false;
  let immediatePlayback = false;
  let needsRender = true;
  let presentation = {};
  let authoredPositions = new Map();
  const renderTargets = new Map();
  const activeCues = [];
  const seenCues = new Set();
  const boundProperties = new Map();
  const appliedBindingValues = new Map();
  const isolatedMaterials = new WeakSet();
  let lastBindingSequence = -1;
  let lastBindingSession = null;
  const raycaster = new THREE.Raycaster();
  const pointer = new THREE.Vector2();

  function material(color, opts = {}) {
    const key = JSON.stringify([color, opts]);
    if (!materialCache.has(key)) materialCache.set(key, new THREE.MeshStandardMaterial({ color, roughness: 0.85, metalness: 0.04, ...opts }));
    return materialCache.get(key);
  }
  function geo(key, create) {
    if (!geometryCache.has(key)) geometryCache.set(key, create());
    return geometryCache.get(key);
  }
  function mesh(geometry, mat, parent, position, scale) {
    const m = new THREE.Mesh(geometry, typeof mat === 'number' ? material(mat) : mat);
    if (position) m.position.set(...position);
    if (scale) m.scale.set(...scale);
    m.castShadow = true;
    m.receiveShadow = true;
    (parent || world).add(m);
    return m;
  }
  function box(parent, x, y, z, w, h, d, color, opts) {
    return mesh(geo('box', () => new THREE.BoxGeometry(1, 1, 1)), material(color, opts), parent, [x, y, z], [w, h, d]);
  }
  function cylinder(parent, x, y, z, rt, rb, h, color, segments = 12, opts) {
    return mesh(geo(`cyl:${rt}:${rb}:${h}:${segments}`, () => new THREE.CylinderGeometry(rt, rb, h, segments)), material(color, opts), parent, [x, y, z]);
  }
  function sphere(parent, x, y, z, r, color, opts) {
    return mesh(geo('sphere', () => new THREE.SphereGeometry(1, 16, 12)), material(color, opts), parent, [x, y, z], [r, r, r]);
  }
  function beam(parent, a, b, radius, color) {
    const start = v(...a), end = v(...b);
    const m = cylinder(parent, 0, 0, 0, radius, radius, start.distanceTo(end), color, 6);
    m.position.copy(start.add(end).multiplyScalar(0.5));
    m.quaternion.setFromUnitVectors(UP, end.sub(v(...a)).normalize());
    return m;
  }
  function glow(parent, position, color, size = 3, opacity = 0.75) {
    const m = new THREE.SpriteMaterial({ map: glowTexture, color, transparent: true, opacity, depthWrite: false, blending: THREE.AdditiveBlending });
    const s = new THREE.Sprite(m);
    s.position.set(...position);
    s.scale.set(size, size, 1);
    parent.add(s);
    return s;
  }
  function warmWindow(parent, x, y, z, w, h, rotation = 0) {
    const frame = new THREE.Group();
    frame.position.set(x, y, z);
    frame.rotation.y = rotation;
    parent.add(frame);
    box(frame, 0, 0, 0, w + 0.18, h + 0.18, 0.13, 0x243d43);
    box(frame, 0, 0, 0.08, w, h, 0.045, 0xffb258, { emissive: 0xff912f, emissiveIntensity: 0.85, roughness: 0.3 });
    box(frame, 0, 0, 0.12, 0.06, h, 0.035, 0x293c40);
    box(frame, 0, 0, 0.12, w, 0.055, 0.035, 0x293c40);
    glow(frame, [0, 0, 0.2], 0xffad50, w * 4, 0.3);
  }
  function lantern(parent, x, y, z, height = 1.9) {
    cylinder(parent, x, y + height * 0.5, z, 0.055, 0.085, height, 0x30424a, 6);
    const top = y + height;
    box(parent, x, top, z, 0.31, 0.47, 0.31, 0xeba558, { emissive: 0xff9f43, emissiveIntensity: 1.1 });
    cylinder(parent, x, top + 0.29, z, 0.02, 0.3, 0.23, 0x253c43, 4);
    cylinder(parent, x, top - 0.29, z, 0.22, 0.22, 0.12, 0x293d43, 4);
    const halo = glow(parent, [x, top, z], 0xffae5c, 2.25, 0.72);
    lamps.push({ halo, phase: random() * TAU });
  }

  // Atmosphere and water are cosmetic. They never change runtime availability.
  const sky = mesh(new THREE.SphereGeometry(370, 32, 20), new THREE.ShaderMaterial({
    side: THREE.BackSide,
    depthWrite: false,
    uniforms: { top: { value: new THREE.Color(0x030b1a) }, horizon: { value: new THREE.Color(0x264755) } },
    vertexShader: 'varying vec3 vP;void main(){vP=position;gl_Position=projectionMatrix*modelViewMatrix*vec4(position,1.0);}',
    fragmentShader: 'uniform vec3 top;uniform vec3 horizon;varying vec3 vP;void main(){float t=clamp(normalize(vP).y,0.0,1.0);vec3 c=mix(horizon,top,pow(t,.5));gl_FragColor=vec4(c,1.);}',
  }), scene);
  sky.castShadow = sky.receiveShadow = false;
  const moon = sphere(scene, -90, 104, -180, 6.1, 0xbbd5d7, { emissive: 0xc1dfe4, emissiveIntensity: 0.7 });
  moon.castShadow = moon.receiveShadow = false;
  glow(scene, [-90, 104, -180], 0x85b1c8, 37, 0.32);

  const ambient = new THREE.HemisphereLight(0x91c7da, 0x12271f, 2.1);
  const moonlight = new THREE.DirectionalLight(0xabc9e9, 2.5);
  moonlight.position.set(-32, 55, 18);
  moonlight.castShadow = true;
  moonlight.shadow.mapSize.set(1024, 1024);
  Object.assign(moonlight.shadow.camera, { left: -28, right: 28, top: 30, bottom: -24, near: 1, far: 130 });
  moonlight.shadow.bias = -0.001;
  moonlight.shadow.normalBias = 0.04;
  const rim = new THREE.DirectionalLight(0x458b93, 1.2);
  rim.position.set(20, 12, -35);
  scene.add(ambient, moonlight, rim);

  const waterUniforms = { time: { value: 0 }, cameraPositionWorld: { value: camera.position }, lightDirection: { value: v(-0.45, 0.8, 0.3) }, shoreCenter: { value: new THREE.Vector2() }, shoreRadius: { value: new THREE.Vector2(16, 14) } };
  const water = mesh(new THREE.PlaneGeometry(550, 550, 120, 120), new THREE.ShaderMaterial({
    uniforms: waterUniforms,
    transparent: false,
    vertexShader: `
      uniform float time; varying vec3 vWorld; varying vec3 vNormal; varying float vWave;
      void main(){
        vec3 p=position; vec4 w=modelMatrix*vec4(p,1.);
        float a=sin(w.x*.31+time*.8+w.z*.12);
        float b=sin(w.z*.44-time*.57+w.x*.16);
        float c=sin(w.x*.85+w.z*.71+time*.9);
        float wave=a*.16+b*.11+c*.035;
        w.y+=wave; vWorld=w.xyz; vWave=wave;
        vNormal=normalize(vec3(-cos(w.x*.31+time*.8+w.z*.12)*.0496-cos(w.z*.44-time*.57+w.x*.16)*.0176,1.,-cos(w.x*.31+time*.8+w.z*.12)*.0192-cos(w.z*.44-time*.57+w.x*.16)*.0484));
        gl_Position=projectionMatrix*viewMatrix*w;
      }`,
    fragmentShader: `
      uniform float time; uniform vec3 cameraPositionWorld; uniform vec3 lightDirection; uniform vec2 shoreCenter; uniform vec2 shoreRadius;
      varying vec3 vWorld;varying vec3 vNormal;varying float vWave;
      void main(){
        vec3 N=normalize(vNormal);vec3 V=normalize(cameraPositionWorld-vWorld);
        float fresnel=pow(1.-max(dot(N,V),0.),3.);
        float sparkle=pow(max(dot(N,normalize(V+lightDirection)),0.),100.);
        float ripples=sin(vWorld.x*1.6+vWorld.z*.7+sin(vWorld.z*1.9-time)*.5+time)*.5+.5;
        vec3 color=mix(vec3(.018,.10,.13),vec3(.12,.25,.29),fresnel);
        color+=vec3(.22,.38,.4)*sparkle*.7;
        color+=vec3(.025,.08,.08)*smoothstep(.91,1.,ripples)*smoothstep(-.1,.2,vWave);
        float shore=length((vWorld.xz-shoreCenter)/shoreRadius);
        float surf=pow(max(0.,sin(shore*32.-time*1.1+sin(atan(vWorld.z-shoreCenter.y,vWorld.x-shoreCenter.x)*9.)*.5)),16.);
        float edge=smoothstep(.91,1.05,shore)*(1.-smoothstep(1.15,1.45,shore));
        color+=vec3(.26,.44,.43)*surf*edge*.46;
        float mist=1.-exp(-length(vWorld.xz-cameraPositionWorld.xz)*.006);
        color=mix(color,vec3(.075,.15,.19),mist*.7);
        gl_FragColor=vec4(color,1.);
      }`,
  }), world, [0, -0.05, 0]);
  water.rotation.x = -Math.PI / 2;
  water.castShadow = water.receiveShadow = false;

  const starPositions = [];
  for (let i = 0; i < 390; i++) {
    const a = random() * TAU, y = 0.15 + random() * 0.84;
    const r = Math.sqrt(1 - y * y) * 280;
    starPositions.push(Math.cos(a) * r, y * 280, Math.sin(a) * r);
  }
  const stars = new THREE.Points(new THREE.BufferGeometry().setAttribute('position', new THREE.Float32BufferAttribute(starPositions, 3)), new THREE.PointsMaterial({ color: 0xaec9d3, size: 0.33, transparent: true, opacity: 0.64, depthWrite: false }));
  scene.add(stars);

  function rockTerrace(cx, cz, radiusX, radiusZ, bottom, top, seed, topColor = 0x334f48) {
    const rng = seeded(seed), n = 32, angles = [], outer = [];
    for (let i = 0; i < n; i++) {
      angles.push(i / n * TAU);
      outer.push(0.92 + rng() * 0.13);
    }
    const positions = [], colors = [];
    const palette = [0x294049, 0x344b50, 0x3d5353, 0x273d45, 0x3d5151].map(c => new THREE.Color(c));
    const grass = new THREE.Color(topColor);
    const add = (a, b, c, color) => { positions.push(...a, ...b, ...c); for (let j = 0; j < 3; j++) colors.push(color.r, color.g, color.b); };
    for (let i = 0; i < n; i++) {
      const next = (i + 1) % n;
      const vertex = (j, y, factor) => [cx + Math.cos(angles[j]) * radiusX * outer[j] * factor, y, cz + Math.sin(angles[j]) * radiusZ * outer[j] * factor];
      const a = vertex(i, bottom, 1.04), b = vertex(next, bottom, 1.04), c = vertex(i, top, 0.91), d = vertex(next, top, 0.91);
      const color = palette[Math.floor(rng() * palette.length)];
      add(a, c, b, color); add(b, c, d, color);
      add(d, c, [cx, top, cz], grass.clone().multiplyScalar(0.92 + rng() * 0.12));
    }
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geometry.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
    geometry.computeVertexNormals();
    mesh(geometry, material(0xffffff, { vertexColors: true, flatShading: true }), landscape);
  }
  function buildLandscape() {
    // Reusable terrain follows source placement; no scenario landmark has a
    // coordinate in this renderer. Seaward piers and upper rooms don't expand
    // the main island's ground footprint.
    const ground = [...places.values()].filter(p => !['lantern_room', 'harbor', 'breakwater'].includes(p.kind));
    if (!ground.length) return;
    const minX = Math.min(...ground.map(p => p.position.x)), maxX = Math.max(...ground.map(p => p.position.x));
    const minZ = Math.min(...ground.map(p => p.position.z)), maxZ = Math.max(...ground.map(p => p.position.z));
    const cx = (minX + maxX) / 2, cz = (minZ + maxZ) / 2;
    const rx = Math.max(8, (maxX - minX) / 2 + 7), rz = Math.max(7, (maxZ - minZ) / 2 + 8);
    const low = Math.max(0.5, Math.min(...ground.map(p => p.position.y)) - 0.25);
    rockTerrace(cx, cz, rx, rz, -1.8, low * 0.49, 101, 0x304843);
    rockTerrace(cx, cz, rx * 0.9, rz * 0.84, low * 0.16, low, 111, 0x344f46);
    const radii = { courtyard: 4.5, lighthouse: 6.6, observatory: 5.9, archive: 5.5 };
    ground.forEach((place, i) => {
      const radius = radii[place.kind] || 4;
      const [x, y, z] = place.position.toArray();
      const top = y - 0.1;
      if (top - low > 1) {
        rockTerrace(x, z, radius, radius * 0.88, low * 0.85, top - 1.2, 450 + i * 13, 0x3b564e);
        rockTerrace(x, z, radius * 0.82, radius * 0.76, top - 1.8, top, 467 + i * 11, 0x3c5d50);
      } else {
        rockTerrace(x, z, radius, radius * 0.84, low * 0.5, top, 558 + i * 17, 0x39544b);
      }
    });
    waterUniforms.shoreCenter.value.set(cx, cz);
    waterUniforms.shoreRadius.value.set(rx * 1.02, rz * 1.02);
    const rng = seeded(8417), dummy = new THREE.Object3D();
    const rocks = new THREE.InstancedMesh(new THREE.DodecahedronGeometry(1, 0), material(0x4a6060, { flatShading: true }), 116);
    for (let i = 0; i < 116; i++) {
      const a = i / 116 * TAU, r = 0.98 + rng() * 0.16;
      dummy.position.set(cx + Math.cos(a) * rx * r, -0.2 + rng() * 0.5, cz + Math.sin(a) * rz * r);
      dummy.rotation.set(rng(), rng(), rng());
      const size = 0.45 + rng() * 1.3;
      dummy.scale.set(size, size * (0.65 + rng()), size * 0.8);
      dummy.updateMatrix(); rocks.setMatrixAt(i, dummy.matrix);
    }
    rocks.castShadow = rocks.receiveShadow = true; landscape.add(rocks);
    const grass = new THREE.InstancedMesh(new THREE.ConeGeometry(0.16, 0.72, 3), material(0x5a8070, { flatShading: true }), 300);
    for (let i = 0; i < 300; i++) {
      const a = rng() * TAU, r = 0.75 + rng() * 0.17;
      dummy.position.set(cx + Math.cos(a) * rx * 0.84 * r, low + 0.24, cz + Math.sin(a) * rz * 0.79 * r);
      dummy.rotation.set((rng() - 0.5) * 0.4, rng() * TAU, (rng() - 0.5) * 0.4);
      const size = 0.6 + rng() * 0.8; dummy.scale.set(size, size, size); dummy.updateMatrix(); grass.setMatrixAt(i, dummy.matrix);
    }
    grass.receiveShadow = true; landscape.add(grass);
    if (!presentation.overview) {
      baseTarget.set(cx, low + 3, cz);
      baseEye.set(cx + rx * 2.1, low + Math.max(rx, rz) * 1.6, cz + rz * 3.2);
    }
    moonlight.position.set(cx - 32, 55, cz + 18);
    moonlight.target.position.set(cx, low, cz);
    scene.add(moonlight.target);
  }

  const rainArray = new Float32Array(390 * 6);
  for (let i = 0; i < 390; i++) {
    const x = (random() - 0.5) * 85, y = random() * 45, z = (random() - 0.5) * 75;
    rainArray.set([x, y, z, x - 0.09, y + 0.6, z + 0.03], i * 6);
  }
  const rainGeometry = new THREE.BufferGeometry();
  rainGeometry.setAttribute('position', new THREE.BufferAttribute(rainArray, 3));
  const rain = new THREE.LineSegments(rainGeometry, new THREE.LineBasicMaterial({ color: 0x89a8b0, transparent: true, opacity: rescueMode ? 0.055 : 0.11, depthWrite: false }));
  world.add(rain);

  function makeRoof(parent, width, depth, y, height, color) {
    const geometry = new THREE.BufferGeometry();
    const w = width / 2, d = depth / 2;
    geometry.setAttribute('position', new THREE.Float32BufferAttribute([
      -w, y, -d, -w, y, d, 0, y + height, d,
      -w, y, -d, 0, y + height, d, 0, y + height, -d,
      w, y, d, w, y, -d, 0, y + height, -d,
      w, y, d, 0, y + height, -d, 0, y + height, d,
      -w, y, d, w, y, d, 0, y + height, d,
      w, y, -d, -w, y, -d, 0, y + height, -d,
    ], 3));
    geometry.computeVertexNormals();
    mesh(geometry, material(color, { roughness: 0.58, metalness: 0.24 }), parent);
    beam(parent, [0, y + height, -d], [0, y + height, d], 0.1, 0x70938d);
  }

  function courtyard(parent) {
    cylinder(parent, 0, -0.02, 0, 3.05, 3.12, 0.2, 0x63746c, 24);
    const ring = mesh(new THREE.RingGeometry(2.62, 2.72, 40), material(0xb09c73), parent, [0, 0.1, 0]);
    ring.rotation.x = -Math.PI / 2;
    for (let i = 0; i < 8; i++) {
      const a = i / 8 * TAU;
      beam(parent, [Math.sin(a) * 0.4, 0.105, Math.cos(a) * 0.4], [Math.sin(a) * 2.4, 0.105, Math.cos(a) * 2.4], 0.025, 0xb2a47e);
    }
    const cottage = new THREE.Group(); cottage.position.set(-2.9, 0, -1.4); cottage.rotation.y = -0.35; parent.add(cottage);
    box(cottage, 0, 1.15, 0, 3.5, 2.3, 2.9, 0x788b7e);
    box(cottage, 0, 0.18, 0, 3.8, 0.36, 3.1, 0x4f605a);
    makeRoof(cottage, 4.0, 3.4, 2.35, 1.55, 0x344d55);
    box(cottage, -0.85, 3.2, -0.7, 0.45, 1.8, 0.5, 0x6a746a);
    box(cottage, -0.35, 0.98, 1.49, 0.78, 1.96, 0.12, 0x263e41);
    warmWindow(cottage, 0.97, 1.33, 1.5, 0.68, 0.8);
    warmWindow(cottage, -1.79, 1.27, 0, 0.8, 0.9, -Math.PI / 2);
    lantern(parent, 2.1, 0, 1.2, 2.2);
    box(parent, 1.8, 0.5, -1.6, 2, 0.14, 0.62, 0x715e45);
    for (const x of [1.1, 2.5]) box(parent, x, 0.24, -1.6, 0.14, 0.48, 0.45, 0x344445);
  }

  function lighthouse(parent) {
    cylinder(parent, 0, 0.13, 0, 3.2, 3.3, 0.42, 0x6d7870, 20);
    cylinder(parent, 0, 6.6, 0, 1.62, 2.17, 13, 0xb6b6a0, 24);
    cylinder(parent, 0, 1.5, 0, 2.11, 2.18, 2.3, 0x657a75, 24);
    for (const [y, rad] of [[5, 1.98], [9.4, 1.79], [12.7, 1.66]]) cylinder(parent, 0, y, 0, rad, rad + 0.02, 0.14, 0x667b75, 24);
    cylinder(parent, 0, 13.3, 0, 2.42, 1.7, 0.55, 0x3b5457, 24);
    cylinder(parent, 0, 13.68, 0, 2.53, 2.53, 0.2, 0x768b80, 24);
    for (let i = 0; i < 18; i++) {
      const a = i / 18 * TAU;
      beam(parent, [Math.cos(a) * 2.32, 13.8, Math.sin(a) * 2.32], [Math.cos(a) * 2.32, 14.75, Math.sin(a) * 2.32], 0.04, 0x31484a);
    }
    for (const y of [14.28, 14.73]) {
      const torus = mesh(new THREE.TorusGeometry(2.32, 0.045, 5, 36), material(0x4b6766), parent, [0, y, 0]);
      torus.rotation.x = Math.PI / 2;
    }
    cylinder(parent, 0, 15.1, 0, 1.61, 1.61, 2.65, 0x47747b, 12, { transparent: true, opacity: 0.24, roughness: 0.08, depthWrite: false });
    for (let i = 0; i < 8; i++) {
      const a = i / 8 * TAU;
      beam(parent, [Math.cos(a) * 1.65, 13.8, Math.sin(a) * 1.65], [Math.cos(a) * 1.65, 16.4, Math.sin(a) * 1.65], 0.09, 0x273f45);
    }
    cylinder(parent, 0, 16.4, 0, 1.83, 1.83, 0.23, 0x455e60, 16);
    cylinder(parent, 0, 17.15, 0, 0.08, 2.0, 1.32, 0x3b6869, 16);
    cylinder(parent, 0, 18.13, 0, 0.045, 0.1, 1.15, 0x7c9d8f, 8);
    for (const y of [3.9, 7.6, 11.2]) warmWindow(parent, 0, y, 2.16 - y * 0.042, 0.43, 0.95);
    box(parent, 0, 1.13, 2.09, 1.02, 2.25, 0.17, 0x253d40);
    box(parent, 0, 1.14, 2.2, 0.7, 1.87, 0.04, 0x785f43);
    lantern(parent, 1.45, 0.32, 2.0, 1.9);
  }

  function observatory(parent) {
    cylinder(parent, 0, 0.13, 0, 3.8, 3.9, 0.4, 0x61736a, 24);
    cylinder(parent, 0, 1.35, 0, 2.48, 2.58, 2.5, 0x8e9c8b, 20);
    cylinder(parent, 0, 2.6, 0, 2.62, 2.62, 0.16, 0x3d585b, 24);
    const dome = mesh(new THREE.SphereGeometry(2.56, 24, 14, 0, TAU, 0, Math.PI / 2), material(0x447877, { metalness: 0.55, roughness: 0.42 }), parent, [0, 2.62, 0]);
    for (let i = 0; i < 8; i++) {
      const points = [], a = i / 8 * TAU;
      for (let j = 0; j <= 20; j++) {
        const th = j / 20 * Math.PI / 2;
        points.push(v(Math.cos(a) * Math.sin(th) * 2.58, 2.62 + Math.cos(th) * 2.58, Math.sin(a) * Math.sin(th) * 2.58));
      }
      mesh(new THREE.TubeGeometry(new THREE.CatmullRomCurve3(points), 20, 0.026, 4, false), material(0x83a798), parent);
    }
    box(parent, 0, 4.62, 0.18, 0.51, 0.11, 2.1, 0x17343c);
    box(parent, 0, 1.1, 2.5, 0.85, 2.15, 0.14, 0x293f43);
    warmWindow(parent, -1.4, 1.35, 2.09, 0.5, 0.92, -0.57);
    warmWindow(parent, 1.4, 1.35, 2.09, 0.5, 0.92, 0.57);
    lantern(parent, 2.9, 0.3, 0.8, 1.9);
    for (const x of [-3.2, -2.55]) cylinder(parent, x, 0.49, 1.5, 0.18, 0.23, 0.7, 0x778679, 8);
    return dome;
  }

  function archive(parent) {
    box(parent, 0, 0.16, 0, 6.7, 0.45, 4.5, 0x67756a);
    box(parent, 0, 1.45, 0, 5.9, 2.65, 3.75, 0x7f8e80);
    makeRoof(parent, 6.45, 4.25, 2.83, 1.55, 0x425d5c);
    for (const x of [-1.9, 1.9]) warmWindow(parent, x, 1.55, 1.91, 1.0, 1.25);
    box(parent, 0, 1.23, 1.95, 0.95, 2.3, 0.16, 0x344c48);
    for (const x of [-2.96, 2.96]) for (let i = 0; i < 5; i++) box(parent, x, 0.5 + i * 0.52, 1.9, 0.34, 0.28, 0.3, 0xa1a68e);
    for (let i = 0; i < 5; i++) box(parent, 0, -i * 0.2, 2.5 + i * 0.33, 2, 0.2, 0.37, 0x829083);
    lantern(parent, -3.3, 0.35, 1.8, 2.1);
    const barrelMat = material(0x736550);
    for (const [x, z] of [[3.4, -0.8], [3.4, 0.2]]) {
      cylinder(parent, x, 0.65, z, 0.36, 0.36, 1.0, 0x786a51, 10);
      for (const y of [0.3, 1]) cylinder(parent, x, y, z, 0.38, 0.38, 0.07, 0x374d4a, 10);
    }
    void barrelMat;
  }

  function harbor(parent) {
    box(parent, 0, -0.22, 0, 5, 0.45, 4.4, 0x4e605b);
    for (let i = 0; i < 18; i++) box(parent, 0, 0.075, -1.9 + i * 0.24, 5, 0.14, 0.205, i % 3 === 0 ? 0x847b62 : 0x686d59);
    for (let i = 0; i < 17; i++) box(parent, 1.0, 0.09, 2.25 + i * 0.31, 2.15, 0.16, 0.27, i % 3 === 0 ? 0x847b62 : 0x6b725d);
    for (const z of [-1.9, 1.8, 4.3, 7.1]) for (const x of [0, 2]) {
      cylinder(parent, x, -0.38, z, 0.13, 0.19, 2.5, 0x546455, 7);
      cylinder(parent, x, 0.53, z, 0.18, 0.18, 0.15, 0x9ba38a, 8);
    }
    const shed = new THREE.Group(); shed.position.set(-1.5, 0, -0.9); parent.add(shed);
    box(shed, 0, 0.95, 0, 2.25, 1.9, 1.7, 0x587a72);
    makeRoof(shed, 2.7, 2.2, 1.94, 0.85, 0x3b5559);
    warmWindow(shed, 0, 1.05, 0.89, 1.08, 0.68);
    lantern(parent, 2.25, 0.3, 0, 2.15);
    lantern(parent, 1.95, 0.1, 6.5, 1.5);
    for (let i = 0; i < 3; i++) box(parent, -1.4 + i * 0.65, 0.36, 1.3, 0.56, 0.66, 0.58, 0x8c8161);
  }

  function breakwater(parent) {
    for (let i = 0; i < 9; i++) {
      const rock = mesh(geo('rock', () => new THREE.DodecahedronGeometry(1, 0)), material(i % 2 ? 0x5b6f69 : 0x475f5e, { flatShading: true }), parent, [(i - 4) * 0.85, -0.42, 0], [1.25, 1.0, 1.45]);
      rock.rotation.y = i * 0.5;
    }
    box(parent, 0, 0.11, 0, 7.6, 0.21, 1.3, 0x748178);
    for (let i = 0; i < 6; i++) cylinder(parent, -3 + i * 1.2, 0.72, -0.7, 0.045, 0.055, 1.3, 0x6d8981, 5);
    beam(parent, [-3, 1.28, -0.7], [3, 1.28, -0.7], 0.04, 0x769087);
    cylinder(parent, 3.2, 0.4, 0, 0.22, 0.3, 0.8, 0x405c5e, 8);
  }

  function entityVisual(def, anchor, index) {
    const object = new THREE.Group();
    const parent = anchor.object;
    parent.add(object);
    const authored = authoredPositions.get(def.id);
    if (authored) object.position.copy(v(...authored).sub(anchor.position));
    else object.position.set(Math.cos(index * 2.4) * 1.5, 0, Math.sin(index * 2.4) * 1.5);
    const data = { id: def.id, kind: def.kind, place: def.at, object, activated: false, open: false, time: 0, labelHeight: 0.3, initial: object.position.clone() };
    switch (def.kind) {
      case 'ring': {
        const ring = mesh(new THREE.TorusGeometry(1, 0.055, 5, 64), new THREE.MeshBasicMaterial({ color: 0xb9e4d2, transparent: true, opacity: 0.8, depthWrite: false, fog: false, toneMapped: false }), object, [0, 0.04, 0]);
        ring.rotation.x = Math.PI / 2;
        const disc = mesh(new THREE.CircleGeometry(0.96, 48), new THREE.MeshBasicMaterial({ color: 0xb9e4d2, transparent: true, opacity: 0.08, side: THREE.DoubleSide, depthWrite: false, fog: false, toneMapped: false }), object, [0, 0.02, 0]);
        disc.rotation.x = -Math.PI / 2;
        ring.castShadow = disc.castShadow = false;
        Object.assign(data, { ring, disc });
        break;
      }
      case 'ferry': {
        const hullShape = new THREE.Shape();
        hullShape.moveTo(-1.25, -3.25); hullShape.lineTo(1.25, -3.25);
        hullShape.lineTo(1.48, 1.6); hullShape.quadraticCurveTo(1.36, 3.0, 0, 3.8);
        hullShape.quadraticCurveTo(-1.36, 3.0, -1.48, 1.6); hullShape.closePath();
        const body = new THREE.Group(); object.add(body);
        const hull = mesh(new THREE.ExtrudeGeometry(hullShape, { depth: 0.8, bevelEnabled: true, bevelSize: 0.17, bevelThickness: 0.13, bevelSegments: 1, steps: 1 }), material(0x9f523d, { roughness: 0.5 }), body);
        // Shape +Y becomes the ferry's local +Z bow, matching its heading and
        // the wake at local -Z. Lift the extrusion to preserve the deck height.
        hull.rotation.x = Math.PI / 2;
        hull.position.y = 0.8;
        box(body, 0, 0.81, -0.15, 2.5, 0.18, 5.75, 0xe0c899);
        box(body, 0, 1.42, -0.6, 2.16, 1.04, 3.15, 0xd5d6bd);
        box(body, 0, 2.04, -0.64, 2.43, 0.2, 3.5, 0x537e7a);
        box(body, 0, 2.58, -1.1, 1.55, 0.93, 1.43, 0xe4dbc0);
        box(body, 0, 3.08, -1.08, 1.77, 0.17, 1.65, 0x446564);
        for (const x of [-1.1, 1.1]) for (let i = 0; i < 4; i++) warmWindow(body, x, 1.45, -1.77 + i * 0.83, 0.53, 0.47, x < 0 ? -Math.PI / 2 : Math.PI / 2);
        warmWindow(body, 0, 2.64, -0.35, 1.0, 0.45);
        cylinder(body, 0.5, 2.62, -2.1, 0.21, 0.26, 1.13, 0xbd9c70, 10);
        beam(body, [0, 3.1, -1.1], [0, 4.2, -1.1], 0.04, 0xc7cbaa);
        const halo = glow(body, [0, 4.2, -1.1], 0xffd791, 3.3, 0.88);
        for (const x of [-1.32, 1.32]) {
          beam(body, [x, 1.2, 1.1], [x, 1.2, 2.3], 0.032, 0xcbd1b2);
          for (const z of [1.1, 1.7, 2.3]) beam(body, [x, 0.9, z], [x, 1.45, z], 0.03, 0xcbd1b2);
        }
        const wake = new THREE.Group(); object.add(wake);
        for (let i = 0; i < 3; i++) {
          const arc = mesh(new THREE.TorusGeometry(1.8 + i * 0.75, 0.045, 4, 30, Math.PI * 0.72), new THREE.MeshBasicMaterial({ color: 0xb7e3db, transparent: true, opacity: 0.24 - i * 0.055, depthWrite: false }), wake, [0, -0.36, -3.2 - i * 0.65]);
          arc.rotation.set(Math.PI / 2, 0, Math.PI * 0.15);
        }
        object.rotation.y = Math.PI;
        Object.assign(data, { body, halo, wake, labelHeight: 3.8 });
        animated.push(data);
        break;
      }
      case 'reef': {
        const rng = seeded(index * 135 + 391);
        for (let i = 0; i < 6; i++) {
          const a = i / 6 * TAU, radius = i === 0 ? 0 : 0.8 + rng() * 0.5;
          const stone = mesh(geo('reef-stone', () => new THREE.DodecahedronGeometry(1, 0)), material(i % 2 ? 0x527270 : 0x789087, { flatShading: true }), object, [Math.cos(a) * radius, i === 0 ? 0.95 : 0.24, Math.sin(a) * radius], [i === 0 ? 1.35 : 1.1, i === 0 ? 1.8 : 0.7, 1.12]);
          stone.rotation.set(rng() * 0.25, rng() * TAU, rng() * 0.22);
        }
        const foam = mesh(new THREE.RingGeometry(2.2, 2.48, 32), new THREE.MeshBasicMaterial({ color: 0xb1d8ca, transparent: true, opacity: 0.28, side: THREE.DoubleSide, depthWrite: false }), object, [0, 0.19, 0]);
        foam.rotation.x = -Math.PI / 2;
        const reveal = mesh(new THREE.RingGeometry(2.55, 2.65, 32), new THREE.MeshBasicMaterial({ color: 0xf4c16c, transparent: true, opacity: 0, side: THREE.DoubleSide, depthWrite: false }), object, [0, 0.23, 0]);
        reveal.rotation.x = -Math.PI / 2;
        Object.assign(data, { foam, reveal });
        break;
      }
      case 'harbor_goal': {
        const ring = mesh(new THREE.TorusGeometry(3.6, 0.095, 5, 64), new THREE.MeshBasicMaterial({ color: 0x83f1bd, transparent: true, opacity: 0.9, depthWrite: false, fog: false, toneMapped: false }), object, [0, 0.04, 0]);
        ring.rotation.x = Math.PI / 2;
        const inner = mesh(new THREE.RingGeometry(2.75, 3.5, 64), new THREE.MeshBasicMaterial({ color: 0x76e6b4, transparent: true, opacity: 0.1, side: THREE.DoubleSide, depthWrite: false, fog: false, toneMapped: false }), object, [0, 0.02, 0]);
        inner.rotation.x = -Math.PI / 2;
        for (const x of [-3.5, 3.5]) {
          cylinder(object, x, 0.75, 0, 0.09, 0.2, 1.6, 0x3b6862, 8);
          glow(object, [x, 1.55, 0], 0x83ffc4, 2.8, 0.82);
        }
        const passengers = new THREE.Group(); object.add(passengers);
        for (let i = 0; i < 32; i++) {
          const person = new THREE.Group(); person.position.set((i % 8 - 3.5) * 0.34, 1.17, -3.2 - Math.floor(i / 8) * 0.4); passengers.add(person);
          cylinder(person, 0, 0.23, 0, 0.1, 0.14, 0.4, i % 3 ? 0xe3b474 : 0x93c9b2, 5);
          sphere(person, 0, 0.54, 0, 0.1, 0xddcfaf);
          person.visible = false;
        }
        Object.assign(data, { ring, inner, passengers }); animated.push(data);
        break;
      }
      case 'beacon': {
        cylinder(object, 0, -0.08, 0, 0.52, 0.62, 0.22, 0x9a9777, 16);
        const lens = cylinder(object, 0, 0.76, 0, 0.65, 0.65, 1.38, 0xb9d9c4, 16, { emissive: 0xffbf68, emissiveIntensity: 0.35, metalness: 0.3, roughness: 0.2 });
        for (let i = 0; i < 8; i++) {
          const torus = mesh(new THREE.TorusGeometry(0.67, 0.03, 5, 24), material(0xc1b17f, { metalness: 0.65, roughness: 0.24 }), object, [0, 0.15 + i * 0.17, 0]);
          torus.rotation.x = Math.PI / 2;
        }
        const halo = glow(object, [0, 0.72, 0], 0xffce82, 5, 0.3);
        const sweep = new THREE.Group(); sweep.position.y = 0.8; object.add(sweep);
        const cone = mesh(new THREE.ConeGeometry(5.5, 60, 28, 1, true), new THREE.MeshBasicMaterial({ color: 0xfde5a9, transparent: true, opacity: 0.035, side: THREE.DoubleSide, depthWrite: false, blending: THREE.AdditiveBlending }), sweep, [0, 0, 30]);
        cone.rotation.x = -Math.PI / 2;
        const cone2 = cone.clone(); cone2.position.z = -30; cone2.rotation.x = Math.PI / 2; sweep.add(cone2);
        sweep.visible = false;
        const light = new THREE.PointLight(0xffc86d, 0, 35, 1.7); light.position.y = 0.8; object.add(light);
        Object.assign(data, { lens, halo, sweep, light });
        animated.push(data);
        break;
      }
      case 'telescope': {
        const telescope = cylinder(object, 0, 0.1, 0.5, 0.2, 0.28, 2.5, 0xc3b28b, 12, { metalness: 0.65, roughness: 0.3 });
        telescope.rotation.x = Math.PI * 0.28;
        const rim = cylinder(object, 0, 0.85, 1.35, 0.3, 0.3, 0.2, 0x394e55, 12); rim.rotation.x = Math.PI * 0.28;
        cylinder(object, 0, -0.43, 0, 0.12, 0.16, 1, 0x708c81, 8);
        break;
      }
      case 'optic': {
        cylinder(object, 0, 0.31, 0, 0.25, 0.34, 0.45, 0x839b90, 12, { metalness: 0.7, roughness: 0.23 });
        sphere(object, 0, 0.64, 0, 0.21, 0x92d3ca, { emissive: 0x5d9d9a, emissiveIntensity: 0.3, roughness: 0.1 });
        break;
      }
      case 'transmitter': {
        box(object, 0, 0.6, 0, 0.85, 1.15, 0.65, 0x456567);
        box(object, 0, 0.7, 0.34, 0.58, 0.23, 0.025, 0xb5cab0, { emissive: 0x86c8af, emissiveIntensity: 0.4 });
        beam(object, [0, 1.1, 0], [0, 5.2, 0], 0.045, 0x90aba0);
        beam(object, [-1.3, 4.6, 0], [1.3, 4.6, 0], 0.032, 0x90aba0);
        beam(object, [0, 2.5, 0], [-1.3, 4.6, 0], 0.024, 0x6a8e88);
        const halo = glow(object, [0, 5.2, 0], 0xa7f0d2, 1.5, 0.32);
        const rings = new THREE.Group(); object.add(rings); rings.visible = false;
        for (let i = 0; i < 3; i++) {
          const ring = mesh(new THREE.TorusGeometry(0.7 + i * 0.6, 0.025, 4, 40), new THREE.MeshBasicMaterial({ color: 0x98ead0, transparent: true, opacity: 0.55, depthWrite: false }), rings, [0, 4.8, 0]);
          ring.rotation.x = Math.PI / 2;
        }
        Object.assign(data, { halo, rings }); animated.push(data);
        break;
      }
      case 'generator': {
        box(object, 0, 0.5, 0, 1.4, 0.85, 0.9, 0x4a6660);
        const flywheel = cylinder(object, 0.77, 0.65, 0, 0.49, 0.49, 0.11, 0xb4a783, 16, { metalness: 0.6, roughness: 0.4 });
        flywheel.rotation.z = Math.PI / 2;
        for (let i = 0; i < 4; i++) box(object, -0.46 + i * 0.29, 0.99, 0, 0.11, 0.15, 0.7, 0x96a48a);
        Object.assign(data, { flywheel }); animated.push(data);
        break;
      }
      case 'chart': {
        for (const x of [-0.55, 0.55]) beam(object, [x, 0, -0.2], [x, 1.1, 0], 0.05, 0x7b775c);
        const board = box(object, 0, 1.05, 0, 1.6, 0.09, 1, 0xa0a389); board.rotation.x = 0.28;
        for (let i = 0; i < 4; i++) {
          const line = box(object, 0.12, 1.1, -0.28 + i * 0.17, 0.93, 0.012, 0.016, 0x496b64); line.rotation.y = i * 0.08;
        }
        glow(object, [0, 1.2, 0], 0xbbe0b8, 2, 0.14);
        break;
      }
      case 'gauge': {
        box(object, 0, 0.5, 0, 0.29, 2.7, 0.14, 0xd3d0ae);
        for (let i = 0; i < 11; i++) box(object, 0, -0.58 + i * 0.22, 0.09, i % 2 ? 0.13 : 0.25, 0.046, 0.022, 0x375c5d);
        break;
      }
      case 'buoy': {
        cylinder(object, 0, 0.34, 0, 0.28, 0.5, 0.7, 0xa55c43, 10);
        cylinder(object, 0, 0.98, 0, 0.075, 0.15, 0.85, 0xa69c7d, 8);
        const halo = glow(object, [0, 1.5, 0], 0xffa56b, 1.6, 0.7);
        Object.assign(data, { halo }); animated.push(data);
        break;
      }
      case 'boat': {
        const hullShape = new THREE.Shape(); hullShape.moveTo(-0.74, -1.65); hullShape.lineTo(0.74, -1.65); hullShape.lineTo(0.86, 0.4); hullShape.quadraticCurveTo(0.8, 1.5, 0, 2); hullShape.quadraticCurveTo(-0.8, 1.5, -0.86, 0.4); hullShape.closePath();
        const hull = mesh(new THREE.ExtrudeGeometry(hullShape, { depth: 0.64, bevelEnabled: true, bevelSize: 0.12, bevelThickness: 0.1, bevelSegments: 1, steps: 1 }), material(0x497675, { roughness: 0.4 }), object);
        hull.rotation.x = -Math.PI / 2;
        box(object, 0, 0.66, -0.2, 1.33, 0.15, 2.56, 0xa89f79);
        box(object, 0, 1.13, -0.6, 1.1, 0.84, 1.15, 0xc2c2a2);
        box(object, 0, 1.65, -0.65, 1.25, 0.16, 1.45, 0x527370);
        warmWindow(object, 0, 1.3, 0.01, 0.69, 0.37);
        beam(object, [0, 1.7, -0.75], [0, 2.7, -0.75], 0.035, 0xb8b899);
        glow(object, [0, 2.7, -0.75], 0xffc284, 1.5, 0.55);
        object.rotation.y = -0.22;
        animated.push(data);
        break;
      }
      default: {
        if (def.kind === 'gate' || def.kind === 'door' || def.kind.endsWith('_door')) {
          const pivot = new THREE.Group(); object.add(pivot);
          box(pivot, 0.5, 0.9, 0, 1, 1.8, 0.13, 0x56756a);
          sphere(pivot, 0.88, 0.9, 0.11, 0.07, 0xd0b989);
          data.pivot = pivot;
        } else {
          cylinder(object, 0, 0.22, 0, 0.3, 0.42, 0.42, 0x749385, 8);
        }
      }
    }
    entities.set(def.id, data);
    return data;
  }

  function routeCurve(from, to) {
    const a = places.get(from), b = places.get(to);
    if (!a || !b) return null;
    const authored = (presentation.routes || []).find(route =>
      (route.from === from && route.to === to) || (route.from === to && route.to === from));
    const intermediate = (authored?.points || []).map(point => v(...point));
    if (authored && authored.from !== from) intermediate.reverse();
    return new THREE.CatmullRomCurve3([a.position.clone(), ...intermediate, b.position.clone()]);
  }

  function drawConnection(connection) {
    const curve = routeCurve(connection.from, connection.to);
    if (!curve) return;
    routes.set(`${connection.from}:${connection.to}`, curve);
    const reversePoints = curve.getPoints(60).reverse();
    routes.set(`${connection.to}:${connection.from}`, new THREE.CatmullRomCurve3(reversePoints));
    const a = places.get(connection.from), b = places.get(connection.to);
    const elevated = Math.abs(a.position.y - b.position.y) > 6;
    const length = curve.getLength(), steps = Math.max(6, Math.floor(length / 0.43));
    for (let i = 1; i < steps; i++) {
      const t = i / steps, p = curve.getPoint(t), tangent = curve.getTangent(t);
      const step = box(graph, p.x, p.y - 0.02, p.z, elevated ? 0.78 : 1.2, 0.14, 0.35, i % 4 ? 0x68796c : 0x809081);
      step.rotation.y = Math.atan2(tangent.x, tangent.z);
      if (!elevated && Math.abs(tangent.y) > 0.18) box(graph, p.x, p.y - 0.3, p.z, 1.25, 0.55, 0.45, 0x53675e).rotation.y = step.rotation.y;
    }
    if (elevated) {
      const railPoints = [];
      for (let i = 0; i <= steps; i++) {
        const t = i / steps, p = curve.getPoint(t), tangent = curve.getTangent(t);
        const side = v(-tangent.z, 0, tangent.x).normalize().multiplyScalar(0.43);
        const foot = p.clone().add(side), top = foot.clone().add(v(0, 0.84, 0));
        railPoints.push(top);
        if (i % 5 === 0) beam(graph, foot.toArray(), top.toArray(), 0.032, 0x6f9590);
      }
      mesh(new THREE.TubeGeometry(new THREE.CatmullRomCurve3(railPoints), steps, 0.04, 5, false), material(0x6f9590), graph);
    }
    for (const t of elevated ? [] : [0.23, 0.73]) {
      const p = curve.getPoint(t), tangent = curve.getTangent(t);
      const side = v(-tangent.z, 0, tangent.x).normalize().multiplyScalar(0.9);
      lantern(graph, p.x + side.x, p.y, p.z + side.z, 1.25);
    }
  }

  const marker = new THREE.Group(); graph.add(marker);
  const markerRing = mesh(new THREE.RingGeometry(0.54, 0.62, 48), new THREE.MeshBasicMaterial({ color: 0xe1cca1, transparent: true, opacity: 0.92, side: THREE.DoubleSide, depthWrite: false }), marker);
  markerRing.rotation.x = -Math.PI / 2;
  const player = new THREE.Group(); marker.add(player);
  const coat = cylinder(player, 0, 0.45, 0, 0.15, 0.23, 0.64, 0xc4a66a, 7);
  sphere(player, 0, 0.92, 0, 0.145, 0xd7c8a4);
  cylinder(player, 0, 1.04, 0, 0.21, 0.21, 0.08, 0x738677, 10);
  for (const x of [-0.09, 0.09]) box(player, x, 0.14, 0, 0.1, 0.3, 0.13, 0x2d4848);
  const playerGlow = glow(player, [0.29, 0.58, 0.1], 0xffd094, 1.35, 0.68);
  void coat;
  const selection = mesh(new THREE.RingGeometry(0.8, 0.85, 48), new THREE.MeshBasicMaterial({ color: 0xe6c687, transparent: true, opacity: 0, side: THREE.DoubleSide, depthWrite: false }), graph);
  selection.rotation.x = -Math.PI / 2;
  marker.visible = !rescueMode;

  const rescueEffects = new THREE.Group(); scene.add(rescueEffects);
  const aimSpot = new THREE.Group(); rescueEffects.add(aimSpot);
  const aimDisc = mesh(new THREE.CircleGeometry(3.1, 48), new THREE.MeshBasicMaterial({ color: 0xffdc8b, transparent: true, opacity: 0.1, depthWrite: false, side: THREE.DoubleSide, blending: THREE.AdditiveBlending }), aimSpot);
  aimDisc.rotation.x = -Math.PI / 2;
  const aimRing = mesh(new THREE.TorusGeometry(2.1, 0.065, 5, 48), new THREE.MeshBasicMaterial({ color: 0xffd27a, transparent: true, opacity: 0.9, depthWrite: false, fog: false, toneMapped: false }), aimSpot);
  aimRing.rotation.x = Math.PI / 2;
  const aimCore = glow(aimSpot, [0, 0.3, 0], 0xffde9a, 3.1, 0.6);
  for (let i = 0; i < 4; i++) {
    const a = i / 4 * TAU;
    const tick = box(aimSpot, Math.cos(a) * 2.55, 0.01, Math.sin(a) * 2.55, 0.55, 0.025, 0.06, 0xffd27a, { emissive: 0xffc56b, emissiveIntensity: 1.2 });
    tick.rotation.y = -a;
  }
  const lightCone = mesh(new THREE.ConeGeometry(3.8, 1, 24, 1, true), new THREE.MeshBasicMaterial({ color: 0xffe4a0, transparent: true, opacity: 0.1, side: THREE.DoubleSide, depthWrite: false, blending: THREE.AdditiveBlending }), rescueEffects);
  lightCone.castShadow = lightCone.receiveShadow = false;
  aimDisc.castShadow = aimRing.castShadow = false;
  const routeGeometry = new THREE.BufferGeometry();
  routeGeometry.setAttribute('position', new THREE.Float32BufferAttribute([0, 0, 0, 0, 0, 0], 3));
  const routeHint = new THREE.Line(routeGeometry, new THREE.LineDashedMaterial({ color: 0xa6e6c5, transparent: true, opacity: 0.7, dashSize: 0.55, gapSize: 0.48, depthWrite: false, fog: false, toneMapped: false }));
  rescueEffects.add(routeHint);
  // Host primitives implement geometry only. All poses, visibility, colors and
  // part animation values below are supplied by evaluated CAVEAT bindings.
  renderTargets.set('light_target', { object: aimSpot, ring: aimRing, disc: aimDisc, core: aimCore });
  renderTargets.set('light_beam', { object: lightCone, primitive: 'beam', from: v(), to: v(0, 1, 0), radius: 3.8 });
  renderTargets.set('guide_line', { object: routeHint, primitive: 'line', from: v(), to: v(0, 1, 0) });
  aimSpot.visible = lightCone.visible = routeHint.visible = false;

  function targetFor(id) { return entities.get(id) || renderTargets.get(id); }

  function bindingPart(data, property) {
    const segments = property.split('.');
    const attribute = segments.pop();
    let object = data.object;
    if (segments.length) {
      let candidate = data;
      for (const segment of segments) {
        if (!Object.prototype.hasOwnProperty.call(candidate, segment)) return null;
        candidate = candidate[segment];
        if (!candidate || typeof candidate !== 'object') return null;
      }
      if (!candidate.isObject3D) return null;
      object = candidate;
    }
    return { object, attribute, root: object === data.object };
  }

  function setMaterialProperty(object, property, value) {
    let changed = false;
    object.traverse(child => {
      if (!child.material) return;
      if (!isolatedMaterials.has(child)) {
        child.material = Array.isArray(child.material) ? child.material.map(item => item.clone()) : child.material.clone();
        isolatedMaterials.add(child);
      }
      for (const material of Array.isArray(child.material) ? child.material : [child.material]) {
        if (property === 'color' && material.color) { material.color.set(typeof value === 'string' ? value : Number(value)); changed = true; }
        if (property === 'opacity') { material.opacity = clamp(Number(value), 0, 1); material.transparent = material.opacity < 1; changed = true; }
        if (property === 'emissive' && 'emissiveIntensity' in material) { material.emissiveIntensity = Number(value); changed = true; }
      }
    });
    return changed;
  }

  function applyBinding(data, property, value) {
    if (/^(from|to)_[xyz]$/.test(property) && data.primitive) {
      const [endpoint, axis] = property.split('_');
      data[endpoint][axis] = Number(value);
      return true;
    }
    if (property === 'radius' && data.primitive === 'beam') { data.radius = Number(value); return true; }
    const part = bindingPart(data, property);
    if (!part) return false;
    const { object, attribute, root } = part;
    if (['x', 'y', 'z'].includes(attribute)) {
      if (root) {
        const position = object.getWorldPosition(v());
        position[attribute] = Number(value);
        if (object.parent) object.parent.worldToLocal(position);
        object.position.copy(position);
      } else object.position[attribute] = Number(value);
      return true;
    }
    if (/^rotation_[xyz]$/.test(attribute)) { object.rotation[attribute.at(-1)] = Number(value); return true; }
    if (/^scale_[xyz]$/.test(attribute)) { object.scale[attribute.at(-1)] = Number(value); return true; }
    if (attribute === 'scale') { object.scale.setScalar(Number(value)); return true; }
    if (attribute === 'visible') { object.visible = Boolean(value); return true; }
    if (attribute === 'count') {
      const count = Math.max(0, Math.floor(Number(value)));
      object.children.forEach((child, index) => { child.visible = index < count; });
      return true;
    }
    if (['color', 'opacity', 'emissive'].includes(attribute)) return setMaterialProperty(object, attribute, value);
    return false;
  }

  function updateBoundGeometry(data) {
    if (data.primitive === 'beam') {
      const middle = data.from.clone().add(data.to).multiplyScalar(0.5);
      if (data.object.parent) data.object.parent.worldToLocal(middle);
      data.object.position.copy(middle);
      data.object.scale.set(data.radius / 3.8, data.from.distanceTo(data.to), data.radius / 3.8);
      const direction = data.from.clone().sub(data.to);
      if (direction.lengthSq() > 0) data.object.quaternion.setFromUnitVectors(UP, direction.normalize());
    } else if (data.primitive === 'line') {
      const start = data.from.clone(), end = data.to.clone();
      data.object.worldToLocal(start); data.object.worldToLocal(end);
      const positions = data.object.geometry.attributes.position;
      positions.setXYZ(0, start.x, start.y, start.z); positions.setXYZ(1, end.x, end.y, end.z);
      positions.needsUpdate = true;
      data.object.computeLineDistances();
    }
  }

  function clearCues() {
    for (const cue of activeCues) { cue.object.removeFromParent(); cue.object.geometry.dispose(); cue.object.material.dispose(); }
    activeCues.length = 0;
    seenCues.clear();
  }

  function playCue(cue) {
    if (cue.kind !== 'ring') return;
    const target = targetFor(cue.target);
    if (!target || !(Number(cue.duration) > 0)) return;
    const radius = Number(cue.radius) > 0 ? Number(cue.radius) : 1;
    const ring = mesh(new THREE.RingGeometry(radius * 0.9, radius * 1.05, 48), new THREE.MeshBasicMaterial({
      color: cue.color, transparent: true, opacity: 0.85, side: THREE.DoubleSide, depthWrite: false, fog: false, toneMapped: false,
    }), scene);
    ring.castShadow = ring.receiveShadow = false;
    ring.rotation.x = -Math.PI / 2;
    ring.position.copy(target.object.getWorldPosition(v())).add(v(0, 0.2, 0));
    activeCues.push({ object: ring, age: 0, duration: Number(cue.duration) });
  }

  function setBindings(snapshot) {
    if (!snapshot) return;
    const sequence = Number(snapshot.sequence ?? 0);
    const session = snapshot.session_id ?? snapshot.source_id ?? null;
    if (sequence < lastBindingSequence || session !== lastBindingSession) {
      clearCues(); appliedBindingValues.clear();
    }
    lastBindingSequence = sequence;
    lastBindingSession = session;
    for (const [id, properties] of Object.entries(snapshot.bindings || {})) {
      const target = targetFor(id);
      if (!target || !properties || typeof properties !== 'object') continue;
      if (!boundProperties.has(id)) boundProperties.set(id, new Set());
      if (!appliedBindingValues.has(id)) appliedBindingValues.set(id, new Map());
      const cache = appliedBindingValues.get(id), entries = Object.entries(properties);
      const changed = entries.filter(([property, value]) => !cache.has(property) || !Object.is(cache.get(property), value));
      let geometryChanged = false;
      for (const [property, value] of entries) {
        // A group material binding can overwrite a child's material binding;
        // retain their ordering when an overlapping property changes.
        const segments = property.split('.'), attribute = segments.pop(), prefix = segments.join('.');
        const overlap = changed.some(([other]) => {
          const parts = other.split('.'), otherAttribute = parts.pop(), otherPrefix = parts.join('.');
          if (['color', 'opacity', 'emissive'].includes(attribute) && attribute === otherAttribute) {
            return !prefix || !otherPrefix || prefix === otherPrefix || prefix.startsWith(`${otherPrefix}.`) || otherPrefix.startsWith(`${prefix}.`);
          }
          return prefix === otherPrefix && (attribute === 'scale' || otherAttribute === 'scale') && attribute.startsWith('scale') && otherAttribute.startsWith('scale');
        });
        if (cache.has(property) && Object.is(cache.get(property), value) && !overlap) continue;
        if (applyBinding(target, property, value)) {
          cache.set(property, value);
          boundProperties.get(id).add(property);
          if (/^(from|to)_[xyz]$/.test(property) || property === 'radius' || /^(x|y|z|rotation_[xyz]|scale(_[xyz])?)$/.test(property)) geometryChanged = true;
        }
      }
      if (geometryChanged) updateBoundGeometry(target);
    }
    for (const [index, cue] of (snapshot.cues || []).entries()) {
      const key = `${session}:${sequence}:${index}:${cue.id}`;
      if (!seenCues.has(key)) { seenCues.add(key); playCue(cue); }
    }
    needsRender = true;
  }

  function updateCues(dt) {
    for (let index = activeCues.length - 1; index >= 0; index--) {
      const cue = activeCues[index];
      cue.age = Math.min(cue.duration, cue.age + dt);
      const progress = cue.age / cue.duration;
      cue.object.material.opacity = (1 - progress) * 0.85;
      cue.object.scale.setScalar(reducedMotion ? 2.4 : 1 + progress * 5);
      if (progress >= 1) {
        cue.object.removeFromParent(); cue.object.geometry.dispose(); cue.object.material.dispose(); activeCues.splice(index, 1);
      }
    }
  }

  function getBindingPresentation() {
    const result = {};
    for (const [id, properties] of boundProperties) {
      const data = targetFor(id);
      if (!data) continue;
      const values = {};
      for (const property of properties) {
        if (/^(from|to)_[xyz]$/.test(property) && data.primitive) {
          const [endpoint, axis] = property.split('_');
          let point;
          if (data.primitive === 'beam') point = data.object.localToWorld(v(0, endpoint === 'from' ? 0.5 : -0.5, 0));
          else {
            const positions = data.object.geometry.attributes.position;
            point = data.object.localToWorld(v().fromBufferAttribute(positions, endpoint === 'from' ? 0 : 1));
          }
          values[property] = point[axis];
          continue;
        }
        if (property === 'radius' && data.primitive === 'beam') { values[property] = data.object.scale.x * 3.8; continue; }
        const part = bindingPart(data, property);
        if (!part) continue;
        const { object, attribute, root } = part;
        if (['x', 'y', 'z'].includes(attribute)) values[property] = (root ? object.getWorldPosition(v()) : object.position)[attribute];
        else if (/^rotation_[xyz]$/.test(attribute)) values[property] = object.rotation[attribute.at(-1)];
        else if (/^scale_[xyz]$/.test(attribute)) values[property] = object.scale[attribute.at(-1)];
        else if (attribute === 'scale') values[property] = object.scale.x;
        else if (attribute === 'visible') values[property] = object.visible;
        else if (attribute === 'count') values[property] = object.children.filter(child => child.visible).length;
        else {
          let material;
          object.traverse(child => { if (!material && child.material) material = Array.isArray(child.material) ? child.material[0] : child.material; });
          if (attribute === 'color' && material?.color) values[property] = material.color.getHex();
          if (attribute === 'opacity' && material) values[property] = material.opacity;
          if (attribute === 'emissive' && material) values[property] = material.emissiveIntensity;
        }
      }
      result[id] = values;
    }
    return { bindings: result, activeCues: activeCues.length };
  }

  function clearGraph() {
    for (const child of [...graph.children]) if (child !== marker && child !== selection) graph.remove(child);
    for (const child of [...landscape.children]) {
      child.traverse(object => object.geometry?.dispose());
      landscape.remove(child);
    }
    places.clear(); entities.clear(); routes.clear(); animated.length = 0; clicks.length = 0; lamps.length = 0;
    boundProperties.clear(); appliedBindingValues.clear(); clearCues(); lastBindingSequence = -1;
    for (const data of renderTargets.values()) data.object.visible = false;
  }

  function instanceStaticArchitecture() {
    // Shared primitive/material batches preserve every transform while avoiding
    // one draw call per plank, lamp frame and stair tread on mobile GPUs.
    graph.updateMatrixWorld(true);
    const batches = new Map();
    for (const child of graph.children) {
      if (child === marker || child === selection) continue;
      child.traverse(object => {
        if (!object.isMesh || object.isInstancedMesh || !object.material?.isMeshStandardMaterial || object.material.transparent) return;
        const key = `${object.geometry.uuid}:${object.material.uuid}`;
        if (!batches.has(key)) batches.set(key, []);
        batches.get(key).push(object);
      });
    }
    for (const items of batches.values()) {
      if (items.length < 3) continue;
      const batch = new THREE.InstancedMesh(items[0].geometry, items[0].material, items.length);
      items.forEach((object, i) => { batch.setMatrixAt(i, object.matrixWorld); object.removeFromParent(); });
      batch.castShadow = batch.receiveShadow = true;
      graph.add(batch);
    }
  }

  function setModel(input) {
    clearGraph();
    model = input?.world || input;
    if (!model?.places) return;
    presentation = model.presentation || {};
    authoredPositions = new Map((presentation.positions || []).map(item => [item.target, item.position]));
    const authoredCameras = new Map((presentation.cameras || []).map(item => [item.place, item]));
    if (presentation.overview) {
      baseEye.set(...presentation.overview.position);
      baseTarget.set(...presentation.overview.target);
    }
    const templates = { courtyard, lighthouse, observatory, archive, harbor, breakwater };
    model.places.forEach((def, index) => {
      const spec = authoredCameras.get(def.id);
      const authored = authoredPositions.get(def.id);
      const position = authored ? v(...authored) : v(Math.cos(index * 2.4) * 9, 3.2, Math.sin(index * 2.4) * 8);
      const object = new THREE.Group(); object.position.copy(position); graph.add(object);
      const anchor = { id: def.id, kind: def.kind, position, object, camera: spec?.position, target: spec?.target };
      places.set(def.id, anchor);
      if (templates[def.kind]) templates[def.kind](object);
      else if (def.kind !== 'lantern_room') cylinder(object, 0, 0, 0, 2.2, 2.3, 0.3, 0x688273, 16);
      const hit = mesh(new THREE.SphereGeometry(def.kind === 'lighthouse' ? 3.4 : 2.7, 10, 8), new THREE.MeshBasicMaterial({ visible: false }), object, [0, def.kind === 'lighthouse' ? 5 : 1.6, 0]);
      hit.userData.place = def.id; clicks.push(hit);
    });
    buildLandscape();
    for (const connection of model.connections || []) drawConnection(connection);
    instanceStaticArchitecture();
    for (const [index, def] of (model.entities || []).entries()) {
      const anchor = places.get(def.at);
      if (anchor) entityVisual(def, anchor, index);
    }
    currentPlace = model.start_at || model.places[0]?.id;
    if (places.has(currentPlace)) marker.position.copy(places.get(currentPlace).position).add(v(0, 0.17, 0));
    reset();
  }

  function tween(duration, update) {
    if (!alive) return Promise.resolve();
    needsRender = true;
    if (reducedMotion || immediatePlayback) { update(1); return Promise.resolve(); }
    return new Promise(resolve => { tweens.add({ duration, age: 0, update, resolve }); });
  }
  function cameraTo(eye, target, duration = 1.1) {
    const fromEye = cameraEye.clone(), fromTarget = cameraTarget.clone();
    yaw = 0; pitch = 0; zoom = 1;
    return tween(duration, t => { cameraEye.lerpVectors(fromEye, eye, smooth(t)); cameraTarget.lerpVectors(fromTarget, target, smooth(t)); });
  }
  function viewFor(place) {
    if (place?.camera) return { eye: v(...place.camera), target: v(...place.target) };
    return { eye: place.position.clone().add(v(15, 12, 18)), target: place.position.clone().add(v(0, 1.3, 0)) };
  }
  function focus(id, duration = 1.25) {
    if (id === 'overview' || !id) return cameraTo(baseEye, baseTarget, duration);
    const place = places.get(id);
    if (!place) return Promise.resolve();
    const shot = viewFor(place);
    return cameraTo(shot.eye, shot.target, duration);
  }
  function pulseAt(position, radius = 1.1) {
    selection.position.copy(position).add(v(0, 0.2, 0));
    return tween(0.85, t => { selection.material.opacity = Math.sin(t * Math.PI) * 0.82; selection.scale.setScalar(radius * (0.7 + t * 0.65)); });
  }
  async function inspect(data) {
    if (!data) return;
    const position = data.object.getWorldPosition(v());
    const place = places.get(data.place);
    const shot = viewFor(place);
    // Keep the architecture in view while making the relevant object readable.
    const eye = shot.eye.clone().lerp(position, 0.37);
    await Promise.all([cameraTo(eye, position.clone().add(v(0, 1, 0)), 0.68), pulseAt(position)]);
  }
  async function open(data, isOpen = true) {
    if (!data?.pivot) return;
    const generation = resetGeneration;
    const from = data.pivot.rotation.y, to = isOpen ? -Math.PI * 0.53 : 0;
    await tween(0.75, t => { data.pivot.rotation.y = THREE.MathUtils.lerp(from, to, smooth(t)); });
    if (generation === resetGeneration) data.open = isOpen;
  }
  async function operate(data) {
    if (!data) return;
    const generation = resetGeneration;
    await inspect(data);
    if (generation !== resetGeneration || !alive) return;
    data.activated = true;
    data.time = 0;
    if (data.kind === 'beacon') {
      data.sweep.visible = true;
      await tween(1.3, t => {
        data.lens.material.emissiveIntensity = 0.35 + t * 4;
        data.halo.material.opacity = 0.3 + t * 0.65;
        data.halo.scale.setScalar(5 + t * 7);
        data.light.intensity = t * 24;
      });
      if (generation === resetGeneration && alive) await focus('overview', 1.6);
    } else if (data.kind === 'transmitter') {
      data.rings.visible = true;
      await tween(0.9, t => { data.halo.material.opacity = 0.4 + t * 0.6; });
    } else if (data.kind === 'boat') {
      // A boat moves only after its actual source-authored operate command.
      await tween(1.1, t => { data.object.rotation.y = THREE.MathUtils.lerp(-0.22, 0.24, smooth(t)); });
    } else {
      await pulseAt(data.object.getWorldPosition(v()), 1.3);
    }
  }
  async function move(command) {
    const generation = resetGeneration;
    const destination = places.get(command.to);
    if (!destination) return;
    const curve = routes.get(`${command.from}:${command.to}`);
    // Never invent a route absent from the source model.
    if (!curve) return;
    const shot = viewFor(destination);
    const fromEye = cameraEye.clone(), fromTarget = cameraTarget.clone();
    yaw = pitch = 0; zoom = 1;
    const duration = clamp(curve.getLength() / 13, 1.05, 2.7);
    await tween(duration, t => {
      const q = smooth(t), p = curve.getPointAt(q), direction = curve.getTangentAt(q);
      marker.position.copy(p).add(v(0, 0.17, 0));
      player.rotation.y = Math.atan2(direction.x, direction.z);
      player.position.y = Math.sin(t * duration * 17) * 0.045;
      cameraEye.lerpVectors(fromEye, shot.eye, q);
      cameraEye.y += Math.sin(t * Math.PI) * 1.3;
      cameraTarget.lerpVectors(fromTarget, shot.target, q);
    });
    if (generation !== resetGeneration || !alive) return;
    player.position.y = 0;
    currentPlace = command.to;
    options.onPlace?.(command.to);
  }

  async function play(execution, playOptions = {}) {
    if (!execution || busy || !alive) return;
    busy = true;
    immediatePlayback = Boolean(playOptions.immediate);
    const generation = resetGeneration;
    try {
      for (const command of execution.commands || []) {
        if (generation !== resetGeneration || !alive) break;
        options.onStatus?.(command);
        switch (command.kind) {
          case 'move': await move(command); break;
          case 'inspect': await inspect(entities.get(command.entity)); break;
          case 'operate': await operate(entities.get(command.entity)); break;
          case 'open': await open(entities.get(command.entity), true); break;
          case 'close': await open(entities.get(command.entity), false); break;
          case 'observe': await pulseAt(marker.position, 1.5); break;
          case 'stay': if (command.place) await focus(command.place, 0.6); break;
          default: break;
        }
      }
    } finally {
      if (generation === resetGeneration) { busy = false; immediatePlayback = false; options.onStatus?.(null); }
    }
  }

  function setWorld(snapshot) {
    const state = snapshot?.world || snapshot?.state || snapshot;
    if (!state) return;
    needsRender = true;
    if (places.has(state.current_place)) {
      currentPlace = state.current_place;
      marker.position.copy(places.get(currentPlace).position).add(v(0, 0.17, 0));
      options.onPlace?.(currentPlace);
    }
    if (Array.isArray(state.open_entities)) for (const data of entities.values()) {
      if (!data.pivot) continue;
      data.open = state.open_entities.includes(data.id);
      data.pivot.rotation.y = data.open ? -Math.PI * 0.53 : 0;
    }
  }

  function reset() {
    needsRender = true;
    resetGeneration++;
    clearCues(); appliedBindingValues.clear(); lastBindingSequence = -1; lastBindingSession = null;
    for (const item of tweens) item.resolve();
    tweens.clear(); busy = false; immediatePlayback = false;
    cameraEye.copy(baseEye); cameraTarget.copy(baseTarget); yaw = pitch = 0; zoom = 1;
    currentPlace = model?.start_at;
    if (places.has(currentPlace)) marker.position.copy(places.get(currentPlace).position).add(v(0, 0.17, 0));
    player.position.y = 0; selection.material.opacity = 0;
    for (const data of entities.values()) {
      data.activated = false; data.time = 0; data.open = false;
      data.object.position.copy(data.initial);
      if (data.pivot) data.pivot.rotation.y = 0;
      if (data.kind === 'beacon') {
        data.sweep.visible = false; data.lens.material.emissiveIntensity = 0.35;
        data.light.intensity = 0; data.halo.material.opacity = 0.3; data.halo.scale.set(5, 5, 1);
      }
      if (data.kind === 'transmitter') { data.rings.visible = false; data.halo.material.opacity = 0.32; }
      if (data.kind === 'boat') data.object.rotation.y = -0.22;
    }
    options.onPlace?.(currentPlace);
  }

  function resize() {
    needsRender = true;
    const width = Math.max(1, canvas.clientWidth || canvas.parentElement?.clientWidth || 1);
    const height = Math.max(1, canvas.clientHeight || canvas.parentElement?.clientHeight || 1);
    const dpr = Math.min(window.devicePixelRatio || 1, 1.7);
    renderer.setPixelRatio(dpr);
    renderer.setSize(width, height, false);
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
  }
  const resizeObserver = new ResizeObserver(resize);
  resizeObserver.observe(canvas);
  resize();

  function onDown(event) {
    if (options.interactiveCamera === false) return;
    if (event.button !== undefined && event.button !== 0) return;
    drag = { x: event.clientX, y: event.clientY, startX: event.clientX, startY: event.clientY, pointerId: event.pointerId };
    canvas.setPointerCapture?.(event.pointerId);
  }
  function onMove(event) {
    if (options.interactiveCamera === false) return;
    if (!drag || busy) return;
    needsRender = true;
    yaw += (event.clientX - drag.x) * 0.004;
    pitch = clamp(pitch + (event.clientY - drag.y) * 0.003, -0.22, 0.48);
    drag.x = event.clientX; drag.y = event.clientY;
  }
  function onUp(event) {
    if (options.interactiveCamera === false) return;
    if (!drag) return;
    const wasClick = Math.hypot(event.clientX - drag.startX, event.clientY - drag.startY) < 6;
    drag = null;
    if (!wasClick || busy) return;
    const rect = canvas.getBoundingClientRect();
    pointer.set((event.clientX - rect.left) / rect.width * 2 - 1, -(event.clientY - rect.top) / rect.height * 2 + 1);
    raycaster.setFromCamera(pointer, camera);
    const target = raycaster.intersectObjects(clicks, false)[0]?.object.userData.place;
    if (target) { focus(target); options.onFocus?.(target); }
  }
  function onCancel() { drag = null; }
  function onWheel(event) {
    if (options.interactiveCamera === false) return;
    event.preventDefault();
    needsRender = true;
    if (!busy) zoom = clamp(zoom + event.deltaY * 0.0005, 0.7, 1.65);
  }
  canvas.addEventListener('pointerdown', onDown);
  canvas.addEventListener('pointermove', onMove);
  canvas.addEventListener('pointerup', onUp);
  canvas.addEventListener('pointercancel', onCancel);
  canvas.addEventListener('wheel', onWheel, { passive: false });
  canvas.style.touchAction = 'none';

  function frame(now) {
    if (!alive) return;
    const wallTime = Math.max(0, (now - lastTime) / 1000);
    const dt = Math.min(0.06, wallTime);
    lastTime = now;
    if (reducedMotion && !needsRender && tweens.size === 0 && activeCues.length === 0) {
      raf = requestAnimationFrame(frame);
      return;
    }
    needsRender = false;
    elapsed += dt;
    for (const item of [...tweens]) {
      item.age += wallTime;
      item.update(Math.min(1, item.age / item.duration));
      if (item.age >= item.duration) { tweens.delete(item); item.resolve(); }
    }
    waterUniforms.time.value = elapsed;
    const view = cameraEye.clone().sub(cameraTarget);
    const spherical = new THREE.Spherical().setFromVector3(view);
    spherical.theta += yaw;
    spherical.phi = clamp(spherical.phi + pitch, 0.2, 1.46);
    // A portrait viewport gets the same island silhouette without clipping.
    spherical.radius *= zoom * (rescueMode ? Math.max(1, Math.min(1.18, 0.44 / camera.aspect)) : Math.max(1, Math.min(1.85, 0.95 / camera.aspect)));
    camera.position.copy(cameraTarget).add(v().setFromSpherical(spherical));
    if (!rescueMode && !busy && !drag && !reducedMotion) { camera.position.x += Math.sin(elapsed * 0.15) * 0.12; camera.position.y += Math.sin(elapsed * 0.19) * 0.08; }
    camera.lookAt(cameraTarget);
    markerRing.material.opacity = 0.67 + Math.sin(elapsed * 2.5) * 0.18;
    playerGlow.material.opacity = 0.6 + Math.sin(elapsed * 3.3) * 0.08;
    for (const lamp of lamps) lamp.halo.material.opacity = 0.64 + Math.sin(elapsed * 2.3 + lamp.phase) * 0.055;
    for (const data of rescueMode ? [] : animated) {
      data.time += dt;
      if (data.kind === 'beacon' && data.activated) data.sweep.rotation.y += dt * 0.3;
      if (data.kind === 'transmitter' && data.activated) {
        const phase = (elapsed * 0.65) % 1;
        data.rings.scale.setScalar(0.7 + phase * 1.7);
        for (const ring of data.rings.children) ring.material.opacity = (1 - phase) * 0.45;
      }
      if (data.kind === 'generator' && data.activated) data.flywheel.rotation.x += dt * 3;
      if (data.kind === 'buoy') {
        data.object.position.y = data.initial.y + Math.sin(elapsed * 1.4) * 0.13;
        data.object.rotation.z = Math.sin(elapsed * 1.1) * 0.065;
        data.halo.material.opacity = 0.35 + Math.max(0, Math.sin(elapsed * 1.5)) * 0.6;
      }
      if (data.kind === 'boat') {
        data.object.position.y = data.initial.y + Math.sin(elapsed * 1.15 + 1) * 0.065;
        data.object.rotation.z = Math.sin(elapsed * 1.3) * 0.024;
        if (data.activated) {
          const distance = Math.min(data.time * 0.82, 29);
          data.object.position.x = data.initial.x + distance * 0.22;
          data.object.position.z = data.initial.z + distance;
        }
      }
    }
    updateCues(wallTime);
    if (!reducedMotion) {
      const p = rainGeometry.attributes.position.array;
      for (let i = 0; i < 390; i++) {
        const n = i * 6, dy = dt * 10;
        p[n] += dt * 1.4; p[n + 3] += dt * 1.4; p[n + 1] -= dy; p[n + 4] -= dy;
        if (p[n + 1] < -1) { p[n + 1] = 42; p[n + 4] = 42.6; p[n] -= 5.8; p[n + 3] -= 5.8; }
      }
      rainGeometry.attributes.position.needsUpdate = true;
    }
    renderer.render(scene, camera);
    raf = requestAnimationFrame(frame);
  }

  function dispose() {
    alive = false; cancelAnimationFrame(raf); resizeObserver.disconnect(); clearCues();
    for (const item of tweens) item.resolve(); tweens.clear();
    canvas.removeEventListener('pointerdown', onDown); canvas.removeEventListener('pointermove', onMove);
    canvas.removeEventListener('pointerup', onUp); canvas.removeEventListener('pointercancel', onCancel); canvas.removeEventListener('wheel', onWheel);
    const geometries = new Set(), materials = new Set();
    scene.traverse(object => {
      if (object.geometry) geometries.add(object.geometry);
      if (Array.isArray(object.material)) object.material.forEach(m => materials.add(m));
      else if (object.material) materials.add(object.material);
    });
    geometries.forEach(g => g.dispose()); materials.forEach(m => m.dispose()); glowTexture.dispose(); renderer.dispose();
  }
  if (options.model) setModel(options.model);
  raf = requestAnimationFrame(frame);
  return {
    play, reset, setModel, setWorld, setBindings, setRescueState: setBindings, getBindingPresentation, focus, resize, dispose,
    setReducedMotion(value) { reducedMotion = Boolean(value); needsRender = true; },
    get currentPlace() { return currentPlace; },
    get idle() { return !busy && tweens.size === 0; },
    getPlaces: () => [...places.values()].map(p => ({ id: p.id, kind: p.kind, position: p.position.toArray() })),
    getScreenPosition(id) {
      const entity = entities.get(id), place = places.get(id);
      if (!entity && !place) return null;
      const position = entity ? entity.object.getWorldPosition(v()).add(v(0, entity.labelHeight, 0)) : place.position.clone().add(v(0, 1.5, 0));
      const p = position.project(camera);
      return { x: (p.x + 1) * 0.5, y: (1 - p.y) * 0.5, visible: p.z > -1 && p.z < 1 && Math.abs(p.x) <= 1 && Math.abs(p.y) <= 1 };
    },
    projected(x, z) {
      const p = v(x, 0.2, z).project(camera);
      return { x: (p.x + 1) * 0.5, y: (1 - p.y) * 0.5, visible: p.z > -1 && p.z < 1 && Math.abs(p.x) <= 1 && Math.abs(p.y) <= 1 };
    },
    pointFromScreen(clientX, clientY) {
      const rect = canvas.getBoundingClientRect();
      pointer.set((clientX - rect.left) / rect.width * 2 - 1, -(clientY - rect.top) / rect.height * 2 + 1);
      raycaster.setFromCamera(pointer, camera);
      const point = raycaster.ray.intersectPlane(new THREE.Plane(UP, -0.2), v());
      return point ? { x: point.x, z: point.z } : null;
    },
    getPresentation: () => ({
      places: [...places.values()].map(p => ({ id: p.id, position: p.position.toArray(), camera: p.camera, target: p.target })),
      entities: [...entities.values()].map(e => ({ id: e.id, position: e.object.getWorldPosition(v()).toArray() })),
      overview: { position: baseEye.toArray(), target: baseTarget.toArray() },
      routes: [...routes.entries()].map(([id, curve]) => ({ id, points: curve.points.map(p => p.toArray()) })),
    }),
    getStats: () => ({ calls: renderer.info.render.calls, triangles: renderer.info.render.triangles, places: places.size, entities: entities.size, routes: routes.size / 2 }),
  };
}
