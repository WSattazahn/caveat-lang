#!/usr/bin/env node
// caveat test [--runtime <dir>] [--json] <file.scenarios.json>...
// caveat explain [--runtime <dir>] [--json] <program.cav> [<events.jsonl>]
// caveat dependents [--runtime <dir>] [--json] <program.cav> <name> [<events.jsonl>]
// caveat validate [--runtime <dir>] [--json] <program.cav>
// caveat check [--runtime <dir>] [--json] [--strict] <program.cav>
// caveat types [--runtime <dir>] [--json] [--from <module>] <program.cav>
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
import { CHECK_SCHEMA, formatCheck } from '../lib/check.mjs';
import { declarations } from '../lib/types.mjs';
import { doctor, formatDoctor } from '../lib/doctor.mjs';
import { runAgentDemo, formatAgentDemo } from '../lib/demo.mjs';
import { serveMcp } from '../lib/mcp.mjs';
import { describeValidation } from '../lib/authoring.mjs';

const USAGE = `CAVEAT Language — Programs that remember why.

Usage:
  caveat-lang test [--runtime <dir>] [--json] <file.scenarios.json>...
  caveat-lang explain [--runtime <dir>] [--json] <program.cav> [<events.jsonl>]
  caveat-lang dependents [--runtime <dir>] [--json] <program.cav> <name> [<events.jsonl>]
  caveat-lang validate [--runtime <dir>] [--json] <program.cav>
  caveat-lang check [--runtime <dir>] [--json] [--strict] <program.cav>
  caveat-lang types [--runtime <dir>] [--json] [--from <module>] <program.cav>
  caveat-lang replay [--runtime <dir>] <program.cav> <events.jsonl>
  caveat-lang serve [--runtime <dir>] <program.cav>
  caveat-lang init [<directory>]
  caveat-lang doctor [--runtime <dir>] [--json]
  caveat-lang demo agent [--runtime <dir>] [--json]
  caveat-lang mcp [--runtime <dir>]
  caveat-lang --version

Both caveat-lang and caveat invoke this CLI. Prefer caveat-lang when other
packages also install a command named caveat.

test runs Caveat scenario files (spec/caveat-scenarios-0.1.md). It first
runs check on each program they name and prints any warnings with the run;
warnings do not stop it.
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

check loads a program and reports patterns worth a second look, each with a
code, a line and a suggestion (spec/caveat-check-0.1.md). Warnings are advice,
not errors. A comment "# caveat check: allow CODE" on the line above silences
one where the pattern is intended. Exit status: 0 it loads, whatever it found;
1 with --strict when there is a warning; 2 it does not load.

types prints TypeScript declarations for a program: each event's payload, the
displayed values and their types, and its state, cue, decision, stream,
evidence, caveat and claim names, read from its interface
(spec/caveat-interface-0.1.md). With --json it prints the interface itself.
Save the output as a .d.ts file beside the host code. Exit status: 0 printed,
2 the program does not load.

replay sends the events and prints one JSON record per line: the initial
snapshot, then each event's outcome, with the snapshot after each accepted
event. Exit status: 0 replayed, 1 an event failed fatally (its record is last),
2 the program or events file could not be used.

serve keeps one session of the program open and answers requests on standard
input, one JSON object per line, with one JSON object per line on standard
output (spec/caveat-serve-0.1.md). Exit status: 0 closed, 1 a fatal outcome
ended the session, 2 the program does not load.

doctor checks Node, package/runtime identity, a real test session and PATH
command ownership. It inspects other executables without running them.
  Exit status: 0 no failed check (warnings may remain), 1 a check failed.

demo agent runs a fixed evidence/correction/reassessment example in memory.
Its illustrative inputs are supplied, not independently verified. Output shows
actual decision grounds and preserved history. It writes no files.
  Exit status: 0 demonstrated, 1 demonstration failed, 2 runtime unavailable.

mcp serves five authoring tools over stdio using MCP 2026-07-28 or 2025-11-25.
It takes inline source, runs each call in a fresh subprocess, and exposes no
persistent session or file access. stdout contains protocol messages only.

init writes the getting-started program, its scenarios and an events file
into a directory (default: the current one). It refuses to overwrite a file.

  --runtime <dir>  directory holding caveat_runtime.js and caveat_runtime_bg.wasm
  --json           print the machine report instead of text
  --strict         (check) exit 1 when there is a warning
  --from <module>  (types) where the declarations import TypedSession from
                   (default caveat-lang/types)`;

// The positional arguments each command takes, as [fewest, most].
const COMMANDS = {
  test: [1, Infinity, 'name at least one scenario file'],
  explain: [1, 2, 'name a program and, optionally, an events file'],
  dependents: [2, 3, 'name a program, the evidence, stream or caveat to ask about and, optionally, an events file'],
  validate: [1, 1, 'name one program'],
  check: [1, 1, 'name one program'],
  types: [1, 1, 'name one program'],
  replay: [2, 2, 'name a program and an events file'],
  serve: [1, 1, 'name one program'],
  init: [0, 1, 'name at most one directory'],
  doctor: [0, 0, 'doctor takes no positional arguments'],
  mcp: [0, 0, 'mcp takes no positional arguments'],
  demo: [1, 1, 'name the demonstration: agent'],
};

function parseArguments(argv) {
  const [command, ...rest] = argv;
  if ((command === '--version' || command === '-v' || command === 'version') && rest.length === 0) return { command: 'version' };
  if (!command || command === 'help' || command === '--help' || command === '-h') return { command: 'help' };
  if (!Object.hasOwn(COMMANDS, command)) return { error: `unknown command ${command}` };
  const options = { command, files: [], json: false, strict: false, runtime: null, from: null };
  for (let index = 0; index < rest.length; index++) {
    const argument = rest[index];
    if (argument === '--json' && !['replay', 'serve', 'init', 'mcp'].includes(command)) options.json = true;
    else if (argument === '--strict' && command === 'check') options.strict = true;
    else if (argument === '--from' && command === 'types') {
      options.from = rest[++index];
      if (!options.from) return { error: '--from needs a module specifier' };
    }
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
  const result = describeValidation(snapshot, program);
  if (options.json) { console.log(JSON.stringify(result, null, 2)); return 0; }
  const joined = items => (items.length ? items.join(', ') : 'none');
  console.log([
    `${path.basename(program)} loads.`,
    `  events: ${joined(result.events.map(event => (event.parameters?.length
      ? `${event.name} (${event.parameters.map(describeParameter).join(', ')})` : event.name)))}`,
    `  reading streams: ${joined(Object.entries(result.reading_streams).map(([name, stream]) => `${name} from ${stream.from}, ${stream.window ? 'window' : 'limit'} ${stream.limit}`))}`,
    `  decision series: ${joined(Object.entries(result.decision_series).map(([name, series]) => `${name}, limit ${series.limit}`))}`,
    `  displayed: ${joined(result.displayed)}`,
  ].join('\n'));
  return 0;
}

async function checkProgram(options) {
  const [program] = options.files;
  const inputs = await readInputs(program);
  if (!inputs) return 2;
  const refuse = message => {
    if (options.json) console.log(JSON.stringify({ schema: CHECK_SCHEMA, program, loads: false, error: message }, null, 2));
    else console.error(`${program} ${message}`);
    return 2;
  };
  if (inputs.source.startsWith('#caveat-bundle')) return refuse('is a bundle; check reads a single-file program');
  const runtime = await loadRuntime(options);
  if (!runtime) return 2;
  let report;
  try { report = runtime.check(inputs.source); } catch (error) { return refuse(`does not load: ${error.message}`); }
  if (options.json) {
    const { schema, diagnostics, suppressed } = report;
    console.log(JSON.stringify({ schema, program, loads: true, strict: options.strict, diagnostics, suppressed }, null, 2));
  } else console.log(formatCheck(report, path.basename(program)));
  return options.strict && report.diagnostics.length ? 1 : 0;
}

async function typesOf(options) {
  const [program] = options.files;
  const inputs = await readInputs(program);
  if (!inputs) return 2;
  const runtime = await loadRuntime(options);
  if (!runtime) return 2;
  let programInterface;
  try { programInterface = runtime.interface(inputs.source); } catch (error) {
    console.error(`${program} does not load: ${error.message}`);
    return 2;
  }
  if (options.json) console.log(JSON.stringify(programInterface, null, 2));
  else process.stdout.write(declarations(programInterface, { program: path.basename(program), ...(options.from ? { from: options.from } : {}) }));
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
    '  npx --no-install caveat-lang test umbrella.scenarios.json',
    '  npx --no-install caveat-lang explain umbrella.cav events.jsonl',
    '  npx --no-install caveat-lang dependents umbrella.cav sky events.jsonl',
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

  // Check each program before the first scenario (spec/caveat-scenarios-0.1.md):
  // the report prints with the run, and an error stops it; warnings run on.
  // A program that cannot be read or does not load is left to its scenarios.
  let stopped = false;
  for (const entry of files) {
    entry.checks = {};
    const names = new Set([entry.doc.source, ...entry.doc.scenarios.map(scenario => scenario.source).filter(Boolean)]);
    for (const name of names) {
      let checked;
      try {
        const source = await readFile(path.resolve(path.dirname(entry.file), name), 'utf8');
        if (source.startsWith('#caveat-bundle')) continue;
        checked = runtime.check(source);
      } catch { continue; }
      const { diagnostics, suppressed } = checked;
      entry.checks[name] = { diagnostics, suppressed };
      if (!options.json && (diagnostics.length || suppressed.length)) console.log(formatCheck(checked, name));
      if (diagnostics.some(diagnostic => diagnostic.severity === 'error')) stopped = true;
    }
  }
  if (stopped) {
    const unrun = files.map(({ file, checks }) => ({ file, sources: {}, check: checks, scenarios: [], passed: 0, failed: 0 }));
    if (options.json) console.log(JSON.stringify(report(unrun, runtime.identity), null, 2));
    console.error('check reported an error; no scenario ran');
    return 2;
  }

  const reports = [];
  for (const { file, doc, checks } of files) {
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
    result.check = checks;
    reports.push(result);
    if (!options.json) console.log(formatFileReport(result));
  }
  const summary = report(reports, runtime.identity);
  if (options.json) console.log(JSON.stringify(summary, null, 2));
  return summary.failed ? 1 : 0;
}

async function diagnose(options) {
  const result = await doctor({ ...(options.runtime ? { runtimeDirectory: options.runtime } : {}) });
  console.log(options.json ? JSON.stringify(result, null, 2) : formatDoctor(result));
  return result.ok ? 0 : 1;
}

async function demonstrate(options) {
  if (options.files[0] !== 'agent') { console.error('unknown demonstration; use demo agent'); return 2; }
  const runtime = await loadRuntime(options);
  if (!runtime) return 2;
  try {
    const result = await runAgentDemo(runtime);
    console.log(options.json ? JSON.stringify(result, null, 2) : formatAgentDemo(result));
    return 0;
  } catch (error) {
    if (options.json) console.log(JSON.stringify({ schema: 'caveat-demo/0.1', demo: 'agent', error: error.message }));
    else console.error(`agent demonstration failed: ${error.message}`);
    return 1;
  }
}

async function main() {
  const options = parseArguments(process.argv.slice(2));
  if (options.command === 'help') { console.log(USAGE); return 0; }
  if (options.command === 'version') {
    const manifest = JSON.parse(await readFile(new URL('../package.json', import.meta.url), 'utf8'));
    console.log(`CAVEAT Language ${manifest.version}`);
    return 0;
  }
  if (options.error) { console.error(`${options.error}\n\n${USAGE}`); return 2; }
  const run = {
    test: testScenarios, explain: explainProgram, dependents: dependentsOf,
    validate: validateProgram, check: checkProgram, types: typesOf, replay: replayProgram, serve: serveProgram, init, doctor: diagnose, demo: demonstrate,
    mcp: options => serveMcp({ runtimeDirectory: options.runtime ?? defaultRuntimeDirectory() }),
  }[options.command];
  return run(options);
}

process.exitCode = await main();
