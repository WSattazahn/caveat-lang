// Post-hoc check, not a registered measure: CR13 says the 4096-byte save bound
// "now holds for play of any length". Long play: each cycle, another creature
// eats each of the five mushrooms (kinds alternate), then 46 s pass so all
// regrow. Reports save bytes after 10, 30 and 60 cycles (300 observations,
// 60 lives, under the 64-life floor) and the time to resume at 60.
//
//   node experiments/glowcap/round7/longplay.mjs [--root=DIR]
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const ROOT = process.argv.find((a) => a.startsWith('--root='))?.slice(7) ?? '/home/claude/glowcap-r7';
const IDS = ['cave', 'pool', 'ruin', 'grove', 'pit'];
const out = {};
for (const id of ['C1', 'C2', 'C3', 'C4', 'T1', 'T2', 'T3']) {
  const entry = path.join(ROOT, 'authors', id, 'impl', id.startsWith('C') ? 'adapter.mjs' : 'glowcap.ts');
  const module = await import(pathToFileURL(entry).href);
  if (module.ready) await module.ready;
  const policy = module.createPolicy();
  const row = { rejected: 0, saveBytes: {} };
  for (let cycle = 1; cycle <= 60; cycle += 1) {
    IDS.forEach((m, i) => {
      try { policy.dispatch({ type: 'witness', id: m, kind: (cycle + i) % 2 ? 'glowcap' : 'duskcap' }); } catch { row.rejected += 1; }
    });
    for (let t = 0; t < 46 * 16; t += 1) policy.dispatch({ type: 'tick', dt: 0.0625 });
    if ([10, 30, 60].includes(cycle)) row.saveBytes[cycle] = Buffer.byteLength(JSON.stringify(policy.save()));
  }
  const text = JSON.stringify(policy.save());
  const start = performance.now();
  const resumed = module.createPolicy(JSON.parse(text));
  const same = JSON.stringify(resumed.view()) === JSON.stringify(policy.view());
  row.resumeMs = Number((performance.now() - start).toFixed(1));
  row.resumedSameView = same;
  out[id] = row;
  console.error(id, JSON.stringify(row));
}
console.log(JSON.stringify(out, null, 2));
