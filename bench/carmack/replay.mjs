#!/usr/bin/env node
// Carmack check 2: byte-identical replay. Runs replay-core.mjs in Node and,
// with --browsers, in each named Playwright browser, and writes one digest
// file per platform. --compare checks digest files from every platform agree.
//
//   node bench/carmack/replay.mjs [--runtime DIR] [--browsers chromium,firefox,webkit] [--out DIR]
//   node bench/carmack/replay.mjs --compare DIR
//
// Exits 1 when a platform's own save/restore check fails or, with --compare,
// when any two digest files differ.
import { createReadStream, existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:http';
import path from 'node:path';
import { loadRuntimeFromDirectory, defaultRuntimeDirectory } from '../../kit/lib/node.mjs';
import { replayAll } from './replay-core.mjs';
import { ROOT } from './workloads.mjs';

const arg = (name) => { const i = process.argv.indexOf(`--${name}`); return i < 0 ? undefined : process.argv[i + 1]; };

// The programs every platform replays: the benchmark gate, the project's
// ledger, and the reactive examples.
export const REPLAY_PROGRAMS = [
  'bench/carmack/programs/release-gate.cav',
  'ledger/project.cav',
  'examples/thermostat_history.cav',
  'examples/ventilation_controller.cav',
  'examples/slime_glow_ability.cav',
  'examples/squeeze_affordance.cav',
  'caveatism/canon/archive.cav',
];

const programs = () => REPLAY_PROGRAMS.map((name) => ({ name, source: readFileSync(path.join(ROOT, name), 'utf8') }));
const OPTIONS = { sequences: Number(arg('sequences') ?? 16), events: Number(arg('events') ?? 80) };

if (arg('compare')) {
  const dir = path.resolve(arg('compare'));
  const files = readdirSync(dir, { recursive: true }).map(String).filter((f) => f.endsWith('.replay.json')).sort();
  if (files.length < 2) { console.error(`need at least two digest files in ${dir}, found ${files.length}`); process.exit(1); }
  const reports = files.map((f) => ({ file: f, ...JSON.parse(readFileSync(path.join(dir, f), 'utf8')) }));
  const [first] = reports;
  let differ = 0;
  for (const r of reports) {
    if (r.combined === first.combined && JSON.stringify(r.digests) === JSON.stringify(first.digests) && r.failures.length === 0) continue;
    differ += 1;
    console.error(`${r.file} (${r.platform}): combined ${r.combined}, ${r.failures.length} save/restore failure(s)`);
    for (const [program, sequences] of Object.entries(r.digests)) {
      for (const [s, hash] of Object.entries(sequences)) {
        if (first.digests[program]?.[s] !== hash) console.error(`  ${program} sequence ${s}: ${hash} against ${first.file}'s ${first.digests[program]?.[s]}`);
      }
    }
  }
  console.log(`${reports.length} platforms: ${reports.map((r) => r.platform).join(', ')}`);
  console.log(differ ? `${differ} platform(s) differ from ${first.platform}` : `all byte-identical: ${first.combined}`);
  process.exit(differ ? 1 : 0);
}

const out = path.resolve(arg('out') ?? path.join(ROOT, 'test-results', 'carmack'));
mkdirSync(out, { recursive: true });
const runtimeDir = path.resolve(arg('runtime') ?? defaultRuntimeDirectory());
let failed = false;

function write(platform, result) {
  const file = path.join(out, `${platform.replace(/[^a-z0-9.-]+/gi, '-')}.replay.json`);
  writeFileSync(file, `${JSON.stringify({ schema: 'caveat-carmack-replay/1', platform, options: OPTIONS, ...result }, null, 2)}\n`);
  console.log(`${platform}: ${result.combined}, ${result.failures.length} save/restore failure(s)`);
  for (const f of result.failures.slice(0, 5)) console.error(`  ${JSON.stringify(f)}`);
  if (result.failures.length) failed = true;
}

const runtime = await loadRuntimeFromDirectory(runtimeDir);
write(`node-${process.platform}-${process.arch}`, await replayAll(runtime, programs(), OPTIONS));

const browsers = arg('browsers')?.split(',').filter(Boolean) ?? [];
if (browsers.length) {
  const playwright = await import('playwright');
  const types = { '.html': 'text/html', '.mjs': 'text/javascript', '.js': 'text/javascript', '.json': 'application/json', '.wasm': 'application/wasm' };
  // Serves the kit, the runtime and the program list; nothing else.
  const roots = { '/kit/': path.join(ROOT, 'kit', 'lib'), '/runtime/': runtimeDir, '/bench/': path.join(ROOT, 'bench', 'carmack') };
  const server = createServer((request, response) => {
    const url = new URL(request.url, 'http://localhost');
    if (url.pathname === '/') { response.writeHead(200, { 'content-type': 'text/html' }).end('<!doctype html><title>replay</title>'); return; }
    if (url.pathname === '/programs.json') { response.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify(programs())); return; }
    const prefix = Object.keys(roots).find((p) => url.pathname.startsWith(p));
    const file = prefix && path.join(roots[prefix], decodeURIComponent(url.pathname.slice(prefix.length)));
    if (!file || !file.startsWith(roots[prefix]) || !existsSync(file)) { response.writeHead(404).end(); return; }
    response.writeHead(200, { 'content-type': types[path.extname(file)] ?? 'application/octet-stream' });
    createReadStream(file).pipe(response);
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const base = `http://127.0.0.1:${server.address().port}`;
  try {
    for (const name of browsers) {
      // CARMACK_CHROMIUM names a Chromium already installed, for machines
      // without Playwright's own download.
      const browser = await playwright[name].launch(name === 'chromium' && process.env.CARMACK_CHROMIUM ? { executablePath: process.env.CARMACK_CHROMIUM } : {});
      try {
        const page = await browser.newPage();
        await page.goto(`${base}/`);
        const result = await page.evaluate(async (options) => {
          const { loadRuntime } = await import('/kit/session.mjs');
          const { replayAll: run } = await import('/bench/replay-core.mjs');
          const runtime = await loadRuntime({ module: new URL('/runtime/caveat_runtime.js', location.href).href, wasm: new URL('/runtime/caveat_runtime_bg.wasm', location.href) });
          const list = await (await fetch('/programs.json')).json();
          return run(runtime, list, options);
        }, OPTIONS);
        write(`${name}-${process.platform}`, { browserVersion: browser.version(), ...result });
      } finally {
        await browser.close();
      }
    }
  } finally {
    server.close();
  }
}
process.exit(failed ? 1 : 0);
