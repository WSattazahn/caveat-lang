// Exercise the host contracts with an injected runtime and the actual CLI.
// Used both by repository tests and against the freshly installed tarball.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';

export const RECOVERY_CASES = [
  {
    name: 'examination-attention', origin: 'limit', code: 'attention_limit',
    source: `budget 1;
caveat stale consequence low;
caveat dusty consequence low;
state kept = 0;
event setup;
event fail;
event resume_work;
on setup set kept = 1;
on fail set kept = 99;
on fail examine stale cost 1;
on fail examine dusty cost 1;
on resume_work set kept = 2;
bind hud.kept = kept;
`,
  },
  {
    name: 'empty-caveated-selection', origin: 'evaluation', code: 'empty_caveated_selection',
    source: `claim known;
evidence memo from "the observed memo";
caveat phantom consequence low;
state basis = 0;
state kept = 0;
decisions plan limit 4;
event setup;
event fail;
event resume_work;
on setup reveal memo supports known;
on setup set basis = qualified(1, memo, phantom);
on setup set kept = 1;
on setup commit plan because enough using basis;
on fail set kept = 99;
on fail reopen plan because caveated(basis, phantom);
on resume_work set kept = 2;
bind hud.kept = kept;
`,
  },
  {
    name: 'unobserved-evidence', origin: 'evaluation', code: 'unobserved_evidence',
    source: `claim known;
evidence memo from "the memo";
caveat stale consequence low;
state kept = 0;
event setup;
event see;
event fail;
event resume_work;
on setup set kept = 1;
on see reveal memo supports known;
on fail set kept = 99;
on fail qualify memo with stale;
on resume_work set kept = 2;
bind hud.kept = kept;
`,
  },
  {
    name: 'unobserved-qualified-read', origin: 'evaluation', code: 'unobserved_evidence',
    source: `claim known;
evidence memo from "the memo";
state kept = 0;
event setup;
event see;
event fail;
event resume_work;
on setup set kept = 1;
on see reveal memo supports known;
on fail set kept = 99;
on fail set kept = kept + qualified(1, memo);
on resume_work set kept = 2;
bind hud.kept = kept;
`,
  },
  {
    name: 'decision-without-value', origin: 'evaluation', code: 'expression',
    source: `state kept = 0;
decisions plan limit 4;
event setup;
event fail;
event resume_work;
on setup set kept = 1;
on setup commit plan because enough;
on fail set kept = 99;
on fail set kept = latest(plan);
on resume_work set kept = 2;
bind hud.kept = kept;
`,
  },
  {
    name: 'not-committed', origin: 'evaluation', code: 'not_committed',
    source: `claim known;
evidence memo from "the memo";
decisions plan limit 2;
state kept = 0;
event setup;
event fail;
event resume_work;
on setup reveal memo supports known;
on setup set kept = 1;
on fail set kept = 99;
on fail reopen plan because memo;
on resume_work set kept = 2;
bind hud.kept = kept;
`,
  },
  {
    name: 'expression', origin: 'evaluation', code: 'expression',
    source: `state kept = 0;
state divisor = 0;
event setup;
event fail;
event resume_work;
on setup set kept = 1;
on fail set kept = 99;
on fail set kept = 1 / divisor;
on resume_work set kept = 2;
bind hud.kept = kept;
`,
  },
  {
    name: 'requirement-failed', origin: 'evaluation', code: 'requirement_failed',
    source: `state kept = 0;
event setup;
event fail;
event resume_work;
on setup set kept = 1;
on fail set kept = 99;
on fail set kept = require(kept < 0, 1);
on resume_work set kept = 2;
bind hud.kept = kept;
`,
  },
  {
    // Each fill event schedules 64; 64 of them leave the table full.
    name: 'scheduled-limit', origin: 'limit', code: 'scheduled_limit', fill: { event: 'load', times: 64 },
    source: `claim known;
evidence memo from "the memo";
caveat stale consequence low;
state kept = 0;
event setup;
event load;
event fail;
event resume_work;
event advance dt min 0 max 1;
clock advance every 1;
on setup reveal memo supports known;
on setup set kept = 1;
${'on load qualify memo with stale after 100;\n'.repeat(64)}on fail set kept = 99;
on fail qualify memo with stale after 100;
on resume_work set kept = 2;
bind hud.kept = kept;
`,
  },
];

function assertRefusal(outcome, fixture) {
  assert.equal(outcome.outcome, 'rejected', fixture.name);
  assert.equal(outcome.origin, fixture.origin, fixture.name);
  assert.equal(outcome.code, fixture.code, fixture.name);
}
const checkpoint = session => ({ save: session.save(), snapshot: session.snapshot(), view: session.view() });

export async function checkRecoveryClient({ runtime, createServer, command, directory, cases = RECOVERY_CASES }) {
  await mkdir(directory, { recursive: true });
  const checks = [];
  for (const fixture of cases) {
    const fills = Array.from({ length: fixture.fill?.times ?? 0 }, () => fixture.fill.event);
    const sequence = 2 + fills.length;
    for (const method of ['dispatch', 'dispatchView']) {
      const session = runtime.open(fixture.source);
      let restored;
      try {
        assert.equal(session.dispatch('setup').outcome, 'accepted');
        for (const event of fills) assert.equal(session.dispatch(event).outcome, 'accepted');
        const before = checkpoint(session);
        assertRefusal(session[method]('fail'), fixture);
        assert.deepEqual(checkpoint(session), before, `${fixture.name}: ${method} rollback`);
        restored = runtime.restore(fixture.source, session.save());
        assert.deepEqual(checkpoint(restored), before, `${fixture.name}: restore after refusal`);
        assertRefusal(restored[method]('fail'), fixture);
        assert.deepEqual(checkpoint(restored), before);
        for (const current of [session, restored]) {
          assert.equal(current[method]('resume_work').outcome, 'accepted');
          assert.equal(current.snapshot().bindings.hud.kept, 2);
          assert.equal(current.snapshot().sequence, sequence);
        }
        assert.deepEqual(checkpoint(restored), checkpoint(session));
      } finally { restored?.close(); session.close(); }
    }

    const server = createServer({ runtime, source: fixture.source, program: fixture.name });
    const send = request => server.handle(JSON.stringify(request));
    try {
      assert.equal(send({ op: 'dispatch', event: 'setup' }).response.outcome, 'accepted');
      for (const event of fills) assert.equal(send({ op: 'dispatch', event }).response.outcome, 'accepted');
      const before = send({ op: 'save' }).response.save;
      const refused = send({ op: 'dispatch', event: 'fail' });
      assert.equal(refused.exit, null, 'a refusal keeps the server running');
      assert.equal(refused.response.ok, true, 'request handling and event acceptance are separate');
      assertRefusal(refused.response, fixture);
      assert.equal(send({ op: 'save' }).response.save, before);
      assert.equal(send({ op: 'dispatch', event: 'resume_work' }).response.outcome, 'accepted');
      assert.equal(send({ op: 'snapshot' }).response.snapshot.bindings.hud.kept, 2);
    } finally { send({ op: 'close' }); }

    const program = path.join(directory, `${fixture.name}.cav`);
    await writeFile(program, fixture.source);
    const requests = [
      { op: 'dispatch', event: 'setup' },
      ...fills.map(event => ({ op: 'dispatch', event })),
      { op: 'save' },
      { op: 'dispatch', event: 'fail' },
      { op: 'save' },
      { op: 'dispatch', event: 'resume_work', snapshot: true },
      { op: 'close' },
    ].map((request, index) => ({ id: index + 1, ...request }));
    const child = spawnSync(command[0], [...command.slice(1), 'serve', program], {
      cwd: directory, input: requests.map(request => JSON.stringify(request)).join('\n') + '\n',
      encoding: 'utf8', timeout: 30000, windowsHide: true, maxBuffer: 64 * 1024 * 1024,
    });
    assert.equal(child.error, undefined, child.error?.message);
    assert.equal(child.status, 0, `${fixture.name}: ${child.stderr}\n${child.stdout}`);
    const [ready, ...all] = child.stdout.trim().split(/\r?\n/).map(line => JSON.parse(line));
    assert.equal(ready.ready, true);
    assert.deepEqual(all.map(response => response.id), requests.map(request => request.id));
    for (const response of all.slice(1, 1 + fills.length)) assert.equal(response.outcome, 'accepted');
    const responses = [all[0], ...all.slice(1 + fills.length)];
    assert.equal(responses[2].ok, true);
    assertRefusal(responses[2], fixture);
    assert.equal(responses[3].save, responses[1].save);
    assert.equal(responses[4].outcome, 'accepted');
    assert.equal(responses[4].snapshot.bindings.hud.kept, 2);
    assert.equal(responses[4].snapshot.sequence, sequence);
    checks.push({ name: fixture.name, origin: fixture.origin, code: fixture.code,
      dispatch: true, dispatchView: true, restoreAndContinue: true, server: true, cli: true });
  }
  return checks;
}
