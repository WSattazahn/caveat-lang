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
const PRESENTATIONS = {
  courtyard: { position: [0, 3.15, 2], camera: [21, 23, 34], target: [0, 6, 2] },
  lighthouse: { position: [-6.4, 6.1, -5.8], camera: [9, 19, 17], target: [-6.4, 11, -5.8] },
  lantern_room: { position: [-6.4, 20, -5.8], camera: [5.5, 27, 10], target: [-6.4, 21, -5.8] },
  observatory: { position: [9.5, 5.2, -3.8], camera: [26, 16, 14], target: [9.5, 6.6, -3.8] },
  archive: { position: [-10.8, 2.8, 6], camera: [3, 13, 24], target: [-10.8, 4, 6] },
  harbor: { position: [5.3, 1.2, 12.2], camera: [24, 13, 32], target: [5.3, 2.4, 12.2] },
  breakwater: { position: [15.5, 1.1, 14.9], camera: [30, 13, 32], target: [15.5, 1.6, 14.9] },
};

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
  const random = seeded();
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x071c2b);
  scene.fog = new THREE.FogExp2(0x092736, 0.0078);
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: false, powerPreference: 'high-performance' });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 1.7));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.22;
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  const camera = new THREE.PerspectiveCamera(43, 1, 0.1, 750);
  const baseEye = v(35, 30, 46);
  const baseTarget = v(0, 6.4, 1.5);
  const cameraEye = baseEye.clone();
  const cameraTarget = baseTarget.clone();
  camera.position.copy(baseEye);
  camera.lookAt(baseTarget);
  const world = new THREE.Group();
  const graph = new THREE.Group();
  scene.add(world, graph);
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

  const waterUniforms = { time: { value: 0 }, cameraPositionWorld: { value: camera.position }, lightDirection: { value: v(-0.45, 0.8, 0.3) } };
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
      uniform float time; uniform vec3 cameraPositionWorld; uniform vec3 lightDirection;
      varying vec3 vWorld;varying vec3 vNormal;varying float vWave;
      void main(){
        vec3 N=normalize(vNormal);vec3 V=normalize(cameraPositionWorld-vWorld);
        float fresnel=pow(1.-max(dot(N,V),0.),3.);
        float sparkle=pow(max(dot(N,normalize(V+lightDirection)),0.),100.);
        float ripples=sin(vWorld.x*1.6+vWorld.z*.7+sin(vWorld.z*1.9-time)*.5+time)*.5+.5;
        vec3 color=mix(vec3(.018,.10,.13),vec3(.12,.25,.29),fresnel);
        color+=vec3(.22,.38,.4)*sparkle*.7;
        color+=vec3(.025,.08,.08)*smoothstep(.91,1.,ripples)*smoothstep(-.1,.2,vWave);
        float shore=length(vec2(vWorld.x/17.5,(vWorld.z-1.)/14.5));
        float surf=pow(max(0.,sin(shore*32.-time*1.1+sin(atan(vWorld.z-1.,vWorld.x)*9.)*.5)),16.);
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
    mesh(geometry, material(0xffffff, { vertexColors: true, flatShading: true }), world);
  }
  rockTerrace(0, 1, 17.2, 14.1, -1.8, 1.25, 101, 0x304843);
  rockTerrace(-1, 0, 15.4, 11.4, 0.4, 2.55, 111, 0x344f46);
  rockTerrace(-2.5, -3, 11.5, 8.4, 2.1, 3.05, 144, 0x38594a);
  rockTerrace(-6.4, -5.8, 6.6, 5.8, 2.4, 4.8, 455, 0x3b564e);
  rockTerrace(-6.4, -5.8, 5.4, 4.9, 4.2, 6, 467, 0x3c5d50);
  rockTerrace(9.5, -3.8, 5.9, 5.3, 1.2, 3.8, 452, 0x3b5749);
  rockTerrace(9.5, -3.8, 4.7, 4.1, 3.2, 5.1, 422, 0x3b574f);
  rockTerrace(-10.8, 6, 5.5, 4.6, 0.2, 2.7, 558, 0x39544b);

  // Instanced coastal boulders and grass keep the draw budget modest on phones.
  const rocks = new THREE.InstancedMesh(new THREE.DodecahedronGeometry(1, 0), material(0x4a6060, { flatShading: true }), 116);
  const dummy = new THREE.Object3D();
  for (let i = 0; i < 116; i++) {
    const a = i / 116 * TAU, r = 0.98 + random() * 0.16;
    dummy.position.set(Math.cos(a) * 17 * r, -0.2 + random() * 0.5, 1 + Math.sin(a) * 14 * r);
    dummy.rotation.set(random(), random(), random());
    const s = 0.45 + random() * 1.3;
    dummy.scale.set(s, s * (0.65 + random()), s * 0.8);
    dummy.updateMatrix(); rocks.setMatrixAt(i, dummy.matrix);
  }
  rocks.castShadow = rocks.receiveShadow = true;
  world.add(rocks);
  const grass = new THREE.InstancedMesh(new THREE.ConeGeometry(0.16, 0.72, 3), material(0x5a8070, { flatShading: true }), 360);
  for (let i = 0; i < 360; i++) {
    const a = random() * TAU, r = 0.75 + random() * 0.21;
    dummy.position.set(-1 + Math.cos(a) * 14 * r, 2.85, Math.sin(a) * 10.6 * r);
    dummy.rotation.set((random() - 0.5) * 0.4, random() * TAU, (random() - 0.5) * 0.4);
    const s = 0.6 + random() * 0.8; dummy.scale.set(s, s, s); dummy.updateMatrix(); grass.setMatrixAt(i, dummy.matrix);
  }
  grass.receiveShadow = true; world.add(grass);

  const rainArray = new Float32Array(390 * 6);
  for (let i = 0; i < 390; i++) {
    const x = (random() - 0.5) * 85, y = random() * 45, z = (random() - 0.5) * 75;
    rainArray.set([x, y, z, x - 0.09, y + 0.6, z + 0.03], i * 6);
  }
  const rainGeometry = new THREE.BufferGeometry();
  rainGeometry.setAttribute('position', new THREE.BufferAttribute(rainArray, 3));
  const rain = new THREE.LineSegments(rainGeometry, new THREE.LineBasicMaterial({ color: 0x89a8b0, transparent: true, opacity: 0.11, depthWrite: false }));
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
    // The continuous spiral gives declared tower moves a physical route.
    const points = [];
    for (let i = 0; i <= 140; i++) {
      const t = i / 140, a = t * TAU * 2.25 + Math.PI / 2, r = 2.34 - t * 0.38;
      points.push(v(Math.cos(a) * r, 0.5 + t * 13, Math.sin(a) * r));
      if (i % 2 === 0) {
        const step = box(parent, Math.cos(a) * r, 0.35 + t * 13, Math.sin(a) * r, 0.68, 0.09, 0.29, 0x668080);
        step.rotation.y = -a;
      }
    }
    const rail = new THREE.CatmullRomCurve3(points.map(p => p.clone().add(v(0, 0.84, 0))));
    mesh(new THREE.TubeGeometry(rail, 110, 0.04, 5, false), material(0x6f9590), parent);
    for (let i = 0; i < points.length; i += 8) {
      const p = points[i]; beam(parent, p.toArray(), p.clone().add(v(0, 0.85, 0)).toArray(), 0.032, 0x6f9590);
    }
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
    const offsets = {
      beacon: [0, 0.13, 0], optic: [0.5, 0.12, 1.15], telescope: [0, 4.75, 0],
      transmitter: [3, 0.2, -0.7], generator: [-2.8, 0.25, 0], chart: [1.35, 0.2, 2.8],
      gauge: [-2.65, -0.5, 1.5], buoy: [3.4, -0.8, 1.4], boat: [3.7, -0.76, 4.5],
    };
    object.position.set(...(offsets[def.kind] || [1.5 + index * 0.35, 0.5, 1.5]));
    const data = { id: def.id, kind: def.kind, place: def.at, object, activated: false, open: false, time: 0, initial: object.position.clone() };
    switch (def.kind) {
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
    const start = a.position.clone(), end = b.position.clone();
    if ((a.kind === 'lighthouse' && b.kind === 'lantern_room') || (b.kind === 'lighthouse' && a.kind === 'lantern_room')) {
      const base = a.kind === 'lighthouse' ? a : b;
      const points = [];
      for (let i = 0; i <= 100; i++) {
        const t = i / 100, angle = t * TAU * 2.25 + Math.PI / 2, r = 2.34 - t * 0.38;
        points.push(base.position.clone().add(v(Math.cos(angle) * r, 0.5 + t * 13, Math.sin(angle) * r)));
      }
      const room = a.kind === 'lantern_room' ? a : b;
      points.push(room.position.clone());
      if (b.kind === 'lighthouse') points.reverse();
      return new THREE.CatmullRomCurve3(points);
    }
    if ((a.kind === 'courtyard' && b.kind === 'harbor') || (a.kind === 'harbor' && b.kind === 'courtyard')) {
      const court = a.kind === 'courtyard' ? a : b, port = a.kind === 'harbor' ? a : b;
      const points = [court.position.clone(), v(2, 3.18, 7.5), v(3.5, 2.75, 9.7), v(4.3, 1.9, 11), port.position.clone()];
      if (a.kind === 'harbor') points.reverse();
      return new THREE.CatmullRomCurve3(points);
    }
    const mid = start.clone().lerp(end, 0.5);
    const direction = end.clone().sub(start);
    mid.add(v(-direction.z, 0, direction.x).normalize().multiplyScalar(0.5));
    // The ground path lies on the upper terraces until its descending steps.
    if (a.kind === 'courtyard' || b.kind === 'courtyard') mid.y = Math.max(3.15, mid.y);
    if (a.kind === 'lighthouse' || b.kind === 'lighthouse') mid.y = Math.max(6.18, mid.y);
    if (a.kind === 'observatory' || b.kind === 'observatory') mid.y = Math.max(5.26, mid.y);
    return new THREE.CatmullRomCurve3([start, mid, end]);
  }

  function drawConnection(connection) {
    const curve = routeCurve(connection.from, connection.to);
    if (!curve) return;
    routes.set(`${connection.from}:${connection.to}`, curve);
    const reversePoints = curve.getPoints(60).reverse();
    routes.set(`${connection.to}:${connection.from}`, new THREE.CatmullRomCurve3(reversePoints));
    const a = places.get(connection.from), b = places.get(connection.to);
    if (a.kind === 'lantern_room' || b.kind === 'lantern_room') return;
    const length = curve.getLength(), steps = Math.max(6, Math.floor(length / 0.43));
    for (let i = 1; i < steps; i++) {
      const t = i / steps, p = curve.getPoint(t), tangent = curve.getTangent(t);
      const step = box(graph, p.x, p.y - 0.02, p.z, 1.2, 0.14, 0.35, i % 4 ? 0x68796c : 0x809081);
      step.rotation.y = Math.atan2(tangent.x, tangent.z);
      if (Math.abs(tangent.y) > 0.18) box(graph, p.x, p.y - 0.3, p.z, 1.25, 0.55, 0.45, 0x53675e).rotation.y = step.rotation.y;
    }
    for (const t of [0.23, 0.73]) {
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

  function clearGraph() {
    for (const child of [...graph.children]) if (child !== marker && child !== selection) graph.remove(child);
    places.clear(); entities.clear(); routes.clear(); animated.length = 0; clicks.length = 0; lamps.length = 0;
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
    const templates = { courtyard, lighthouse, observatory, archive, harbor, breakwater };
    const seenKinds = new Map();
    model.places.forEach((def, index) => {
      const spec = PRESENTATIONS[def.kind];
      const position = spec ? v(...spec.position) : v(Math.cos(index * 2.4) * 9, 3.2, Math.sin(index * 2.4) * 8);
      const n = seenKinds.get(def.kind) || 0;
      seenKinds.set(def.kind, n + 1);
      if (n) position.x += n * 7;
      const object = new THREE.Group(); object.position.copy(position); graph.add(object);
      const anchor = { id: def.id, kind: def.kind, position, object, camera: spec?.camera, target: spec?.target };
      places.set(def.id, anchor);
      if (templates[def.kind]) templates[def.kind](object);
      else if (def.kind !== 'lantern_room') cylinder(object, 0, 0, 0, 2.2, 2.3, 0.3, 0x688273, 16);
      const hit = mesh(new THREE.SphereGeometry(def.kind === 'lighthouse' ? 3.4 : 2.7, 10, 8), new THREE.MeshBasicMaterial({ visible: false }), object, [0, def.kind === 'lighthouse' ? 5 : 1.6, 0]);
      hit.userData.place = def.id; clicks.push(hit);
    });
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
    resetGeneration++;
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
    if (event.button !== undefined && event.button !== 0) return;
    drag = { x: event.clientX, y: event.clientY, startX: event.clientX, startY: event.clientY, pointerId: event.pointerId };
    canvas.setPointerCapture?.(event.pointerId);
  }
  function onMove(event) {
    if (!drag || busy) return;
    yaw += (event.clientX - drag.x) * 0.004;
    pitch = clamp(pitch + (event.clientY - drag.y) * 0.003, -0.22, 0.48);
    drag.x = event.clientX; drag.y = event.clientY;
  }
  function onUp(event) {
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
    event.preventDefault();
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
    const dt = Math.min(0.06, (now - lastTime) / 1000);
    lastTime = now; elapsed += dt;
    for (const item of [...tweens]) {
      item.age += dt;
      item.update(Math.min(1, item.age / item.duration));
      if (item.age >= item.duration) { tweens.delete(item); item.resolve(); }
    }
    waterUniforms.time.value = elapsed;
    const view = cameraEye.clone().sub(cameraTarget);
    const spherical = new THREE.Spherical().setFromVector3(view);
    spherical.theta += yaw;
    spherical.phi = clamp(spherical.phi + pitch, 0.2, 1.46);
    // A portrait viewport gets the same island silhouette without clipping.
    spherical.radius *= zoom * Math.max(1, Math.min(1.85, 0.95 / camera.aspect));
    camera.position.copy(cameraTarget).add(v().setFromSpherical(spherical));
    if (!busy && !drag && !reducedMotion) { camera.position.x += Math.sin(elapsed * 0.15) * 0.12; camera.position.y += Math.sin(elapsed * 0.19) * 0.08; }
    camera.lookAt(cameraTarget);
    markerRing.material.opacity = 0.67 + Math.sin(elapsed * 2.5) * 0.18;
    playerGlow.material.opacity = 0.6 + Math.sin(elapsed * 3.3) * 0.08;
    for (const lamp of lamps) lamp.halo.material.opacity = 0.64 + Math.sin(elapsed * 2.3 + lamp.phase) * 0.055;
    for (const data of animated) {
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
    alive = false; cancelAnimationFrame(raf); resizeObserver.disconnect();
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
    play, reset, setModel, setWorld, focus, resize, dispose,
    setReducedMotion(value) { reducedMotion = Boolean(value); },
    get currentPlace() { return currentPlace; },
    get idle() { return !busy && tweens.size === 0; },
    getPlaces: () => [...places.values()].map(p => ({ id: p.id, kind: p.kind, position: p.position.toArray() })),
    getScreenPosition(id) {
      const place = places.get(id); if (!place) return null;
      const p = place.position.clone().add(v(0, 1.5, 0)).project(camera);
      return { x: (p.x + 1) * 0.5, y: (1 - p.y) * 0.5, visible: p.z > -1 && p.z < 1 };
    },
    getStats: () => ({ calls: renderer.info.render.calls, triangles: renderer.info.render.triangles, places: places.size, entities: entities.size, routes: routes.size / 2 }),
  };
}
