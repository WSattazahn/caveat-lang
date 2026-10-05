import test from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { readFile } from 'node:fs/promises';
import { checkPackageDocuments, checkReadmeLinks, checkRegistryIdentity, packageBlock, renderPackageDocument, trackedFiles, validatePackageDocument } from '../../scripts/kit-docs.mjs';

const previous = { name: 'caveat-lang', version: '0.1.0-rc.8' };
const candidate = { name: 'caveat-lang', version: '0.1.0-rc.9' };
const readme = manifest => `# CAVEAT Language\n\n${packageBlock(manifest, 'identity')}\n\nIntroduced in 0.1.0-rc.7, caveat-lang is the unambiguous command.\n\n${packageBlock(manifest, 'starter')}\n`;

test('a version change regenerates identity and install commands while retaining history', () => {
  const changed = renderPackageDocument('README.md', readme(previous), candidate);
  assert.match(changed, /Package: \*\*`caveat-lang@0\.1\.0-rc\.9`\*\*/);
  assert.match(changed, /^npm install caveat-lang@0\.1\.0-rc\.9$/m);
  assert.doesNotMatch(changed, /0\.1\.0-rc\.8/);
  assert.match(changed, /Introduced in 0\.1\.0-rc\.7/);
  validatePackageDocument('README.md', changed, candidate);
  assert.equal(renderPackageDocument('README.md', changed, candidate), changed);
});

test('mentioning the current version elsewhere does not excuse stale package or install blocks', () => {
  assert.throws(() => validatePackageDocument('README.md', readme(previous) + '\nDevelopment version 0.1.0-rc.9\n', candidate), /blocks are stale/);
  const staleInstall = readme(candidate).replace('npm install caveat-lang@0.1.0-rc.9', 'npm install caveat-lang@0.1.0-rc.8');
  assert.throws(() => validatePackageDocument('README.md', staleInstall, candidate), /blocks are stale/);
});

test('installation examples outside generated blocks cannot select tags or another version', () => {
  for (const command of [
    'npm install caveat-lang@next', 'npm i caveat-lang@latest', 'npm install caveat-lang',
    'npm install --save-exact caveat-lang@0.1.0-rc.8',
  ]) assert.throws(() => validatePackageDocument('README.md', `${readme(candidate)}\n\`\`\`sh\n${command}\n\`\`\`\n`, candidate), /install example must pin/);
});

test('quoted installation examples cannot select stale versions, tags or an unpinned package', () => {
  for (const quote of ["'", '"']) {
    for (const specifier of ['caveat-lang@0.1.0-rc.8', 'caveat-lang@next', 'caveat-lang@latest', 'caveat-lang']) {
      for (const command of ['npm install', 'npm i --save-exact']) {
        const example = command + ' ' + quote + specifier + quote;
        assert.throws(() => validatePackageDocument('README.md', readme(candidate) + '\n' + example + '\n', candidate), /install example must pin/, example);
      }
    }
  }
});

test('balanced single and double quotes preserve exact-version installation examples', () => {
  for (const quote of ["'", '"']) {
    for (const command of ['npm install', 'npm i --save-exact']) {
      const example = command + ' ' + quote + 'caveat-lang@0.1.0-rc.9' + quote;
      assert.doesNotThrow(() => validatePackageDocument('README.md', readme(candidate) + '\n' + example + '\n', candidate), example);
    }
  }
});

test('dated publication receipts and current-channel notices cannot enter immutable guides', () => {
  for (const notice of [
    '> npm publication verified 2026-10-02: caveat-lang@0.1.0-rc.8',
    '`next` names rc.8; `latest` remains rc.5.',
    'npm publication remains pending.',
    'This checkout is unpublished rc.9 development.',
    '[Publication receipt](https://example.test/npm-publication-verification.json).',
    '> retained Linux artifact SHA256 `deadbeef`.',
  ]) assert.throws(() => validatePackageDocument('README.md', `${readme(candidate)}\n${notice}\n`, candidate), /moving publication claim/);
});

test('genuine feature history and an external release index remain valid', () => {
  const text = readme(candidate) + '\nHistorical rc.6 had only the caveat executable.\n[Release records](https://github.com/WSattazahn/caveat-lang/releases) record availability.\n';
  validatePackageDocument('README.md', text, candidate);
  assert.match(text, /Historical rc\.6/);
});

test('missing, duplicate, malformed and mismatched generation markers fail closed', () => {
  const valid = readme(candidate);
  for (const invalid of [
    valid.replace(/<!-- \/?caveat-package:identity -->\n?/g, ''),
    valid + packageBlock(candidate, 'identity'),
    valid.replace('<!-- caveat-package:identity -->', '<!-- caveat-package:identity-->'),
    valid.replace('<!-- /caveat-package:identity -->', '<!-- /caveat-package:install -->'),
    valid.replace('<!-- /caveat-package:starter -->', ''),
  ]) assert.throws(() => renderPackageDocument('README.md', invalid, candidate), /package (?:block|marker)|package markers/);
});

test('invalid manifest data cannot generate shell installation commands', () => {
  for (const manifest of [
    { ...candidate, version: '0.1.0; echo bad' },
    { ...candidate, version: '0.1.0\nother-command' },
    { ...candidate, name: 'some-other-package' },
  ]) assert.throws(() => renderPackageDocument('README.md', readme(candidate), manifest));
});

test('all current package-owned guides pass the manifest consistency gate', async () => {
  const kit = fileURLToPath(new URL('../', import.meta.url));
  const checked = await checkPackageDocuments(kit);
  assert(checked.some(document => document.file === 'README.md' && document.blocks.includes('starter')));
  assert(checked.some(document => document.file === 'docs/GETTING_STARTED.md' && document.blocks.includes('install')));
});

test('the package README links resolve on npm and Socket', async () => {
  const blob = 'https://github.com/WSattazahn/caveat-lang/blob/main/';
  for (const link of ['docs/AGENT_START.md', './examples/agent-evidence/README.md', 'docs/reference/spec/caveat-serve-0.1.md#serve', '../LICENSE']) {
    assert.throws(() => validatePackageDocument('README.md', `${readme(candidate)}\n[guide](${link})\n`, candidate), /relative links 404/, link);
  }
  validatePackageDocument('README.md', `${readme(candidate)}\n[guide](${blob}kit/docs/AGENT_START.md) and [index](#install)\n`, candidate);

  const tracked = new Set(['kit/docs/AGENT_START.md', 'spec/caveat-serve-0.1.md']);
  checkReadmeLinks(`[a](${blob}kit/docs/AGENT_START.md) [b](${blob}spec/caveat-serve-0.1.md#changes) [c](https://example.com/x)`, tracked);
  for (const link of ['kit/docs/reference/spec/caveat-serve-0.1.md', 'kit/docs/MISSING.md', 'docs/AGENT_START.md']) {
    assert.throws(() => checkReadmeLinks(`[x](${blob}${link})`, tracked), /does not track/, link);
  }

  const links = checkReadmeLinks(await readFile(new URL('../README.md', import.meta.url), 'utf8'), trackedFiles(fileURLToPath(new URL('../../', import.meta.url))));
  assert(links.length >= 16, `expected the README's GitHub links, found ${links.length}`);
});

test('the MCP server name matches the GitHub login exactly and the versions agree (rc.14)', async () => {
  const name = 'io.github.WSattazahn/caveat-lang';
  const manifest = { ...candidate, mcpName: name };
  const server = { name, version: candidate.version, packages: [{ registryType: 'npm', identifier: 'caveat-lang', version: candidate.version }] };
  assert.deepEqual(checkRegistryIdentity(manifest, server), { name, version: candidate.version });
  const lowercase = 'io.github.wsattazahn/caveat-lang';
  assert.throws(() => checkRegistryIdentity({ ...manifest, mcpName: lowercase }, server), /mcpName must be/);
  assert.throws(() => checkRegistryIdentity(manifest, { ...server, name: lowercase }), /server\.json: name must be/);
  assert.throws(() => checkRegistryIdentity({ ...manifest, mcpName: undefined }, server), /mcpName must be/);
  assert.throws(() => checkRegistryIdentity(manifest, { ...server, version: previous.version }), /version must equal/);
  assert.throws(() => checkRegistryIdentity(manifest, { ...server, packages: [{ ...server.packages[0], version: previous.version }] }), /package version must equal/);
  assert.throws(() => checkRegistryIdentity(manifest, { ...server, packages: [] }), /one npm package entry/);

  const current = JSON.parse(await readFile(new URL('../package.json', import.meta.url), 'utf8'));
  const declared = JSON.parse(await readFile(new URL('../../server.json', import.meta.url), 'utf8'));
  checkRegistryIdentity(current, declared);
});
