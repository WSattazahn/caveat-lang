// Finds a Python interpreter, 3.9 or later, for the agent-evidence example's
// caller tests, and runs them. CAVEAT_PYTHON names one to use instead.
import { spawnSync } from 'node:child_process';
import { readdir } from 'node:fs/promises';

const CANDIDATES = [['python3'], ['python'], ['py', '-3']];

export function findPython() {
  const candidates = process.env.CAVEAT_PYTHON ? [[process.env.CAVEAT_PYTHON]] : CANDIDATES;
  for (const [command, ...args] of candidates) {
    const probe = spawnSync(command, [...args, '-c', 'import sys; print(sys.version_info >= (3, 9))'], { encoding: 'utf8' });
    if (probe.status === 0 && probe.stdout.trim() === 'True') return [command, ...args];
  }
  throw new Error(`No Python 3.9 or later was found (tried ${candidates.map(words => words.join(' ')).join(', ')}). `
    + 'The agent-evidence example\'s caller tests need one; install Python or set CAVEAT_PYTHON.');
}

// Runs test_caller.py in `directory` with `caveat` started by `command`, a
// list of words, or by the example's default when it is null. Bytecode is
// off, so the run leaves the directory as it found it; the result says what
// changed.
export async function runCallerTests(directory, command = null) {
  const before = (await readdir(directory)).sort();
  const [python, ...args] = findPython();
  const env = { ...process.env, PYTHONDONTWRITEBYTECODE: '1' };
  if (command) env.CAVEAT_COMMAND = JSON.stringify(command);
  else delete env.CAVEAT_COMMAND;
  const result = spawnSync(python, [...args, '-B', '-m', 'unittest', '-v', 'test_caller'], { cwd: directory, env, encoding: 'utf8' });
  const after = (await readdir(directory)).sort();
  // unittest reports on standard error; Windows ends its lines with CRLF.
  const report = (result.stderr ?? '').replace(/\r\n/g, '\n');
  return { ...result, report, python: [python, ...args].join(' '), before, after };
}
