#!/usr/bin/env node
// caveat test [--runtime <dir>] [--json] <file.scenarios.json>...
// caveat explain [--runtime <dir>] [--json] <program.cav> [<events.jsonl>]
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { defaultRuntimeDirectory, loadRuntimeFromDirectory } from '../lib/node.mjs';
import { formatFileReport, parseScenarioFile, report, runScenarioFile } from '../lib/scenarios.mjs';
import { explain, formatExplanation, parseEvents } from '../lib/explain.mjs';

const USAGE = `Usage:
  caveat test [--runtime <dir>] [--json] <file.scenarios.json>...
  caveat explain [--runtime <dir>] [--json] <program.cav> [<events.jsonl>]

test runs Caveat scenario files (spec/caveat-scenarios-0.1.md).
  Exit status: 0 every scenario passed, 1 a scenario failed, 2 a file was
  invalid or could not be run (nothing runs).

explain sends the events, one JSON object per line such as
{"event": "read", "payload": {"value": 17}}, then shows each decision with what
it rests on and its history, the evidence and its caveats, and why each
displayed value is what it is. Refused events are listed and change nothing.
  Exit status: 0 explained, 1 an event failed fatally (the explanation is of
  the session before it), 2 the program or events file could not be used.

  --runtime <dir>  directory holding caveat_runtime.js and caveat_runtime_bg.wasm
  --json           print the machine report instead of text`;

function parseArguments(argv) {
  const [command, ...rest] = argv;
  if (!command || command === 'help' || command === '--help' || command === '-h') return { command: 'help' };
  if (command !== 'test' && command !== 'explain') return { error: `unknown command ${command}` };
  const options = { command, files: [], json: false, runtime: null };
  for (let index = 0; index < rest.length; index++) {
    const argument = rest[index];
    if (argument === '--json') options.json = true;
    else if (argument === '--runtime') {
      options.runtime = rest[++index];
      if (!options.runtime) return { error: '--runtime needs a directory' };
    } else if (argument.startsWith('--')) return { error: `unknown option ${argument}` };
    else options.files.push(argument);
  }
  if (command === 'explain') {
    if (options.files.length < 1 || options.files.length > 2) return { error: 'name a program and, optionally, an events file' };
  } else if (!options.files.length) return { error: 'name at least one scenario file' };
  return options;
}

async function explainProgram(options) {
  const [program, eventsFile] = options.files;
  let source;
  let events = [];
  try { source = await readFile(program, 'utf8'); } catch (error) { console.error(`cannot read ${program}: ${error.message}`); return 2; }
  if (eventsFile) {
    try { events = parseEvents(await readFile(eventsFile, 'utf8')); } catch (error) { console.error(`INVALID ${eventsFile}: ${error.message}`); return 2; }
  }
  let runtime;
  try { runtime = await loadRuntimeFromDirectory(options.runtime ?? defaultRuntimeDirectory()); } catch (error) {
    console.error(`cannot load the Caveat runtime: ${error.message}`);
    return 2;
  }
  let session;
  try { session = runtime.open(source); } catch (error) { console.error(`${program} does not load: ${error.message}`); return 2; }

  // A fatal event ends the session; the explanation is of the last good state.
  const sent = [];
  let snapshot = session.snapshot();
  let failed = false;
  for (const { event, payload } of events) {
    try {
      const { snapshot: after, ...outcome } = session.dispatch(event, payload);
      sent.push({ event, payload, outcome });
      if (outcome.outcome === 'accepted') snapshot = after;
    } catch (error) {
      sent.push({ event, payload, outcome: { outcome: 'fatal', kind: error.kind ?? null, message: error.message } });
      failed = true;
      break;
    }
  }
  if (!failed) session.close();
  const result = explain(snapshot, sent);
  if (options.json) console.log(JSON.stringify({ program, ...result }, null, 2));
  else {
    console.log(formatExplanation(result, path.basename(program)));
    if (failed) console.log(`\nEvent ${sent.length} failed; the explanation above is of the session before it.`);
  }
  return failed ? 1 : 0;
}

async function main() {
  const options = parseArguments(process.argv.slice(2));
  if (options.command === 'help') { console.log(USAGE); return 0; }
  if (options.error) { console.error(`${options.error}\n\n${USAGE}`); return 2; }
  if (options.command === 'explain') return explainProgram(options);

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
  let runtime;
  try { runtime = await loadRuntimeFromDirectory(directory); } catch (error) {
    console.error(`cannot load the Caveat runtime: ${error.message}`);
    return 2;
  }

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

process.exitCode = await main();
