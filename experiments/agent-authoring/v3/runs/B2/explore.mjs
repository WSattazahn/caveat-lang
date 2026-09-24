// Explore a pond program: node explore.mjs <file.cav> <event:payloadJSON>...
// Example: node explore.mjs pond.cav measure:'{"cm":12}' decide crack
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const [file, ...steps] = process.argv.slice(2);
const source = await readFile(file, 'utf8');
const runtime = await loadRuntimeFromDirectory();
const session = runtime.open(source);

const show = (label) => {
  const s = session.snapshot();
  console.log(`== ${label}  seq=${s.sequence}`);
  console.log('  hud:', JSON.stringify(s.bindings.hud));
  console.log('  grounds:', JSON.stringify(s.commitment_grounds));
  for (const e of s.decision_journal) {
    console.log(`  journal: ${e.commitment} ${e.change} value=${e.value} because=${e.because.join(',')} caveats=${e.caveats.join(',')}`);
  }
};

show('initial');
for (const step of steps) {
  const i = step.indexOf(':');
  const event = i < 0 ? step : step.slice(0, i);
  const payload = i < 0 ? {} : JSON.parse(step.slice(i + 1));
  const out = session.dispatch(event, payload);
  if (out.outcome === 'rejected') {
    console.log(`== ${step}: REJECTED ${out.origin}/${out.code} ${out.message}`);
  } else {
    show(step);
  }
}
if (process.env.FULL) console.log(JSON.stringify(session.snapshot(), null, 1));
session.close();
