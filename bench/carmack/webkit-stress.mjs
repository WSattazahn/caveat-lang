#!/usr/bin/env node
// Investigation harness for the intermittent WebKit trap in
// webreactivesession_new. Loads the runtime in many fresh WebKit pages and
// counts traps; prints a summary line per configuration.
//
//   node bench/carmack/webkit-stress.mjs --runtime DIR [--pages N] [--replays N] [--browser webkit]
//
// JSC_* environment variables are passed to the browser, so engine options
// (JSC_useOMGJIT=0 and the like) can be compared.
import { createReadStream, existsSync, readFileSync } from 'node:fs';
import { createServer } from 'node:http';
import path from 'node:path';
import { ROOT } from './workloads.mjs';

const arg = (name, fallback) => { const i = process.argv.indexOf(`--${name}`); return i < 0 ? fallback : process.argv[i + 1]; };
const runtimeDir = path.resolve(arg('runtime', 'dist/pkg-reactive'));
const pages = Number(arg('pages', 30));
const replays = Number(arg('replays', 2));
const browserName = arg('browser', 'webkit');
const garbage = Number(arg('garbage', 0));
const REPLAY_PROGRAMS = ['bench/carmack/programs/release-gate.cav', 'ledger/project.cav', 'examples/thermostat_history.cav', 'examples/ventilation_controller.cav', 'examples/slime_glow_ability.cav', 'examples/squeeze_affordance.cav', 'caveatism/canon/archive.cav'];
const programs = REPLAY_PROGRAMS.map((name) => ({ name, source: readFileSync(path.join(ROOT, name), 'utf8') }));

const playwright = await import('playwright');
const types = { '.html': 'text/html', '.mjs': 'text/javascript', '.js': 'text/javascript', '.json': 'application/json', '.wasm': 'application/wasm' };
const roots = { '/kit/': path.join(ROOT, 'kit', 'lib'), '/runtime/': runtimeDir, '/bench/': path.join(ROOT, 'bench', 'carmack') };
const server = createServer((request, response) => {
  const url = new URL(request.url, 'http://localhost');
  if (url.pathname === '/') { response.writeHead(200, { 'content-type': 'text/html' }).end('<!doctype html><title>stress</title>'); return; }
  if (url.pathname === '/programs.json') { response.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify(programs)); return; }
  const prefix = Object.keys(roots).find((p) => url.pathname.startsWith(p));
  const file = prefix && path.join(roots[prefix], decodeURIComponent(url.pathname.slice(prefix.length)));
  if (!file || !file.startsWith(roots[prefix]) || !existsSync(file)) { response.writeHead(404).end(); return; }
  response.writeHead(200, { 'content-type': types[path.extname(file)] ?? 'application/octet-stream' });
  createReadStream(file).pipe(response);
});
await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
const base = `http://127.0.0.1:${server.address().port}`;
const jsc = Object.fromEntries(Object.entries(process.env).filter(([k]) => k.startsWith('JSC_')));
const browser = await playwright[browserName].launch({ env: { ...process.env }, ...(browserName === 'chromium' && process.env.CARMACK_CHROMIUM ? { executablePath: process.env.CARMACK_CHROMIUM } : {}) });
const version = browser.version();
const failures = [];
let startups = 0;
try {
  // Fresh pages: load the runtime and open every program three times.
  for (let i = 0; i < pages; i += 1) {
    const page = await browser.newPage();
    try {
      await page.goto(`${base}/`);
      const result = await page.evaluate(async (garbage) => {
        // Garbage: churn the JS heap so collections land inside wasm calls.
        let keep = [];
        const churn = () => { for (let i = 0; i < garbage * 2000; i += 1) keep.push({ i, s: `x${i}`, a: [i, i + 1] }); if (keep.length > garbage * 20000) keep = []; };
        let timer = garbage ? setInterval(churn, 1) : null;
        const ns = await import('/runtime/caveat_runtime.js');
        await ns.default({ module_or_path: new URL('/runtime/caveat_runtime_bg.wasm', location.href) });
        const list = await (await fetch('/programs.json')).json();
        const memory = () => undefined;
        let opened = 0;
        for (let round = 0; round < 3; round += 1) {
          for (const program of list) {
            try {
              if (garbage) churn();
              new ns.WebReactiveSession(program.source).free();
              opened += 1;
            } catch (error) {
              if (String(error?.message).startsWith('PASS_MISMATCH')) { clearInterval(timer); return { trap: String(error.message), program: program.name, round, opened }; }
              if (error instanceof WebAssembly.RuntimeError) { clearInterval(timer); return { trap: String(error.message), program: program.name, round, opened, memory: memory() }; }
            }
          }
        }
        clearInterval(timer);
        return { opened };
      }, garbage);
      startups += 1;
      if (result.trap) { failures.push({ kind: 'startup', page: i, ...result }); console.log(`page ${i}: TRAP ${result.trap} (${result.program}, round ${result.round}, after ${result.opened})`); }
    } catch (error) {
      failures.push({ kind: 'startup', page: i, error: String(error.message ?? error).slice(0, 300) });
      console.log(`page ${i}: ERROR ${String(error.message ?? error).slice(0, 300)}`);
    } finally {
      await page.close();
    }
  }
  for (let r = 0; r < replays; r += 1) {
    const page = await browser.newPage();
    try {
      await page.goto(`${base}/`);
      await page.evaluate(async () => {
        const { loadRuntime } = await import('/kit/session.mjs');
        const { replayAll } = await import('/bench/replay-core.mjs');
        const runtime = await loadRuntime({ module: new URL('/runtime/caveat_runtime.js', location.href).href, wasm: new URL('/runtime/caveat_runtime_bg.wasm', location.href) });
        const list = await (await fetch('/programs.json')).json();
        return replayAll(runtime, list, { sequences: 16, events: 80 });
      });
      console.log(`replay ${r}: ok`);
    } catch (error) {
      failures.push({ kind: 'replay', replay: r, error: String(error.message ?? error).slice(0, 300) });
      console.log(`replay ${r}: ERROR ${String(error.message ?? error).slice(0, 300)}`);
    } finally {
      await page.close();
    }
  }
} finally {
  await browser.close();
  server.close();
}
console.log(`SUMMARY ${browserName} ${version} jsc=${JSON.stringify(jsc)} garbage=${garbage} pages=${startups}/${pages} replays=${replays} failures=${failures.length}`);
process.exit(0);
