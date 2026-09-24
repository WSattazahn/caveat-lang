// Load pond.cav and print what happens after each event.
// Usage: node explore.mjs measure:12 measure:15 crack decide ...
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const source = await readFile(new URL('./pond.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();
const session = runtime.open(source);

const show = (label) => {
  const s = session.snapshot();
  console.log(`${label} seq=${s.sequence} hud=${JSON.stringify(s.bindings.hud)}`);
};
show('initial');
for (const arg of process.argv.slice(2)) {
  const [event, value] = arg.split(':');
  const payload = value === undefined ? {} : { cm: Number(value) };
  const outcome = session.dispatch(event, payload);
  if (outcome.outcome === 'rejected') {
    console.log(`${arg}: REJECTED ${outcome.origin}/${outcome.code} "${outcome.message}"`);
    continue;
  }
  show(arg);
}
const s = session.snapshot();
console.log('commitment_grounds', JSON.stringify(s.commitment_grounds));
console.log('decision_journal');
for (const e of s.decision_journal) console.log('  ', JSON.stringify(e));
console.log('relations', JSON.stringify(s.relations));
session.close();
