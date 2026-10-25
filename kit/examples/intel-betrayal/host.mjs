// A game's side of the intel mechanic. The game sends what happened in the
// story; the program says which of the player's plans rested on a lie, and
// each plan keeps the reasons it was made on. The story is a fixed list of
// events, saved after the player's choice and resumed in a later session.
//
//   node host.mjs [--json]
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
import { decisionHistory } from 'caveat-lang/session';

const here = path.dirname(fileURLToPath(import.meta.url));

// The view fields this game reads after each event; the rest is not built.
const FIELDS = ['bindings', 'commitments', 'decision_series', 'decision_journal'];

export function playIntel({ runtime, source }) {
  const opened = [];
  const lines = [];
  let session = runtime.open(source);
  opened.push(session);
  let view = session.view();
  const send = (line, event, payload) => {
    lines.push(line);
    const outcome = session.dispatchView(event, payload, { fields: FIELDS });
    if (outcome.outcome !== 'accepted') {
      lines.push(`  (${outcome.message})`);
      return outcome;
    }
    view = outcome.view;
    return outcome;
  };
  try {
    send('Ocelot radios: the north gate is clear.', 'intel', { informant: 'ocelot', gate: 'north', says: 'clear' });
    send('Eva radios: the south gate is guarded.', 'intel', { informant: 'eva', gate: 'south', says: 'guarded' });
    send('You try the south gate.', 'go', { gate: 'south' });
    send('You go north.', 'go', { gate: 'north' });
    const save = session.save({ compact: true, positional: true });
    lines.push(`  [saved: ${save.length} bytes]`);

    // A later play session resumes the save with the same source.
    session = runtime.restore(source, save);
    opened.push(session);
    send('Ocelot was working for the other side.', 'betray', { informant: 'ocelot' });
    send('You confront Ocelot again.', 'betray', { informant: 'ocelot' });

    // The debrief: each plan's grounds, named by what the evidence is about.
    const sources = Object.fromEntries(session.snapshot().symbols
      .filter(symbol => symbol.kind === 'evidence').map(symbol => [symbol.name, symbol.source]));
    const history = decisionHistory(view, 'north_plan').map(revision => ({
      revision: revision.revision,
      status: revision.status,
      made_on: revision.committed.because.map(name => sources[name] ?? name),
      reopened_by: revision.reopened?.because.map(name => sources[name] ?? name) ?? [],
    }));
    return { saveBytes: save.length, lines, plan: view.bindings.north.plan, history };
  } finally {
    for (const item of opened) item.close();
  }
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const runtime = await loadRuntimeFromDirectory();
  const result = playIntel({ runtime, source: await readFile(path.join(here, 'intel.cav'), 'utf8') });
  if (process.argv.includes('--json')) {
    console.log(JSON.stringify(result, null, 2));
  } else {
    console.log(result.lines.join('\n'));
    console.log(`\nThe north plan: ${result.plan}`);
    for (const revision of result.history) {
      console.log(`  ${revision.revision} (${revision.status}) was made on: ${revision.made_on.join('; ')}`);
      if (revision.reopened_by.length) console.log(`    and reopened by: ${revision.reopened_by.join('; ')}`);
    }
  }
}
