// The fixed workloads the Carmack checks time. Each is a program and an event
// sequence, replayed on a fresh session per repetition. Kept small and
// deterministic so every platform and every release runs the same events.
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, '..', '..');

// A release gate: three components, three checks each, reports that are
// flagged, unflagged and withdrawn, and decisions that reopen. Each round is
// eight events per component; two rounds for each of the three components.
function releaseGate() {
  const round = (component) => [
    ['report', { component, check: 'tests', outcome: 'pass' }],
    ['report', { component, check: 'audit', outcome: 'pass' }],
    ['report', { component, check: 'review', outcome: 'pass' }],
    ['decide', { component }],
    ['flag', { component, check: 'tests', reason: 'flaky' }],
    ['decide', { component }],
    ['unflag', { component, check: 'tests', reason: 'flaky' }],
    ['withdraw', { component, check: 'audit', named_component: component, named_check: 'audit', named_number: 0 }],
  ];
  const events = [];
  for (let i = 0; i < 2; i += 1) for (const c of ['api', 'web', 'worker']) events.push(...round(c));
  return events;
}

// The project's own decision ledger, every recorded event in order.
function ledger() {
  return readFileSync(path.join(root, 'ledger', 'events.jsonl'), 'utf8')
    .split('\n').filter(Boolean).map((line) => JSON.parse(line))
    .map(({ event, payload }) => [event, payload ?? {}]);
}

export const WORKLOADS = [
  { name: 'release-gate', program: path.join(here, 'programs', 'release-gate.cav'), events: releaseGate(), reps: 200 },
  { name: 'ledger', program: path.join(root, 'ledger', 'project.cav'), events: ledger(), reps: 20 },
];

export const ROOT = root;
