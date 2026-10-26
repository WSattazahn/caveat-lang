// Exercise 3: keep the session across restarts. Run it twice.
import { readFile, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const runtime = await loadRuntimeFromDirectory();
const source = await readFile('support.cav', 'utf8');

if (!existsSync('session.save')) {
  const session = runtime.open(source);
  for (const [event, payload] of [['order_lookup', { total: 120 }], ['carrier_lookup', { days: 9 }], ['issue_refund', {}]]) {
    session.dispatch(event, payload);
  }
  await writeFile('session.save', session.save({ compact: true }));  // save() returns a string
  console.log('Saved after the refund. Run me again.');
} else {
  const session = runtime.restore(source, await readFile('session.save', 'utf8'));
  console.log('Restored. Refund again:', session.dispatch('issue_refund', {}).message);
}
