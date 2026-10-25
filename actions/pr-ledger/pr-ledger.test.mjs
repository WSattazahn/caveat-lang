// The PR ledger's host: what it gathers from GitHub, how it keeps the record
// in chapters, and what Caveat says rested on what. Runs on this
// repository's kit after `npm run build`, or on the released package named
// by CAVEAT_PR_LEDGER_KIT.
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {
  affected, appendMerge, appendWrong, authorOf, checkRunOk, describeMerge, gatherMerge, limitFor, loadKit, loadRecord, main,
  renderProgram, replay, restedOn, reviewVerdicts, sourceId, TEMPLATE,
} from './pr-ledger.mjs';

const kit = await loadKit();
const template = await readFile(TEMPLATE, 'utf8');
const repository = 'octo/widgets';

const run = (name, conclusion = 'success', status = 'completed') => ({ name, status, conclusion });
const review = (login, state) => ({ user: { login }, state });

// A fake GitHub holding merged pull requests by number.
function github(pulls) {
  return async url => {
    const bare = url.replace(/[?&]per_page=100&page=\d+$/, '');
    const first = /[?&]page=1$/.test(url) || !/[?&]page=\d+$/.test(url);
    const number = Number(/\/pulls\/(\d+)(?:\/|$)/.exec(bare)?.[1]);
    const pr = pulls.find(pull => pull.number === number || bare.includes(`/commits/${pull.head}/`));
    if (bare === `/repos/${repository}/pulls?state=closed&base=main&sort=updated&direction=desc`) {
      return first ? pulls.map(pull => ({ number: pull.number, merged_at: pull.merged_at })) : [];
    }
    if (!pr) throw new Error(`unexpected ${url}`);
    if (bare === `/repos/${repository}/pulls/${pr.number}`) {
      return { number: pr.number, title: `PR ${pr.number}`, merged_at: pr.merged_at, head: { sha: pr.head }, user: pr.user ?? { login: 'dev', type: 'User' }, body: pr.body ?? '' };
    }
    if (bare === `/repos/${repository}/commits/${pr.head}/check-runs?filter=latest`) return { check_runs: first ? pr.runs ?? [] : [] };
    if (bare === `/repos/${repository}/commits/${pr.head}/status`) return { statuses: pr.statuses ?? [] };
    if (bare === `/repos/${repository}/pulls/${pr.number}/reviews`) return first ? pr.reviews ?? [] : [];
    if (bare === `/repos/${repository}/pulls/${pr.number}/commits`) return first ? pr.commits ?? [] : [];
    throw new Error(`unexpected ${url}`);
  };
}
const pull = (number, extra = {}) => ({ number, head: String(number).padStart(40, 'a'), merged_at: `2026-10-${String(number % 28 + 1).padStart(2, '0')}T00:00:00Z`, ...extra });

async function record(pulls) {
  const api = github(pulls);
  const chapters = [];
  for (const { number } of pulls) appendMerge(chapters, await gatherMerge({ api, repository, number }));
  return chapters;
}

test('what GitHub says becomes passed, failed or nothing', () => {
  assert.equal(checkRunOk(run('a')), 1);
  assert.equal(checkRunOk(run('a', 'failure')), 0);
  assert.equal(checkRunOk(run('a', 'cancelled')), 0);
  assert.equal(checkRunOk(run('a', 'skipped')), null);
  assert.equal(checkRunOk(run('a', 'neutral')), null);
  assert.equal(checkRunOk(run('a', null, 'in_progress')), null);
  assert.deepEqual(reviewVerdicts([review('ann', 'CHANGES_REQUESTED'), review('ann', 'APPROVED'), review('bo', 'COMMENTED'), review('cy', 'APPROVED'), review('cy', 'DISMISSED')]),
    [{ role: 'review', name: 'ann', ok: 1 }]);
});

test('the author is the coding agent the pull request names, else the account', () => {
  assert.deepEqual(authorOf({ user: { login: 'dev' }, body: 'Fix.\n\nGenerated with [Claude Code](https://claude.com/claude-code)' }), { role: 'author', name: 'Claude Code', agent: true });
  assert.deepEqual(authorOf({ user: { login: 'dev' }, body: '' }, [{ commit: { message: 'x\n\nCo-authored-by: Codex <codex@example.com>' } }]), { role: 'author', name: 'Codex', agent: true });
  assert.deepEqual(authorOf({ user: { login: 'Copilot', type: 'Bot' } }), { role: 'author', name: 'Copilot', agent: true });
  assert.deepEqual(authorOf({ user: { login: 'renovate[bot]', type: 'Bot' } }), { role: 'author', name: 'renovate', agent: true });
  assert.deepEqual(authorOf({ user: { login: 'dev', type: 'User' }, body: 'By hand.' }), { role: 'author', name: 'dev', agent: false });
});

test('a merge rests on its passing checks, approving reviews and author, and nothing else', async () => {
  const chapters = await record([
    pull(1, { runs: [run('unit'), run('lint', 'failure'), run('docs', 'skipped')], statuses: [{ context: 'deploy', state: 'success' }],
      reviews: [review('ann', 'APPROVED'), review('bo', 'CHANGES_REQUESTED')], body: 'Generated with [Claude Code](x)' }),
  ]);
  const { lines } = describeMerge(kit, template, chapters, 1);
  assert.deepEqual(lines.slice(2), [
    '  rested on:',
    '    check "deploy" (check_deploy@1)',
    '    check "unit" (check_unit@1)',
    '    review "ann" (review_ann@1)',
    '    author "Claude Code" (author_claude_code@1)',
    '  merged despite: check "lint", review "bo"',
  ]);
});

test('a check found wrong on one pull request names the merge that rested on it, and only that one', async () => {
  const runs = [run('unit'), run('e2e')];
  const chapters = await record([pull(1, { runs }), pull(2, { runs }), pull(3, { runs: [run('unit')] }), pull(4, { runs: [run('unit'), run('e2e', 'failure')] })]);
  assert.deepEqual(restedOn(kit, template, chapters, { role: 'check', name: 'e2e' }).map(merge => merge.pr), [1, 2]);
  assert.deepEqual(affected(kit, template, chapters), []);
  assert.equal(appendWrong(chapters, { role: 'check', name: 'e2e', pr: 2 }), 1);
  assert.equal(appendWrong(chapters, { role: 'check', name: 'e2e', pr: 2 }), 0, 'already found wrong');
  assert.equal(appendWrong(chapters, { role: 'check', name: 'e2e', pr: 4 }), 0, 'a failed check was not rested on');
  const rows = affected(kit, template, chapters);
  assert.deepEqual(rows.map(merge => [merge.pr, merge.wrong, merge.occurrence, merge.status]), [[2, 'check "e2e"', 'check_e2e@2', 'superseded']]);
  // The finding withdraws the reading; the merge's grounds stay as recorded.
  const { snapshot } = replay(kit, template, chapters[0]);
  assert.ok(snapshot.commitment_grounds['main@2'].evidence.includes('check_e2e@2'));
});

test('an agent found wrong across a range lists every merge it wrote there', async () => {
  const agent = { body: 'Generated with [Claude Code](x)' };
  const chapters = await record([pull(10, { ...agent, runs: [run('unit')] }), pull(11, { runs: [run('unit')] }), pull(12, { ...agent }), pull(13, { ...agent })]);
  assert.equal(appendWrong(chapters, { role: 'author', name: 'Claude Code', from: 10, to: 12, note: 'a model regression' }), 2);
  assert.deepEqual(affected(kit, template, chapters).map(merge => merge.pr), [10, 12]);
  assert.deepEqual(restedOn(kit, template, chapters, { role: 'author', name: 'Claude Code' }).map(merge => [merge.pr, merge.found_wrong.length > 0]), [[10, true], [12, true], [13, false]]);
});

test('chapters close when the program could not declare the history', async () => {
  assert.equal(limitFor(15), 64);
  assert.equal(limitFor(24), 40);
  const many = Array.from({ length: 70 }, (_, i) => pull(i + 1, { runs: [run('unit')] }));
  const chapters = await record(many);
  assert.deepEqual(chapters.map(chapter => chapter.entries.length), [64, 6]);
  const wide = await record(Array.from({ length: 30 }, (_, i) => pull(i + 1, { runs: Array.from({ length: 30 }, (_, j) => run(`job ${j}`)) })));
  assert.deepEqual(wide.map(chapter => chapter.entries.length), [30]);
  const wider = await record(Array.from({ length: 40 }, (_, i) => pull(i + 1, { runs: Array.from({ length: 30 }, (_, j) => run(`job ${j}`)) })));
  assert.deepEqual(wider.map(chapter => chapter.entries.length), [30, 10]);  // 31 sources: 30 readings each
  for (const chapter of [...chapters, ...wider]) assert.doesNotThrow(() => kit.runtime.open(renderProgram(template, chapter)).close());
  assert.equal(appendWrong(wider, { role: 'check', name: 'job 3', from: 29, to: 32 }), 4);
  assert.deepEqual(affected(kit, template, wider).map(merge => [merge.chapter, merge.pr]), [[1, 29], [1, 30], [2, 31], [2, 32]]);
});

test('source names become distinct identifiers', () => {
  const taken = new Set(['check_unit']);
  assert.equal(sourceId('check', 'Unit', taken), 'check_unit_2');
  assert.equal(sourceId('check', 'MCP worker / ubuntu-24.04', new Set()), 'check_mcp_worker_ubuntu_24_04');
  assert.equal(sourceId('check', '!!!', new Set()), 'check');
});

test('the command line records, finds wrong and reports, and the record replays by hand', async t => {
  const dir = await mkdtemp(path.join(os.tmpdir(), 'pr-ledger-'));
  t.after(() => rm(dir, { recursive: true, force: true }));
  const api = github([pull(5, { runs: [run('unit')] }), pull(6, { runs: [run('unit')], reviews: [review('ann', 'APPROVED')] })]);
  const out = [];
  const options = { api, kit, log: line => out.push(line), now: () => '2026-10-25T00:00:00Z' };
  await main(['backfill', '--last', '2', '--repo', repository, '--dir', dir], options);
  await main(['record', '6', '--repo', repository, '--dir', dir], options);
  assert.deepEqual(out.splice(0), ['#5 recorded in chapter 1: 2 facts.', '#6 recorded in chapter 1: 3 facts.', '#6 is already recorded.']);
  await main(['wrong', '--review', 'ann', '--pr', '6', '--note', 'approved without running it', '--dir', dir], options);
  assert.equal(out[0], 'Found wrong: review "ann" on 1 merge.');
  assert.match(out[1], /^#6 {2}merged .* {2}in force {2}PR 6\n {4}rested on review "ann", found wrong \(review_ann@1\)$/);
  out.length = 0;
  await main(['rested-on', '--check', 'unit', '--dir', dir], options);
  assert.equal(out.length, 2);
  const [chapter] = await loadRecord(dir);
  assert.equal(chapter.entries.at(-1).note, 'approved without running it');
  const events = (await readFile(path.join(dir, 'chapter-001', 'events.jsonl'), 'utf8')).trim().split('\n').map(line => JSON.parse(line));
  assert.deepEqual(events.at(-1), { event: 'wrong', payload: { name: 'review_ann', index: 1 } });
  assert.equal(await readFile(path.join(dir, 'chapter-001', 'program.cav'), 'utf8'), renderProgram(template, chapter));
});
