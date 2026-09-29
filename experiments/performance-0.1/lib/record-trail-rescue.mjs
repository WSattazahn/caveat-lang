// Regenerates inputs/trail-rescue-episodes.json: the runtime-level events the
// registered Trail Rescue scenarios send, as the page's own policy translates
// them (web/trail-rescue-policy.js). One episode per scenario; `resume` steps
// are left out because a restore reproduces the same state; events the
// runtime refuses are kept, since a host pays for them too.
//
//   node experiments/performance-0.1/lib/record-trail-rescue.mjs --runtime=DIR
//
// DIR holds caveat_runtime.js and caveat_runtime_bg.wasm. The recorded events
// do not depend on which runtime translates them; the file records the hash
// of the one used.
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { harnessDirectory, repositoryRoot, sha256 } from './workloads.mjs';

const runtime = process.argv.find((arg) => arg.startsWith('--runtime='))?.slice('--runtime='.length);
if (!runtime) throw new Error('usage: record-trail-rescue.mjs --runtime=DIR');
const glue = await import(pathToFileURL(path.join(runtime, 'caveat_runtime.js')).href);
const wasm = await readFile(path.join(runtime, 'caveat_runtime_bg.wasm'));
await glue.default({ module_or_path: wasm });
const { createPolicyFromSession } = await import(pathToFileURL(path.join(repositoryRoot, 'web', 'trail-rescue-policy.js')).href);
const source = await readFile(path.join(repositoryRoot, 'game', 'trail_rescue.cav'), 'utf8');
const contractText = await readFile(path.join(repositoryRoot, 'experiments', 'trail-rescue', 'scenarios.json'), 'utf8');
const contract = JSON.parse(contractText);

let steps = null;
class RecordingSession extends glue.WebReactiveSession {
  dispatch_view(event, payload) {
    steps?.push([event, payload]);
    return super.dispatch_view(event, payload);
  }
}

const episodes = [];
for (const scenario of contract.scenarios) {
  steps = [];
  const policy = createPolicyFromSession(RecordingSession, source, undefined);
  let refused = 0;
  for (const scenarioStep of scenario.steps) {
    if (scenarioStep.op !== 'dispatch') continue;
    try { policy.dispatch(scenarioStep.event); } catch { refused += 1; }
  }
  policy.free?.();
  episodes.push({ scenario: scenario.id, refusedByPolicyOrRuntime: refused, steps });
  steps = null;
}

const output = {
  schema: 'caveat-performance-episodes/0.1',
  generatedBy: 'experiments/performance-0.1/lib/record-trail-rescue.mjs',
  from: {
    scenarios: { path: 'experiments/trail-rescue/scenarios.json', sha256: sha256(contractText) },
    policy: { path: 'web/trail-rescue-policy.js', sha256: sha256(await readFile(path.join(repositoryRoot, 'web', 'trail-rescue-policy.js'))) },
    program: { path: 'game/trail_rescue.cav', sha256: sha256(source) },
    runtimeWasmSha256: sha256(wasm),
  },
  episodes,
};
const file = path.join(harnessDirectory, 'inputs', 'trail-rescue-episodes.json');
await writeFile(file, `${JSON.stringify(output, null, 2)}\n`);
console.log(`${episodes.length} episodes, ${episodes.reduce((sum, episode) => sum + episode.steps.length, 0)} events -> ${file}`);
