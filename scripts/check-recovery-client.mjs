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
    for (const method of ['dispatch', 'dispatchView']) {
      const session = runtime.open(fixture.source);
      let restored;
      try {
        assert.equal(session.dispatch('setup').outcome, 'accepted');
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
          assert.equal(current.snapshot().sequence, 2);
        }
        assert.deepEqual(checkpoint(restored), checkpoint(session));
      } finally { restored?.close(); session.close(); }
    }

    const server = createServer({ runtime, source: fixture.source, program: fixture.name });
    const send = request => server.handle(JSON.stringify(request));
    try {
      assert.equal(send({ op: 'dispatch', event: 'setup' }).response.outcome, 'accepted');
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
      { id: 1, op: 'dispatch', event: 'setup' },
      { id: 2, op: 'save' },
      { id: 3, op: 'dispatch', event: 'fail' },
      { id: 4, op: 'save' },
      { id: 5, op: 'dispatch', event: 'resume_work', snapshot: true },
      { id: 6, op: 'close' },
    ];
    const child = spawnSync(command[0], [...command.slice(1), 'serve', program], {
      cwd: directory, input: requests.map(request => JSON.stringify(request)).join('\n') + '\n',
      encoding: 'utf8', timeout: 30000, windowsHide: true,
    });
    assert.equal(child.error, undefined, child.error?.message);
    assert.equal(child.status, 0, `${fixture.name}: ${child.stderr}\n${child.stdout}`);
    const [ready, ...responses] = child.stdout.trim().split(/\r?\n/).map(line => JSON.parse(line));
    assert.equal(ready.ready, true);
    assert.deepEqual(responses.map(response => response.id), [1, 2, 3, 4, 5, 6]);
    assert.equal(responses[2].ok, true);
    assertRefusal(responses[2], fixture);
    assert.equal(responses[3].save, responses[1].save);
    assert.equal(responses[4].outcome, 'accepted');
    assert.equal(responses[4].snapshot.bindings.hud.kept, 2);
    assert.equal(responses[4].snapshot.sequence, 2);
    checks.push({ name: fixture.name, origin: fixture.origin, code: fixture.code,
      dispatch: true, dispatchView: true, restoreAndContinue: true, server: true, cli: true });
  }
  return checks;
}
