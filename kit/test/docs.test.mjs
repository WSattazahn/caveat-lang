// The documentation that ships in the package: every source exists, every
// link in the kit's own documents resolves inside the package, the
// getting-started guide and the worked example, followed step by step, print
// what they show, and the one-page reference quotes only the worked example.
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
  for (const document of ['README.md', 'docs/README.md', 'docs/GETTING_STARTED.md', 'docs/REFERENCE.md', 'docs/WORKED_EXAMPLE.md']) {
    const markdown = await readFile(path.join(kit, document), 'utf8');
    for (const target of relativeLinks(markdown)) {
      const resolved = path.posix.normalize(path.posix.join(path.posix.dirname(document), target));
      assert.ok(files.has(resolved), `${document} links to ${target}, which is not in the package`);
    }
  }
});

// A version change must say what the new version is: a release candidate, or
// a development version that is not released.
test('the kit README names the package\'s version', async () => {
  const { version } = JSON.parse(await readFile(path.join(kit, 'package.json'), 'utf8'));
  const readme = await readFile(path.join(kit, 'README.md'), 'utf8');
  assert.ok(readme.includes(version), `README.md does not mention ${version}`);
});

test('the getting-started guide marks its files, edits and commands', async () => {
  const steps = guideSteps(await readFile(path.join(kit, 'docs/GETTING_STARTED.md'), 'utf8'));
  assert.deepEqual(steps.map(step => step.kind), ['file', 'file', 'run', 'edit', 'run', 'edit', 'file', 'run', 'file', 'run', 'run']);
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

// Follows a kit document in a fresh directory. The repository has no installed
// package: run the command and import the library from this checkout instead.
async function followInCheckout(document) {
  const stamp = new Date().toISOString().replace(/[:.]/g, '-');
  const directory = path.join(repo, 'test-results', 'kit-docs', `${path.basename(document, '.md')}-${stamp}`);
  await mkdir(directory, { recursive: true });
  const cli = path.join(kit, 'bin', 'caveat.mjs');
  return followGuide(await readFile(path.join(kit, document), 'utf8'), {
    directory,
    prepare: (name, content) => content.replace("'caveat-lang/node'", `'${pathToFileURL(path.join(kit, 'lib', 'node.mjs')).href}'`),
    run: (command, cwd) => {
      const args = command.startsWith('npx --no-install caveat ')
        ? [cli, ...command.slice('npx --no-install caveat '.length).split(' ')]
        : command.split(' ').slice(1);
      return spawnSync(process.execPath, args, { cwd, encoding: 'utf8' });
    },
  });
}

function assertPrintsWhatItShows(results) {
  for (const result of results) {
    assert.equal(result.status, result.expected.includes('FAIL') ? 1 : 0, `${result.command}\n${result.stdout}${result.stderr}`);
    assert.equal(result.actual, result.expected, result.command);
  }
}

test('following the getting-started guide prints what it shows', async () => {
  const results = await followInCheckout('docs/GETTING_STARTED.md');
  assert.equal(results.length, 5);
  assertPrintsWhatItShows(results);
});

// Each caveat block with the line before it, which must be a marker.
function programBlocks(markdown) {
  const lines = markdown.split(/\r?\n/);
  return lines.flatMap((line, index) => {
    if (!line.startsWith('```caveat')) return [];
    const end = lines.indexOf('```', index + 1);
    return [{ at: index + 1, marker: lines[index - 1] ?? '', body: lines.slice(index + 1, end) }];
  });
}

// A program on the worked example that no step saves and runs could say
// anything; the page has none.
test('every program in the worked example is a file it runs', async () => {
  const markdown = await readFile(path.join(kit, 'docs/WORKED_EXAMPLE.md'), 'utf8');
  const blocks = programBlocks(markdown);
  assert.ok(blocks.length > 0);
  for (const block of blocks) {
    assert.match(block.marker, /^<!-- file: \S+\.cav -->$/, `the caveat block at line ${block.at} is not a file the page runs`);
  }
  assert.deepEqual(guideSteps(markdown).map(step => step.kind), ['file', 'run', 'run', 'file', 'run', 'file', 'run']);
});

test('following the worked example prints what it shows', async () => {
  const results = await followInCheckout('docs/WORKED_EXAMPLE.md');
  assert.deepEqual(results.map(result => result.command.split(' ')[3]), ['validate', 'check', 'test', 'explain']);
  assertPrintsWhatItShows(results);
});

// The one-page reference quotes the worked example's program rather than
// showing programs of its own, and stays one page.
test('every excerpt on the one-page reference is from the worked example\'s program', async () => {
  const markdown = await readFile(path.join(kit, 'docs/REFERENCE.md'), 'utf8');
  const example = guideSteps(await readFile(path.join(kit, 'docs/WORKED_EXAMPLE.md'), 'utf8'));
  const normalize = lines => lines.map(line => line.trim()).filter(Boolean).join('\n');
  const blocks = programBlocks(markdown);
  assert.ok(blocks.length > 0);
  for (const block of blocks) {
    const name = /^<!-- excerpt: (\S+\.cav) -->$/.exec(block.marker)?.[1];
    assert.ok(name, `the caveat block at line ${block.at} is not marked as an excerpt`);
    const program = example.find(step => step.kind === 'file' && step.name === name);
    assert.ok(program, `the worked example has no ${name}`);
    assert.ok(normalize(program.content.split(/\r?\n/)).includes(normalize(block.body)),
      `the excerpt at line ${block.at} is not in the worked example's ${name}`);
  }
  assert.ok(markdown.split(/\r?\n/).length <= 200, 'the one-page reference is longer than a page');
});
