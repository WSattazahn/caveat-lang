// A main agent hands work to two sub-agents, with the ledger as the save.
// Each agent's session is scoped to its own events; the two sub-agents'
// work is merged by replay, and what no longer applies is reported. No model
// is called: each agent is a fixed list of events.
//
//   node host.mjs [--json]
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { explain, formatExplanation } from 'caveat-lang/explain';
import { replayOnto, scopeSession } from 'caveat-lang/handoff';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const here = path.dirname(fileURLToPath(import.meta.url));

const outcomeOf = outcome => (outcome.outcome === 'accepted' ? 'accepted' : `${outcome.origin}/${outcome.code}`);

export function runHandoff({ runtime, source }) {
  const names = runtime.interface(source).events.map(item => item.name);
  const mainEvents = names.filter(name => name.startsWith('main_'));
  const subEvents = names.filter(name => name.startsWith('sub_'));
  const opened = [];
  const open = saved => { const session = saved === undefined ? runtime.open(source) : runtime.restore(source, saved); opened.push(session); return session; };
  const steps = [];
  const send = (agent, session, event, payload = {}) => {
    const outcome = session.dispatch(event, payload);
    steps.push({ agent, event, outcome: outcomeOf(outcome), message: outcome.message ?? null });
    return outcome;
  };
  try {
    // 1. The main agent plans, closes off renames, rules out a rewrite, grants edit and hands off.
    const main = scopeSession(open(), mainEvents);
    for (const event of ['main_request', 'main_notify', 'main_rule_out_rewrite']) send('main', main, event);
    send('main', main, 'main_grant', { scope: 'edit' });
    send('main', main, 'main_handoff');
    const ledger = main.save({ compact: true });
    const report = explain(main.snapshot());
    const brief = formatExplanation(report, 'handoff.cav');

    // 2. Two sub-agents restore the ledger, each in a session scoped to sub_* events.
    const a = scopeSession(open(ledger), subEvents);
    const b = scopeSession(open(ledger), subEvents);
    send('a', a, 'sub_rename');
    send('a', a, 'main_return');
    send('a', a, 'sub_edit');
    send('a', a, 'sub_report', { result: 'pass', coverage: 'linux' });
    send('b', b, 'sub_rewrite');
    send('b', b, 'sub_edit');
    send('b', b, 'sub_object');

    // 3. The main agent merges: A's session, then B's accepted events replayed onto it.
    const merged = open(a.save());
    const replay = replayOnto(scopeSession(merged, subEvents), b.accepted);
    const back = scopeSession(merged, mainEvents);
    send('main', back, 'main_ship');
    send('main', back, 'main_return');
    send('main', back, 'main_ship_on_linux');
    const snapshot = merged.snapshot();
    return {
      ledgerBytes: ledger.length,
      foreclosed: Object.fromEntries(report.decisions.filter(item => item.foreclosed_by?.length).map(item => [item.name, item.foreclosed_by])),
      brief: brief.slice(brief.indexOf('Decisions'), brief.indexOf('Evidence')).trimEnd(),
      steps,
      replayed: replay.accepted.map(item => item.event),
      conflicts: replay.conflicts.map(item => ({ event: item.event, outcome: outcomeOf(item.outcome), message: item.outcome.message })),
      plan: snapshot.decision_journal.filter(item => item.decision === 'plan').map(item => ({ change: item.change, because: item.because })),
      ship: snapshot.commitment_grounds['ship@1'],
    };
  } finally {
    for (const session of opened) session.close();
  }
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const runtime = await loadRuntimeFromDirectory();
  const result = runHandoff({ runtime, source: await readFile(path.join(here, 'handoff.cav'), 'utf8') });
  if (process.argv.includes('--json')) {
    console.log(JSON.stringify(result, null, 2));
  } else {
    console.log(`ledger handed over: ${result.ledgerBytes} bytes\n\n${result.brief}\n`);
    for (const step of result.steps) console.log(`${step.agent.padEnd(4)} ${step.event.padEnd(20)} ${step.outcome}${step.message && step.outcome !== 'accepted' ? `: ${step.message}` : ''}`);
    console.log(`\nreplayed from b: ${result.replayed.join(', ')}`);
    for (const conflict of result.conflicts) console.log(`conflict: ${conflict.event} ${conflict.outcome}: ${conflict.message}`);
    console.log(`plan: ${result.plan.map(item => `${item.change} (${item.because.join(', ')})`).join('; ')}`);
    console.log(`ship@1 rests on ${result.ship.evidence.join(', ')}; caveats ${result.ship.caveats.join(', ')}`);
  }
}
