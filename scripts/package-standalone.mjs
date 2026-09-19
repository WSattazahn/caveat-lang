// Portable games embed the actual CAVEAT runtime and their source programs.
import { readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
const root = new URL('../dist/', import.meta.url);
const read = file => readFile(new URL(file, root), 'utf8');
const moduleURL = value => `data:text/javascript;base64,${Buffer.from(value).toString('base64')}`;
const wasm = (await readFile(new URL('pkg/caveat_runtime_bg.wasm', root))).toString('base64');
const runtime = await read('pkg/caveat_runtime.js');
const core = await read('vendor/three.core.js');
const three = (await read('vendor/three.module.js')).replaceAll("'./three.core.js'", "'#three-core'");
const world = (await read('beacon-world.js')).replace("'./vendor/three.module.js'", "'#three'");
const rescueWorld = (await read('rescue-world.js')).replace("'./beacon-world.js'", "'#world'");
const license = (await read('vendor/THREE-LICENSE.txt')).replaceAll('--', '—');
for (const game of [
  {entry:'rescue',source:'light_the_way.cav',output:'Light-the-Way.html'},
  {entry:'last-beacon',source:'the_last_beacon.cav',output:'The-Last-Beacon.html'},
]) {
  const source = await read(game.source);
  let main = (await read(`${game.entry}.js`))
    .replace("'./pkg/caveat_runtime.js'", "'#runtime'")
    .replace("'./beacon-world.js'", "'#world'")
    .replace("'./rescue-world.js'", "'#rescue-world'");
  const fetchCall = `await fetch('./${game.source}')`;
  if (!main.includes(fetchCall) || !main.includes('await init();')) throw Error(`Cannot embed ${game.entry}: boot contract changed`);
  main = main.replace(fetchCall, `new Response(${JSON.stringify(source)})`)
    .replace('await init();', `await init({module_or_path:Uint8Array.from(atob(${JSON.stringify(wasm)}),char=>char.charCodeAt(0))});`);
  const imports = {'#three-core':moduleURL(core),'#three':moduleURL(three),'#runtime':moduleURL(runtime),'#world':moduleURL(world),'#rescue-world':moduleURL(rescueWorld),'#main':moduleURL(main)};
  let html = await read(`${game.entry}.html`);
  html = html.replace(`<link rel="stylesheet" href="./${game.entry}.css">`, `<style>${await read(`${game.entry}.css`)}</style>`)
    .replace('href="./" aria-label="Caveat games"', 'href="#" aria-label="The Last Beacon"')
    .replace('<script src="./beacon-bootstrap.js"></script>', `<script>${await read('beacon-bootstrap.js')}</script>`)
    .replace(`<script type="module" src="./${game.entry}.js"></script>`, `<script type="importmap">${JSON.stringify({imports})}</script><script type="module">import '#main';</script>`)
    .replace('</body>', `<!-- Three.js license\n${license}\n-->\n</body>`);
  const output = new URL(game.output, root);
  await writeFile(output, html);
  console.log(`Portable game: ${fileURLToPath(output)} (${(Buffer.byteLength(html)/1024/1024).toFixed(1)} MiB)`);
}
