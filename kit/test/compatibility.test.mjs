// docs/COMPATIBILITY.md against the repository: the specifications it calls
// stable say so, the codes, schemas, library exports and MCP tools it promises
// exist, and every promise names CI steps that run.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readdir, readFile } from 'node:fs/promises';
import path from 'node:path';
import { repo } from './helpers.mjs';

const commitment = await readFile(path.join(repo, 'docs', 'COMPATIBILITY.md'), 'utf8');
const section = heading => {
  const start = commitment.indexOf(`\n${heading}\n`);
  assert.ok(start >= 0, `COMPATIBILITY.md has no "${heading}"`);
  const rest = commitment.slice(start + heading.length + 2);
  const end = rest.search(/\n#{2,3} /);
  return end < 0 ? rest : rest.slice(0, end);
};
const ticked = text => [...text.matchAll(/`([^`]+)`/g)].map(match => match[1]);
const sorted = values => [...values].sort();

test('every specification\'s status line says whether the compatibility commitment covers it', async () => {
  const specs = (await readdir(path.join(repo, 'spec'))).filter(name => name.endsWith('.md')).sort();
  const listed = ticked(section('### Specifications')).filter(name => name.endsWith('.md'));
  const notStable = new Set(listed.filter(name => name !== 'caveat-0.1.md'));
  for (const name of notStable) assert.ok(specs.includes(name), `COMPATIBILITY.md lists ${name}, which spec/ does not have`);
  for (const name of specs) {
    const status = (await readFile(path.join(repo, 'spec', name), 'utf8')).split('\n')[2] ?? '';
    assert.match(status, /^\*\*Status:\*\* /, `spec/${name}: the line under its title is not its status`);
    if (notStable.has(name)) assert.match(status, /not part of the \[compatibility commitment\]/, `spec/${name} is listed as not stable`);
    else assert.match(status, /stable from 1\.\d+\.\d+ under the \[compatibility commitment\]/, `spec/${name} is stable by COMPATIBILITY.md`);
  }
});

test('the permanent rejection codes are Dispatch 0.1\'s catalog and the kit\'s declared codes', async () => {
  const text = section('### Rejection origins and codes');
  const promised = ticked(text.slice(text.indexOf('are permanent:'), text.indexOf('A 1.x release')));
  assert.equal(promised.length, 28);
  const dispatch = await readFile(path.join(repo, 'spec', 'caveat-dispatch-0.1.md'), 'utf8');
  const catalog = [...dispatch.matchAll(/^\| `(\w+)` \| `(\w+)` \|/gm)];
  assert.deepEqual(sorted(promised), sorted(new Set(catalog.map(([, , code]) => code))));
  assert.deepEqual(sorted(ticked(text.slice(0, text.indexOf('The codes')))), sorted(new Set(catalog.map(([, origin]) => origin))));
  const types = await readFile(path.join(repo, 'kit', 'lib', 'session.d.mts'), 'utf8');
  const declared = /export type KnownRejectionCode =([^;]+);/.exec(types)[1];
  assert.deepEqual(sorted(promised), sorted([...declared.matchAll(/'(\w+)'/g)].map(match => match[1])));
});

test('every promised schema is a name the kit or the runtime writes', async () => {
  const schemas = ticked(section('### Schemas')).filter(name => /^caveat-[a-z-]+\/\d+\.\d+$/.test(name));
  assert.ok(schemas.length >= 10);
  const sources = [];
  for (const [directory, suffix] of [['kit/lib', '.mjs'], ['runtime/src', '.rs']]) {
    for (const name of await readdir(path.join(repo, directory))) {
      if (name.endsWith(suffix)) sources.push(await readFile(path.join(repo, directory, name), 'utf8'));
    }
  }
  const text = sources.join('\n');
  for (const schema of schemas) assert.ok(text.includes(`'${schema}'`) || text.includes(`"${schema}"`), `${schema} is not written anywhere`);
});

test('every promise names CI steps that the runtime workflow runs', async () => {
  const workflow = await readFile(path.join(repo, '.github', 'workflows', 'runtime.yml'), 'utf8');
  const steps = new Set([...workflow.matchAll(/^\s*- name: (.+)$/gm)].map(match => match[1].trim()));
  const rows = section('## How each promise is checked').split('\n').filter(line => line.startsWith('| ') && !line.startsWith('| Promise') && !line.startsWith('| ---'));
  assert.ok(rows.length >= 7);
  for (const row of rows) {
    const cells = row.split(' | ');
    const named = [...cells.at(-1).matchAll(/"([^"]+)"/g)].map(match => match[1]);
    assert.ok(named.length, `"${cells[0].slice(2)}" names no CI step`);
    for (const step of named) assert.ok(steps.has(step), `"${step}" is not a step of the runtime workflow`);
  }
});

test('the stable library exports are the package\'s exports, apart from the runtime\'s paths', async () => {
  const row = section('## What is stable').split('\n').find(line => line.startsWith('| Library |'));
  assert.ok(row, 'COMPATIBILITY.md has no Library row');
  const promised = ticked(row.split(' | ')[1]).filter(name => name.startsWith('./'));
  const manifest = JSON.parse(await readFile(path.join(repo, 'kit', 'package.json'), 'utf8'));
  assert.deepEqual(sorted(promised), sorted(Object.keys(manifest.exports).filter(entry => entry !== './runtime/*')));
});

test('the stable MCP tools are the tools the MCP server lists', async () => {
  const row = section('## What is stable').split('\n').find(line => line.startsWith('| MCP tools |'));
  assert.ok(row, 'COMPATIBILITY.md has no MCP tools row');
  const { MCP_TOOLS } = await import('../lib/mcp.mjs');
  assert.deepEqual(ticked(row.split(' | ')[1]), MCP_TOOLS.map(tool => tool.name));
});
