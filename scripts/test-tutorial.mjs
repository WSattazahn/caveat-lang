// Runs the agent undo-list tutorial against this checkout's kit: every command
// it shows, every output line it quotes, and the exercise solutions.
// Usage: node scripts/test-tutorial.mjs [KIT_DIRECTORY]
import { execFileSync } from 'node:child_process';
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const tutorial = join(root, 'docs/tutorials/agent-undo-list');
const kit = resolve(process.argv[2] ?? join(root, 'kit'));
const cli = join(kit, 'bin/caveat.mjs');

const work = mkdtempSync(join(tmpdir(), 'caveat-tutorial-'));
const failures = [];
try {
  mkdirSync(join(work, 'node_modules'));
  symlinkSync(kit, join(work, 'node_modules/caveat-lang'), 'dir');
  cpSync(tutorial, work, { recursive: true });

  const run = (command, args) => {
    try {
      return execFileSync(command, args, { cwd: work, encoding: 'utf8' });
    } catch (error) {
      failures.push(`${command} ${args.join(' ')} exited ${error.status}: ${error.stdout}${error.stderr}`);
      return '';
    }
  };

  const text = readFileSync(join(tutorial, 'README.md'), 'utf8');
  const blocks = kind => [...text.matchAll(new RegExp('```' + kind + '\\n([\\s\\S]*?)```', 'g'))].map(m => m[1]);

  // Every Caveat or agent command the tutorial shows, in order.
  let output = '';
  for (const line of blocks('sh').flatMap(block => block.split('\n')).map(l => l.trim())) {
    const caveat = line.match(/^npx --no-install caveat-lang (.+)$/);
    if (caveat) output += run(process.execPath, [cli, ...caveat[1].split(/\s+/)]);
    else if (line.startsWith('node ')) output += run(process.execPath, line.slice(5).split(/\s+/));
  }

  // Every output line the tutorial quotes appears in what the commands printed.
  for (const line of blocks('text').flatMap(block => block.split('\n'))) {
    const quoted = line.trimEnd();
    if (quoted.trim() === '' || quoted.trim() === '...') continue;
    if (!output.includes(quoted.trim())) failures.push(`quoted output not printed: ${quoted.trim()}`);
  }
  if (!output.includes('support.cav: no warnings.')) failures.push('check did not print "support.cav: no warnings."');

  // Exercise solutions.
  run(process.execPath, [cli, 'check', '--strict', 'solutions/order-correction.cav']);
  const order = run(process.execPath, [cli, 'explain', 'solutions/order-correction.cav', 'solutions/events-order.jsonl']);
  for (const expected of ['refund@1 = null  reopened', 'carrier_claim@1 = null  in force', 'apology_credit@1 = null  in force']) {
    if (!order.includes(expected)) failures.push(`exercise 2: expected "${expected}"`);
  }
  run(process.execPath, ['solutions/save-restore.mjs']);
  if (!run(process.execPath, ['solutions/save-restore.mjs']).includes('Already refunded.')) {
    failures.push('exercise 3: the restored session did not refuse a second refund');
  }
} finally {
  rmSync(work, { recursive: true, force: true });
}

if (failures.length > 0) {
  console.error(failures.join('\n'));
  process.exit(1);
}
console.log('The agent undo-list tutorial runs as written.');
