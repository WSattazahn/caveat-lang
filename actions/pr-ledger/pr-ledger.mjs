// A record of merged pull requests kept in Caveat: what each merge rested on
// (its passing checks, its approving reviews and the agent or person that
// wrote it), and which merges rested on something later found wrong.
//
//   node pr-ledger.mjs record PR [--dir DIR]
//   node pr-ledger.mjs backfill [--last N] [--dir DIR]
//   node pr-ledger.mjs wrong (--check NAME | --review LOGIN | --author NAME) [--pr N | --from N --to N] [--note TEXT] [--dir DIR]
//   node pr-ledger.mjs rested-on (--check NAME | --review LOGIN | --author NAME) [--pr N] [--dir DIR]
//   node pr-ledger.mjs affected [--dir DIR]
//   node pr-ledger.mjs show PR [--dir DIR]
//
// The record is DIR (default pr-ledger/): one folder per chapter, each with
// chapter.json (the record), and program.cav and events.jsonl rendered from
// it, so `caveat explain DIR/chapter-001/program.cav DIR/chapter-001/events.jsonl`
// works by hand. A chapter holds what one program's declared history can:
// 1,024 across its decision revisions and its sources' readings, so at most
// 64 merges, and fewer when the merges rest on more than 15 sources.
//
// The runtime is the released caveat-lang package in CAVEAT_PR_LEDGER_KIT
// (action.yml installs it, pinned by checksum), or this repository's kit.
// GITHUB_REPOSITORY names the repository, GITHUB_TOKEN is used when set.
import assert from 'node:assert/strict';
import { mkdir, readdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
export const TEMPLATE = path.join(here, 'pr-ledger.cav.template');
export const SCHEMA = 'caveat-pr-ledger/0.1';
const CAPACITY = 1024;     // declared history capacity of one program: readings plus decision revisions
const MERGES = 64;         // merges per chapter: the `main` series' limit
const FINDINGS = 256;      // findings per chapter: the largest renewable limit

// ---- What GitHub says about one merged pull request -----------------------

const CHECK_PASSED = new Set(['success']);
const CHECK_FAILED = new Set(['failure', 'cancelled', 'timed_out', 'action_required', 'startup_failure']);
const STATUS_FAILED = new Set(['failure', 'error']);

// 1 passed, 0 failed, null when it says nothing (skipped, neutral, still running).
export function checkRunOk(run) {
  if (run.status !== 'completed') return null;
  if (CHECK_PASSED.has(run.conclusion)) return 1;
  return CHECK_FAILED.has(run.conclusion) ? 0 : null;
}
export const statusOk = status => (status.state === 'success' ? 1 : STATUS_FAILED.has(status.state) ? 0 : null);

// The latest review of each reviewer that approved or asked for changes.
export function reviewVerdicts(reviews) {
  const latest = new Map();
  for (const review of reviews) {
    const login = review.user?.login;
    if (!login) continue;
    if (review.state === 'APPROVED') latest.set(login, 1);
    else if (review.state === 'CHANGES_REQUESTED') latest.set(login, 0);
    else if (review.state === 'DISMISSED') latest.delete(login);
  }
  return [...latest].map(([name, ok]) => ({ role: 'review', name, ok }));
}

// Coding agents named by a pull request's author, body or commit trailers.
const AGENTS = [
  ['Claude Code', /\bclaude[ -]code\b|^claude\b/i],
  ['Codex', /\bcodex\b/i],
  ['Copilot', /\bcopilot\b/i],
  ['Devin', /\bdevin\b/i],
  ['Cursor', /\bcursor(?:agent)?\b/i],
  ['Jules', /\bjules\b/i],
  ['Aider', /\baider\b/i],
];
const agentIn = text => AGENTS.find(([, pattern]) => pattern.test(text))?.[0];
const GENERATED = /\bgenerated (?:with|by) \[?([^\]\n(]+)/gi;
const TRAILER = /^co-authored-by:\s*([^<\n]+?)\s*</gim;

// The author a merge rests on: a coding agent when one is named, else the
// account that opened it. Bots are agents.
export function authorOf(pull, commits = []) {
  const login = pull.user?.login ?? 'unknown';
  const named = [...(pull.body ?? '').matchAll(GENERATED)].map(match => match[1].trim())
    .concat(commits.flatMap(commit => [...(commit.commit?.message ?? '').matchAll(TRAILER)].map(match => match[1].trim())));
  for (const name of [login, ...named]) {
    const agent = agentIn(name);
    if (agent) return { role: 'author', name: agent, agent: true };
  }
  if (pull.user?.type === 'Bot' || login.endsWith('[bot]')) return { role: 'author', name: login.replace(/\[bot\]$/, ''), agent: true };
  return { role: 'author', name: login, agent: false };
}

export async function pages(api, url, key) {
  const values = [];
  for (let page = 1; ; page++) {
    const response = await api(`${url}${url.includes('?') ? '&' : '?'}per_page=100&page=${page}`);
    const list = key ? response[key] : response;
    assert.ok(Array.isArray(list), `GitHub did not return ${key ?? 'a list'} for ${url}.`);
    values.push(...list);
    if (list.length < 100) return values;
  }
}

// The facts one merged pull request rests on, in the order they are sent.
export async function gatherMerge({ api, repository, number }) {
  const pull = await api(`/repos/${repository}/pulls/${number}`);
  assert.ok(pull.merged_at, `#${number} has not merged.`);
  const head = pull.head.sha;
  const runs = await pages(api, `/repos/${repository}/commits/${head}/check-runs?filter=latest`, 'check_runs');
  const statuses = (await api(`/repos/${repository}/commits/${head}/status`)).statuses ?? [];
  const reviews = await pages(api, `/repos/${repository}/pulls/${number}/reviews`, null);
  const commits = await pages(api, `/repos/${repository}/pulls/${number}/commits`, null);
  const checks = new Map();
  for (const run of runs) {
    const ok = checkRunOk(run);
    if (ok !== null && !checks.has(run.name)) checks.set(run.name, ok);
  }
  for (const status of statuses) {
    const ok = statusOk(status);
    if (ok !== null && !checks.has(status.context)) checks.set(status.context, ok);
  }
  const author = authorOf(pull, commits);
  const facts = [
    ...[...checks].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)).map(([name, ok]) => ({ role: 'check', name, ok })),
    ...reviewVerdicts(reviews).sort((a, b) => (a.name < b.name ? -1 : 1)),
    { role: 'author', name: author.name, ok: 1 },
  ];
  return { pr: number, title: pull.title, merged_at: pull.merged_at, head, merge_commit: pull.merge_commit_sha ?? null, agent: author.agent, facts };
}

// ---- The record: chapters of merges and findings --------------------------

const sourceKey = (role, name) => `${role}\u0000${name}`;

function emptyChapter(number) {
  return { schema: SCHEMA, chapter: number, sources: [], entries: [] };
}

// Readings each source has in a chapter, counting both outcomes.
function counts(chapter) {
  const count = new Map();
  for (const entry of chapter.entries) {
    if (entry.type === 'merge') for (const fact of entry.facts) count.set(fact.source, (count.get(fact.source) ?? 0) + 1);
  }
  return count;
}
const merges = chapter => chapter.entries.filter(entry => entry.type === 'merge');
export const limitFor = sources => Math.min(MERGES, Math.floor((CAPACITY - MERGES) / Math.max(1, sources)));

// Whether the merge fits the chapter: every source's readings within the
// per-source limit the program can declare once the merge's new sources join.
function fits(chapter, merge) {
  if (merges(chapter).length >= MERGES) return false;
  const ids = new Map(chapter.sources.map(source => [sourceKey(source.role, source.name), source.id]));
  const count = counts(chapter);
  let sources = chapter.sources.length;
  for (const fact of merge.facts) {
    const id = ids.get(sourceKey(fact.role, fact.name));
    if (id) count.set(id, (count.get(id) ?? 0) + 1);
    else sources++;
  }
  const limit = limitFor(sources);
  return limit >= 1 && [...count.values()].every(value => value <= limit);
}

// A readable identifier for a source, unique in its chapter: the role and
// the name in lower case, `check_core_semantics`.
export function sourceId(role, name, taken) {
  const stem = `${role}_${name.toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_+|_+$/g, '')}`.replace(/_+$/, '').slice(0, 40).replace(/_+$/, '');
  let id = stem;
  for (let n = 2; taken.has(id); n++) id = `${stem}_${n}`;
  return id;
}

function addMerge(chapter, merge) {
  const ids = new Map(chapter.sources.map(source => [sourceKey(source.role, source.name), source.id]));
  const taken = new Set(ids.values());
  const facts = merge.facts.map(fact => {
    const key = sourceKey(fact.role, fact.name);
    if (!ids.has(key)) {
      const id = sourceId(fact.role, fact.name, taken);
      taken.add(id);
      chapter.sources.push({ id, role: fact.role, name: fact.name });
      ids.set(key, id);
    }
    return { source: ids.get(key), ok: fact.ok };
  });
  chapter.entries.push({ type: 'merge', pr: merge.pr, title: merge.title, merged_at: merge.merged_at, head: merge.head,
    merge_commit: merge.merge_commit, agent: merge.agent, facts });
}

export function renderProgram(template, chapter) {
  const limit = limitFor(chapter.sources.length);
  const entities = chapter.sources.length
    ? chapter.sources.map(source => `entity ${source.id} kind source at repo;  # ${source.role}: ${source.name.replace(/[\r\n]/g, ' ')}`).join('\n')
    : 'entity none kind source at repo;  # no sources yet';
  return template.replace('# @SOURCES@', entities).replaceAll('@LIMIT@', String(limit))
    .replaceAll('@MERGES@', String(MERGES)).replaceAll('@FINDINGS@', String(FINDINGS));
}

export function renderEvents(chapter) {
  const events = [];
  for (const entry of chapter.entries) {
    if (entry.type === 'merge') {
      events.push({ event: 'pr', payload: { number: entry.pr } });
      for (const fact of entry.facts) events.push({ event: 'saw', payload: { name: fact.source, ok: fact.ok } });
      events.push({ event: 'merge', payload: {} });
    } else {
      events.push({ event: 'wrong', payload: { name: entry.source, index: entry.index } });
    }
  }
  return events;
}

export async function loadRecord(dir) {
  let names = [];
  try {
    names = (await readdir(dir)).filter(name => /^chapter-\d+$/.test(name)).sort();
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }
  const chapters = [];
  for (const name of names) {
    const chapter = JSON.parse(await readFile(path.join(dir, name, 'chapter.json'), 'utf8'));
    assert.equal(chapter.schema, SCHEMA, `${name}/chapter.json is not a ${SCHEMA} record.`);
    chapters.push(chapter);
  }
  return chapters;
}

export async function saveChapter(dir, chapter, template) {
  const folder = path.join(dir, `chapter-${String(chapter.chapter).padStart(3, '0')}`);
  await mkdir(folder, { recursive: true });
  await writeFile(path.join(folder, 'chapter.json'), `${JSON.stringify(chapter, null, 1)}\n`);
  await writeFile(path.join(folder, 'program.cav'), renderProgram(template, chapter));
  await writeFile(path.join(folder, 'events.jsonl'), renderEvents(chapter).map(event => JSON.stringify(event)).join('\n') + '\n');
}

export function recorded(chapters, number) {
  return chapters.some(chapter => merges(chapter).some(merge => merge.pr === number));
}

// Adds a merge to the last chapter, or to a new one when it does not fit.
// Returns the chapter changed, or null when the merge is already recorded.
export function appendMerge(chapters, merge) {
  if (recorded(chapters, merge.pr)) return null;
  let chapter = chapters.at(-1);
  if (!chapter || !fits(chapter, merge)) {
    chapter = emptyChapter((chapter?.chapter ?? 0) + 1);
    assert.ok(fits(chapter, merge), `#${merge.pr} rests on more sources than one program can hold.`);
    chapters.push(chapter);
  }
  addMerge(chapter, merge);
  return chapter;
}

// ---- Asking the record ----------------------------------------------------

export async function loadKit(directory = process.env.CAVEAT_PR_LEDGER_KIT || path.join(here, '..', '..', 'kit')) {
  const lib = name => pathToFileURL(path.join(directory, 'lib', name)).href;
  const [node, explain] = await Promise.all([import(lib('node.mjs')), import(lib('explain.mjs'))]);
  return { runtime: await node.loadRuntimeFromDirectory(node.defaultRuntimeDirectory()), explain };
}

// Replays a chapter and returns its last snapshot and what was sent.
export function replay(kit, template, chapter) {
  const session = kit.runtime.open(renderProgram(template, chapter));
  const sent = [];
  let snapshot = session.snapshot();
  try {
    for (const { event, payload } of renderEvents(chapter)) {
      const { snapshot: after, ...outcome } = session.dispatch(event, payload);
      assert.equal(outcome.outcome, 'accepted', `chapter ${chapter.chapter}: the program refused ${event} ${JSON.stringify(payload)}: ${outcome.message}`);
      sent.push({ event, payload, outcome, sequence: after.sequence, effects: after.effects });
      snapshot = after;
    }
  } finally {
    session.close();
  }
  return { snapshot, sent };
}

const ROLES = { check: 'check', review: 'review', author: 'author' };
export function matchSources(chapter, role, name) {
  return chapter.sources.filter(source => source.role === role && (name === undefined || source.name === name));
}

// The reading index of a source's fact on one merge (1-based), or null.
export function readingIndex(chapter, sourceId, pr) {
  let index = 0;
  for (const entry of chapter.entries) {
    if (entry.type !== 'merge') continue;
    for (const fact of entry.facts) {
      if (fact.source !== sourceId) continue;
      index++;
      if (entry.pr === pr) return index;
    }
  }
  return null;
}

// Every merge whose recorded grounds hold the subject (a source or one of its
// readings), as Caveat's dependents report says. Lineage is left out: only
// what a merge's commit read is what it rested on.
export function mergesResting(kit, snapshot, chapter, subject) {
  const report = kit.explain.dependents(snapshot, subject);
  const byPr = new Map(merges(chapter).map(merge => [merge.pr, merge]));
  return report.decisions.filter(decision => decision.basis === 'grounds').map(decision => ({
    pr: decision.value,
    revision: decision.id,
    status: decision.status,
    via: decision.via,
    found_wrong: decision.withdrawn.map(withdrawal => withdrawal.evidence),
    merged_at: byPr.get(decision.value)?.merged_at ?? null,
    title: byPr.get(decision.value)?.title ?? null,
  }));
}

const labelOf = (chapter, id) => {
  const source = chapter.sources.find(entry => entry.id === id);
  return source ? `${source.role} "${source.name}"` : id;
};
const occurrenceSource = occurrence => occurrence.replace(/@\d+$/, '');

// Every merge that rested on something later found wrong.
export function affected(kit, template, chapters) {
  const rows = [];
  for (const chapter of chapters) {
    const { snapshot } = replay(kit, template, chapter);
    const withdrawn = new Set(kit.explain.dependents(snapshot, 'withdrawn').withdrawals.map(withdrawal => withdrawal.evidence));
    for (const occurrence of withdrawn) {
      for (const merge of mergesResting(kit, snapshot, chapter, occurrence)) {
        rows.push({ chapter: chapter.chapter, ...merge, wrong: labelOf(chapter, occurrenceSource(occurrence)), occurrence });
      }
    }
  }
  return rows.sort((a, b) => a.pr - b.pr || (a.wrong < b.wrong ? -1 : 1));
}

// Merges whose grounds hold a source, or its reading on one pull request.
export function restedOn(kit, template, chapters, { role, name, pr }) {
  const rows = [];
  for (const chapter of chapters) {
    const sources = matchSources(chapter, role, name);
    if (!sources.length) continue;
    const { snapshot } = replay(kit, template, chapter);
    for (const source of sources) {
      const index = pr === undefined ? null : readingIndex(chapter, source.id, pr);
      if (pr !== undefined && index === null) continue;
      const subject = index === null ? source.id : `${source.id}@${index}`;
      for (const merge of mergesResting(kit, snapshot, chapter, subject)) rows.push({ chapter: chapter.chapter, ...merge, source: labelOf(chapter, source.id) });
    }
  }
  return rows.sort((a, b) => a.pr - b.pr);
}

// Records a finding against a source on the merges named (one, a range, or
// all), as one `wrong` event each. Returns how many readings it withdrew.
export function appendWrong(chapters, { role, name, pr, from, to, note, at }) {
  let withdrawn = 0;
  for (const chapter of chapters) {
    for (const source of matchSources(chapter, role, name)) {
      const done = new Set(chapter.entries.filter(entry => entry.type === 'wrong' && entry.source === source.id).map(entry => entry.index));
      for (const merge of merges(chapter)) {
        if (pr !== undefined && merge.pr !== pr) continue;
        if (from !== undefined && merge.pr < from) continue;
        if (to !== undefined && merge.pr > to) continue;
        const index = readingIndex(chapter, source.id, merge.pr);
        if (index === null || done.has(index)) continue;
        const fact = merge.facts.find(entry => entry.source === source.id);
        if (fact.ok !== 1) continue;   // a failed check or a request for changes was not rested on
        assert.ok(chapter.entries.filter(entry => entry.type === 'wrong').length < FINDINGS, `chapter ${chapter.chapter} holds no more findings.`);
        chapter.entries.push({ type: 'wrong', source: source.id, pr: merge.pr, index, note: note ?? null, at: at ?? null });
        done.add(index);
        withdrawn++;
      }
    }
  }
  return withdrawn;
}

export function describeMerge(kit, template, chapters, number) {
  const chapter = chapters.find(entry => recorded([entry], number));
  assert.ok(chapter, `#${number} is not in the record.`);
  const { snapshot, sent } = replay(kit, template, chapter);
  const merge = merges(chapter).find(entry => entry.pr === number);
  const revision = `main@${merges(chapter).indexOf(merge) + 1}`;
  const grounds = snapshot.commitment_grounds?.[revision]?.evidence ?? [];
  const withdrawn = new Set(kit.explain.dependents(snapshot, 'withdrawn').withdrawals.map(withdrawal => withdrawal.evidence));
  const lines = [`#${number} ${merge.title ?? ''}`.trimEnd(), `  merged ${merge.merged_at} at ${merge.head.slice(0, 7)}; ${revision} in chapter ${chapter.chapter}`, '  rested on:'];
  const order = new Map(chapter.sources.map((source, position) => [source.id, position]));
  const position = occurrence => order.get(occurrenceSource(occurrence)) ?? -1;
  for (const occurrence of [...grounds].sort((a, b) => position(a) - position(b))) {
    lines.push(`    ${labelOf(chapter, occurrenceSource(occurrence))} (${occurrence})${withdrawn.has(occurrence) ? '  found wrong' : ''}`);
  }
  const failed = merge.facts.filter(fact => fact.ok === 0).map(fact => labelOf(chapter, fact.source));
  if (failed.length) lines.push(`  merged despite: ${failed.join(', ')}`);
  return { lines, explanation: kit.explain.formatExplanation(kit.explain.explain(snapshot, sent), `chapter ${chapter.chapter}`) };
}

// ---- The command line ------------------------------------------------------

export function githubApi() {
  const base = process.env.GITHUB_API_URL || 'https://api.github.com';
  const headers = { accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28' };
  if (process.env.GITHUB_TOKEN) headers.authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  return async url => {
    const response = await fetch(`${base}${url}`, { headers });
    assert.ok(response.ok, `GitHub request failed (${response.status}) for ${url}.`);
    return response.json();
  };
}

export function parseArgs(args) {
  const options = { _: [] };
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (!arg.startsWith('--')) options._.push(arg);
    else if (['--explain', '--json'].includes(arg)) options[arg.slice(2)] = true;
    else {
      assert.ok(i + 1 < args.length, `${arg} needs a value.`);
      options[arg.slice(2)] = args[++i];
    }
  }
  return options;
}

const number = (value, what) => {
  if (value === undefined) return undefined;
  const parsed = Number(value);
  assert.ok(Number.isSafeInteger(parsed) && parsed > 0, `${what} must be a pull request number.`);
  return parsed;
};

function subjectOf(options) {
  const given = Object.keys(ROLES).filter(role => options[role] !== undefined);
  assert.equal(given.length, 1, 'Name one of --check NAME, --review LOGIN or --author NAME.');
  return { role: given[0], name: options[given[0]] };
}

const row = merge => `#${merge.pr}  merged ${merge.merged_at ?? '?'}  ${merge.status}${merge.title ? `  ${merge.title}` : ''}`;

export async function main(args, { api = githubApi(), kit, log = console.log, now = () => new Date().toISOString() } = {}) {
  const options = parseArgs(args);
  const [command, target] = options._;
  const dir = path.resolve(options.dir ?? 'pr-ledger');
  const repository = options.repo ?? process.env.GITHUB_REPOSITORY;
  const template = await readFile(TEMPLATE, 'utf8');
  const chapters = await loadRecord(dir);
  const runtime = () => kit ?? loadKit();

  if (command === 'record' || command === 'backfill') {
    assert.ok(repository, 'Set GITHUB_REPOSITORY or pass --repo OWNER/NAME.');
    let numbers;
    if (command === 'record') numbers = [number(target, 'record')];
    else {
      const last = number(options.last ?? '30', '--last');
      // Pages come newest update first, and a pull request is updated when it
      // merges, so once a page's oldest update is older than the Nth newest
      // merge so far, no later page holds a newer merge.
      const closed = [];
      for (let page = 1; ; page++) {
        const list = await api(`/repos/${repository}/pulls?state=closed&base=${options.base ?? 'main'}&sort=updated&direction=desc&per_page=100&page=${page}`);
        closed.push(...list.filter(pull => pull.merged_at));
        const nth = closed.map(pull => pull.merged_at).sort().reverse()[last - 1];
        if (list.length < 100 || (nth && list.at(-1).updated_at < nth)) break;
      }
      numbers = closed.sort((a, b) => (a.merged_at < b.merged_at ? 1 : -1)).slice(0, last)
        .sort((a, b) => (a.merged_at < b.merged_at ? -1 : 1)).map(pull => pull.number);
    }
    const changed = new Set();
    for (const pr of numbers) {
      if (recorded(chapters, pr)) { log(`#${pr} is already recorded.`); continue; }
      const merge = await gatherMerge({ api, repository, number: pr });
      const chapter = appendMerge(chapters, merge);
      changed.add(chapter);
      log(`#${pr} recorded in chapter ${chapter.chapter}: ${merge.facts.length} facts.`);
    }
    const loaded = await runtime();
    for (const chapter of changed) {
      replay(loaded, template, chapter);   // the record must replay before it is written
      await saveChapter(dir, chapter, template);
    }
    return 0;
  }

  if (command === 'wrong') {
    const subject = subjectOf(options);
    const count = appendWrong(chapters, { ...subject, pr: number(options.pr, '--pr'), from: number(options.from, '--from'), to: number(options.to, '--to'), note: options.note, at: now() });
    assert.ok(count > 0, `No merge in the record rested on ${subject.role} "${subject.name}" there, or it is already found wrong.`);
    const loaded = await runtime();
    for (const chapter of chapters) {
      replay(loaded, template, chapter);
      await saveChapter(dir, chapter, template);
    }
    log(`Found wrong: ${subject.role} "${subject.name}" on ${count} merge${count === 1 ? '' : 's'}.`);
    options._ = ['affected'];
  }

  if (command === 'wrong' || command === 'affected') {
    const rows = affected(await runtime(), template, chapters);
    if (options.json) log(JSON.stringify(rows, null, 2));
    else if (!rows.length) log('No merge in the record rested on anything found wrong.');
    else for (const merge of rows) log(`${row(merge)}\n    rested on ${merge.wrong}, found wrong (${merge.occurrence})`);
    return 0;
  }

  if (command === 'rested-on') {
    const subject = subjectOf(options);
    const rows = restedOn(await runtime(), template, chapters, { ...subject, pr: number(options.pr, '--pr') });
    if (options.json) log(JSON.stringify(rows, null, 2));
    else if (!rows.length) log(`No merge in the record rested on ${subject.role} "${subject.name}".`);
    else for (const merge of rows) log(`${row(merge)}${merge.found_wrong.length ? '  (found wrong)' : ''}`);
    return 0;
  }

  if (command === 'show') {
    const { lines, explanation } = describeMerge(await runtime(), template, chapters, number(target, 'show'));
    log(lines.join('\n'));
    if (options.explain) log(`\n${explanation}`);
    return 0;
  }

  throw new Error('Usage: pr-ledger.mjs record PR | backfill [--last N] | wrong (--check|--review|--author) NAME [--pr N | --from N --to N] [--note TEXT] | rested-on (--check|--review|--author) NAME [--pr N] | affected | show PR [--explain]');
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  main(process.argv.slice(2)).then(status => { process.exitCode = status; }, error => { console.error(error.message); process.exitCode = 2; });
}
