#!/usr/bin/env node
// Turns a release's facts (GitHub runs, merges, the tag, the npm-publish
// approval, the publish and the verification record) into release.cav's
// events, and runs them through `caveat serve`.
//
//   node release-ledger.mjs events FACTS.jsonl          the events, as JSON lines
//   node release-ledger.mjs run FACTS.jsonl [--explain OUT.json] [--events OUT.jsonl]
//
// `run` prints one line per fact and ends with the serve session's explain
// report; --explain writes that report and --events the events sent, so
// `caveat explain --json release.cav OUT.jsonl` reproduces it. It exits 1 if
// the session fails. Refusals are outcomes, not failures. The facts are
// whatever the caller collected: this driver checks their shape, not their
// truth, and fetches nothing.
import { spawn } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import readline from 'node:readline';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const PROGRAM = path.join(here, 'release.cav');
const CLI = path.join(here, '..', '..', 'kit', 'bin', 'caveat.mjs');
const SLOTS = new Set(['pr1', 'pr2', 'pr3', 'pr4', 'pr5', 'pr6', 'pr7', 'pr8']);
const SHA = /^[0-9a-f]{40}$/;
// A run's conclusion: anything else (cancelled, skipped, timed_out, …) is not
// a result and sends nothing.
const RESULTS = { success: 'passed', failure: 'failed' };

function sha(fact, field) {
  const value = fact[field];
  if (typeof value !== 'string' || !SHA.test(value)) throw new Error(`${fact.fact}: ${field} must be a full lowercase SHA`);
  return value;
}

// One fact to at most one event. `slots` maps GitHub pull request numbers to
// the plan's pull requests and is filled by `plan` facts.
export function translate(fact, slots) {
  const slot = number => slots.get(Number(number)) ?? 'unlisted';
  switch (fact.fact) {
    case 'plan':
      for (const [number, name] of Object.entries(fact.prs ?? {})) {
        if (!SLOTS.has(name)) throw new Error(`plan: ${name} is not a plan pull request (pr1 to pr8)`);
        slots.set(Number(number), name);
      }
      return { skip: `${slots.size} plan pull requests` };
    case 'open':
      return { event: 'opened', payload: { commit: sha(fact, 'sha') } };
    case 'pr_run': {
      const target = slot(fact.pr);
      if (target === 'unlisted') return { skip: `#${fact.pr} is not a plan pull request` };
      const result = RESULTS[fact.conclusion];
      if (!result) return { skip: `conclusion ${fact.conclusion} is not a result` };
      return { event: 'pr_run', payload: { target, head: sha(fact, 'head_sha'), base: sha(fact, 'base_sha'), result } };
    }
    case 'merge':
      return { event: 'merged', payload: { target: slot(fact.pr), head: sha(fact, 'head_sha'), commit: sha(fact, 'merge_commit_sha') } };
    case 'main_run': {
      const result = RESULTS[fact.conclusion];
      if (!result) return { skip: `conclusion ${fact.conclusion} is not a result` };
      return { event: 'main_run', payload: { commit: sha(fact, 'head_sha'), result } };
    }
    case 'decide':
      return { event: 'decide', payload: {} };
    case 'tag':
      return { event: 'tagged', payload: { commit: sha(fact, 'sha') } };
    case 'approval':
      return { event: 'approved', payload: { commit: sha(fact, 'sha') } };
    case 'hand_go':
      return { event: 'hand_go', payload: {} };
    case 'publish':
      if (!['workflow', 'hand'].includes(fact.route)) throw new Error('publish: route must be workflow or hand');
      return { event: 'published', payload: { route: fact.route, commit: sha(fact, 'sha') } };
    case 'verification': {
      // A verify-publication artifact (caveat-npm-publication-verification/0.1)
      // or a release's publication record (caveat-npm-publication-record/0.1):
      // both carry the revision and these two checks.
      const record = fact.record;
      const { registryBytesMatch, provenanceAttested } = record?.checks ?? {};
      if (typeof registryBytesMatch !== 'boolean' || typeof provenanceAttested !== 'boolean') {
        throw new Error('verification: checks.registryBytesMatch and checks.provenanceAttested must be true or false');
      }
      return { event: 'verified', payload: {
        commit: sha(record, 'revision'),
        bytes: registryBytesMatch ? 'match' : 'mismatch',
        provenance: provenanceAttested ? 'present' : 'absent',
      } };
    }
    case 'record':
      return { event: 'record', payload: {} };
    default:
      throw new Error(`unknown fact ${JSON.stringify(fact.fact)}`);
  }
}

// Reads a facts file. A verification fact may name its record by `file`,
// relative to the facts file, instead of carrying it.
export async function readFacts(file) {
  const facts = [];
  for (const [index, line] of (await readFile(file, 'utf8')).split('\n').entries()) {
    if (!line.trim()) continue;
    const fact = JSON.parse(line);
    if (fact.fact === 'verification' && fact.file) {
      fact.record = JSON.parse(await readFile(path.resolve(path.dirname(file), fact.file), 'utf8'));
    }
    facts.push({ line: index + 1, fact });
  }
  return facts;
}

export function eventsOf(facts) {
  const slots = new Map();
  return facts.map(({ line, fact }) => ({ line, fact, ...translate(fact, slots) }));
}

// A `caveat serve` session of release.cav, one request at a time.
export async function serve({ cli = CLI } = {}) {
  const child = spawn(process.execPath, [cli, 'serve', PROGRAM], { stdio: ['pipe', 'pipe', 'inherit'] });
  const lines = readline.createInterface({ input: child.stdout })[Symbol.asyncIterator]();
  const next = async () => {
    const { value, done } = await lines.next();
    if (done) throw new Error('caveat serve ended');
    return JSON.parse(value);
  };
  const ready = await next();
  if (!ready.ready) throw new Error(`release.cav does not load: ${ready.error?.message}`);
  let id = 0;
  return {
    async request(op, fields = {}) {
      id += 1;
      child.stdin.write(`${JSON.stringify({ id, op, ...fields })}\n`);
      const response = await next();
      if (!response.ok) throw new Error(`${op}: ${response.error.kind}: ${response.error.message}`);
      return response;
    },
    close() { child.stdin.end(); },
  };
}

export async function run(facts, options) {
  const session = await serve(options);
  const steps = [];
  try {
    for (const step of eventsOf(facts)) {
      if (step.skip) { steps.push({ ...step, outcome: 'skipped' }); continue; }
      const response = await session.request('dispatch', { event: step.event, payload: step.payload });
      steps.push({ ...step, outcome: response.outcome, origin: response.origin, code: response.code, message: response.message });
    }
    const { report } = await session.request('explain');
    return { steps, report };
  } finally {
    session.close();
  }
}

async function main([command, file, ...rest]) {
  if (!['events', 'run'].includes(command) || !file) {
    throw new Error('usage: release-ledger.mjs events FACTS.jsonl | run FACTS.jsonl [--explain OUT.json] [--events OUT.jsonl]');
  }
  const facts = await readFacts(file);
  const sent = eventsOf(facts).filter(step => !step.skip).map(({ event, payload }) => JSON.stringify({ event, payload }));
  if (command === 'events') { process.stdout.write(sent.map(line => `${line}\n`).join('')); return; }
  const option = name => { const at = rest.indexOf(name); return at >= 0 ? rest[at + 1] : undefined; };
  const { steps, report } = await run(facts);
  for (const step of steps) {
    const what = step.skip ? `skipped: ${step.skip}`
      : step.outcome === 'accepted' ? `${step.event} accepted`
      : `${step.event} refused (${step.origin}/${step.code}): ${step.message}`;
    console.log(`${String(step.line).padStart(4)}  ${step.fact.fact.padEnd(12)} ${what}`);
  }
  if (option('--explain')) await writeFile(option('--explain'), `${JSON.stringify(report, null, 2)}\n`);
  if (option('--events')) await writeFile(option('--events'), sent.map(line => `${line}\n`).join(''));
  console.log('');
  for (const { name, value } of report.displayed) if (name.startsWith('repo.')) console.log(`${name} = ${JSON.stringify(value)}`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch(error => { console.error(error.message); process.exitCode = 1; });
}
