// The documentation that ships in the package: every source exists, every
// link in the kit's own documents resolves inside the package, and the
// getting-started guide, followed step by step, prints what it shows.
import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { followGuide, guideSteps } from './guide.mjs';

const kit = fileURLToPath(new URL('../', import.meta.url));
const repo = path.join(kit, '..');
const manifest = JSON.parse(await readFile(path.join(kit, 'pack-docs.json'), 'utf8'));

// Every path the packed tarball will hold, relative to the package root.
async function packageFiles() {
  const files = new Set(['README.md', 'package.json', 'LICENSE', 'THIRD_PARTY_NOTICES.md',
    'runtime/caveat_runtime.js', 'runtime/caveat_runtime_bg.wasm', 'runtime/build-info.json']);
  for (const directory of ['bin', 'lib', 'docs']) {
    for (const entry of await readdir(path.join(kit, directory), { withFileTypes: true })) {
      if (entry.isFile()) files.add(`${directory}/${entry.name}`);
    }
  }
  for (const file of manifest.reference) files.add(`docs/reference/${file}`);
  return files;
}

export function relativeLinks(markdown) {
  return [...markdown.matchAll(/\]\(([^)\s]+)\)/g)].map(match => match[1])
    .filter(target => !/^(https?:|mailto:|#)/.test(target))
    .map(target => target.split('#')[0]);
}

test('every packaged documentation source exists', () => {
  for (const file of manifest.reference) {
    assert.ok(existsSync(path.join(repo, file)), `${file} is missing`);
  }
});

test('links in the kit documentation resolve inside the package', async () => {
  const files = await packageFiles();
  for (const document of ['README.md', 'docs/README.md', 'docs/GETTING_STARTED.md']) {
    const markdown = await readFile(path.join(kit, document), 'utf8');
    for (const target of relativeLinks(markdown)) {
      const resolved = path.posix.normalize(path.posix.join(path.posix.dirname(document), target));
      assert.ok(files.has(resolved), `${document} links to ${target}, which is not in the package`);
    }
  }
});

test('the getting-started guide marks its files, edits and commands', async () => {
  const steps = guideSteps(await readFile(path.join(kit, 'docs/GETTING_STARTED.md'), 'utf8'));
  assert.deepEqual(steps.map(step => step.kind), ['file', 'file', 'run', 'edit', 'run', 'edit', 'file', 'run']);
});

test('the authoring guide\'s script runs against its example', async () => {
  const guide = await readFile(path.join(repo, 'docs', 'AI_AUTHORING.md'), 'utf8');
  const script = /```js\n([\s\S]*?)```/.exec(guide)?.[1];
  assert.ok(script, 'the authoring guide has a script');
  const directory = path.join(repo, 'test-results', 'kit-docs', `authoring-${Date.now()}`);
  await mkdir(directory, { recursive: true });
  await writeFile(path.join(directory, 'thermostat_history.cav'), await readFile(path.join(repo, 'examples', 'thermostat_history.cav')));
  await writeFile(path.join(directory, 'script.mjs'),
    script.replace("'caveat-lang/node'", `'${pathToFileURL(path.join(kit, 'lib', 'node.mjs')).href}'`));
  const result = spawnSync(process.execPath, ['script.mjs'], { cwd: directory, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /^accepted \[/);
  assert.match(result.stdout, /commitment: 'heating@1'/);
});

test('following the getting-started guide prints what it shows', async () => {
  const directory = path.join(repo, 'test-results', 'kit-docs', new Date().toISOString().replace(/[:.]/g, '-'));
  await mkdir(directory, { recursive: true });
  const cli = path.join(kit, 'bin', 'caveat.mjs');
  const results = await followGuide(await readFile(path.join(kit, 'docs/GETTING_STARTED.md'), 'utf8'), {
    directory,
    // The repository has no installed package: run the command and import the
    // library from this checkout instead.
    prepare: (name, content) => content.replace("'caveat-lang/node'", `'${pathToFileURL(path.join(kit, 'lib', 'node.mjs')).href}'`),
    run: (command, cwd) => {
      const args = command.startsWith('npx --no-install caveat ')
        ? [cli, ...command.slice('npx --no-install caveat '.length).split(' ')]
        : command.split(' ').slice(1);
      return spawnSync(process.execPath, args, { cwd, encoding: 'utf8' });
    },
  });
  assert.equal(results.length, 3);
  for (const result of results) {
    assert.equal(result.status, result.expected.includes('FAIL') ? 1 : 0, `${result.command}\n${result.stdout}${result.stderr}`);
    assert.equal(result.actual, result.expected, result.command);
  }
});
