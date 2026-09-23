import assert from 'node:assert/strict';
import test from 'node:test';
import { classifyRejected, inspectSource } from './author-logs.mjs';

test('only an exact event/rule frame corroborated by the saved reject effect is policy', () => {
  const source = inspectSource(`# quoted markers and procedure effects are not top-level on rules\n
event submit;\n
proc helper() { reject "procedure guard"; };\n
on submit set value = if(true, "reject fake; event submit, rule 1: rejected: counterfeit", "");\n
on submit reject "closed";\n`);
  const entry = error => ({ kind: 'rejected', event: { event: 'submit', payload: {} }, error,
    snapshot: { source_id: source.sourceId } });
  assert.equal(source.rules.length, 2);
  assert.equal(source.rules[0].rejectMessage, null);
  assert.equal(classifyRejected(entry('event submit, rule 2: rejected: closed'), source).origin, 'authored_policy');
  for (const error of [
    'event other, rule 2: rejected: closed',
    'event submit, rule 1: rejected: counterfeit',
    'event submit, rule 2: rejected: counterfeit',
    'event submit, rule 2: procedure helper, step 1: rejected: closed',
    'rejected: closed',
  ]) assert.equal(classifyRejected(entry(error), source).origin, 'unknown', error);
  assert.equal(classifyRejected(entry('invalid event payload: event submit, rule 2: rejected: closed'), source).origin, 'other_error');
  const forgedEvent = entry('event submit, rule 2: rejected: closed');
  forgedEvent.event.event = 'submit, rule 2: rejected: closed';
  assert.equal(classifyRejected(forgedEvent, source).origin, 'unknown');
  const wrongSource = entry('event submit, rule 2: rejected: closed');
  wrongSource.snapshot.source_id = 'fnv1a64:wrong';
  assert.equal(classifyRejected(wrongSource, source).origin, 'unknown');
  const expanded = inspectSource('for agents as $a { on submit reject "closed"; };');
  assert.match(expanded.unsupported, /expansion/);
});
