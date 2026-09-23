// Supplemental run of the unchanged 24 registered Trail Rescue histories.
// Deliberately does not execute the original harness's additional/fuzz checks.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { gzipSync, gunzipSync } from 'node:zlib';
import { createPolicyFromSession } from '../../web/trail-rescue-policy.js';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../..');
const schema = 'caveat-dispatch/0.1';
const historicalBase = '3980c3d';
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const clone = value => JSON.parse(JSON.stringify(value));
const atPointer = (value, pointer) => pointer.split('/').slice(1).reduce((object, key) => object?.[key.replace(/~1/g, '/').replace(/~0/g, '~')], value);

export class CoreRejection extends Error {
  constructor(outcome) { super(outcome.message); this.name = 'CoreRejection'; this.outcome = outcome; }
}

export class AuditFatal extends Error {
  constructor(message, cause) { super(message, { cause }); this.name = 'AuditFatal'; }
}

function validateOutcome(outcome) {
  assert.equal(outcome?.schema, schema, 'unknown dispatch outcome schema');
  if (outcome.outcome === 'accepted') {
    assert(outcome.snapshot && typeof outcome.snapshot === 'object' && !Array.isArray(outcome.snapshot), 'accepted result lacks snapshot');
  } else if (outcome.outcome === 'rejected') {
    assert(['policy', 'input', 'evaluation', 'limit'].includes(outcome.origin), 'unknown/reserved core rejection origin');
    assert.equal(typeof outcome.code, 'string');
    assert(outcome.code.length > 0, 'rejection code is empty');
    assert.equal(typeof outcome.message, 'string');
    if (outcome.origin === 'policy') assert.equal(outcome.code, 'reject', 'unknown policy rejection code');
  } else throw new Error(`unexpected returned outcome: ${outcome.outcome}`);
}

// Only the explicit returned-rejected branch creates CoreRejection. Everything
// that escapes core execution, response parsing, or output reading is fatal.
export function instrumentSession(Session, registry = []) {
  return class InstrumentedSession {
    constructor(source) { this.initialize(new Session(source)); }

    initialize(inner) {
      this.inner = inner; this.id = registry.length + 1; this.records = [];
      this.coreCalls = 0; this.poisoned = false; registry.push(this);
    }

    static restore(source, saved) {
      const restored = Object.create(this.prototype);
      restored.initialize(Session.restore(source, saved));
      return restored;
    }

    ensureHealthy() {
      if (this.poisoned) throw new AuditFatal('session was discarded after a fatal dispatch');
    }

    read(method) {
      this.ensureHealthy();
      try { return this.inner[method](); }
      catch (error) {
        this.poisoned = true;
        throw new AuditFatal(`fatal ${method} read: ${String(error)}`, error);
      }
    }

    view() { return this.read('view'); }
    save() { return this.read('save'); }
    snapshot() { return this.read('snapshot'); }

    capture() {
      this.ensureHealthy();
      try {
        const raw = { save: this.save(), view: this.view(), snapshot: this.snapshot() };
        return { values: Object.fromEntries(Object.entries(raw).map(([key, value]) => [key, JSON.parse(value)])),
          hashes: Object.fromEntries(Object.entries(raw).map(([key, value]) => [key, sha256(value)])) };
      } catch (error) {
        this.poisoned = true;
        throw error instanceof AuditFatal ? error : new AuditFatal(`fatal state capture: ${String(error)}`, error);
      }
    }

    dispatch_view(event, payload) {
      this.ensureHealthy();
      const record = { session: this.id, event, payload, outcome: null, responseSha256: null, before: null, after: null };
      this.records.push(record);
      let outcome;
      let shown;
      try {
        const before = this.capture();
        record.before = before.hashes;
        this.coreCalls++;
        const raw = this.inner.dispatch_outcome(event, payload);
        record.responseSha256 = sha256(raw);
        outcome = JSON.parse(raw);
        validateOutcome(outcome);
        record.outcome = outcome;
        const after = this.capture();
        record.after = after.hashes;
        if (outcome.outcome === 'rejected') {
          assert.deepEqual(after.values, before.values, 'core rejection changed save/view/snapshot');
          record.atomic = true;
        } else {
          assert.deepEqual(outcome.snapshot, after.values.snapshot, 'accepted report is not the complete current snapshot');
          shown = JSON.stringify(after.values.view);
        }
      } catch (error) {
        this.poisoned = true;
        // Do not read save/view/snapshot or call further WASM after an exception.
        record.fatal = { error: String(error), coreCalls: this.coreCalls };
        throw new AuditFatal(`fatal structured dispatch for ${event}: ${String(error)}`, error);
      }
      if (outcome.outcome === 'rejected') throw new CoreRejection(outcome);
      return shown;
    }

    free() {
      if (!this.poisoned) this.inner.free();
      // A trapped instance is discarded, never probed to prove it survived.
    }
  };
}

export function sendAudited(handle, event) {
  const { policy, session } = handle;
  const before = session.capture();
  const publicBefore = policy.view();
  const callsBefore = session.coreCalls;
  const recordsBefore = session.records.length;
  let caught = null;
  try { policy.dispatch(event); } catch (error) { caught = error; }
  const records = session.records.slice(recordsBefore);
  let category;
  if (caught instanceof CoreRejection && records.length === 1 && records[0].outcome?.outcome === 'rejected') {
    category = `core_${caught.outcome.origin}_rejection`;
  } else if (caught && session.coreCalls === callsBefore && records.length === 0) {
    category = 'adapter_exception';
  } else if (caught) {
    throw caught instanceof AuditFatal ? caught : new AuditFatal(`exception after core dispatch: ${String(caught)}`, caught);
  } else {
    assert.equal(records.length, 1, 'adapter accepted without exactly one core call');
    assert.equal(records[0].outcome?.outcome, 'accepted', 'adapter accepted without a core accepted report');
    category = 'core_accepted';
  }
  const accepted = category === 'core_accepted';
  let atomic = null;
  if (!accepted) {
    const after = session.capture();
    assert.deepEqual(after.values, before.values, 'refusal changed save/view/snapshot');
    assert.deepEqual(policy.view(), publicBefore, 'refusal changed adapter projection');
    atomic = { save: true, view: true, snapshot: true, projection: true, before: before.hashes, after: after.hashes };
  }
  return { session: session.id, accepted, category,
    ...(category === 'adapter_exception' ? { error: String(caught), coreCalls: 0 } : {}),
    core: records[0] ?? null, atomic };
}

function emptyCounts() {
  return { dispatchSteps: 0, core_accepted: 0, core_policy_rejection: 0, core_input_rejection: 0,
    core_evaluation_rejection: 0, core_limit_rejection: 0, adapter_exception: 0, fatal: 0 };
}

function count(into, attempt) { into.dispatchSteps++; into[attempt.category]++; }

async function fileEvidence(file, unchanged = false) {
  const bytes = await readFile(path.join(root, file));
  const result = { file, bytes: bytes.length, sha256: sha256(bytes) };
  if (unchanged) {
    const historical = execFileSync('git', ['show', `${historicalBase}:${file}`], { cwd: root, maxBuffer: 8 * 1024 * 1024 });
    result.baselineSha256 = sha256(historical);
    result.baselineByteMatch = sha256(bytes) === result.baselineSha256;
    if (!result.baselineByteMatch) {
      // The existing scenarios checkout uses CRLF; preserve its actual bytes
      // while proving that line endings are the entire difference from Git.
      assert.equal(bytes.toString('utf8').replaceAll('\r\n', '\n'), historical.toString('utf8'), `historical file changed: ${file}`);
      result.baselineDifference = 'working-tree CRLF versus Git LF only';
    }
    result.unchangedFrom = historicalBase;
  }
  return result;
}

export async function runTrailRescueAudit() {
  const report = { schema: 1, kind: 'new_supplemental_runtime_run', started: new Date().toISOString(), pass: false,
    scope: 'Exactly the 24 registered scenarios; original five additional checks (including fuzz) are excluded. Frozen historical results are not rescored.',
    expectationInterpretation: 'The unchanged legacy reject booleans specify admission only. Actual core origins and adapter exceptions are recorded separately; adapter exceptions never count as policy evidence.',
    historicalBase, evidence: [], counts: { registered: emptyCounts(), withShadows: emptyCounts(), explicitResumes: 0, finalResumes: 0 },
    scenarios: [], traceSha256: null };
  let active = null;
  let live = [];
  try {
    for (const file of ['game/trail_rescue.cav', 'web/trail-rescue-policy.js', 'experiments/trail-rescue/adapter.mjs',
      'experiments/trail-rescue/scenarios.json', 'experiments/trail-rescue/harness.mjs']) report.evidence.push(await fileEvidence(file, true));
    for (const file of ['dist/pkg-reactive/caveat_runtime.js', 'dist/pkg-reactive/caveat_runtime_bg.wasm', 'dist/build-info.json',
      'runtime/src/reactive.rs', 'runtime/src/reactive_outcome.rs', 'runtime/src/web.rs', 'spec/caveat-dispatch-0.1.md',
      'experiments/dispatch-audit/trail-rescue.mjs', 'experiments/dispatch-audit/trail-rescue.test.mjs']) {
      report.evidence.push(await fileEvidence(file));
    }
    report.build = JSON.parse(await readFile(path.join(root, 'dist/build-info.json'), 'utf8'));
    report.checkout = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
    const { default: init, WebReactiveSession } = await import('../../dist/pkg-reactive/caveat_runtime.js');
    assert.equal(typeof WebReactiveSession.prototype.dispatch_outcome, 'function', 'new structured WASM build is required');
    await init({ module_or_path: await readFile(path.join(root, 'dist/pkg-reactive/caveat_runtime_bg.wasm')) });
    const source = await readFile(path.join(root, 'game/trail_rescue.cav'), 'utf8');
    const contract = JSON.parse(await readFile(path.join(root, 'experiments/trail-rescue/scenarios.json'), 'utf8'));
    assert.equal(contract.scenarios.length, 24, 'registered scenario inventory changed');
    assert.deepEqual(contract.scenarios.map(scenario => scenario.id), Array.from({ length: 24 }, (_, index) => `TR${String(index + 1).padStart(2, '0')}`));
    const registry = [];
    const Session = instrumentSession(WebReactiveSession, registry);
    const create = saved => {
      const policy = createPolicyFromSession(Session, source, saved);
      return { policy, session: registry.at(-1) };
    };
    for (const scenario of contract.scenarios) {
      active = { id: scenario.id, title: scenario.title, pass: false, steps: [], finalResume: null };
      report.scenarios.push(active);
      let handle = create(); live = [handle];
      assert.deepEqual(handle.policy.view(), contract.initialView, `${scenario.id}: initial projection`);
      for (const [index, step] of scenario.steps.entries()) {
        const trace = { step: index + 1, op: step.op, expectedProjection: step.expect ?? {} };
        active.steps.push(trace);
        if (step.op === 'resume') {
          const before = handle.session.capture();
          const restored = create(clone(handle.policy.save()));
          live.push(restored);
          assert.deepEqual(restored.policy.view(), handle.policy.view(), 'resume changed the full projection');
          const after = restored.session.capture();
          assert.deepEqual(after.values, before.values, 'resume changed save/view/snapshot');
          trace.restore = { from: handle.session.id, to: restored.session.id, before: before.hashes, after: after.hashes };
          handle = restored; report.counts.explicitResumes++;
        } else {
          assert.equal(step.op, 'dispatch');
          trace.event = step.event; trace.expectedAccepted = !step.reject; trace.attempts = [];
          for (const shadow of live) {
            let attempt;
            try { attempt = sendAudited(shadow, step.event); }
            catch (error) {
              trace.attempts.push({ session: shadow.session.id, category: 'fatal', error: String(error), core: shadow.session.records.at(-1) ?? null });
              count(report.counts.withShadows, { category: 'fatal' });
              if (shadow === live[0]) count(report.counts.registered, { category: 'fatal' });
              throw error;
            }
            trace.attempts.push(attempt); count(report.counts.withShadows, attempt);
          }
          count(report.counts.registered, trace.attempts[0]);
          assert.equal(trace.attempts[0].accepted, !step.reject, `${scenario.id} step ${index + 1}: unexpected admission`);
          assert(trace.attempts.every(attempt => attempt.accepted === trace.attempts[0].accepted), 'resumed shadow disagreed on acceptance');
          assert(trace.attempts.every(attempt => attempt.category === trace.attempts[0].category), 'resumed shadow disagreed on rejection origin');
          for (const shadow of live) assert.deepEqual(shadow.policy.view(), handle.policy.view(), 'resumed shadow diverged');
        }
        const projection = handle.policy.view();
        for (const [pointer, expected] of Object.entries(step.expect ?? {})) assert.deepEqual(atPointer(projection, pointer), expected, `${scenario.id} step ${index + 1} ${pointer}`);
        trace.projection = projection;
      }
      const before = handle.session.capture();
      const final = create(clone(handle.policy.save())); live.push(final);
      assert.deepEqual(final.policy.view(), handle.policy.view(), 'final resume changed the full projection');
      const after = final.session.capture();
      assert.deepEqual(after.values, before.values, 'final resume changed save/view/snapshot');
      active.finalResume = { from: handle.session.id, to: final.session.id, before: before.hashes, after: after.hashes };
      report.counts.finalResumes++; active.pass = true;
      for (const shadow of live) shadow.policy.free();
      live = []; active = null;
    }
    // Recheck hashes after the run: the evidence cannot silently change during it.
    for (const item of report.evidence) assert.equal((await fileEvidence(item.file)).sha256, item.sha256, `input changed during run: ${item.file}`);
    report.pass = true;
  } catch (error) {
    report.failure = { scenario: active?.id ?? null, step: active?.steps.at(-1)?.step ?? null, error: String(error) };
    if (active) active.error = String(error);
  } finally {
    for (const handle of live) {
      try { handle.policy.free(); } catch (error) { report.cleanupError = String(error); report.pass = false; }
    }
    report.traceSha256 = sha256(JSON.stringify(report.scenarios));
    report.finished = new Date().toISOString();
  }
  return report;
}

export function trailMarkdown(report, archive = null) {
  const tally = label => {
    const counts = report.counts[label];
    return `| ${label === 'registered' ? 'Registered dispatch steps' : 'All attempts including restored shadows'} | ${counts.dispatchSteps} | ${counts.core_accepted} | ${counts.core_policy_rejection} | ${counts.core_input_rejection} | ${counts.core_evaluation_rejection} | ${counts.core_limit_rejection} | ${counts.adapter_exception} | ${counts.fatal} |`;
  };
  const refusals = report.scenarios.flatMap(scenario => scenario.steps.flatMap(step => (step.attempts ?? []).slice(0, 1)
    .filter(attempt => attempt.category !== 'core_accepted').map(attempt => {
      const core = attempt.core?.outcome;
      return `| ${scenario.id} | ${step.step} | ${attempt.category} | ${core?.code ?? ''} | ${(core?.message ?? attempt.error ?? '').replaceAll('|', '\\|')} |`;
    })));
  return `# Trail Rescue supplemental structured-outcome audit\n\n` +
    `This is a **new supplemental run on the new structured runtime**, not a rescore or reconstruction of the frozen historical results. Status: **${report.pass ? 'PASS' : 'FAIL'}** (${report.scenarios.filter(scenario => scenario.pass).length}/24 scenarios).\n\n` +
    `Run \`node experiments/dispatch-audit/trail-rescue.mjs\` after building the current WASM. It runs the exact 24 histories in \`experiments/trail-rescue/scenarios.json\`, through the unchanged \`web/trail-rescue-policy.js\` adapter and unchanged \`game/trail_rescue.cav\`. Their contents are checked against commit \`${historicalBase}\` before replay. Actual working-tree and baseline SHA-256 hashes are retained: the existing scenarios checkout differs only by CRLF versus Git LF, which is explicitly verified without editing the file. The other historical input files match baseline bytes. The original harness and adapter files remain unchanged. The original five additional checks, including its 200 × 80 seeded fuzz run, are outside this supplemental audit.\n\n` +
    `The delegate translates a returned core rejection into a private tagged exception solely to satisfy the unchanged adapter interface. Exceptions before any core call are \`adapter_exception\`, not core policy or structured host refusals. Every escaped core exception, protocol/JSON failure, or output failure stops the run as fatal; a failed core session is discarded without further save/view/snapshot probes. The legacy scenario reject booleans still check admission only. Actual structured origins below are separate evidence, not a retroactive policy interpretation.\n\n` +
    `| Scope | Dispatches | Accepted | Policy rejected | Input rejected | Evaluation rejected | Limit rejected | Adapter exceptions | Fatal |\n|---|---:|---:|---:|---:|---:|---:|---:|---:|\n${tally('registered')}\n${tally('withShadows')}\n\n` +
    `Checked every declared projection and admission expectation, every refusal's save/view/snapshot and public projection atomicity, ${report.counts.explicitResumes} explicit restores with shadow continuation, and ${report.counts.finalResumes} final restores. Accepted reports are compared to the complete current snapshot. The JSON captures the complete core reports, adapter exception messages, exact events, projections, restore/atomicity hashes, and scenario traces.\n\n` +
    `## Refusals in registered histories\n\n| Scenario | Step | Observed category | Code | Message |\n|---|---:|---|---|---|\n${refusals.join('\n')}\n\n` +
    `## Reproduction evidence\n\nThe complete JSON report is stored in [result-trail-rescue.json.gz](result-trail-rescue.json.gz). ${archive ? `It contains ${archive.uncompressedBytes.toLocaleString('en-US')} bytes of JSON in ${archive.compressedBytes.toLocaleString('en-US')} compressed bytes; decompression was verified byte-for-byte after writing. Archive SHA-256: \`${archive.sha256}\`. ` : ''}All core reports, snapshots, events, and provenance remain in the archive. Read it from the repository root with:\n\n` +
    '```sh\nnode -e "process.stdout.write(require(\'node:zlib\').gunzipSync(require(\'node:fs\').readFileSync(\'experiments/dispatch-audit/result-trail-rescue.json.gz\')))"\n```\n\n' +
    `Trace SHA-256: \`${report.traceSha256}\`. The JSON records the runtime JS/WASM hashes, build metadata, runtime source/spec hashes, script hash, source/adapter/scenario/harness hashes, and start/finish times. Restored shadow attempts are separated from registered steps to avoid inflating the scenario rejection count.\n` +
    (report.failure ? `\nFailure: ${report.failure.error}\n` : '');
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const report = await runTrailRescueAudit();
  const json = Buffer.from(`${JSON.stringify(report, null, 2)}\n`);
  const compressed = gzipSync(json, { level: 9 });
  const archiveFile = path.join(here, 'result-trail-rescue.json.gz');
  await writeFile(archiveFile, compressed);
  assert.deepEqual(gunzipSync(await readFile(archiveFile)), json, 'written archive does not round-trip to the complete report');
  const archive = { uncompressedBytes: json.length, compressedBytes: compressed.length, sha256: sha256(compressed) };
  await writeFile(path.join(here, 'TRAIL_RESCUE.md'), trailMarkdown(report, archive));
  console.log(JSON.stringify({ pass: report.pass, scenariosPassed: report.scenarios.filter(scenario => scenario.pass).length, counts: report.counts, archive, failure: report.failure }, null, 2));
  if (!report.pass) process.exitCode = 1;
}
