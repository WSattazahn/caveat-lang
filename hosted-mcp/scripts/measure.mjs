// Time per tool call for the starter program, with one runtime kept per
// process as the Worker keeps one per isolate:   npm run measure
// Workers bill CPU, not wall time; the free plan allows 10 ms per request.
// A call does no I/O, so its main-thread wall time approximates its CPU.
// Node and workerd share V8, so this estimates the Worker; the deployed
// Worker's own CPU figures are in the Cloudflare dashboard.
import { register } from 'node:module';
import { readFile } from 'node:fs/promises';

register('./measure-hooks.mjs', import.meta.url);
const { runAuthoringOperation } = await import('../node_modules/caveat-lang/lib/authoring.mjs');
const pkg = new URL('../node_modules/caveat-lang/', import.meta.url);
const source = await readFile(new URL('templates/umbrella.cav', pkg), 'utf8');
const scenarios = { ...JSON.parse(await readFile(new URL('templates/umbrella.scenarios.json', pkg), 'utf8')), source: 'inline.cav' };
const events = (await readFile(new URL('templates/events.jsonl', pkg), 'utf8')).trim().split('\n').map(line => JSON.parse(line));
const calls = [
  ['caveat_validate', { source }],
  ['caveat_check', { source }],
  ['caveat_test', { source, scenarios }],
  ['caveat_explain', { source, events }],
  ['caveat_dependents', { source, subject: 'forecast', events }],
];
const time = async fn => { const start = performance.now(); await fn(); return performance.now() - start; };
console.log(`loading the runtime (at isolate start in the Worker): ${(await time(() => runAuthoringOperation('caveat_validate', { source }))).toFixed(1)} ms`);
for (const [name, args] of calls) {
  // The first call of each kind runs before the JIT has warmed up.
  const first = await time(() => runAuthoringOperation(name, args));
  for (let index = 0; index < 200; index++) await runAuthoringOperation(name, args);
  const runs = [];
  for (let index = 0; index < 101; index++) runs.push(await time(() => runAuthoringOperation(name, args)));
  runs.sort((a, b) => a - b);
  console.log(`${name}: first ${first.toFixed(1)} ms; warm median ${runs[50].toFixed(2)} ms, 95th percentile ${runs[95].toFixed(2)} ms`);
}
