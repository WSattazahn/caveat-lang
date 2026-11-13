// Round-1 carry (README.md): when a push to a pull request adds nothing but
// lines at the end of ledger/events.jsonl, the runtime jobs that never read
// that file keep the result round 1 gave them on the same tree instead of
// running again. This runs in the regression scope job of
// .github/workflows/runtime.yml on every pull request run. It records the
// tree the run tests, so a later push can carry this run's result, and
// decides whether this run may carry an earlier one. Any doubt, and any
// error, runs everything.
//
//   node scripts/ci-reuse/reuse.mjs
//
// Reads GITHUB_EVENT_PATH, GITHUB_EVENT_NAME, GITHUB_REPOSITORY,
// GITHUB_RUN_ID, GITHUB_TOKEN, GITHUB_API_URL, GITHUB_OUTPUT,
// GITHUB_STEP_SUMMARY, GAMES, LEGACY_DOOR and ROUND1_REUSE. Always exits 0.
import { spawnSync } from 'node:child_process';
import { appendFileSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export const WORKFLOW = '.github/workflows/runtime.yml';
export const LEDGER_EVENTS = 'ledger/events.jsonl';
export const SCOPE_JOB = 'regression scope';
export const CARRY_JOB = 'round-1 carry';
export const CARRY_STEP = "Carry round 1's result";
export const RECORD_TITLE = 'Tested tree';
export const DECISION_TITLE = 'Round-1 carry';
export const MAX_AGE_MINUTES = 60;
export const REQUEST_TIMEOUT_MS = 10_000;

// The jobs that may carry round 1's result, by job id and by the check names
// GitHub shows for them. None of them reads ledger/events.jsonl, and each
// takes the ledger events out of its own checkout in a full run, so a read
// fails there instead of passing unseen. Not here: core semantics replays the
// ledger; packed kit dependency security reads today's advisories, and
// Evidence games browser QA packs the kit it audits; the jobs that end within
// a minute or two of core semantics; and every other workflow.
export const CARRIED = Object.freeze([
  { job: 'native', names: ['native runtime'] },
  { job: 'native-reactive', names: ['native runtime, reactive-only and diagnostic features'] },
  { job: 'compatibility', names: ['rc.15 compatibility and departure gates'] },
  { job: 'lean-conformance', names: ['Lean runtime comparison'] },
  { job: 'reproducible', names: ['reproducible WebAssembly'] },
  { job: 'last-beacon', names: ['The Last Beacon browser QA'] },
  { job: 'light-the-way', names: ['Light the Way (chromium)', 'Light the Way (webkit)'] },
]);
export const CARRIED_NAMES = Object.freeze(CARRIED.flatMap(entry => entry.names));

// A release pull request's final head always runs everything: its version or
// release records change (docs/CONSOLIDATION_PLAN.md, "Run runtime.yml at the
// final review head").
export const isReleaseFile = file => file === 'kit/package.json' || file.startsWith('docs/releases/');

const HEX40 = /^[0-9a-f]{40}$/;
const RECORD = /^tree=([0-9a-f]{40}) head=([0-9a-f]{40}) games=(true|false) legacy_door=(true|false) reuse=(true|false)$/;
const short = revision => String(revision).slice(0, 12);

// The record a pull request run leaves on its regression scope job.
export function formatRecord({ tree, head, games, legacy_door, reuse }) {
  const text = `tree=${tree} head=${head} games=${games} legacy_door=${legacy_door} reuse=${reuse}`;
  return RECORD.test(text) ? text : null;
}

export function parseRecord(text) {
  const match = RECORD.exec(typeof text === 'string' ? text : '');
  return match && { tree: match[1], head: match[2], games: match[3], legacy_door: match[4], reuse: match[5] === 'true' };
}

// Workflow command data: % and line breaks would end or change the command.
export const escapeData = text => String(text).replaceAll('%', '%25').replaceAll('\r', '%0D').replaceAll('\n', '%0A');

// Git in the checkout: the runtime workflow's regression scope job fetches
// all history, so the previous head and the base are here unless a branch
// was rewritten.
export function localGit(cwd = process.cwd()) {
  const git = (args, binary = false) => {
    const result = spawnSync('git', args, { cwd, encoding: binary ? 'buffer' : 'utf8', maxBuffer: 1 << 30 });
    if (result.error) throw result.error;
    return result;
  };
  const text = args => {
    const result = git(args);
    if (result.status !== 0) throw new Error(`git ${args[0]} failed: ${String(result.stderr).trim()}`);
    return result.stdout;
  };
  return {
    hasCommit: revision => git(['cat-file', '-e', `${revision}^{commit}`]).status === 0,
    tree: revision => text(['rev-parse', '--verify', '--end-of-options', `${revision}^{tree}`]).trim(),
    // [{ oldMode, newMode, oldId, newId, status, path }] between two trees.
    diffTrees: (from, to) => {
      const fields = text(['diff-tree', '-r', '--raw', '--no-renames', '--full-index', '-z', from, to]).split('\0');
      const changes = [];
      for (let index = 0; index + 1 < fields.length; index += 2) {
        const [oldMode, newMode, oldId, newId, status] = fields[index].replace(/^:/, '').split(' ');
        changes.push({ oldMode, newMode, oldId, newId, status, path: fields[index + 1] });
      }
      return changes;
    },
    blob: id => {
      const result = git(['cat-file', 'blob', id], true);
      if (result.status !== 0) throw new Error(`git cat-file failed: ${String(result.stderr).trim()}`);
      return result.stdout;
    },
    // The files a pull request changes: from where its head left the base.
    changedFiles: (base, head) => text(['diff', '--name-only', '--no-renames', '-z', `${base}...${head}`]).split('\0').filter(Boolean),
  };
}

// Null when the tree of `to` is the tree of `from` with lines added at the end
// of ledger/events.jsonl and nothing else; otherwise what differs.
export function appendOnly(git, from, to) {
  if (!git.hasCommit(from)) return `the previous head ${short(from)} is not in this checkout (was the branch rewritten?)`;
  const changes = git.diffTrees(`${from}^{tree}`, `${to}^{tree}`);
  if (!changes.length) return 'the tree did not change since the previous head';
  const others = changes.filter(change => change.path !== LEDGER_EVENTS);
  if (others.length) {
    const names = others.slice(0, 3).map(change => change.path).join(', ');
    return `files other than ${LEDGER_EVENTS} changed since the previous head (${names}${others.length > 3 ? ', ...' : ''})`;
  }
  const [change] = changes;
  if (change.status !== 'M' || change.oldMode !== '100644' || change.newMode !== '100644') {
    return `${LEDGER_EVENTS} was not edited in place (${change.status}, mode ${change.oldMode} to ${change.newMode})`;
  }
  const before = git.blob(change.oldId);
  const after = git.blob(change.newId);
  if (after.length <= before.length || !after.subarray(0, before.length).equals(before)) {
    return `${LEDGER_EVENTS} changed other than by lines added at its end`;
  }
  if ((before.length && before[before.length - 1] !== 0x0a) || after[after.length - 1] !== 0x0a) {
    return `${LEDGER_EVENTS} has a line without its newline`;
  }
  return null;
}

const listOf = (value, what) => {
  if (!Array.isArray(value)) throw new Error(`GitHub did not return ${what}`);
  return value;
};

// Whether this run may carry round 1's result. Refusals are checked cheapest
// first, so a run on new code asks GitHub nothing.
export async function decide({ env, event, git, api, now }) {
  const refuse = reason => ({ reuse: false, reason });
  if (env.ROUND1_REUSE === 'off') return refuse('turned off by the repository variable ROUND1_REUSE');
  if (env.GITHUB_EVENT_NAME !== 'pull_request') return refuse(`a ${env.GITHUB_EVENT_NAME || 'unnamed'} run, not a pull request run`);
  if (event.action !== 'synchronize') return refuse(`the pull request was ${event.action || 'changed'}, not pushed to`);
  const pull = event.pull_request;
  const repository = env.GITHUB_REPOSITORY;
  if (!pull || pull.head?.repo?.full_name !== repository) return refuse('the pull request comes from another repository');
  if (!HEX40.test(event.before ?? '')) return refuse('the push names no previous head');
  if (!HEX40.test(pull.head?.sha ?? '') || !HEX40.test(pull.base?.sha ?? '')) return refuse('the pull request names no head or base revision');
  const release = git.changedFiles(pull.base.sha, pull.head.sha).filter(isReleaseFile);
  if (release.length) return refuse(`a release pull request (it changes ${release.slice(0, 2).join(', ')}), so its head runs everything`);
  const content = appendOnly(git, event.before, 'HEAD');
  if (content) return refuse(content);

  const runs = listOf((await api(`/repos/${repository}/actions/workflows/runtime.yml/runs?event=pull_request&head_sha=${event.before}&per_page=100`)).workflow_runs, 'workflow runs');
  const earlier = runs.filter(run => run.id < Number(env.GITHUB_RUN_ID) && run.path === WORKFLOW && run.head_sha === event.before
    && run.head_branch === pull.head.ref && run.head_repository?.full_name === repository);
  if (!earlier.length) return refuse(`no earlier run of this pull request tested the previous head ${short(event.before)}`);
  const round1 = earlier.reduce((newest, run) => (run.id > newest.id ? run : newest));
  const url = `${round1.html_url}/attempts/${round1.run_attempt}`;
  if (round1.status !== 'completed' || round1.conclusion !== 'success') {
    return refuse(`round 1 (${url}) is ${round1.status}${round1.conclusion ? `, ${round1.conclusion}` : ''}, not a success`);
  }
  const finished = Date.parse(round1.updated_at);
  if (!Number.isFinite(finished) || now - finished > MAX_AGE_MINUTES * 60_000) {
    return refuse(`round 1 (${url}) finished more than ${MAX_AGE_MINUTES} minutes ago`);
  }
  const jobs = listOf((await api(`/repos/${repository}/actions/runs/${round1.id}/jobs?filter=latest&per_page=100`)).jobs, 'jobs');
  const scope = jobs.find(job => job.name === SCOPE_JOB);
  if (!scope) return refuse(`round 1 (${url}) has no ${SCOPE_JOB} job`);
  const records = listOf(await api(`/repos/${repository}/check-runs/${scope.id}/annotations?per_page=100`), 'annotations')
    .filter(annotation => annotation.annotation_level === 'notice' && annotation.title === RECORD_TITLE);
  if (records.length !== 1) return refuse(`round 1 (${url}) left ${records.length} tested-tree records, not one`);
  const record = parseRecord(records[0].message);
  if (!record) return refuse(`round 1's tested-tree record does not read: ${records[0].message}`);
  if (record.head !== event.before) return refuse(`round 1 recorded head ${short(record.head)}, not the previous head ${short(event.before)}`);
  if (record.tree !== git.tree(event.before)) {
    return refuse('round 1 tested a merge with another main than this head holds (main moved, or the head was behind it)');
  }
  if (record.reuse) return refuse('round 1 itself carried an earlier result');
  if (record.games !== env.GAMES || record.legacy_door !== env.LEGACY_DOOR) return refuse('round 1 decided a different set of browser QA jobs');
  for (const name of CARRIED_NAMES) {
    const job = jobs.find(candidate => candidate.name === name);
    if (job?.conclusion !== 'success') return refuse(`round 1's ${name} did not run to success`);
    const carry = (job.steps ?? []).find(step => step.name === CARRY_STEP);
    if (carry?.conclusion !== 'skipped') return refuse(`round 1's ${name} did not run in full`);
  }
  return {
    reuse: true, url, tree: record.tree, finished: round1.updated_at,
    reason: `round 1 (${url}) ran these jobs to success on tree ${record.tree}; this push only adds lines at the end of ${LEDGER_EVENTS}`,
  };
}

export function githubApi(env) {
  const base = env.GITHUB_API_URL || 'https://api.github.com';
  if (!env.GITHUB_TOKEN) throw new Error('GITHUB_TOKEN is not set');
  return async request => {
    const response = await fetch(`${base}${request}`, {
      headers: { authorization: `Bearer ${env.GITHUB_TOKEN}`, accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28' },
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    });
    if (!response.ok) throw new Error(`GitHub answered ${response.status} for ${request.replace(/\?.*$/, '')}`);
    return response.json();
  };
}

export async function main({ env = process.env, now = Date.now(), git = localGit(), api, log = console.log } = {}) {
  let record = null;
  let decision;
  try {
    const event = JSON.parse(readFileSync(env.GITHUB_EVENT_PATH, 'utf8'));
    record = formatRecord({ tree: git.tree('HEAD'), head: event.pull_request?.head?.sha, games: env.GAMES, legacy_door: env.LEGACY_DOOR, reuse: false });
    decision = await decide({ env, event, git, api: api ?? githubApi(env), now });
  } catch (error) {
    decision = { reuse: false, reason: `the decision failed (${error.message})` };
    log(`::warning title=${DECISION_TITLE}::${escapeData(`Running everything: ${decision.reason}`)}`);
  }
  // The record is left even when this run carries nothing, so the next push
  // can carry this run's result; a carrying run records reuse=true, and no
  // later run carries from it.
  if (record) log(`::notice title=${RECORD_TITLE}::${decision.reuse ? record.replace(/reuse=false$/, 'reuse=true') : record}`);
  else log(`::warning title=${DECISION_TITLE}::This run left no tested-tree record, so the next push runs everything.`);
  log(`::notice title=${DECISION_TITLE}::${escapeData(decision.reuse ? `Carrying round 1's result: ${decision.reason}.` : `Running everything: ${decision.reason}.`)}`);
  const outputs = decision.reuse
    ? { reuse: 'true', url: decision.url, tree: decision.tree, finished: decision.finished, names: CARRIED_NAMES.join('; ') }
    : { reuse: 'false', url: '', tree: '', finished: '', names: '' };
  if (env.GITHUB_STEP_SUMMARY) {
    appendFileSync(env.GITHUB_STEP_SUMMARY, `### Round-1 carry\n\n${decision.reuse ? 'Carrying' : 'Running everything'}: ${decision.reason}.\n`);
  }
  // One write, last: a half-written reuse=true cannot happen.
  if (env.GITHUB_OUTPUT) appendFileSync(env.GITHUB_OUTPUT, `${Object.entries(outputs).map(([key, value]) => `${key}=${value}`).join('\n')}\n`);
  return { decision, record };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  main().catch(error => console.log(`::warning title=${DECISION_TITLE}::${escapeData(`Running everything: ${error.message}`)}`));
}
