// Is a pull request done? Fetches what GitHub says about it, sends each fact
// to done.cav, and prints the program's verdict with its reasons. Advisory:
// it exits 0 whatever the verdict unless --strict is given.
//
//   node scripts/done-check/done.mjs PR [--explain] [--json] [--strict]
//
// GITHUB_TOKEN is used when set; in a Claude session, run it with
// NODE_USE_ENV_PROXY=1 NODE_EXTRA_CA_CERTS=/root/.ccr/ca-bundle.crt and no
// GITHUB_TOKEN so requests go through the session's proxy.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
export const PROGRAM = path.join(here, 'done.cav');
export const REPOSITORY = 'WSattazahn/caveat-lang';
// The CI job that runs the corpus and save sweeps (departure, rc.15
// compatibility, View 0.2 delta) over every tracked program.
export const SWEEP_CHECK = 'rc.15 compatibility and departure gates';
const LEDGER = 'ledger/events.jsonl';
const PASSING = new Set(['success', 'neutral', 'skipped']);

async function pages(api, url, key, stop = () => false) {
  const values = [];
  for (let page = 1; ; page++) {
    const response = await api(`${url}${url.includes('?') ? '&' : '?'}per_page=100&page=${page}`);
    const list = key ? response[key] : response;
    assert.ok(Array.isArray(list), `GitHub did not return ${key ?? 'a list'} for ${url}.`);
    values.push(...list);
    if (list.length < 100 || stop(list)) return values;
  }
}

export const touchesRuntime = name => name.startsWith('runtime/') || name.startsWith('spec/');
export const checkOutcome = run => (run.status !== 'completed' ? 'running' : PASSING.has(run.conclusion) ? 'passed' : 'failed');

// The pull requests merged to main after the one that last changed the ledger.
export async function ledgerOwed(api, repository, exclude) {
  const [last] = await api(`/repos/${repository}/commits?sha=main&path=${LEDGER}&per_page=1`);
  if (!last) return { since: null, owed: [] };
  const carriers = await api(`/repos/${repository}/commits/${last.sha}/pulls`);
  const carrier = carriers.find(pull => pull.merged_at && pull.base?.ref === 'main');
  const since = carrier?.merged_at ?? last.commit?.committer?.date;
  const closed = await pages(api, `/repos/${repository}/pulls?state=closed&base=main&sort=updated&direction=desc`, null,
    list => list.every(pull => pull.updated_at < since));
  const owed = closed.filter(pull => pull.merged_at && pull.merged_at > since && pull.number !== carrier?.number && pull.number !== exclude)
    .map(pull => pull.number).sort((a, b) => a - b);
  return { since, carrier: carrier?.number ?? null, owed };
}

export async function loadKit(directory = process.env.CAVEAT_GATE_KIT || path.join(here, '..', '..', 'kit')) {
  const lib = name => pathToFileURL(path.join(directory, 'lib', name)).href;
  const [node, explain] = await Promise.all([import(lib('node.mjs')), import(lib('explain.mjs'))]);
  return { runtime: await node.loadRuntimeFromDirectory(node.defaultRuntimeDirectory()), explain };
}

// The facts for one pull request, in the order the program takes them.
export async function gatherFacts({ api, repository = REPOSITORY, number }) {
  const pull = await api(`/repos/${repository}/pulls/${number}`);
  const head = pull.head.sha;
  const comparison = await api(`/repos/${repository}/compare/${pull.base.ref}...${head}`);
  const runs = await pages(api, `/repos/${repository}/commits/${head}/check-runs?filter=latest`, 'check_runs');
  const files = (await pages(api, `/repos/${repository}/pulls/${number}/files`, null)).map(file => file.filename);
  const ledger = await ledgerOwed(api, repository, number);
  const events = [
    { event: 'head', payload: { contains_main: comparison.behind_by === 0 ? 1 : 0 } },
    ...runs.map(run => ({ event: 'check', payload: { outcome: checkOutcome(run), sweep: run.name === SWEEP_CHECK ? 1 : 0 } })),
    { event: 'files', payload: { runtime: files.some(touchesRuntime) ? 1 : 0 } },
    { event: 'ledger', payload: { owed: Math.min(ledger.owed.length, 999) } },
    { event: 'decide', payload: {} },
  ];
  return { pull: { number, head, base: pull.base.ref, behind_by: comparison.behind_by, title: pull.title, merged_at: pull.merged_at ?? null },
    runs: runs.map(run => ({ name: run.name, outcome: checkOutcome(run) })), files, ledger, changesLedger: files.includes(LEDGER), events };
}

// Sends the facts and returns the verdict, its reasons and the record.
export async function decide({ facts, kit }) {
  const session = kit.runtime.open(await readFile(PROGRAM, 'utf8'));
  const sent = [];
  let snapshot = session.snapshot();
  try {
    for (const { event, payload } of facts.events) {
      const { snapshot: after, ...outcome } = session.dispatch(event, payload);
      sent.push({ event, payload, outcome, ...(outcome.outcome === 'accepted' ? { sequence: after.sequence, effects: after.effects } : {}) });
      assert.equal(outcome.outcome, 'accepted', `done.cav refused ${event}: ${outcome.message}`);
      snapshot = after;
    }
  } finally {
    session.close();
  }
  const pr = snapshot.bindings.pr;
  return { ready: pr.verdict.startsWith('ready'), reasons: pr, explanation: kit.explain.formatExplanation(kit.explain.explain(snapshot, sent), 'done.cav'), sent };
}

function githubApi() {
  const base = process.env.GITHUB_API_URL || 'https://api.github.com';
  const headers = { accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28' };
  if (process.env.GITHUB_TOKEN) headers.authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  return async url => {
    const response = await fetch(`${base}${url}`, { headers });
    assert.ok(response.ok, `GitHub request failed (${response.status}) for ${url}.`);
    return response.json();
  };
}

export function report(facts, verdict) {
  const { pull, ledger } = facts;
  const failing = facts.runs.filter(run => run.outcome !== 'passed').map(run => `${run.name} (${run.outcome})`);
  const behind = pull.behind_by ? ` (${pull.behind_by} commit${pull.behind_by === 1 ? '' : 's'} behind ${pull.base})` : '';
  return [
    `#${pull.number} at ${pull.head.slice(0, 7)}: ${verdict.reasons.verdict}${pull.merged_at ? ` (already merged, ${pull.merged_at})` : ''}`,
    `  main: ${verdict.reasons.main}${behind}`,
    `  checks: ${verdict.reasons.checks}${failing.length ? `: ${failing.join(', ')}` : ''}`,
    `  sweep: ${verdict.reasons.sweep}`,
    `  ledger: ${verdict.reasons.ledger}${ledger.owed.length ? ` (${ledger.owed.map(n => `#${n}`).join(', ')}, merged since #${ledger.carrier ?? '?'} changed it)` : ''}`
      + (facts.changesLedger ? '; this PR changes the ledger, so check it carries them' : ''),
  ].join('\n');
}

async function main() {
  const args = process.argv.slice(2);
  const number = Number(args.find(arg => /^\d+$/.test(arg)));
  assert.ok(Number.isSafeInteger(number), 'Usage: node scripts/done-check/done.mjs PR [--explain] [--json] [--strict]');
  const facts = await gatherFacts({ api: githubApi(), number });
  const verdict = await decide({ facts, kit: await loadKit() });
  if (args.includes('--json')) console.log(JSON.stringify({ pull: facts.pull, ready: verdict.ready, reasons: verdict.reasons, owed: facts.ledger.owed }, null, 2));
  else console.log(report(facts, verdict));
  if (args.includes('--explain')) console.log(`\n${verdict.explanation}`);
  if (args.includes('--strict') && !verdict.ready) process.exitCode = 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  main().catch(error => { console.error(error.message); process.exitCode = 2; });
}
