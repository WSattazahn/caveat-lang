// Execute canonical specifications against the checkout runtime. These checks
// protect the displayed symbol association and the complete public view table.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { loadRuntimeFromDirectory } from '../lib/node.mjs';

const runtime = await loadRuntimeFromDirectory(fileURLToPath(new URL('../../dist/pkg-reactive/', import.meta.url)));

// Read the canonical example, without copying its source into a test fixture.
// It shows static declarations; the reactive host additionally needs an event.
test('the source-text specification example labels its declared evidence and preserves escaped text', async () => {
  const markdown = await readFile(new URL('../../spec/caveat-text-0.1.md', import.meta.url), 'utf8');
  const blocks = [...markdown.matchAll(/```caveat\r?\n([\s\S]*?)```/g)];
  assert.equal(blocks.length, 1, 'the source-text specification has one complete declaration example');
  const session = runtime.open(`${blocks[0][1]}\nevent inspect;\n`);
  try {
    const snapshot = session.snapshot();
    const label = 'The log says "return".\nTrust is provisional.';
    assert.deepEqual(snapshot.scenes, ['The beacon is silent; the sky is full of voices.']);
    assert.deepEqual(snapshot.labels, { archive: label });
    const archive = snapshot.symbols.find(symbol => symbol.name === 'archive');
    assert.ok(archive, 'the example declares archive');
    assert.equal(archive.kind, 'evidence');
    assert.equal(archive.source, 'https://station.example/log;revision=2#signal');
    assert.equal(archive.display, label, 'display must attach to the evidence, not a separately quoted name');
    assert.equal(session.dispatch('inspect').outcome, 'accepted');
  } finally {
    session.close();
  }
});

test('the view specification lists every returned field, including the accumulated decision journal', async () => {
  const markdown = await readFile(new URL('../../spec/caveat-view-0.1.md', import.meta.url), 'utf8');
  const fields = markdown.split('## API')[0].split(/\r?\n/)
    .filter(line => line.startsWith('|'))
    .flatMap(line => [...line.split('|')[1].matchAll(/`([^`]+)`/g)].map(match => match[1]));
  const session = runtime.open(`
    claim ready;
    evidence meter from "meter reading";
    readings samples from meter limit 4;
    decisions plan limit 4 reopened by samples;
    event read value min 0 max 10;
    event decide;
    on read sample samples = value supports ready;
    on decide commit plan because enough using latest(samples);
  `);
  const checkView = () => {
    const view = session.view();
    const snapshot = session.snapshot();
    assert.deepEqual([...fields].sort(), Object.keys(view).sort(), 'the field table matches the runtime surface');
    assert.equal(view.schema, 'caveat-reactive-view/0.1');
    for (const field of fields.filter(field => field !== 'schema')) {
      assert.deepEqual(view[field], snapshot[field], `${field} agrees with the snapshot`);
    }
    return view;
  };
  try {
    assert.deepEqual(checkView().decision_journal, []);
    for (const [event, payload] of [['read', { value: 2 }], ['decide', {}], ['read', { value: 3 }]]) {
      assert.equal(session.dispatch(event, payload).outcome, 'accepted');
      checkView();
    }
    const journal = checkView().decision_journal;
    assert.deepEqual(journal.map(entry => entry.change), ['committed', 'reopened']);
    assert.deepEqual(journal.map(entry => entry.because), [['samples@1'], ['samples@2']]);
    assert.deepEqual(journal.map(entry => entry.value), [2, 2]);
  } finally {
    session.close();
  }
});

// F257: the reactive profile's function limit counts the program's own
// declarations; the standard library prelude is excluded by the number stated.
test('the reactive function limit admits 128 program functions and excludes the stated prelude count', async () => {
  const markdown = await readFile(new URL('../../spec/caveat-reactive-0.2.md', import.meta.url), 'utf8');
  const stated = markdown.match(/A program may declare (\d+) functions of its own; the (\d+) standard library\s+functions are not counted/);
  assert.ok(stated, 'the reactive profile states the function limit');
  const [limit, preludeCount] = [Number(stated[1]), Number(stated[2])];
  const prelude = await readFile(new URL('../../runtime/prelude.cav', import.meta.url), 'utf8');
  assert.equal(prelude.split(/\r?\n/).filter(line => /^fn\s/.test(line)).length, preludeCount);

  const program = count => [
    ...Array.from({ length: count }, (_, index) => `fn f${index}() = ${index};`),
    'state total = 0;',
    'event go;',
    `on go set total = total + f${count - 1}() + abs(-1);`,
  ].join('\n');
  const session = runtime.open(program(limit));
  try {
    assert.equal(session.dispatch('go').outcome, 'accepted');
    assert.equal(session.snapshot().values.total, limit);
  } finally {
    session.close();
  }
  assert.throws(() => runtime.open(program(limit + 1)), /function limit 128/);
});
