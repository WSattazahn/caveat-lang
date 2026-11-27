// Which files of a pull request does a person still need to check? Reads what
// GitHub says about the pull request and what the repository's tests and specs
// say about its files, sends each fact to a copy of shortlist.cav.template,
// and prints the files that rest on nothing but the word of the agent that
// wrote them, or on a test or spec that has changed since.
//
//   node scripts/review-shortlist/shortlist.mjs PR [--json] [--out DIR]
//
// Run it in a clone of the repository: it fetches the pull request's head
// and main with git. --out writes program.cav, events.jsonl and facts.json,
// so `caveat explain`, `caveat dependents` and `caveat undo-list` work on the
// result by hand. GITHUB_REPOSITORY names the repository (default
// WSattazahn/caveat-lang), GITHUB_TOKEN is used when set; in a Claude session
// run it with NODE_USE_ENV_PROXY=1 NODE_EXTRA_CA_CERTS=/root/.ccr/ca-bundle.crt.
// The runtime is this repository's kit after `npm run build`, or the
// caveat-lang package folder named by CAVEAT_SHORTLIST_KIT.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { authorOf, checkRunOk, githubApi, pages, statusOk } from '../../actions/pr-ledger/pr-ledger.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
export const TEMPLATE = path.join(here, 'shortlist.cav.template');
const MAX_FILES = 300;   // each file's decision series declares two revisions of the program's 1,024
const SPEC_NAME = /\b(caveat-[a-z0-9-]+-\d+\.\d+(?:\.\d+)?)\.md\b/g;
const AGENT_FOOTER = /generated (?:with|by) \[?claude code|🤖 generated with/i;

// A test file: test code under a tests folder or named for tests, or a
// scenarios file. Fixtures, recorded results and prose are not tests.
const TEST_CODE = /\.(?:mjs|cjs|js|mts|ts|py|rs|sh)$/;
export const isTest = name => name.endsWith('.scenarios.json') || (TEST_CODE.test(name)
  && !/(^|\/)(fixtures|results|runs)\//.test(name)
  && (/(^|\/)(tests?|__tests__)\//.test(name) || /(^|[._-])tests?([._-]|$)/.test(path.basename(name))));
export const isSpec = name => /^spec\/[^/]+\.md$/.test(name);

// How a test names a file: by its base name with the extension, which is how
// an import, a fixture path or a command line spells it. A name too generic
// to mean one file (index.js, mod.rs) names nothing.
const GENERIC = new Set(['index', 'readme', 'main', 'mod', 'lib', 'package', 'action', 'test', 'tests', 'types', 'utils', 'build', 'config', 'license', 'changelog']);
export function nameOf(name) {
  const base = path.basename(name);
  const stem = base.replace(/(\.d)?\.[^.]+$/, '');
  return base.includes('.') && stem.length >= 4 && !GENERIC.has(stem.toLowerCase()) ? base : null;
}

// A person, not an agent: not a bot, and not text an agent posted through a
// person's account (it carries the agent's footer).
const byPerson = item => item.user && item.user.type !== 'Bot' && !item.user.login?.endsWith('[bot]') && !AGENT_FOOTER.test(item.body ?? '');

// ---- What GitHub says ------------------------------------------------------

export async function gatherPull({ api, repository, number }) {
  const pull = await api(`/repos/${repository}/pulls/${number}`);
  const head = pull.head.sha;
  const commits = await pages(api, `/repos/${repository}/pulls/${number}/commits`, null);
  const files = await pages(api, `/repos/${repository}/pulls/${number}/files`, null);
  const runs = await pages(api, `/repos/${repository}/commits/${head}/check-runs?filter=latest`, 'check_runs');
  const statuses = (await api(`/repos/${repository}/commits/${head}/status`)).statuses ?? [];
  const reviews = await pages(api, `/repos/${repository}/pulls/${number}/reviews`, null);
  const comments = await pages(api, `/repos/${repository}/pulls/${number}/comments`, null);
  const results = [...runs.map(checkRunOk), ...statuses.map(statusOk)].filter(ok => ok !== null);
  const author = authorOf(pull, commits);
  const approvers = new Map();
  for (const review of reviews) {
    if (!byPerson(review)) continue;
    if (review.state === 'APPROVED' && review.commit_id === head) approvers.set(review.user.login, true);
    else if (['CHANGES_REQUESTED', 'DISMISSED'].includes(review.state)) approvers.delete(review.user.login);
  }
  return {
    number, title: pull.title, head, base: pull.base.ref, merged_at: pull.merged_at ?? null, merge_commit: pull.merged_at ? pull.merge_commit_sha : null,
    author: author.name, agent: author.agent,
    cites: [...new Set([...(pull.body ?? '').matchAll(SPEC_NAME)].map(match => `spec/${match[1]}.md`))],
    files: files.map(file => ({ name: file.filename, status: file.status })),
    checks: results.length > 0 && results.every(ok => ok === 1) ? 1 : 0,
    checks_seen: results.length,
    approved_by: [...approvers.keys()],
    commented: [...new Set(comments.filter(byPerson).map(comment => comment.path))],
  };
}

// ---- What the repository says ----------------------------------------------

export function gitIn(cwd) {
  return (...args) => execFileSync('git', args, { cwd, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 });
}

// The tests that name each changed file and the specs each one names, at the
// pull request's head, and which of them changed on main since the pull
// request merged (or, while it is open, since it branched).
export function gatherLinks({ git, pull, main = 'origin/main' }) {
  const changed = new Set(pull.files.map(file => file.name));
  const tree = git('ls-tree', '-r', '--name-only', pull.head).split('\n').filter(Boolean);
  const tests = tree.filter(isTest);
  const specs = new Set(tree.filter(isSpec));
  const named = [];
  const stems = new Map();
  for (const file of pull.files) {
    const stem = nameOf(file.name);
    if (stem) stems.set(file.name, stem);
  }
  const wanted = [...new Set(stems.values())];
  const hits = new Map();   // test -> stems it mentions
  for (let i = 0; i < wanted.length && tests.length; i += 200) {
    const patterns = wanted.slice(i, i + 200).flatMap(stem => ['-e', stem]);
    let out = '';
    try {
      out = git('grep', '-o', '-F', ...patterns, pull.head, '--', ...tests);
    } catch (error) {
      if (error.status !== 1) throw error;   // 1: no match
    }
    for (const line of out.split('\n').filter(Boolean)) {
      const [, test, stem] = /^[^:]+:(.+):([^:]+)$/.exec(line) ?? [];
      if (!test) continue;
      if (!hits.has(test)) hits.set(test, new Set());
      hits.get(test).add(stem);
    }
  }
  for (const file of pull.files) {
    const stem = stems.get(file.name);
    if (!stem) continue;
    for (const [test, found] of hits) {
      if (test !== file.name && found.has(stem)) named.push({ file: file.name, test, origin: changed.has(test) ? 'added' : 'before' });
    }
  }
  const cited = [];
  for (const file of pull.files) {
    if (file.status === 'removed') continue;
    const text = git('show', `${pull.head}:${file.name}`);
    // A file rests on a spec it names that the pull request was built
    // against: one it changes, or one its description cites. Docs name many
    // specs in passing; those are not what the change rests on.
    const found = new Set([...text.matchAll(SPEC_NAME)].map(match => `spec/${match[1]}.md`));
    for (const spec of found) {
      if (specs.has(spec) && spec !== file.name && (changed.has(spec) || pull.cites.includes(spec))) {
        cited.push({ file: file.name, spec, origin: changed.has(spec) ? 'added' : 'before' });
      }
    }
  }
  const since = pull.merge_commit ?? git('merge-base', pull.head, main).trim();
  const later = [];
  for (const name of new Set([...named.map(link => link.test), ...cited.map(link => link.spec)])) {
    const [commit] = git('log', '--format=%h %s', `${since}..${main}`, '--', name).split('\n').filter(Boolean).reverse();
    if (commit) later.push({ name, kind: isSpec(name) ? 'spec' : 'test', commit });
  }
  return { named, cited, later };
}

// ---- The program and its events --------------------------------------------

const comment = text => text.replace(/[\r\n]/g, ' ');

export function buildFacts(pull, links) {
  assert.ok(pull.files.length <= MAX_FILES, `#${pull.number} changes ${pull.files.length} files; the shortlist takes at most ${MAX_FILES}.`);
  const ids = { file: new Map(), test: new Map(), spec: new Map() };
  const id = (kind, prefix, name) => {
    if (!ids[kind].has(name)) ids[kind].set(name, `${prefix}${ids[kind].size + 1}`);
    return ids[kind].get(name);
  };
  for (const file of pull.files) id('file', 'f', file.name);
  for (const link of links.named) id('test', 't', link.test);
  for (const link of links.cited) id('spec', 's', link.spec);
  const by = pull.agent ? 'agent' : 'person';
  const events = [
    { event: 'checks', payload: { ok: pull.checks } },
    ...pull.files.map(file => ({ event: 'wrote', payload: { file: ids.file.get(file.name), by } })),
    ...links.named.map(link => ({ event: 'named', payload: { file: ids.file.get(link.file), test: ids.test.get(link.test), origin: link.origin } })),
    ...links.cited.map(link => ({ event: 'cited', payload: { file: ids.file.get(link.file), spec: ids.spec.get(link.spec), origin: link.origin } })),
    ...pull.commented.filter(name => ids.file.has(name)).map(name => ({ event: 'commented', payload: { file: ids.file.get(name) } })),
    ...(pull.approved_by.length ? [{ event: 'approved', payload: {} }] : []),
    ...pull.files.map(file => ({ event: 'decide', payload: { file: ids.file.get(file.name) } })),
    ...links.later.map(change => change.kind === 'test'
      ? { event: 'test_changed', payload: { test: ids.test.get(change.name) } }
      : { event: 'spec_changed', payload: { spec: ids.spec.get(change.name) } }),
  ];
  const names = Object.fromEntries(Object.entries(ids).map(([kind, map]) => [kind, Object.fromEntries([...map].map(([name, key]) => [key, name]))]));
  return { pull, links, names, events };
}

export function renderProgram(template, facts) {
  const entities = (kind, place, none) => {
    const rows = Object.entries(facts.names[kind]).map(([key, name]) => `entity ${key} kind ${kind} at ${place};  # ${comment(name)}`);
    return rows.length ? rows.join('\n') : `entity ${none} kind ${kind} at ${place};  # none`;
  };
  return template.replace('# @FILES@', entities('file', 'pr', 'no_file'))
    .replace('# @TESTS@', entities('test', 'suite', 'no_test'))
    .replace('# @SPECS@', entities('spec', 'docs', 'no_spec'));
}

export async function loadKit(directory = process.env.CAVEAT_SHORTLIST_KIT || path.join(here, '..', '..', 'kit')) {
  const lib = name => pathToFileURL(path.join(directory, 'lib', name)).href;
  const node = await import(lib('node.mjs'));
  return { runtime: await node.loadRuntimeFromDirectory(node.defaultRuntimeDirectory()) };
}

// Sends the facts and returns, for each file, whether it is on the list and why.
export function decide({ kit, template, facts }) {
  const session = kit.runtime.open(renderProgram(template, facts));
  let snapshot = session.snapshot();
  try {
    for (const { event, payload } of facts.events) {
      const { snapshot: after, ...outcome } = session.dispatch(event, payload);
      assert.equal(outcome.outcome, 'accepted', `the program refused ${event} ${JSON.stringify(payload)}: ${outcome.message}`);
      snapshot = after;
    }
  } finally {
    session.close();
  }
  const rows = Object.entries(facts.names.file).map(([key, name]) => {
    const tests = facts.links.named.filter(link => link.file === name);
    const specs = facts.links.cited.filter(link => link.file === name);
    const changed = facts.links.later.filter(change => tests.some(link => link.test === change.name) || specs.some(link => link.spec === change.name));
    return { file: name, listed: snapshot.bindings[key].listed === 1, why: snapshot.bindings[key].why, tests, specs, changed };
  });
  return { rows, grounds: snapshot.commitment_grounds };
}

export function report(facts, { rows }) {
  const { pull } = facts;
  const listed = rows.filter(row => row.listed);
  const lines = [
    `#${pull.number} ${pull.title}`,
    `  written by ${pull.author}${pull.agent ? ' (an agent)' : ''}; ${pull.checks ? `all ${pull.checks_seen} check results passed` : pull.checks_seen ? 'not every check passed' : 'no check results'} on ${pull.head.slice(0, 7)}; `
      + (pull.approved_by.length ? `approved at its head by ${pull.approved_by.join(', ')}` : 'no approval from a person at its head'),
    '',
    listed.length ? `Check these ${listed.length} of ${rows.length} files:` : `Nothing to check: none of its ${rows.length} files rests on the agent's word alone.`,
  ];
  for (const row of listed) {
    lines.push(`  ${row.file}`, `      ${row.why}`);
    for (const change of row.changed.slice(0, 3)) lines.push(`      ${change.kind} ${change.name} changed in ${change.commit}`);
    if (row.changed.length > 3) lines.push(`      and ${row.changed.length - 3} more changed`);
    const added = row.tests.filter(link => link.origin === 'added').map(link => link.test);
    if (added.length && !row.changed.length) lines.push(`      tests it wrote: ${added.join(', ')}`);
  }
  const off = rows.filter(row => !row.listed);
  if (off.length) {
    lines.push('', 'Off the list:');
    for (const row of off) {
      const prior = row.tests.filter(link => link.origin === 'before').map(link => link.test);
      lines.push(`  ${row.file}: ${row.why}${prior.length && row.why.startsWith('tests') ? ` (${prior.join(', ')})` : ''}`);
    }
  }
  return lines.join('\n');
}

export async function main(args, { api = githubApi(), git = gitIn(process.cwd()), kit, log = console.log } = {}) {
  const number = Number(args.find(arg => /^\d+$/.test(arg)));
  assert.ok(Number.isSafeInteger(number), 'Usage: node scripts/review-shortlist/shortlist.mjs PR [--json] [--out DIR]');
  const repository = process.env.GITHUB_REPOSITORY || 'WSattazahn/caveat-lang';
  const pull = await gatherPull({ api, repository, number });
  git('fetch', '--quiet', 'origin', `+refs/pull/${number}/head:refs/remotes/origin/pr/${number}`, `+refs/heads/main:refs/remotes/origin/main`);
  const facts = buildFacts(pull, gatherLinks({ git, pull }));
  const template = await readFile(TEMPLATE, 'utf8');
  const result = decide({ kit: kit ?? await loadKit(), template, facts });
  const out = args.indexOf('--out');
  if (out >= 0) {
    const dir = path.resolve(args[out + 1]);
    await mkdir(dir, { recursive: true });
    await writeFile(path.join(dir, 'program.cav'), renderProgram(template, facts));
    await writeFile(path.join(dir, 'events.jsonl'), facts.events.map(event => JSON.stringify(event)).join('\n') + '\n');
    await writeFile(path.join(dir, 'facts.json'), `${JSON.stringify({ pull, links: facts.links, names: facts.names }, null, 1)}\n`);
  }
  if (args.includes('--json')) log(JSON.stringify({ pull: { number, title: pull.title, head: pull.head, author: pull.author, agent: pull.agent }, files: result.rows }, null, 2));
  else log(report(facts, result));
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  main(process.argv.slice(2)).then(status => { process.exitCode = status; }, error => { console.error(error.message); process.exitCode = 2; });
}
