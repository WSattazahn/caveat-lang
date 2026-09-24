import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
const source = await readFile('pond.cav', 'utf8');
const runtime = await loadRuntimeFromDirectory();
const session = runtime.open(source);
for (const [e, p] of [['scan', { cm: 14 }], ['measure', { cm: 9 }], ['crack', {}]]) session.dispatch(e, p);
const s = session.snapshot();
console.log(Object.keys(s).join(' '));
console.log(JSON.stringify(s.relations));
console.log(JSON.stringify(s.reading_streams, null, 1));
console.log(JSON.stringify(s.decision_series));
console.log(JSON.stringify(s.symbols).slice(0, 1500));
for (const [e, p] of [['measure', { cm: 'x' }], ['measure', { cm: '12' }], ['crack', { cm: 1 }], ['scan', {}], ['scan', { cm: 61 }], ['nope', {}], ['scan', { cm: 3, extra: 1 }], ['decide', null], ['decide', []]]) {
  try { console.log(e, JSON.stringify(p), JSON.stringify(session.dispatch(e, p))); } catch (err) { console.log(e, JSON.stringify(p), 'THROW', err.kind, err.message); }
}
session.close();
