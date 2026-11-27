#!/usr/bin/env node
// WebKit startup check. WebKit 26.0 (Playwright 1.55) trapped in about 1% of
// fresh pages inside webreactivesession_new ("call_indirect to a null table
// entry", "Out of bounds memory access", "Unreachable code"); the traps
// stopped with its baseline (BBQ) tier or OSR tier-up switched off, and do not
// occur in WebKit 27.2. This loads the runtime in many fresh pages, opens
// every replay program three times in each, and exits 1 on any trap.
//
//   node bench/carmack/webkit-stress.mjs --runtime DIR [--pages N] [--browser webkit]
//
// JSC_* environment variables reach the browser, so engine options
// (JSC_useBBQJIT=0 and the like) can be compared.
import { createReadStream, existsSync, readFileSync } from 'node:fs';
import { createServer } from 'node:http';
import path from 'node:path';
import { ROOT } from './workloads.mjs';

const arg = (name, fallback) => { const i = process.argv.indexOf(`--${name}`); return i < 0 ? fallback : process.argv[i + 1]; };
const runtimeDir = path.resolve(arg('runtime', 'dist/pkg-reactive'));
const pages = Number(arg('pages', 200));
const browserName = arg('browser', 'webkit');
// The same programs as REPLAY_PROGRAMS in replay.mjs, which runs on import.
const PROGRAMS = ['bench/carmack/programs/release-gate.cav', 'ledger/project.cav', 'examples/thermostat_history.cav', 'examples/ventilation_controller.cav', 'examples/slime_glow_ability.cav', 'examples/squeeze_affordance.cav', 'caveatism/canon/archive.cav'];
const programs = PROGRAMS.map((name) => ({ name, source: readFileSync(path.join(ROOT, name), 'utf8') }));

const playwright = await import('playwright');
const types = { '.html': 'text/html', '.js': 'text/javascript', '.json': 'application/json', '.wasm': 'application/wasm' };
const server = createServer((request, response) => {
  const url = new URL(request.url, 'http://localhost');
  if (url.pathname === '/') { response.writeHead(200, { 'content-type': 'text/html' }).end('<!doctype html><title>webkit check</title>'); return; }
  if (url.pathname === '/programs.json') { response.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify(programs)); return; }
  const file = url.pathname.startsWith('/runtime/') && path.join(runtimeDir, decodeURIComponent(url.pathname.slice('/runtime/'.length)));
  if (!file || !file.startsWith(runtimeDir) || !existsSync(file)) { response.writeHead(404).end(); return; }
  response.writeHead(200, { 'content-type': types[path.extname(file)] ?? 'application/octet-stream' });
  createReadStream(file).pipe(response);
});
await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
const base = `http://127.0.0.1:${server.address().port}`;
const jsc = Object.fromEntries(Object.entries(process.env).filter(([k]) => k.startsWith('JSC_')));
const browser = await playwright[browserName].launch({ env: { ...process.env } });
const version = browser.version();
const failures = [];
try {
  for (let i = 0; i < pages; i += 1) {
    const page = await browser.newPage();
    try {
      await page.goto(`${base}/`);
      const result = await page.evaluate(async () => {
        const ns = await import('/runtime/caveat_runtime.js');
        await ns.default({ module_or_path: new URL('/runtime/caveat_runtime_bg.wasm', location.href) });
        const list = await (await fetch('/programs.json')).json();
        let opened = 0;
        for (let round = 0; round < 3; round += 1) {
          for (const program of list) {
            try {
              new ns.WebReactiveSession(program.source).free();
              opened += 1;
            } catch (error) {
              // A refusal is the program's answer; only a trap is a failure.
              if (error instanceof WebAssembly.RuntimeError) return { trap: String(error.message), program: program.name, round, opened };
            }
          }
        }
        return { opened };
      });
      if (result.trap) {
        failures.push({ page: i, ...result });
        console.log(`page ${i}: TRAP ${result.trap} (${result.program}, round ${result.round}, after ${result.opened})`);
      }
    } catch (error) {
      failures.push({ page: i, error: String(error.message ?? error).slice(0, 300) });
      console.log(`page ${i}: ERROR ${String(error.message ?? error).slice(0, 300)}`);
    } finally {
      await page.close();
    }
  }
} finally {
  await browser.close();
  server.close();
}
console.log(`SUMMARY ${browserName} ${version} jsc=${JSON.stringify(jsc)} pages=${pages} failures=${failures.length}`);
process.exit(failures.length ? 1 : 0);
