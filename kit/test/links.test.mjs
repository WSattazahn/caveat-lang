// The link rewrite applied to repository documents copied into the package,
// and the documents pack-docs.json copies: every link one keeps resolves
// inside the package, and every link it rewrites names a repository file.
import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { isRelative, markdownLinks, resolveLink, rewriteLinks } from './links.mjs';

const kit = fileURLToPath(new URL('../', import.meta.url));
const repo = path.join(kit, '..');
const { reference } = JSON.parse(await readFile(path.join(kit, 'pack-docs.json'), 'utf8'));
const REVISION = '0123456789abcdef0123456789abcdef01234567';
const BLOB = `https://github.com/WSattazahn/caveat-lang/blob/${REVISION}`;
const shipped = file => ['spec/caveat-dispatch-0.1.md', 'examples/thermostat_history.cav', 'docs/AI_AUTHORING.md'].includes(file);
const rewrite = markdown => rewriteLinks(markdown, { from: 'spec/caveat-save-0.1.md', revision: REVISION, shipped });

test('a relative link to a file in the package is kept', () => {
  const markdown = 'See [dispatch](caveat-dispatch-0.1.md#origins), [the example](../examples/thermostat_history.cav)'
    + ' and [the guide](../docs/AI_AUTHORING.md).\n';
  assert.deepEqual(rewrite(markdown), { text: markdown, rewritten: [] });
});

test('a relative link to a file outside the package is rewritten, keeping its anchor', () => {
  const { text, rewritten } = rewrite('Measured in [Glowcap](../experiments/glowcap/RESULTS.md#round-3) and [the draft](caveat-0.5-draft.md).\n');
  assert.equal(text, `Measured in [Glowcap](${BLOB}/experiments/glowcap/RESULTS.md#round-3) and [the draft](${BLOB}/spec/caveat-0.5-draft.md).\n`);
  assert.deepEqual(rewritten.map(link => link.target), ['../experiments/glowcap/RESULTS.md#round-3', 'caveat-0.5-draft.md']);
});

test('absolute URLs, mail links and anchors are kept', () => {
  const markdown = '[web](https://example.com/a.md), [mail](mailto:a@example.com), [here](#changes), [proto](//example.com/x).\n';
  assert.deepEqual(rewrite(markdown), { text: markdown, rewritten: [] });
});

test('links inside fenced code blocks and inline code are untouched', () => {
  const markdown = [
    'Before [outside](../game/a.cav).',
    '',
    '```markdown',
    'A [link](../game/b.cav) in a fence.',
    '```',
    '',
    '  ~~~text',
    '  [indented](../game/c.cav)',
    '  ~~~',
    '',
    'Inline `[code](../game/d.cav)` and ``a `[tick](../game/e.cav)` span``,',
    'but [`code` text](../game/f.cav) is a link.',
    '',
  ].join('\n');
  const { text, rewritten } = rewrite(markdown);
  assert.deepEqual(rewritten.map(link => link.target), ['../game/a.cav', '../game/f.cav']);
  assert.equal(text, markdown.replace('(../game/a.cav)', `(${BLOB}/game/a.cav)`).replace('(../game/f.cav)', `(${BLOB}/game/f.cav)`));
});

test('image links, titles, angle brackets and reference definitions are rewritten the same way', () => {
  const { text } = rewrite([
    '![diagram](../docs/diagram.png) and [titled](../docs/x.md "Title") and [angled](<../docs/y.md>).',
    '',
    '[ref]: ../docs/z.md#part',
    '[kept]: caveat-dispatch-0.1.md',
    '',
  ].join('\n'));
  assert.equal(text, [
    `![diagram](${BLOB}/docs/diagram.png) and [titled](${BLOB}/docs/x.md "Title") and [angled](<${BLOB}/docs/y.md>).`,
    '',
    `[ref]: ${BLOB}/docs/z.md#part`,
    '[kept]: caveat-dispatch-0.1.md',
    '',
  ].join('\n'));
});

test('the revision must be a full commit SHA, and a link may not leave the repository', () => {
  assert.throws(() => rewriteLinks('', { from: 'a.md', revision: 'main', shipped }), /not a full commit SHA/);
  assert.throws(() => rewriteLinks('[up](../../x.md)', { from: 'spec/a.md', revision: REVISION, shipped }), /outside the repository/);
});

// The documents the package copies: a link a copy keeps resolves inside the
// package, and a link it rewrites names a file in this checkout.
test('every link in a copied document resolves inside the package or is rewritten to a repository file', async () => {
  const copied = new Set(reference);
  let rewrittenLinks = 0;
  for (const from of reference.filter(file => file.endsWith('.md'))) {
    const { text, rewritten } = rewriteLinks(await readFile(path.join(repo, from), 'utf8'), { from, revision: REVISION, shipped: file => copied.has(file) });
    for (const { target } of markdownLinks(text).filter(link => isRelative(link.target))) {
      assert.ok(copied.has(resolveLink(from, target).path), `${from} keeps ${target}, which is not in the package`);
    }
    for (const { target } of rewritten) {
      const file = resolveLink(from, target).path;
      assert.ok(existsSync(path.join(repo, file)), `${from} links to ${target}, and ${file} is not in the repository`);
    }
    rewrittenLinks += rewritten.length;
  }
  assert.ok(rewrittenLinks > 0, 'no copied document links outside the package');
});
