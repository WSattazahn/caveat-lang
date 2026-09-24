import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const file = process.argv[2] ?? 'pond.cav';
const steps = JSON.parse(process.argv[3] ?? '[]');
const source = await readFile(file, 'utf8');
const runtime = await loadRuntimeFromDirectory();
const session = runtime.open(source);
const show = (label) => {
  const s = session.snapshot();
  console.log(label, JSON.stringify(s.bindings.hud), 'seq', s.sequence);
};
show('initial');
for (const [event, payload] of steps) {
  const o = session.dispatch(event, payload ?? {});
  if (o.outcome === 'rejected') console.log(`${event} ${JSON.stringify(payload ?? {})}: REJECTED ${o.origin}/${o.code} ${o.message}`);
  else show(`${event} ${JSON.stringify(payload ?? {})}:`);
}
const s = session.snapshot();
console.log('grounds', JSON.stringify(s.commitment_grounds));
console.log('journal');
for (const e of s.decision_journal) console.log(' ', JSON.stringify(e));
console.log('bases', JSON.stringify(s.commitment_bases));
session.close();
