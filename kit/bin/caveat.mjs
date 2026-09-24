#!/usr/bin/env node
// caveat test [--runtime <dir>] [--json] <file.scenarios.json>...
// caveat explain [--runtime <dir>] [--json] <program.cav> [<events.jsonl>]
// caveat dependents [--runtime <dir>] [--json] <program.cav> <name> [<events.jsonl>]
// caveat validate [--runtime <dir>] [--json] <program.cav>
// caveat replay [--runtime <dir>] <program.cav> <events.jsonl>
// caveat serve [--runtime <dir>] <program.cav>
// caveat init [<directory>]
import { existsSync } from 'node:fs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import readline from 'node:readline';
import { fileURLToPath } from 'node:url';
import { defaultRuntimeDirectory, loadRuntimeFromDirectory } from '../lib/node.mjs';
import { formatFileReport, parseScenarioFile, report, runScenarioFile } from '../lib/scenarios.mjs';
import { dependents, explain, formatDependents, formatExplanation, parseEvents } from '../lib/explain.mjs';
import { createServer } from '../lib/serve.mjs';

const USAGE = `Usage:
  caveat test [--runtime <dir>] [--json] <file.scenarios.json>...
  caveat explain [--runtime <dir>] [--json] <program.cav> [<events.jsonl>]
  caveat dependents [--runtime <dir>] [--json] <program.cav> <name> [<events.jsonl>]
  caveat validate [--runtime <dir>] [--json] <program.cav>
  caveat replay [--runtime <dir>] <program.cav> <events.jsonl>
  caveat serve [--runtime <dir>] <program.cav>
  caveat init [<directory>]

test runs Caveat scenario files (spec/caveat-scenarios-0.1.md).
  Exit status: 0 every scenario passed, 1 a scenario failed, 2 a file was
  invalid or could not be run (nothing runs).

explain sends the events, one JSON object per line such as
{"event": "read", "payload": {"value": 17}}, then shows each decision with what
it rests on and its history, the evidence and its caveats, and why each
displayed value is what it is. Refused events are listed and change nothing.
  Exit status: 0 explained, 1 an event failed fatally (the explanation is of
  the session before it), 2 the program or events file could not be used.

dependents sends the events the same way, then shows what rests on one piece
of evidence, reading stream or caveat: the decisions and values based on it or
that it could have influenced, the decision changes it caused, and the
displayed values that cite it. Exit status as for explain; 2 also when the
program declares no such name.

validate loads a program and lists its events, histories and displayed values.
  Exit status: 0 it loads, 2 it does not.

replay sends the events and prints one JSON record per line: the initial
snapshot, then each event's outcome, with the snapshot after each accepted
event. Exit status: 0 replayed, 1 an event failed fatally (its record is last),
2 the program or events file could not be used.

serve keeps one session of the program open and answers requests on standard
input, one JSON object per line, with one JSON object per line on standard
output (spec/caveat-serve-0.1.md). Exit status: 0 closed, 1 a fatal outcome
ended the session, 2 the program does not load.

init writes the getting-started program, its scenarios and an events file
into a directory (default: the current one). It refuses to overwrite a file.

  --runtime <dir>  directory holding caveat_runtime.js and caveat_runtime_bg.wasm
  --json           print the machine report instead of text`;

// The positional arguments each command takes, as [fewest, most].
const COMMANDS = {
  test: [1, Infinity, 'name at least one scenario file'],
  explain: [1, 2, 'name a program and, optionally, an events file'],
  dependents: [2, 3, 'name a program, the evidence, stream or caveat to ask about and, optionally, an events file'],
  validate: [1, 1, 'name one program'],
  replay: [2, 2, 'name a program and an events file'],
  serve: [1, 1, 'name one program'],
  init: [0, 1, 'name at most one directory'],
};

function parseArguments(argv) {
  const [command, ...rest] = argv;
  if (!command || command === 'help' || command === '--help' || command === '-h') return { command: 'help' };
  if (!Object.hasOwn(COMMANDS, command)) return { error: `unknown command ${command}` };
  const options = { command, files: [], json: false, runtime: null };
  for (let index = 0; index < rest.length; index++) {
    const argument = rest[index];
    if (argument === '--json' && !['replay', 'serve', 'init'].includes(command)) options.json = true;
    else if (argument === '--runtime' && command !== 'init') {
      options.runtime = rest[++index];
      if (!options.runtime) return { error: '--runtime needs a directory' };
    } else if (argument.startsWith('--')) return { error: `unknown option ${argument} for ${command}` };
    else options.files.push(argument);
  }
  const [fewest, most, message] = COMMANDS[command];
  if (options.files.length < fewest || options.files.length > most) return { error: message };
  return options;
}

async function loadRuntime(options) {
  try { return await loadRuntimeFromDirectory(options.runtime ?? defaultRuntimeDirectory()); } catch (error) {
    console.error(`cannot load the Caveat runtime: ${error.message}`);
    return null;
  }
}

// Reads the program and, when named, the events file. Null after reporting
// why either cannot be used.
async function readInputs(program, eventsFile) {
  let source;
  try { source = await readFile(program, 'utf8'); } catch (error) { console.error(`cannot read ${program}: ${error.message}`); return null; }
  let events = [];
  if (eventsFile) {
    try {
      const text = await readFile(eventsFile, 'utf8');
      events = parseEvents(text);
      // parseEvents skips blank lines; keep each event's line in the file.
      const lines = text.split(/\r?\n/).flatMap((line, index) => (line.trim() ? [index + 1] : []));
      events.forEach((event, index) => { event.line = lines[index]; });
    } catch (error) { console.error(`INVALID ${eventsFile}: ${error.message}`); return null; }
  }
  return { source, events };
}

// Sends the events in order. A fatal outcome ends the session, and the result
// describes the session before it.
function sendAll(session, events, each = () => {}) {
  const sent = [];
  let snapshot = session.snapshot();
  for (const { event, payload, line } of events) {
    try {
      const { snapshot: after, ...outcome } = session.dispatch(event, payload);
      sent.push({ event, payload, outcome });
      if (outcome.outcome === 'accepted') snapshot = after;
      each({ event, payload, line, outcome, snapshot: outcome.outcome === 'accepted' ? after : null });
    } catch (error) {
      const outcome = { outcome: 'fatal', kind: error.kind ?? null, message: error.message };
      sent.push({ event, payload, outcome });
      each({ event, payload, line, outcome, snapshot: null });
      return { sent, snapshot, failed: true };
    }
  }
  session.close();
  return { sent, snapshot, failed: false };
}

async function openProgram(options, program, eventsFile) {
  const inputs = await readInputs(program, eventsFile);
  if (!inputs) return null;
  const runtime = await loadRuntime(options);
  if (!runtime) return null;
  try { return { ...inputs, runtime, session: runtime.open(inputs.source) }; } catch (error) {
    console.error(`${program} does not load: ${error.message}`);
    return null;
  }
}

async function explainProgram(options) {
  const [program, eventsFile] = options.files;
  const opened = await openProgram(options, program, eventsFile);
  if (!opened) return 2;
  const { sent, snapshot, failed } = sendAll(opened.session, opened.events);
  const result = explain(snapshot, sent);
  if (options.json) console.log(JSON.stringify({ program, ...result }, null, 2));
  else {
    console.log(formatExplanation(result, path.basename(program)));
    if (failed) console.log(`\nEvent ${sent.length} failed; the explanation above is of the session before it.`);
  }
  return failed ? 1 : 0;
}

async function dependentsOf(options) {
  const [program, subject, eventsFile] = options.files;
  const opened = await openProgram(options, program, eventsFile);
  if (!opened) return 2;
  const { sent, snapshot, failed } = sendAll(opened.session, opened.events);
  let result;
  try { result = dependents(snapshot, subject); } catch (error) { console.error(error.message); return 2; }
  if (options.json) console.log(JSON.stringify({ program, events: sent, ...result }, null, 2));
  else {
    console.log(formatDependents(result, path.basename(program), sent.length));
    if (failed) console.log(`\nEvent ${sent.length} failed; the answer above is about the session before it.`);
  }
  return failed ? 1 : 0;
}

// As the source declares it: `target kind mushroom`, `sort in glowcap
// duskcap` (spec/caveat-typed-parameters-0.1.md), `commit id`
// (spec/caveat-identifiers-0.1.md) or `value 0..100`.
function describeParameter(parameter) {
  const { entity, member, identifier } = parameter.domain ?? {};
  if (entity) return `${parameter.name} kind ${entity.kind}`;
  if (member) return `${parameter.name} in ${member.members.join(' ')}`;
  if (identifier) return `${parameter.name} id`;
  return `${parameter.name} ${parameter.min}..${parameter.max}`;
}

async function validateProgram(options) {
  const [program] = options.files;
  const inputs = await readInputs(program);
  if (!inputs) return 2;
  const runtime = await loadRuntime(options);
  if (!runtime) return 2;
  let session;
  try { session = runtime.open(inputs.source); } catch (error) {
    if (options.json) console.log(JSON.stringify({ schema: 'caveat-validate/0.1', program, loads: false, error: error.message }, null, 2));
    else console.error(`${program} does not load: ${error.message}`);
    return 2;
  }
  const snapshot = session.snapshot();
  session.close();
  const result = {
    schema: 'caveat-validate/0.1', program, loads: true,
    events: snapshot.events ?? [],
    reading_streams: Object.fromEntries(Object.entries(snapshot.reading_streams ?? {})
      .map(([name, stream]) => [name, { from: stream.template, limit: stream.limit }])),
    decision_series: Object.fromEntries(Object.entries(snapshot.decision_series ?? {})
      .map(([name, series]) => [name, { limit: series.limit }])),
    displayed: Object.entries(snapshot.bindings ?? {}).flatMap(([target, properties]) =>
      Object.keys(properties).map(property => `${target}.${property}`)),
  };
  if (options.json) { console.log(JSON.stringify(result, null, 2)); return 0; }
  const joined = items => (items.length ? items.join(', ') : 'none');
  console.log([
    `${path.basename(program)} loads.`,
    `  events: ${joined(result.events.map(event => (event.parameters?.length
      ? `${event.name} (${event.parameters.map(describeParameter).join(', ')})` : event.name)))}`,
    `  reading streams: ${joined(Object.entries(result.reading_streams).map(([name, stream]) => `${name} from ${stream.from}, limit ${stream.limit}`))}`,
    `  decision series: ${joined(Object.entries(result.decision_series).map(([name, series]) => `${name}, limit ${series.limit}`))}`,
    `  displayed: ${joined(result.displayed)}`,
  ].join('\n'));
  return 0;
}

async function replayProgram(options) {
  const [program, eventsFile] = options.files;
  const opened = await openProgram(options, program, eventsFile);
  if (!opened) return 2;
  const write = record => console.log(JSON.stringify(record));
  const initial = opened.session.snapshot();
  write({ record: 'initial', sequence: initial.sequence, snapshot: initial });
  // A refused event leaves the sequence where it was.
  let sequence = initial.sequence;
  const { failed } = sendAll(opened.session, opened.events, ({ event, payload, line, outcome, snapshot }) => {
    if (outcome.outcome === 'fatal') { write({ record: 'fatal', line, event, payload, kind: outcome.kind, message: outcome.message }); return; }
    const { schema: _schema, ...result } = outcome;
    if (snapshot) sequence = snapshot.sequence;
    write({ record: 'event', line, event, payload, ...result, sequence, ...(snapshot ? { snapshot } : {}) });
  });
  return failed ? 1 : 0;
}

async function serveProgram(options) {
  const [program] = options.files;
  const write = value => process.stdout.write(`${JSON.stringify(value)}\n`);
  const inputs = await readInputs(program);
  if (!inputs) return 2;
  const runtime = await loadRuntime(options);
  if (!runtime) return 2;
  let server;
  try { server = createServer({ runtime, source: inputs.source, program: path.basename(program) }); } catch (error) {
    write({ schema: 'caveat-serve/0.1', ready: false, error: { kind: 'load', message: error.message } });
    return 2;
  }
  write(server.ready);
  const lines = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
  let status = 0;
  let stopped = false;
  for await (const line of lines) {
    if (!line.trim()) continue;
    const { response, exit } = server.handle(line);
    write(response);
    if (typeof exit === 'number') { status = exit; stopped = true; break; }
  }
  if (!stopped) server.close();
  process.stdin.destroy();
  return status;
}

const TEMPLATES = ['umbrella.cav', 'umbrella.scenarios.json', 'events.jsonl'];

async function init(options) {
  const directory = options.files[0] ?? '.';
  const existing = TEMPLATES.filter(name => existsSync(path.join(directory, name)));
  if (existing.length) {
    console.error(`${existing.map(name => path.join(directory, name)).join(', ')} already exist${existing.length === 1 ? 's' : ''}; nothing written`);
    return 2;
  }
  const templates = fileURLToPath(new URL('../templates/', import.meta.url));
  await mkdir(directory, { recursive: true });
  for (const name of TEMPLATES) await writeFile(path.join(directory, name), await readFile(path.join(templates, name)));
  const where = directory === '.' ? '' : ` in ${directory}`;
  console.log([
    `Wrote ${TEMPLATES.slice(0, -1).join(', ')} and ${TEMPLATES.at(-1)}${where}, the program from docs/GETTING_STARTED.md.`,
    'Next:',
    '  npx --no-install caveat test umbrella.scenarios.json',
    '  npx --no-install caveat explain umbrella.cav events.jsonl',
    '  npx --no-install caveat dependents umbrella.cav sky events.jsonl',
  ].join('\n'));
  return 0;
}

async function testScenarios(options) {
  const files = [];
  let invalid = false;
  for (const file of options.files) {
    try {
      files.push({ file, doc: parseScenarioFile(await readFile(file, 'utf8')) });
    } catch (error) {
      invalid = true;
      console.error(`INVALID ${file}: ${error.message}`);
    }
  }
  if (invalid) return 2;

  const directory = options.runtime ?? defaultRuntimeDirectory();
  let runtime = await loadRuntime(options);
  if (!runtime) return 2;

  const reports = [];
  for (const { file, doc } of files) {
    const hashes = {};
    const readSource = async relative => {
      const text = await readFile(path.resolve(path.dirname(file), relative), 'utf8');
      hashes[relative] = createHash('sha256').update(text).digest('hex');
      return text;
    };
    const result = await runScenarioFile(doc, {
      runtime, file, readSource,
      reload: async () => { runtime = await loadRuntimeFromDirectory(directory); return runtime; },
    });
    result.sources = hashes;
    reports.push(result);
    if (!options.json) console.log(formatFileReport(result));
  }
  const summary = report(reports, runtime.identity);
  if (options.json) console.log(JSON.stringify(summary, null, 2));
  return summary.failed ? 1 : 0;
}

async function main() {
  const options = parseArguments(process.argv.slice(2));
  if (options.command === 'help') { console.log(USAGE); return 0; }
  if (options.error) { console.error(`${options.error}\n\n${USAGE}`); return 2; }
  const run = {
    test: testScenarios, explain: explainProgram, dependents: dependentsOf,
    validate: validateProgram, replay: replayProgram, serve: serveProgram, init,
  }[options.command];
  return run(options);
}

process.exitCode = await main();
