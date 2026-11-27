// The npm release gate's host. scripts/release-publish.mjs `select` asks it
// which Runtime run and artifact may be published for the tagged revision.
// GitHub's answers are sent to release-gate.cav one fact per event, and the
// program decides; this file only fetches, sends, and turns the program's
// refusal into the message the gate has always given. It records every event
// so the decision can be explained afterwards (caveat explain).
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
export const PROGRAM = path.join(here, 'release-gate.cav');
const RUNTIME_WORKFLOW = '.github/workflows/runtime.yml';
const CANDIDATE_ARTIFACT = 'kit-package-candidate';
const SHA = /^[0-9a-f]{40}$/;
const same = (a, b) => typeof a === 'string' && a.toLowerCase() === b.toLowerCase();
const bit = value => (value ? 1 : 0);

// CAVEAT_GATE_KIT names an unpacked caveat-lang package (publish-npm.yml
// installs a pinned release there); otherwise this checkout's kit/ and its
// built runtime.
export function kitDirectory() {
  return process.env.CAVEAT_GATE_KIT || path.join(here, '..', '..', 'kit');
}

export async function loadKit(directory = kitDirectory()) {
  const lib = name => pathToFileURL(path.join(directory, 'lib', name)).href;
  const [node, explain] = await Promise.all([import(lib('node.mjs')), import(lib('explain.mjs'))]);
  const runtime = await node.loadRuntimeFromDirectory(node.defaultRuntimeDirectory());
  return { runtime, explain, directory };
}

async function pages(api, url, key) {
  const values = [];
  for (let page = 1; ; page++) {
    const response = await api(`${url}${url.includes('?') ? '&' : '?'}per_page=100&page=${page}`);
    assert.ok(Array.isArray(response[key]), `GitHub did not return ${key}.`);
    values.push(...response[key]);
    if (response[key].length < 100) return values;
  }
}

// The program's own refusals, raised as the gate raised them before it was a
// Caveat program: the same message, and for the two that were equality checks
// the same check, so Node's assert appends the same detail. Other refusals
// pass through unchanged.
function refuse(literal, { revision, run, artifacts }) {
  switch (literal) {
    case 'No Runtime run from a push to main tested the revision.':
      assert.fail(`No Runtime run from a push to main tested ${revision}.`);
    case 'The newest Runtime run has not succeeded.':
      assert.fail(`The newest Runtime run for ${revision} (${run?.id}) has not succeeded.`);
    case 'The chosen run must have exactly one kit-package-candidate artifact.':
      assert.equal(artifacts?.length, 1, `Runtime run ${run?.id} must have exactly one ${CANDIDATE_ARTIFACT} artifact.`);
      break;
    case 'The tested kit-package-candidate has expired; it cannot be published.':
      assert.equal(artifacts?.[0]?.expired, false, `The tested ${CANDIDATE_ARTIFACT} of run ${run?.id} has expired; it cannot be published.`);
      break;
  }
  assert.fail(literal);
}

// A record of the session: each event sent, its outcome, and what explain
// needs to name the event that observed each ground.
function recorder(session, context) {
  const sent = [];
  let snapshot = session.snapshot();
  const send = (event, payload = {}) => {
    const { snapshot: after, ...outcome } = session.dispatch(event, payload);
    sent.push({ event, payload, outcome, ...(outcome.outcome === 'accepted' ? { sequence: after.sequence, effects: after.effects } : {}) });
    if (outcome.outcome === 'accepted') { snapshot = after; return after; }
    try {
      if (outcome.origin === 'policy' && outcome.code === 'reject') refuse(outcome.message, context);
      assert.fail(`The release gate refused ${event} (${outcome.origin}/${outcome.code}): ${outcome.message}`);
    } catch (error) {
      error.gate = { sent, snapshot: () => snapshot };
      throw error;
    }
  };
  return { send, sent, snapshot: () => snapshot };
}

// The same answer as before (the newest Runtime run pushed to main for the
// revision, and its one unexpired candidate artifact), now decided by
// release-gate.cav. Returns the record with it.
export async function gateRuntimeCandidate({ api, repository, revision, kit }) {
  assert.match(revision, SHA, 'Invalid release revision.');
  const { runtime } = kit ?? await loadKit();
  const session = runtime.open(await readFile(PROGRAM, 'utf8'));
  const context = { revision, run: null, artifacts: null };
  const gate = recorder(session, context);
  try {
    const workflow = await api(`/repos/${repository}/actions/workflows/runtime.yml`);
    const runs = (await pages(api, `/repos/${repository}/actions/workflows/runtime.yml/runs?event=push&branch=main&head_sha=${revision}`, 'workflow_runs'))
      .sort((a, b) => a.id - b.id);
    for (const run of runs) {
      gate.send('run', { id: run.id,
        workflow: bit(run.workflow_id === workflow.id && run.path?.split('@')[0] === RUNTIME_WORKFLOW),
        repository: bit(same(run.repository?.full_name, repository) && same(run.head_repository?.full_name, repository)),
        push: bit(run.event === 'push'), main: bit(run.head_branch === 'main'), revision: bit(run.head_sha === revision),
        completed: bit(run.status === 'completed'), success: bit(run.conclusion === 'success') });
    }
    context.run = runs.at(-1) ?? null;
    gate.send('choose_run');
    const run = context.run;
    const artifacts = (await pages(api, `/repos/${repository}/actions/runs/${run.id}/artifacts`, 'artifacts'))
      .filter(artifact => artifact.name === CANDIDATE_ARTIFACT);
    context.artifacts = artifacts;
    for (const artifact of artifacts) {
      gate.send('artifact', { id: artifact.id, expired: bit(artifact.expired !== false),
        belongs: bit(artifact.workflow_run?.id === run.id && artifact.workflow_run?.head_sha === revision) });
    }
    const after = gate.send('decide');
    const bindings = after.bindings?.release ?? {};
    assert.equal(bindings.allowed, 'allowed', 'The release gate did not allow the release.');
    const [artifact] = artifacts;
    assert.ok(bindings.run === run.id && bindings.artifact === artifact.id, 'The release gate allowed a different run or artifact.');
    return { runtime_run_id: run.id, artifact_id: artifact.id, record: gate };
  } finally {
    session.close();
  }
}

// The record as files: the events (replay with `caveat explain`) and the
// explanation itself.
export function recordFiles(record, explainModule, title = 'release-gate.cav') {
  const events = record.sent.filter(entry => entry.outcome.outcome === 'accepted')
    .map(({ event, payload }) => JSON.stringify({ event, payload })).join('\n');
  const explanation = explainModule.formatExplanation(explainModule.explain(record.snapshot(), record.sent), title);
  const refused = record.sent.find(entry => entry.outcome.outcome !== 'accepted');
  return { 'events.jsonl': `${events}\n`, 'explain.txt': `${explanation}\n${refused ? `\nRefused ${refused.event}: ${refused.outcome.message}\n` : ''}` };
}
