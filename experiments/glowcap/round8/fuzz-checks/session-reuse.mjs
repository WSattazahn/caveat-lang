import { pathToFileURL } from 'node:url';
const m = await import(pathToFileURL(process.argv[2]).href);
if (m.ready) await m.ready;
for (let i = 0; i < 400; i++) {
  let p;
  try { p = m.createPolicy(); } catch (e) { console.log('fail at', i, String(e?.message ?? e).slice(0, 300), process.memoryUsage().arrayBuffers); break; }
  for (let t = 0; t < 2000; t++) { try { p.dispatch({ type: 'tick', dt: 0.0625 }); } catch {} }
  p.view(); const s = JSON.stringify(p.save()); m.createPolicy(JSON.parse(s)).view();
  if (i % 50 === 0) console.log(i, Math.round(process.memoryUsage().arrayBuffers / 1e6), 'MB');
}
