// A tiny support agent. It looks things up with its tools, asks Caveat before
// each action, and acts only when Caveat records the decision.
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
import { decisionHistory } from 'caveat-lang/session';

// Stand-ins for real tools. The carrier API is wrong about this package.
const tools = {
  orderDatabase: () => ({ total: 120 }),
  carrierApi: () => ({ days: 9 }),       // "no scan for 9 days": looks lost
  carrierRecheck: () => ({ days: 1 }),   // a second lookup: scanned yesterday
};

const runtime = await loadRuntimeFromDirectory();
const session = runtime.open(await readFile('support.cav', 'utf8'));

function send(event, payload = {}) {
  const result = session.dispatchView(event, payload, { fields: ['bindings'] });
  const note = result.outcome === 'accepted' ? 'ok' : `refused: ${result.message}`;
  console.log(`  ${event.padEnd(18)} ${note}`);
  return result.outcome === 'accepted';
}

console.log('The agent works the ticket:');
send('issue_refund');                                  // too early: refused
send('order_lookup', tools.orderDatabase());
send('carrier_lookup', tools.carrierApi());
send('customer_message');
for (const action of ['issue_refund', 'file_claim', 'send_credit']) {
  if (send(action)) { /* here a real agent would call the payments or carrier API */ }
}

console.log('\nA second carrier lookup disagrees:', tools.carrierRecheck());
send('carrier_correction', tools.carrierRecheck());
send('issue_refund');                                  // now refused

console.log('\nUndo list:');
for (const action of ['refund', 'carrier_claim', 'apology_credit']) {
  const [first] = decisionHistory(session.view(), action);
  const verdict = first.status === 'reopened' ? 'UNDO' : 'keep';
  console.log(`  ${verdict.padEnd(4)} ${action.padEnd(15)} taken at #${first.committed.sequence} because ${first.committed.because.join(', ')}`
    + (first.reopened ? `; reopened at #${first.reopened.sequence} because ${first.reopened.because.join(', ')}` : ''));
}
