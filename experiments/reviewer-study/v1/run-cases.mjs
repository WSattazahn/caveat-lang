// Runs one program through every history and case and prints, per file, each
// line's outcome and the view after it. Public at stage 1; the expected views
// are sealed, and score.mjs compares the two after phase two.
//
//   node run-cases.mjs --arm caveat --program gate.cav --modules DIR [--study DIR] [--out FILE]
//   node run-cases.mjs --arm ts --program gate.ts --modules DIR [--study DIR] [--out FILE]
//
// DIR is a node_modules directory holding the pinned caveat-lang (both arms
// read nothing else from it) or the pinned typescript.
import { mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) =>
  (index % 2 === 0 ? [...pairs, [value.replace(/^--/, ''), all[index + 1]]] : pairs), []));
for (const key of ['arm', 'program', 'modules']) if (!args[key]) throw new Error(`--${key} is required`);
const study = path.resolve(args.study ?? here);
const modules = path.resolve(args.modules);

const readLines = async file => (await readFile(file, 'utf8')).split('\n').filter(line => line.trim()).map(line => JSON.parse(line));
const files = [];
for (const dir of ['histories', 'cases']) {
  for (const name of (await readdir(path.join(study, dir))).filter(name => name.endsWith('.jsonl')).sort()) files.push(`${dir}/${name}`);
}

// The view the task requires, and nothing else.
const pick = view => ({ status: view.status, head: view.head, revisions: view.revisions, reason: view.reason });

async function caveatArm() {
  const { loadRuntimeFromDirectory, defaultRuntimeDirectory } = await import(pathToFileURL(path.join(modules, 'caveat-lang', 'lib', 'node.mjs')));
  const runtime = await loadRuntimeFromDirectory(defaultRuntimeDirectory());
  const source = await readFile(args.program, 'utf8');
  return lines => {
    const session = runtime.open(source);
    try {
      const steps = [];
      for (const [index, entry] of lines.entries()) {
        let accepted;
        try {
          const outcome = session.dispatch(entry.event, entry.payload ?? {});
          accepted = outcome.outcome === 'accepted';
        } catch (error) {
          steps.push({ line: index + 1, fatal: String(error?.message ?? error) });
          return steps;
        }
        const gate = session.view().bindings.gate ?? {};
        steps.push({ line: index + 1, accepted, view: pick(gate) });
      }
      return steps;
    } finally { if (session.state === 'open') session.close(); }
  };
}

async function tsArm() {
  const ts = (await import(pathToFileURL(path.join(modules, 'typescript', 'lib', 'typescript.js')))).default;
  const source = await readFile(args.program, 'utf8');
  const { outputText } = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 } });
  const directory = await mkdtemp(path.join(tmpdir(), 'rs1-ts-'));
  const compiled = path.join(directory, 'gate.mjs');
  await writeFile(compiled, outputText);
  const { createPolicy } = await import(pathToFileURL(compiled));
  await rm(directory, { recursive: true, force: true });
  return lines => {
    const policy = createPolicy();
    const steps = [];
    for (const [index, entry] of lines.entries()) {
      let accepted = true;
      try { policy.dispatch(structuredClone(entry)); } catch { accepted = false; }
      steps.push({ line: index + 1, accepted, view: pick(policy.view()) });
    }
    return steps;
  };
}

const runOne = args.arm === 'caveat' ? await caveatArm() : args.arm === 'ts' ? await tsArm() : null;
if (!runOne) throw new Error('--arm is caveat or ts');
const result = {};
for (const file of files) result[file] = runOne(await readLines(path.join(study, file)));
const text = `${JSON.stringify(result, null, 1)}\n`;
if (args.out) await writeFile(args.out, text, { flag: 'wx' }); else process.stdout.write(text);
