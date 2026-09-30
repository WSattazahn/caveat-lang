// The agent-evidence example: its program loads and checks clean, and its
// Python caller, driving this checkout's `caveat serve`, reaches the
// conclusions its README states.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { runCallerTests } from './python.mjs';

const kit = fileURLToPath(new URL('../', import.meta.url));
const example = path.join(kit, 'examples', 'agent-evidence');
const cli = path.join(kit, 'bin', 'caveat.mjs');
const caveat = (...args) => spawnSync(process.execPath, [cli, ...args], { cwd: example, encoding: 'utf8' });

test('the example program loads and has no warnings', () => {
  const validated = caveat('validate', 'assessment.cav');
  assert.equal(validated.status, 0, validated.stdout + validated.stderr);
  assert.match(validated.stdout, /^assessment\.cav loads\.\n {2}events: assess, erase, observe \(confidence 0\.\.100\), retract\n/);
  const checked = caveat('check', '--strict', 'assessment.cav');
  assert.equal(checked.status, 0, checked.stdout + checked.stderr);
  assert.equal(checked.stdout.trim(), 'assessment.cav: no warnings.');
});

// Each situation in the README's table is a test in test_caller.py; they all
// run, against a real server, and pass. Nothing is left in the example.
test('the Python caller\'s tests pass against this checkout\'s caveat serve', async () => {
  const result = await runCallerTests(example, [process.execPath, cli]);
  assert.equal(result.status, 0, `${result.python}\n${result.stdout}${result.stderr}`);
  for (const name of [
    'test_a_rejected_required_operation_fails_the_attempt_despite_an_old_approval',
    'test_withdrawn_evidence_reopens_the_approval_by_the_programs_policy',
    'test_a_forbidden_action_is_refused',
    'test_a_later_attempt_with_replacement_evidence_is_judged_on_its_own',
    'test_an_earlier_approval_does_not_stand_in_for_this_attempt',
    'test_a_malformed_request_changes_nothing',
    'test_a_program_that_does_not_load_is_reported',
  ]) assert.match(result.report, new RegExp(`^${name} .* ok$`, 'm'), `${name} did not pass`);
  assert.match(result.report, /\nOK\n?$/);
  assert.doesNotMatch(result.report, /ResourceWarning/, 'the caller leaves a pipe or file open');
  assert.deepEqual(result.after, result.before, 'the tests left files in the example');
});
