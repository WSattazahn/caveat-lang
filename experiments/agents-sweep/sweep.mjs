// Corpus sweep for Agents 0.1 (spec/caveat-agents-0.1.md, "Corpus sweep").
// The profile adds the statement `agent NAME [sends EVENT, …];`. This reads
// every tracked Caveat program and reports:
//   - every statement that begins with the word `agent`: before this profile
//     such a statement did not load;
//   - every other use of the words `agent` or `sends` outside quoted text and
//     comments, which must keep meaning what it meant.
// With --against DIR, where DIR holds a published package's runtime/ (for
// example node_modules/caveat-lang), it also loads each tracked program in
// that runtime and in this checkout's (dist/pkg-reactive, from npm run build)
// and compares the interface and the initial snapshot byte for byte. A
// program that does not load in one must not load in the other.
// Run from any directory:
//   node experiments/agents-sweep/sweep.mjs [--against DIR] [--write]
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../..');
const listed = spawnSync('git', ['ls-files', '-z', '--', '*.cav'], { cwd: root, encoding: 'utf8' });
assert.equal(listed.status, 0, listed.stderr);
const programs = listed.stdout.split('\0').filter(Boolean).sort();
assert.ok(programs.length > 0, 'the tracked Caveat corpus must not be empty');

// `#` or `//` comments and quoted text carry no source words.
function code(text) {
  let out = '';
  let inString = false;
  for (let i = 0; i < text.length; i += 1) {
    const c = text[i];
    if (inString) {
      if (c === '\\') { i += 1; continue; }
      if (c === '"') { inString = false; out += '"'; }
      continue;
    }
    if (c === '"') { inString = true; out += '"'; continue; }
    if (c === '#' || (c === '/' && text[i + 1] === '/')) {
      while (i < text.length && text[i] !== '\n') i += 1;
      out += '\n';
      continue;
    }
    out += c;
  }
  return out;
}

const statements = [];
const words = [];
for (const file of programs) {
  const text = code(readFileSync(join(root, file), 'utf8'));
  // A statement begins at the start of the text or after a `;`.
  for (const match of text.matchAll(/(^|;)\s*agent\b/g)) {
    statements.push({ file, line: text.slice(0, match.index + match[0].length).split('\n').length });
  }
  text.split('\n').forEach((line, index) => {
    for (const match of line.matchAll(/\b(agent|sends)\b/g)) words.push({ file, line: index + 1, word: match[1] });
  });
}

const results = { programs: programs.length, agent_statements: statements, words };

const against = process.argv.indexOf('--against');
if (against >= 0) {
  const published = resolve(process.argv[against + 1]);
  const load = async directory => {
    const { loadRuntimeFromDirectory } = await import(pathToFileURL(join(directory, 'lib', 'node.mjs')));
    return loadRuntimeFromDirectory(join(directory, 'runtime'));
  };
  const { loadRuntimeFromDirectory } = await import(pathToFileURL(join(root, 'kit', 'lib', 'node.mjs')));
  const before = await load(published);
  const after = await loadRuntimeFromDirectory(join(root, 'dist', 'pkg-reactive'));
  const compared = { loaded: 0, refused_by_both: 0, differences: [] };
  for (const file of programs) {
    const source = readFileSync(join(root, file), 'utf8');
    const read = runtime => {
      try {
        const session = runtime.open(source);
        const result = { interface: JSON.stringify(runtime.interface(source)), snapshot: session.snapshotText() };
        session.close();
        return result;
      } catch (error) {
        return { error: error.message };
      }
    };
    const old = read(before);
    const now = read(after);
    if (old.error !== undefined && now.error !== undefined) { compared.refused_by_both += 1; continue; }
    if (old.error !== undefined || now.error !== undefined) {
      compared.differences.push({ file, before: old.error ?? 'loads', after: now.error ?? 'loads' });
      continue;
    }
    compared.loaded += 1;
    for (const key of ['interface', 'snapshot']) {
      if (old[key] !== now[key]) compared.differences.push({ file, differs: key });
    }
  }
  results.against = { package: JSON.parse(readFileSync(join(published, 'package.json'), 'utf8')).version, ...compared };
}

const json = `${JSON.stringify(results, null, 2)}\n`;
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), json);
process.stdout.write(json);
