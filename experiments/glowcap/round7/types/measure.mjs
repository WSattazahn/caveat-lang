// rc.13 PR 9's measurement: round 7's four Caveat adapters, checked against
// declarations `caveat types` generates from each author's own program.
//
//   npm run build && node experiments/glowcap/round7/types/measure.mjs [--json]
//
// For each of C1-C4 it copies runs/<ID>/impl/glowcap.cav and the annotated
// adapter in this directory (the round's adapter plus the lines diffed below)
// into a scratch directory, writes glowcap.d.ts with the kit's `caveat types`,
// links the kit as `caveat-lang`, and runs TypeScript 5.9.3 with
// `--checkJs --noEmit` against @types/node 22.18.6. It prints the added lines
// and every diagnostic. The round's own files are not changed.
//
// As a control, it then makes one misspelling at a time in each annotated
// adapter (below) and reports which ones the check catches: a new diagnostic
// on the misspelled line.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { cp, mkdir, mkdtemp, readFile, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = fileURLToPath(new URL('./', import.meta.url));
const round = path.join(here, '..');
const kit = path.join(here, '../../../../kit');
const AUTHORS = ['C1', 'C2', 'C3', 'C4'];
const TYPESCRIPT = 'typescript@5.9.3';
const NODE_TYPES = '@types/node@22.18.6';
const json = process.argv.includes('--json');

// Each misspelling replaces the first occurrence of its text.
const MUTATIONS = [
  ['binding property', '.present', '.presnt'],
  ['binding target', 'b.slime.', 'b.slim.'],
  ['payload field', 'target:', 'targte:'],
  ['event name', "['tick'", "['tock'"],
  ['decision series', '.trust', '.trusts'],
];

const run = (command, args, cwd) => {
  const result = spawnSync(command, args, { cwd, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, shell: process.platform === 'win32' });
  if (result.error) throw result.error;
  return result;
};

// Lines the annotation added to the round's adapter, as `diff` reports them.
function added(original, annotated) {
  const before = new Map();
  for (const line of original.split('\n')) before.set(line, (before.get(line) ?? 0) + 1);
  const lines = [];
  for (const line of annotated.split('\n')) {
    const count = before.get(line) ?? 0;
    if (count) before.set(line, count - 1); else lines.push(line);
  }
  const removed = [...before].flatMap(([line, count]) => Array(count).fill(line));
  return { added: lines, removed };
}

const scratch = await mkdtemp(path.join(tmpdir(), 'caveat-round7-types-'));
try {
  const tools = path.join(scratch, 'tools');
  await mkdir(tools);
  await writeFile(path.join(tools, 'package.json'), '{"private": true}\n');
  const installed = run('npm', ['install', '--no-save', '--no-audit', '--no-fund', TYPESCRIPT, NODE_TYPES], tools);
  assert.equal(installed.status, 0, installed.stderr);
  const tsc = path.join(tools, 'node_modules/typescript/bin/tsc');

  const results = [];
  for (const author of AUTHORS) {
    const impl = path.join(round, 'runs', author, 'impl');
    const work = path.join(scratch, author);
    await mkdir(path.join(work, 'node_modules/@types'), { recursive: true });
    await cp(path.join(impl, 'glowcap.cav'), path.join(work, 'glowcap.cav'));
    await cp(path.join(here, author, 'adapter.mjs'), path.join(work, 'adapter.mjs'));
    await writeFile(path.join(work, 'package.json'), '{"type": "module"}\n');
    await symlink(kit, path.join(work, 'node_modules/caveat-lang'), 'junction');
    await symlink(path.join(tools, 'node_modules/@types/node'), path.join(work, 'node_modules/@types/node'), 'junction');
    const types = run(process.execPath, [path.join(kit, 'bin/caveat.mjs'), 'types', 'glowcap.cav'], work);
    assert.equal(types.status, 0, `${author}: ${types.stderr}`);
    await writeFile(path.join(work, 'glowcap.d.ts'), types.stdout);
    const check = () => {
      const checked = run(process.execPath, [tsc, '--checkJs', '--allowJs', '--noEmit', '--module', 'nodenext', '--target', 'es2022',
        '--lib', 'es2022,esnext.disposable', '--types', 'node', 'adapter.mjs'], work);
      return { exit: checked.status, diagnostics: checked.stdout.split('\n').filter(line => /error TS\d+/.test(line)) };
    };
    const { exit, diagnostics } = check();
    const original = await readFile(path.join(impl, 'adapter.mjs'), 'utf8');
    const annotated = await readFile(path.join(here, author, 'adapter.mjs'), 'utf8');
    const controls = [];
    for (const [what, from, to] of MUTATIONS) {
      const at = annotated.indexOf(from);
      if (at < 0) { controls.push({ what, applies: false }); continue; }
      const line = annotated.slice(0, at).split('\n').length;
      await writeFile(path.join(work, 'adapter.mjs'), annotated.slice(0, at) + to + annotated.slice(at + from.length));
      const mutated = check();
      const caught = mutated.diagnostics.some(found => found.startsWith(`adapter.mjs(${line},`) && !diagnostics.includes(found));
      controls.push({ what, applies: true, line, caught });
    }
    await writeFile(path.join(work, 'adapter.mjs'), annotated);
    const diff = added(original, annotated);
    results.push({
      author,
      adapterLines: original.trimEnd().split('\n').length,
      added: diff.added.filter(line => line.trim()),
      removed: diff.removed.filter(line => line.trim()),
      declarationLines: types.stdout.trimEnd().split('\n').length,
      exit,
      diagnostics,
      controls,
    });
  }
  if (json) console.log(JSON.stringify({ typescript: TYPESCRIPT, nodeTypes: NODE_TYPES, results }, null, 2));
  else {
    for (const result of results) {
      console.log(`${result.author}: adapter ${result.adapterLines} lines; ${result.added.length} added, ${result.removed.length} changed; declarations ${result.declarationLines} lines; tsc exit ${result.exit}, ${result.diagnostics.length} diagnostic(s)`);
      for (const line of result.removed) console.log(`  - ${line}`);
      for (const line of result.added) console.log(`  + ${line}`);
      for (const line of result.diagnostics) console.log(`  ! ${line}`);
      for (const control of result.controls) {
        console.log(`  control ${control.what}: ${control.applies ? `line ${control.line}, ${control.caught ? 'caught' : 'not caught'}` : 'no such text'}`);
      }
    }
  }
} finally {
  await rm(scratch, { recursive: true, force: true });
}
