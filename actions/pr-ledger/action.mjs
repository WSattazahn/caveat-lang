// The Action's step: fetch the record from its branch, run one pr-ledger.mjs
// command on it, and push the record back when the command changed it.
// action.yml passes the inputs as INPUT_* variables; nothing reaches a shell.
import { spawnSync } from 'node:child_process';
import { appendFileSync, mkdtempSync, rmSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { main } from './pr-ledger.mjs';

const input = name => {
  const value = process.env[`INPUT_${name.toUpperCase()}`];
  return value === undefined || value === '' ? undefined : value;
};
const repository = process.env.GITHUB_REPOSITORY;
const server = process.env.GITHUB_SERVER_URL || 'https://github.com';
const branch = input('branch') ?? 'caveat-pr-ledger';
const folder = input('path') ?? 'pr-ledger';
const command = input('command') ?? 'record';
const token = input('token');
const writes = ['record', 'backfill', 'wrong'].includes(command);

const auth = token ? ['-c', `http.${server}/.extraheader=AUTHORIZATION: basic ${Buffer.from(`x-access-token:${token}`).toString('base64')}`] : [];
function git(cwd, args, { allowFailure = false } = {}) {
  const result = spawnSync('git', [...auth, ...args], { cwd, encoding: 'utf8' });
  if (result.status !== 0 && !allowFailure) throw new Error(`git ${args[0]} failed: ${result.stderr.trim()}`);
  return result;
}

function checkout() {
  const dir = mkdtempSync(path.join(process.env.RUNNER_TEMP || os.tmpdir(), 'pr-ledger-'));
  git(dir, ['init', '--quiet']);
  git(dir, ['remote', 'add', 'origin', `${server}/${repository}.git`]);
  const exists = git(dir, ['ls-remote', '--exit-code', '--heads', 'origin', branch], { allowFailure: true }).status === 0;
  if (exists) {
    git(dir, ['fetch', '--quiet', '--depth', '1', 'origin', branch]);
    git(dir, ['checkout', '--quiet', '-b', branch, 'FETCH_HEAD']);
  } else {
    git(dir, ['checkout', '--quiet', '--orphan', branch]);
  }
  return dir;
}

function args() {
  const list = [command];
  if (['record', 'show'].includes(command)) list.push(input('pr') ?? '');
  for (const name of ['check', 'review', 'author', 'from', 'to', 'note', 'last', 'base']) {
    if (input(name) !== undefined) list.push(`--${name}`, input(name));
  }
  if (command === 'wrong' || command === 'rested-on') {
    if (input('pr') !== undefined) list.push('--pr', input('pr'));
  }
  return list;
}

const lines = [];
const log = line => { lines.push(line); console.log(line); };

for (let attempt = 1; ; attempt++) {
  const dir = checkout();
  lines.length = 0;
  try {
    await main([...args(), '--dir', path.join(dir, folder), '--repo', repository], { log });
    if (!writes) break;
    git(dir, ['add', '--all', folder]);
    if (git(dir, ['diff', '--cached', '--quiet'], { allowFailure: true }).status === 0) break;
    git(dir, ['-c', 'user.name=github-actions[bot]', '-c', 'user.email=41898282+github-actions[bot]@users.noreply.github.com',
      'commit', '--quiet', '-m', `pr-ledger: ${args().filter(Boolean).join(' ')}`]);
    // Another merge may have pushed first: start again from its record.
    if (git(dir, ['push', '--quiet', 'origin', `HEAD:refs/heads/${branch}`], { allowFailure: true }).status === 0) break;
    if (attempt === 3) throw new Error(`could not push the record to ${branch} after 3 attempts.`);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

if (process.env.GITHUB_STEP_SUMMARY && lines.length) {
  appendFileSync(process.env.GITHUB_STEP_SUMMARY, `### PR ledger: ${command}\n\n\`\`\`\n${lines.join('\n')}\n\`\`\`\n`);
}
