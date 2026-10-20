import assert from 'node:assert/strict';
import test from 'node:test';
import { HOST_EVENTS, SOURCES_PROGRAM, TRANSLATION_CONTROLS, controlEvents, history, modelRequest, runtimeStates } from './lean-sources-histories.mjs';
import { canonical, firstMismatch } from './verify-lean-sources.mjs';

test('every host event is declared and has exactly one rule in the program', () => {
  for (const name of Object.keys(HOST_EVENTS)) {
    assert.match(SOURCES_PROGRAM, new RegExp('^event ' + name + ';$', 'm'));
    assert.equal(SOURCES_PROGRAM.match(new RegExp('^on ' + name + ' ', 'gm'))?.length, 1, name);
  }
  assert.equal(SOURCES_PROGRAM.match(/^on /gm).length, Object.keys(HOST_EVENTS).length);
});

test('histories are deterministic and only run events whose guards hold', () => {
  const first = [...history(7n, 200)];
  assert.deepEqual([...history(7n, 200)], first);
  assert.equal(first.length, 200);
  let committed = false;
  let fSeen = false;
  for (const event of first) {
    if (event === 'recall') assert.ok(committed);
    if (event === 'decide') { assert.ok(!committed); committed = true; }
    if (event === 'renew_ef') assert.ok(fSeen);
    if (event === 'reveal_f') fSeen = true;
    if (event === 'renew_f') fSeen = false;
  }
});

test('a refused event changes nothing a guard reads', () => {
  const events = [];
  const generator = history(11n, 300);
  let next = generator.next();
  while (!next.done) {
    events.push(next.value);
    next = generator.next(next.value !== 'reveal_f' && next.value !== 'decide');
  }
  assert.ok(!events.includes('recall'));
  assert.ok(!events.includes('renew_ef'));
});

test('requests and projections have the protocol shape', () => {
  const request = modelRequest('t', ['reveal_e', 'read']);
  assert.deepEqual(request.events[1], [{ set: { state: 'v', body: [{ evidence: 'e' }], guard: [] } }]);
  assert.deepEqual(runtimeStates({ qualified_values: { v: { provenance: { caveats: ['b', 'a', 'a'] } } } }).v,
    { lineage: ['a', 'b'], grounds: [] });
});

test('controls change one translation and comparisons ignore key order only', () => {
  for (const control of TRANSLATION_CONTROLS) {
    const events = controlEvents(control);
    assert.notDeepEqual(events[control.event].model, HOST_EVENTS[control.event].model);
    for (const name of Object.keys(HOST_EVENTS)) if (name !== control.event) assert.equal(events[name], HOST_EVENTS[name]);
  }
  assert.equal(canonical({ b: 1, a: [2, { d: 3, c: 4 }] }), canonical({ a: [2, { c: 4, d: 3 }], b: 1 }));
  assert.equal(firstMismatch([{ a: 1, b: 2 }], [{ b: 2, a: 1 }]), null);
  assert.equal(firstMismatch([{ a: 1 }], [{ a: 2 }]).index, 0);
  assert.equal(firstMismatch([{ a: 1 }], []).index, 0);
});
