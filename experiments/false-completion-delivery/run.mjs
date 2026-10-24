// Runs the shared wrong-delivery fixture through a Caveat decision ledger.
// The host (this file) copies, reads back, hashes and parses. Caveat compares
// the read-back with the frozen contract, decides, and keeps the grounds.
// Exit 0 means every case ran and matched its pre-frozen expected outcome;
// it is not the acceptance of any delivery.
import { createHash } from 'node:crypto';
import { copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const specBytes = await readFile(new URL('./task-spec.json', import.meta.url));
const spec = JSON.parse(specBytes.toString('utf8'));
const sourceBytes = await readFile(new URL('./delivery.cav', import.meta.url));
const source = sourceBytes.toString('utf8');
const runtime = await loadRuntimeFromDirectory();
const pkg = JSON.parse(await readFile(new URL('./node_modules/caveat-lang/package.json', import.meta.url), 'utf8'));

async function readBack(root, destination) {
  let bytes;
  try {
    bytes = await readFile(join(root, destination));
  } catch (error) {
    return { destination, exists: false, sha256: null, json_parse_ok: false, readback_error: error.code ?? 'error', json_error: null };
  }
  const observation = { destination, exists: true, sha256: sha256(bytes), json_parse_ok: false, readback_error: null, json_error: null };
  try {
    JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
    observation.json_parse_ok = true;
  } catch (error) {
    observation.json_error = error.name;
  }
  return observation;
}

function send(session, log, event, payload = {}) {
  const outcome = session.dispatch(event, payload);
  log.push({ event, payload, outcome: outcome.outcome, ...(outcome.message ? { message: outcome.message } : {}) });
  if (outcome.outcome === 'fatal') throw new Error(`fatal on ${event}: ${outcome.message}`);
  return outcome;
}

const readBackPayload = observation => ({
  destination: observation.destination,
  exists: observation.exists ? 'present' : 'missing',
  sha256: observation.sha256 ?? 'absent',
  json: observation.json_parse_ok ? 'valid' : 'invalid',
});

async function withFixtures(fn) {
  const root = await mkdtemp(join(tmpdir(), 'caveat-delivery-'));
  try {
    for (const [relative, text] of Object.entries(spec.fixtures)) {
      await mkdir(dirname(join(root, relative)), { recursive: true });
      await writeFile(join(root, relative), text);
    }
    return await fn(root);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

async function deliver(root, copySource, destination) {
  await mkdir(dirname(join(root, destination)), { recursive: true });
  await copyFile(join(root, copySource), join(root, destination));
}

function verdictOf(snapshot) {
  return {
    verdict: snapshot.bindings.delivery.verdict,
    digest: snapshot.bindings.delivery.digest,
    json: snapshot.bindings.delivery.json,
    grounds: snapshot.commitment_grounds,
  };
}

async function runCase(item, contract) {
  return withFixtures(async root => {
    const log = [];
    const session = runtime.open(source);
    send(session, log, 'freeze_contract', { approved_sha256: contract.approved_sha256, destination: contract.destination });
    send(session, log, 'agent_reports_done');
    await deliver(root, item.copy_source, contract.destination);
    const observation = await readBack(root, contract.destination);
    send(session, log, 'read_back', readBackPayload(observation));
    const judged = send(session, log, 'judge');
    const result = verdictOf(judged.snapshot);
    return { id: item.id, copy_source: item.copy_source, approved_sha256: contract.approved_sha256,
      observation, ...result, expected: item.expected, matches_expected: result.verdict === item.expected, events: log };
  });
}

// Beyond the shared contract: what a ledger adds once a verdict exists.
async function extensions() {
  const contract = spec.contract;
  const results = [];

  results.push(await withFixtures(async root => {
    const log = [];
    const session = runtime.open(source);
    send(session, log, 'freeze_contract', { approved_sha256: contract.approved_sha256, destination: contract.destination });
    send(session, log, 'agent_reports_done');
    const refused = send(session, log, 'judge');
    const verdict = session.snapshot().bindings.delivery.verdict;
    const ok = refused.outcome === 'rejected' && verdict === 'none';
    return { id: 'report_without_readback', description: 'The agent reports done and nothing is read back: no verdict can be made.',
      verdict, refusal: refused.message ?? null, matches_expected: ok, events: log };
  }));

  results.push(await withFixtures(async root => {
    const log = [];
    const session = runtime.open(source);
    send(session, log, 'freeze_contract', { approved_sha256: contract.approved_sha256, destination: contract.destination });
    await deliver(root, 'reviewed/artifact.json', 'elsewhere/artifact.json');
    const observation = await readBack(root, 'elsewhere/artifact.json');
    const refused = send(session, log, 'read_back', readBackPayload(observation));
    return { id: 'readback_of_wrong_path', description: 'The correct bytes read back from an undeclared path are refused as evidence.',
      observation, refusal: refused.message ?? null, matches_expected: refused.outcome === 'rejected', events: log };
  }));

  results.push(await withFixtures(async root => {
    const log = [];
    const session = runtime.open(source);
    send(session, log, 'freeze_contract', { approved_sha256: contract.approved_sha256, destination: contract.destination });
    send(session, log, 'agent_reports_done');
    await deliver(root, 'reviewed/artifact.json', contract.destination);
    send(session, log, 'read_back', readBackPayload(await readBack(root, contract.destination)));
    const first = verdictOf(send(session, log, 'judge').snapshot);
    await deliver(root, 'stale/artifact.json', contract.destination);
    const reread = send(session, log, 'read_back', readBackPayload(await readBack(root, contract.destination)));
    const afterOverwrite = verdictOf(reread.snapshot);
    const second = verdictOf(send(session, log, 'judge').snapshot);
    const journal = session.snapshot().decision_journal;
    const ok = first.verdict === 'accept' && afterOverwrite.verdict === 'reopened' && second.verdict === 'reject'
      && JSON.stringify(second.grounds['verdict@1']) === JSON.stringify(first.grounds['verdict@1']);
    return { id: 'overwritten_after_accept', description: 'Accepted, then the destination is overwritten with the stale bytes and read back again: the acceptance is reopened, a new verdict rejects, and the first verdict keeps its original grounds.',
      first, after_overwrite: afterOverwrite.verdict, second, decision_journal: journal, matches_expected: ok, events: log };
  }));

  return results;
}

const cases = [];
for (const item of spec.cases) cases.push(await runCase(item, spec.contract));
const controls = [];
for (const item of spec.controls) controls.push(await runCase(item, item.contract));
const extended = await extensions();

const all = [...cases, ...controls, ...extended];
const report = {
  schema: 'caveat-delivery-observations/0.1',
  synthetic: true,
  task_spec_sha256: sha256(specBytes),
  program: 'delivery.cav',
  program_sha256: sha256(sourceBytes),
  runtime: `caveat-lang@${pkg.version}`,
  node: process.version,
  cases,
  controls,
  extended,
  all_matched_expected: all.every(item => item.matches_expected),
  runner_exit_meaning: '0: every case ran and matched its pre-frozen expected outcome. Not the acceptance of any delivery.',
};

if (process.argv.includes('--json')) {
  console.log(JSON.stringify(report, null, 2));
} else {
  console.log(`SYNTHETIC local fixtures. ${report.runtime}, ${report.node}, task-spec ${report.task_spec_sha256.slice(0, 12)}, program ${report.program_sha256.slice(0, 12)}`);
  for (const item of [...cases, ...controls]) {
    console.log(`${item.id}: digest ${item.digest}, JSON ${item.json} -> ${item.verdict} (expected ${item.expected})  grounds ${JSON.stringify(item.grounds['verdict@1'].evidence)}`);
  }
  for (const item of extended) {
    const tail = item.refusal ? `refused: "${item.refusal}"` : `${item.first.verdict} -> ${item.after_overwrite} -> ${item.second.verdict}; verdict@1 grounds still ${JSON.stringify(item.second.grounds['verdict@1'].evidence)}`;
    console.log(`${item.id}: ${tail}  (${item.matches_expected ? 'as expected' : 'NOT as expected'})`);
  }
  console.log(`Runner exit: ${report.all_matched_expected ? 0 : 1} (${report.runner_exit_meaning})`);
}
process.exitCode = report.all_matched_expected ? 0 : 1;
