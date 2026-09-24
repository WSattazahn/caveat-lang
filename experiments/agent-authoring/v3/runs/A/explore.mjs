import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const source = await readFile(new URL('./pond.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();
const session = runtime.open(source);

const steps = JSON.parse(process.argv[2] ?? '[]');
const show = (label) => {
  const s = session.snapshot();
  console.log(label, JSON.stringify(s.bindings.hud));
};
show('initial');
for (const [event, payload] of steps) {
  const outcome = session.dispatch(event, payload ?? {});
  if (outcome.outcome === 'rejected') {
    console.log(`${event} ${JSON.stringify(payload ?? {})}: REJECTED ${outcome.origin}/${outcome.code} ${outcome.message}`);
  } else {
    show(`${event} ${JSON.stringify(payload ?? {})}:`);
  }
}
const s = session.snapshot();
console.log('journal:');
for (const e of s.decision_journal) console.log('  ', JSON.stringify(e));
console.log('grounds:', JSON.stringify(s.commitment_grounds));
console.log('sequence:', s.sequence);
session.close();
