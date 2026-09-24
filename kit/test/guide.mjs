// Follows docs/GETTING_STARTED.md the way a reader does. Invisible markers in
// the guide say what each block is:
//   <!-- file: NAME -->          the next code block is a file to save
//   <!-- edit: NAME: "A" -> "B" --> replace A with B in that file
//   <!-- run: COMMAND -->        run COMMAND; its output is the next text block
// Used by kit/test/docs.test.mjs (repository) and scripts/test-kit-package.mjs
// (the installed package).
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

function nextBlock(lines, from, language) {
  for (let index = from; index < lines.length; index++) {
    const fence = /^```(\w*)/.exec(lines[index]);
    if (!fence || (language && fence[1] !== language)) continue;
    const body = [];
    for (let line = index + 1; line < lines.length; line++) {
      if (lines[line].startsWith('```')) return body.join('\n');
      body.push(lines[line]);
    }
    throw new Error(`unclosed code block at line ${index + 1}`);
  }
  throw new Error(`no ${language ?? 'code'} block after line ${from}`);
}

export function guideSteps(markdown) {
  const lines = markdown.split(/\r?\n/);
  const steps = [];
  lines.forEach((line, index) => {
    const marker = /^<!-- (file|edit|run): (.*) -->$/.exec(line.trim());
    if (!marker) return;
    const [, kind, argument] = marker;
    if (kind === 'file') steps.push({ kind, name: argument, content: `${nextBlock(lines, index + 1)}\n` });
    if (kind === 'run') steps.push({ kind, command: argument, expected: nextBlock(lines, index + 1, 'text') });
    if (kind === 'edit') {
      const edit = /^(\S+): "(.*)" -> "(.*)"$/.exec(argument);
      if (!edit) throw new Error(`cannot read edit marker at line ${index + 1}`);
      steps.push({ kind, name: edit[1], from: edit[2], to: edit[3] });
    }
  });
  return steps;
}

// run(command, directory) returns { status, stdout, stderr }. prepare(name,
// content) may adapt a file for the environment (the repository has no
// installed package to import). Returns one result per run marker.
export async function followGuide(markdown, { directory, run, prepare = (name, content) => content }) {
  const results = [];
  for (const step of guideSteps(markdown)) {
    const file = step.name && path.join(directory, step.name);
    if (step.kind === 'file') {
      await writeFile(file, prepare(step.name, step.content));
    } else if (step.kind === 'edit') {
      const text = await readFile(file, 'utf8');
      if (!text.includes(step.from)) throw new Error(`${step.name} does not contain "${step.from}"`);
      await writeFile(file, text.split(step.from).join(step.to));
    } else {
      const outcome = await run(step.command, directory);
      results.push({ command: step.command, expected: step.expected.trim(), ...outcome, actual: outcome.stdout.trim() });
    }
  }
  return results;
}
