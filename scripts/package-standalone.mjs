// Produce a portable game with real CAVEAT WASM and all graphics embedded.
// No network, local server, CDNs, or substitute JavaScript game logic required.
import { readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
const root = new URL('../dist/', import.meta.url);
const read = file => readFile(new URL(file, root), 'utf8');
const moduleURL = value => `data:text/javascript;base64,${Buffer.from(value).toString('base64')}`;
const wasm = (await readFile(new URL('pkg/caveat_runtime_bg.wasm', root))).toString('base64');
const story = await read('the_last_beacon.cav');
const runtime = await read('pkg/caveat_runtime.js');
const core = await read('vendor/three.core.js');
const three = (await read('vendor/three.module.js')).replaceAll("'./three.core.js'", "'#three-core'");
const world = (await read('beacon-world.js')).replace("'./vendor/three.module.js'", "'#three'");
let main = (await read('last-beacon.js')).replace("'./pkg/caveat_runtime.js'", "'#runtime'").replace("'./beacon-world.js'", "'#world'");
main = main.replace("await fetch('./the_last_beacon.cav')", `new Response(${JSON.stringify(story)})`);
main = main.replace('await init();', `await init({module_or_path:Uint8Array.from(atob(${JSON.stringify(wasm)}),char=>char.charCodeAt(0))});`);
const imports = {'#three-core':moduleURL(core),'#three':moduleURL(three),'#runtime':moduleURL(runtime),'#world':moduleURL(world),'#main':moduleURL(main)};
let html = await read('last-beacon.html');
html = html.replace('<link rel="stylesheet" href="./last-beacon.css">', `<style>${await read('last-beacon.css')}</style>`);
html = html.replace('href="./" aria-label="Caveat games"', 'href="#" aria-label="The Last Beacon"');
html = html.replace('<script src="./beacon-bootstrap.js"></script>', `<script>${await read('beacon-bootstrap.js')}</script>`);
html = html.replace('<script type="module" src="./last-beacon.js"></script>', `<script type="importmap">${JSON.stringify({imports})}</script><script type="module">import '#main';</script>`);
const license = (await read('vendor/THREE-LICENSE.txt')).replaceAll('--', '—');
html = html.replace('</body>', `<!-- Three.js license\n${license}\n-->\n</body>`);
const output = new URL('The-Last-Beacon.html', root);
await writeFile(output, html);
console.log(`Portable game: ${fileURLToPath(output)} (${(Buffer.byteLength(html)/1024/1024).toFixed(1)} MiB)`);
