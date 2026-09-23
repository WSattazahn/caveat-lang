#!/usr/bin/env node
// caveat test [--runtime <dir>] [--json] <file.scenarios.json>...
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { defaultRuntimeDirectory, loadRuntimeFromDirectory } from '../lib/node.mjs';
import { formatFileReport, parseScenarioFile, report, runScenarioFile } from '../lib/scenarios.mjs';

const USAGE = `Usage:
  caveat test [--runtime <dir>] [--json] <file.scenarios.json>...

Runs Caveat scenario files (spec/caveat-scenarios-0.1.md).
  --runtime <dir>  directory holding caveat_runtime.js and caveat_runtime_bg.wasm
  --json           print the machine report instead of text

Exit status: 0 every scenario passed, 1 a scenario failed, 2 a file was invalid
or could not be run (nothing runs).`;

function parseArguments(argv) {
  const [command, ...rest] = argv;
  if (!command || command === 'help' || command === '--help' || command === '-h') return { command: 'help' };
  if (command !== 'test') return { error: `unknown command ${command}` };
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
  if (!options.files.length) return { error: 'name at least one scenario file' };
  return options;
}

async function main() {
  const options = parseArguments(process.argv.slice(2));
  if (options.command === 'help') { console.log(USAGE); return 0; }
  if (options.error) { console.error(`${options.error}\n\n${USAGE}`); return 2; }

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
