// Artifact-local documentation blocks. Registry state and release receipts belong
// on the website/release record, not in an immutable npm package.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const KIT_DOCUMENTS = [
  'README.md', 'docs/README.md', 'docs/GETTING_STARTED.md', 'docs/REFERENCE.md',
  'docs/WORKED_EXAMPLE.md', 'docs/NAMES.md', 'docs/AGENT_START.md', 'docs/MCP.md',
  'examples/agent-evidence/README.md', 'examples/agent-evidence/QUALIFICATION.md',
  'examples/agent-evidence/BRANCHING.md',
];

const BLOCKS = {
  'README.md': ['identity', 'starter'],
  'docs/GETTING_STARTED.md': ['identity', 'install'],
  'docs/AGENT_START.md': ['identity', 'starter'],
  'docs/NAMES.md': ['identity'],
  'docs/MCP.md': ['identity'],
  'examples/agent-evidence/README.md': ['identity', 'agent-install'],
};

export function packageIdentity(manifest) {
  assert.equal(manifest.name, 'caveat-lang', 'documentation is for the caveat-lang package');
  assert.match(manifest.version, /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/, 'invalid package version');
  return `${manifest.name}@${manifest.version}`;
}

export function installationCommands(manifest, kind) {
  const commands = ['npm init -y', `npm install ${packageIdentity(manifest)}`];
  if (kind === 'install') return commands;
  if (kind === 'starter') return [...commands,
    'npx --no-install caveat-lang --version',
    'npx --no-install caveat-lang doctor',
    'npx --no-install caveat-lang demo agent',
    'npx --no-install caveat-lang init',
    'npx --no-install caveat-lang test umbrella.scenarios.json',
    'npx --no-install caveat-lang explain umbrella.cav events.jsonl',
  ];
  if (kind === 'agent-install') return [...commands,
    'cp -r node_modules/caveat-lang/examples/agent-evidence .',
    'cd agent-evidence',
    'npx --no-install caveat-lang validate assessment.cav',
    'npx --no-install caveat-lang check assessment.cav',
    'python3 -B -m unittest -v test_caller',
  ];
  throw new Error(`unknown installation block ${kind}`);
}

export function packageBlock(manifest, id) {
  const content = id === 'identity'
    ? `Package: **\`${packageIdentity(manifest)}\`**.`
    : ['```sh', ...installationCommands(manifest, id), '```'].join('\n');
  return `<!-- caveat-package:${id} -->\n${content}\n<!-- /caveat-package:${id} -->`;
}

function blocksIn(file, text) {
  const tokens = [...text.matchAll(/^<!-- (\/?)caveat-package:([a-z-]+) -->$/gm)];
  assert.equal(tokens.length, [...text.matchAll(/<!--\s*\/?caveat-package:/g)].length, `${file}: malformed package marker`);
  assert.equal(tokens.length % 2, 0, `${file}: unclosed package block`);
  const blocks = [];
  for (let index = 0; index < tokens.length; index += 2) {
    const [open, close] = [tokens[index], tokens[index + 1]];
    assert.equal(open[1], '', `${file}: unexpected closing marker`);
    assert.equal(close[1], '/', `${file}: nested package block`);
    assert.equal(open[2], close[2], `${file}: mismatched package markers`);
    blocks.push({ id: open[2], start: open.index, end: close.index + close[0].length });
  }
  assert.deepEqual(blocks.map(block => block.id), BLOCKS[file] ?? [], `${file}: expected package blocks are missing, duplicated, or reordered`);
  return blocks;
}

export function renderPackageDocument(file, source, manifest) {
  packageIdentity(manifest);
  const text = source.replaceAll('\r\n', '\n');
  const blocks = blocksIn(file, text);
  let rendered = text;
  for (const { id, start, end } of blocks.reverse()) {
    rendered = rendered.slice(0, start) + packageBlock(manifest, id) + rendered.slice(end);
  }
  return rendered;
}

// Deliberately focused on the kit's own current guides. Archived release records
// and copied specifications retain their dated history and separate scope.
export function validatePackageDocument(file, source, manifest) {
  const identity = packageIdentity(manifest);
  const text = source.replaceAll('\r\n', '\n');
  assert.equal(text, renderPackageDocument(file, text, manifest), `${file}: generated package blocks are stale; run node scripts/kit-docs.mjs --write`);
  const movingClaims = [
    /npm publication (?:verified|pending)/i,
    /(?:publication|npm version) (?:is |remains )?(?:available and )?(?:verified|pending|unpublished)/i,
    /(?:next|latest)[`'"*]*\s+(?:tag\s+)?(?:names|points to|remains|is)\s+(?:on\s+)?(?:rc\.|\d+\.\d+\.\d+)/i,
    /(?:this checkout|repository|package) (?:is |stays |remains )?(?:\*\*)?(?:unpublished|(?:\d+\.\d+\.\d+-)?rc\.\d+ development)/i,
    /npm-publication(?:-verification|-receipts|\.json)|\[publication receipt\]/i,
    /retained Linux artifact SHA256/i,
  ];
  if (file === 'README.md') {
    const relative = relativeReadmeLinks(text);
    assert.deepEqual(relative, [], `${file}: relative links 404 on npm and Socket; link ${REPOSITORY_BLOB}<path> instead`);
  }
  for (const pattern of movingClaims) assert(!pattern.test(text), `${file}: moving publication claim belongs in an external release record (${pattern})`);
  // Executable install examples must select these exact bytes' package version,
  // never a moving dist-tag. Ordinary history such as "introduced in rc.7" stays.
  for (const command of text.matchAll(/\bnpm\s+(?:install|i)\s+([^\r\n`]+)/g)) {
    for (const token of command[1].split(/\s+/)) {
      // A package token may be wrapped in a balanced pair of shell quotes.
      const specifier = token.replace(/^(['"])(.*)\1$/, '$2');
      if (specifier === 'caveat-lang' || specifier.startsWith('caveat-lang@')) {
        assert.equal(specifier, identity, `${file}: install example must pin ${identity}`);
      }
    }
  }
  return { file, identity, blocks: (BLOCKS[file] ?? []).slice() };
}

// npm and Socket resolve a package README's relative links against the
// repository root, not `repository.directory`, so they 404 there. The README
// links to GitHub instead; the shipped copies are in the package itself.
const REPOSITORY_BLOB = 'https://github.com/WSattazahn/caveat-lang/blob/main/';

function markdownLinks(text) {
  return [...text.matchAll(/\]\(([^)\s]+)\)/g)].map(match => match[1]);
}

export function relativeReadmeLinks(text) {
  return markdownLinks(text).filter(link => !/^(?:[a-z][a-z0-9+.-]*:|#)/i.test(link));
}

// A link must name a file tracked on main: generated files such as
// kit/docs/reference/ exist after a build but not on GitHub.
export function checkReadmeLinks(text, tracked) {
  const links = markdownLinks(text).filter(link => link.startsWith(REPOSITORY_BLOB));
  const missing = links.filter(link => !tracked.has(decodeURIComponent(link.slice(REPOSITORY_BLOB.length).replace(/#.*$/, ''))));
  assert.deepEqual(missing, [], 'README.md: links name files the repository does not track');
  return links;
}

export function trackedFiles(repository) {
  return new Set(execFileSync('git', ['ls-files', '-z'], { cwd: repository, encoding: 'utf8' }).split('\0').filter(Boolean));
}

export async function checkPackageDocuments(directory, { write = false } = {}) {
  const manifest = JSON.parse(await readFile(path.join(directory, 'package.json'), 'utf8'));
  const reports = [];
  for (const file of KIT_DOCUMENTS) {
    const source = await readFile(path.join(directory, file), 'utf8');
    const rendered = renderPackageDocument(file, source, manifest);
    validatePackageDocument(file, write ? rendered : source, manifest);
    if (write && source !== rendered) await writeFile(path.join(directory, file), rendered);
    reports.push({ file, identity: packageIdentity(manifest), blocks: (BLOCKS[file] ?? []).slice() });
  }
  return reports;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  assert(args.length === 1 && ['--check', '--write'].includes(args[0]), 'usage: node scripts/kit-docs.mjs --check|--write');
  const kit = fileURLToPath(new URL('../kit/', import.meta.url));
  const reports = await checkPackageDocuments(kit, { write: args[0] === '--write' });
  console.log(`${args[0] === '--write' ? 'Generated' : 'Checked'} ${reports.length} package documents for ${reports[0].identity}.`);
  const links = checkReadmeLinks(await readFile(path.join(kit, 'README.md'), 'utf8'), trackedFiles(path.resolve(kit, '..')));
  console.log(`Checked ${links.length} README links against the files tracked in the repository.`);
}
