// Post-hoc: opens one program's policy up to 400 times in one process, each
// with 2,000 ticks, a view, a save and a resume, never calling free(), and
// prints when creating a policy first throws. The fuzz runs synchronously, so
// the runtime's finalizers cannot free the sessions in between.
//
//   node fuzz-checks/session-reuse.mjs /home/claude/glowcap-r8/authors/C3/impl/adapter.mjs
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
