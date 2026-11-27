// Summary counts are over reviewed revisions and exact identifier strings;
// they do not turn labels into source identities or a failing run into success.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { formatSummary, LEDGER_SUMMARY_SCHEMA, reviewLedgers, summarizeLedgers } from '../lib/ledgers.mjs';
import { faulting, faultingRuntimeDirectory } from './fatal-fixture.mjs';
import { real } from './helpers.mjs';

const cli = fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url));
const caveat = (...args) => spawnSync(process.execPath, [cli, ...args], { encoding: 'utf8' });
const events = (...names) => names.map(event => ({ event, payload: {} }));
const completed = events('observe', 'decide', 'outcome');
const jsonl = sent => sent.map(value => JSON.stringify(value)).join('\n') + '\n';

function program({ origin = 'provider A', reading = false, two = false, caveated = true, materialized = true } = {}) {
  const evidence = reading ? 'measurement' : 'scan';
  const value = reading ? 'latest(scan)' : 'qualified(1, scan)';
  return `claim supported;
evidence ${evidence} from "${origin}";
${reading ? 'readings scan from measurement limit 2;' : ''}
evidence other_scan from "another provider";
evidence outcome_receipt from "a recorded outcome";
caveat risk consequence material;
${caveated ? `risk qualifies ${evidence};\n${two ? 'risk qualifies other_scan;' : ''}` : ''}
decisions choose limit 2;
event observe; event decide; event outcome; event fault;
on observe ${reading ? 'sample scan = 1' : 'reveal scan'} supports supported;
on observe reveal other_scan supports supported;
on decide commit choose because enough using ${value} + ${value}${two ? ' + qualified(1, other_scan)' : ''};
on outcome reveal outcome_receipt;
on outcome resolve choose = 2 because outcome_receipt${caveated && materialized ? ' materialized risk' : ''};`;
}

function ledger(name, source = program(), sent = completed) {
  return { program: `${name}.cav`, events_file: `${name}.events.jsonl`, source, events: sent };
}

test('summary counts a shared caveat once per revision and each carrier once', () => {
  const entries = reviewLedgers(source => real.open(source), [ledger('two', program({ two: true }))]);
  assert.equal(entries.length, 1);
  assert.equal(entries[0].fatal, null);
  assert.deepEqual(entries[0].grounds.map(item => item.evidence).sort(), ['other_scan', 'scan']);
  assert.deepEqual(summarizeLedgers(entries), {
    schema: LEDGER_SUMMARY_SCHEMA, revisions: 1,
    caveats: [{ caveat: 'risk', retained: 1, came_true: 1,
      by_evidence: [{ evidence: 'other_scan', retained: 1, came_true: 1 }, { evidence: 'scan', retained: 1, came_true: 1 }] }],
  });
  const repeated = summarizeLedgers([...entries, ...entries]);
  assert.equal(repeated.revisions, 2, 'duplicate supplied entries count again');
  assert.equal(repeated.caveats[0].retained, 2);
  assert.deepEqual(repeated.caveats[0].by_evidence.map(item => item.retained), [2, 2]);
});

test('summary pools equal names across programs but keeps occurrence identifiers separate', () => {
  const entries = reviewLedgers(source => real.open(source), [
    ledger('a', program()),
    ledger('b', program({ origin: 'unrelated provider B', materialized: false })),
    ledger('c', program({ reading: true })),
  ]);
  assert.ok(entries.every(entry => !entry.error && entry.fatal === null));
  assert.equal(new Set(entries.map(entry => entry.program_sha256)).size, 3);
  assert.deepEqual(summarizeLedgers(entries), {
    schema: LEDGER_SUMMARY_SCHEMA, revisions: 3,
    caveats: [{ caveat: 'risk', retained: 3, came_true: 2,
      by_evidence: [{ evidence: 'scan', retained: 2, came_true: 1 }, { evidence: 'scan@1', retained: 1, came_true: 1 }] }],
  });
});

test('summary omits errors and absent outcomes but counts outcomes with no caveated ground', () => {
  const entries = reviewLedgers(source => real.open(source), [
    ledger('plain', program({ caveated: false })),
    ledger('unresolved', program(), events('observe', 'decide')),
    ledger('broken', 'claim;', []),
  ]);
  assert.equal(entries.filter(entry => entry.error).length, 1);
  assert.equal(entries.filter(entry => !entry.error).length, 1);
  const summary = summarizeLedgers(entries);
  assert.deepEqual(summary, { schema: LEDGER_SUMMARY_SCHEMA, revisions: 1, caveats: [] });
  assert.deepEqual(formatSummary(summary), []);
  assert.deepEqual(summarizeLedgers([]), { schema: LEDGER_SUMMARY_SCHEMA, revisions: 0, caveats: [] });
});

test('a fatal event after an outcome preserves its count and the CLI failure status', async () => {
  const source = program();
  const sent = [...completed, ...events('fault')];
  const runtime = faulting().runtime;
  const entries = reviewLedgers(text => runtime.open(text), [ledger('failure', source, sent)]);
  assert.match(entries[0].fatal, /fixture fault/);
  assert.equal(summarizeLedgers(entries).caveats[0].came_true, 1);
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-summary-fatal-'));
  try {
    await writeFile(path.join(directory, 'failure.cav'), source);
    await writeFile(path.join(directory, 'failure.events.jsonl'), jsonl(sent));
    const fakeRuntime = await faultingRuntimeDirectory();
    for (const json of [false, true]) {
      const args = ['review', '--runtime', fakeRuntime, '--all', ...(json ? ['--json'] : []), directory];
      const plain = caveat(...args);
      const summary = caveat(...args, '--summary');
      assert.equal(plain.status, 1, plain.stderr);
      assert.equal(summary.status, 1, summary.stderr);
      assert.equal(summary.stderr, plain.stderr);
      if (json) {
        const rows = summary.stdout.trim().split('\n').map(JSON.parse);
        assert.equal(rows.length, 2);
        assert.match(rows[0].fatal, /fixture fault/);
        assert.deepEqual(rows[1], summarizeLedgers(entries));
        assert.equal(summary.stdout.slice(0, summary.stdout.lastIndexOf('{"schema":"caveat-ledger-summary/0.1"')), plain.stdout);
      } else {
        assert.match(summary.stdout, /fixture fault/);
        assert.match(summary.stdout, /Across the set \(1 revision with an outcome\)/);
        assert.equal(summary.stdout.replace(/Across the set[^]*?\n\n(?=1 revision)/, ''), plain.stdout);
      }
    }
  } finally { await rm(directory, { recursive: true, force: true }); }
});

test('summary stays opt-in for errors and empty caveat sets and is refused outside review --all', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-summary-empty-'));
  try {
    const sourcePath = path.join(directory, 'plain.cav');
    const eventsPath = path.join(directory, 'plain.events.jsonl');
    await writeFile(sourcePath, program({ caveated: false }));
    await writeFile(eventsPath, jsonl(completed));
    const plain = caveat('review', '--all', directory);
    const summary = caveat('review', '--all', '--summary', directory);
    assert.equal(plain.status, 0, plain.stderr);
    assert.equal(summary.status, 0, summary.stderr);
    assert.equal(summary.stdout, plain.stdout, 'no empty text block');
    const json = caveat('review', '--all', '--summary', '--json', directory);
    assert.equal(json.status, 0, json.stderr);
    assert.deepEqual(JSON.parse(json.stdout.trim().split('\n').at(-1)), { schema: LEDGER_SUMMARY_SCHEMA, revisions: 1, caveats: [] });
    for (const args of [
      ['review', '--summary', sourcePath, 'choose@1', eventsPath],
      ['explain', '--summary', sourcePath, eventsPath],
      ['check', '--summary', sourcePath],
    ]) {
      const refused = caveat(...args);
      assert.equal(refused.status, 2, refused.stderr);
      assert.match(refused.stderr, /--summary is read with --all|unknown option --summary/);
      assert.equal(refused.stdout, '');
    }
    await writeFile(path.join(directory, 'broken.cav'), 'claim;');
    await writeFile(path.join(directory, 'broken.events.jsonl'), '');
    const broken = caveat('review', '--all', '--summary', '--json', directory);
    assert.equal(broken.status, 2);
    assert.match(broken.stderr, /broken\.cav does not load/);
    assert.deepEqual(JSON.parse(broken.stdout.trim().split('\n').at(-1)), { schema: LEDGER_SUMMARY_SCHEMA, revisions: 1, caveats: [] });
  } finally { await rm(directory, { recursive: true, force: true }); }
});
