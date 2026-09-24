import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
const source = await readFile(new URL('../ferry.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();
const s = runtime.open(source);
const ev = [['gust',{kt:12}],['gust',{kt:20.5}],['wave',{m:1}],['decide',{}],['wave',{m:1.2}],['decide',{}],['gust',{kt:35}],['gust',{kt:40}],['decide',{}],['warning',{}],['warning',{}]];
for (const [e,p] of ev) {
  const o = s.dispatch(e,p);
  const snap = s.snapshot();
  console.log(e, JSON.stringify(p), o.outcome, o.message ?? '', JSON.stringify(snap.bindings.hud));
}
const snap = s.snapshot();
console.log(JSON.stringify(snap.commitment_grounds), JSON.stringify(snap.decision_journal, null, 0));
