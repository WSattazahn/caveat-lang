// Weekly guard for the game browser QA retries: a retry turns a flaky
// red X into a warning, so count the warnings and open (or update) an
// issue for any job and command that needed its retry too often.
// Keep counting separate from GitHub's transport so it is testable.
import { pathToFileURL } from 'node:url';

export const RETRY_MARK = 'trying once more';
export const DAYS = 7;
export const THRESHOLD = 2; // more than this many retries in DAYS opens an issue

function requireValue(condition, message) {
  if (!condition) throw new Error(message);
}

// What was retried, from a warning the retry wrappers leave:
//   'npm run test:beacon' failed (exit 1); trying once more
//   playwright install webkit failed or stalled; trying once more
export function retriedCommand(message) {
  if (typeof message !== 'string' || !message.trim().endsWith(RETRY_MARK)) return null;
  return message.replace(/ failed\b.*$/s, '').replace(/^'(.*)'$/, '$1').trim();
}

// checkRuns: [{ name, html_url, annotations: [{ annotation_level, message }] }]
export function tallyRetries(checkRuns) {
  const counts = new Map();
  for (const run of checkRuns) {
    for (const annotation of run.annotations || []) {
      if (annotation.annotation_level !== 'warning') continue;
      const command = retriedCommand(annotation.message);
      if (!command) continue;
      const key = `${run.name}\u0000${command}`;
      const entry = counts.get(key) || { job: run.name, command, count: 0, examples: [] };
      entry.count += 1;
      if (entry.examples.length < 10 && run.html_url) entry.examples.push(run.html_url);
      counts.set(key, entry);
    }
  }
  return [...counts.values()].sort((a, b) => b.count - a.count || a.job.localeCompare(b.job));
}

export function issueTitle({ job, command }) {
  return `Game QA retries: ${job} / ${command}`;
}

export function issueBody(entry, { days = DAYS, threshold = THRESHOLD } = {}) {
  return [
    `\`${entry.command}\` in **${entry.job}** needed its automatic retry ${entry.count} times in the last ${days} days (more than ${threshold} opens this issue).`,
    '',
    'A retry hides a red X, so a test that is getting flakier shows up here instead. Find the cause; do not raise the limit.',
    '',
    'Runs that retried:',
    ...entry.examples.map(url => `- ${url}`),
  ].join('\n');
}

export function overThreshold(tally, threshold = THRESHOLD) {
  return tally.filter(entry => entry.count > threshold);
}

async function pages(api, path, key) {
  const values = [];
  for (let page = 1; ; page++) {
    const response = await api(`${path}${path.includes('?') ? '&' : '?'}per_page=100&page=${page}`);
    const list = key ? response[key] : response;
    requireValue(Array.isArray(list), `GitHub did not return ${key || 'a list'} for ${path}.`);
    values.push(...list);
    if (list.length < 100) return values;
  }
}

// runtime.yml plays the games on every change; pages.yml plays the live site.
export const WORKFLOWS = ['runtime.yml', 'pages.yml'];

export async function collect({ api, repository, since }) {
  const runs = [];
  for (const workflow of WORKFLOWS) {
    runs.push(...await pages(api,
      `/repos/${repository}/actions/workflows/${workflow}/runs?created=%3E%3D${since.toISOString().slice(0, 10)}`,
      'workflow_runs'));
  }
  const checkRuns = [];
  for (const run of runs) {
    if (!run.check_suite_id) continue;
    const suiteRuns = await pages(api, `/repos/${repository}/check-suites/${run.check_suite_id}/check-runs?filter=all`, 'check_runs');
    for (const checkRun of suiteRuns) {
      if (!(checkRun.output?.annotations_count > 0)) continue;
      const annotations = await pages(api, `/repos/${repository}/check-runs/${checkRun.id}/annotations`);
      checkRuns.push({ name: checkRun.name, html_url: checkRun.html_url, annotations });
    }
  }
  return checkRuns;
}

export async function report({ api, repository, tally, threshold = THRESHOLD, log = console.log }) {
  const flagged = overThreshold(tally, threshold);
  if (!flagged.length) return [];
  const open = await pages(api, `/repos/${repository}/issues?state=open`);
  const actions = [];
  for (const entry of flagged) {
    const title = issueTitle(entry);
    const body = issueBody(entry, { threshold });
    const existing = open.find(issue => !issue.pull_request && issue.title === title);
    if (existing) {
      await api(`/repos/${repository}/issues/${existing.number}/comments`, { method: 'POST', body: { body } });
      actions.push({ action: 'commented', number: existing.number, title });
    } else {
      const created = await api(`/repos/${repository}/issues`, { method: 'POST', body: { title, body } });
      actions.push({ action: 'opened', number: created.number, title });
    }
    log(`${actions.at(-1).action} #${actions.at(-1).number}: ${title}`);
  }
  return actions;
}

function githubApi(token) {
  requireValue(token, 'GITHUB_TOKEN is required.');
  const base = process.env.GITHUB_API_URL || 'https://api.github.com';
  return async (path, { method = 'GET', body } = {}) => {
    const response = await fetch(`${base}${path}`, { method, headers: {
      authorization: `Bearer ${token}`,
      accept: 'application/vnd.github+json',
      'X-GitHub-Api-Version': '2022-11-28',
      ...(body ? { 'content-type': 'application/json' } : {}),
    }, body: body ? JSON.stringify(body) : undefined });
    requireValue(response.ok, `GitHub request failed (${response.status}) for ${method} ${path}.`);
    return response.json();
  };
}

async function main() {
  const repository = process.env.GITHUB_REPOSITORY;
  requireValue(/^[\w.-]+\/[\w.-]+$/.test(repository || ''), 'Invalid GITHUB_REPOSITORY.');
  const api = githubApi(process.env.GITHUB_TOKEN);
  const since = new Date(Date.now() - DAYS * 24 * 60 * 60 * 1000);
  const tally = tallyRetries(await collect({ api, repository, since }));
  console.log(tally.length
    ? tally.map(entry => `${entry.count}\t${entry.job}\t${entry.command}`).join('\n')
    : `No game QA retries since ${since.toISOString()}.`);
  await report({ api, repository, tally });
}

if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
