// Candidate-side W01-W20 regressions from the bounded 590fae5 adversarial pass.
// See fixtures/adversarial/README.md for provenance and coverage limits.
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import test from 'node:test';
import {real} from './helpers.mjs';

const clone = value => JSON.parse(JSON.stringify(value));
const normalized = value => value && typeof value === 'object'
  ? Array.isArray(value) ? value.map(normalized) : Object.fromEntries(Object.keys(value).sort().map(k => [k, normalized(value[k])]))
  : value;
const equal = (a, b, label) => assert.deepEqual(normalized(a), normalized(b), label);
const wd = snapshot => snapshot.withdrawals ?? [];
const STATIC = 'withdrawal-static.cav';
const OWN = 'withdrawal-own-grounds.cav';
const HISTORY = 'withdrawal-history.cav';
const CHAIN = 'withdrawal-reason-chain.cav';

const cases = [
  ['W01', STATIC, 'unchanged-guarantee', 'Unmodified withdrawal save restores exactly and continues identically.'],
  ['W02', STATIC, 'malformed-save-rejected', 'Unknown withdrawal subject is refused.'],
  ['W03', STATIC, 'malformed-save-rejected', 'Unknown withdrawal reason is refused.'],
  ['W04', STATIC, 'malformed-save-rejected', 'Declared but unobserved subject, with matching withdrawn edge and source event, is refused.'],
  ['W05', STATIC, 'malformed-save-rejected', 'Declared but unobserved reason, with matching source event, is refused.'],
  ['W06', STATIC, 'malformed-save-rejected', 'Duplicate withdrawal target is refused.'],
  ['W07', STATIC, 'malformed-save-rejected', 'Zero withdrawal sequence is refused.'],
  ['W08', STATIC, 'malformed-save-rejected', 'Withdrawal sequence after save sequence is refused.'],
  ['W09', STATIC, 'malformed-save-rejected', 'Undeclared withdrawal event is refused.'],
  ['W10', STATIC, 'malformed-save-rejected', 'Declared event with no reachable withdrawal is refused.'],
  ['W11', STATIC, 'malformed-save-rejected', 'Withdrawal record without its withdrawn graph edge is refused.'],
  ['W12', STATIC, 'malformed-save-rejected', 'Withdrawn graph edge without its record is refused.'],
  ['W13', STATIC, 'malformed-save-rejected', 'Unknown withdrawal-record field is refused.'],
  ['W14', STATIC, 'known-host-trust-limit', 'Source-possible edited withdrawal metadata is accepted even when that source guard was false; restore does not authenticate history.'],
  ['W15', STATIC, 'known-host-trust-limit', 'Coordinated withdrawal-record and graph-edge omission is accepted; restore does not authenticate history.'],
  ['W16', STATIC, 'unchanged-guarantee', 'Repeated withdrawal before and after restore keeps the first exact record/reason and emits no repeated withdraw effect; reason qualification remains.'],
  ['W17', STATIC, 'unchanged-guarantee', 'Policy refusal and late binding failure roll back withdrawal and preserve snapshot/save.'],
  ['W18', OWN, 'approved-lifecycle-difference', 'Two own-ground holders pin the self-reason across restore; failed last-root releases roll back; accepted final release archives it after final release.'],
  ['W19', HISTORY, 'approved-lifecycle-difference', 'Restore while historical holder pins reason; later read retains withdrawn/stale lineage and own grounds; final holder removal archives exact reason/reading after final holder release.'],
  ['W20', CHAIN, 'unchanged-guarantee', 'Distinct required withdrawal reason chain survives four cycles, restore and continuation; predicate carries actual qualified reason.'],
].map(([id, fixture, classification, expectation]) => ({ id, fixture, classification, expectation, originalFindingReplica: false }));


const errorValue = error => ({kind: error.kind, message: error.message});
function context(source) {
  const sessions = [];
  return {
    open() { const s = real.open(source); sessions.push(s); return s; },
    restore(text) { const s = real.restore(source, text); sessions.push(s); return s; },
    event(session, event, expected = 'accepted') {
      const result = session.dispatch(event, {});
      assert.equal(result.outcome, expected, `${event}: ${JSON.stringify(result)}`);
      return result;
    },
    drain(session) { return session.drainArchive(); },
    close() { for (const session of sessions) session.close(); },
  };
}

const edge = r => r[0] === 'withdrawn' && r[1] === 'qualifies' && r[2] === 'subject';
const removeEdge = save => { save.graph.relations = save.graph.relations.filter(r => !edge(r)); };
const mutations = {
  W02: s => { s.withdrawals[0].evidence = 'unknown_subject'; },
  W03: s => { s.withdrawals[0].because = 'unknown_reason'; },
  W04: s => { s.withdrawals[0].evidence = 'unseen'; s.withdrawals[0].event = 'unobserved_subject'; for (const r of s.graph.relations) if (edge(r)) r[2] = 'unseen'; },
  W05: s => { s.withdrawals[0].because = 'unseen'; s.withdrawals[0].event = 'unobserved_reason'; },
  W06: s => { s.withdrawals.push(clone(s.withdrawals[0])); },
  W07: s => { s.withdrawals[0].sequence = 0; },
  W08: s => { s.withdrawals[0].sequence = s.sequence + 1; },
  W09: s => { s.withdrawals[0].event = 'unknown_event'; },
  W10: s => { s.withdrawals[0].event = 'noop'; },
  W11: removeEdge,
  W12: s => { delete s.withdrawals; },
  W13: s => { s.withdrawals[0].forged_extra = true; },
  W14: s => { s.withdrawals[0].because = 'second'; s.withdrawals[0].event = 'dormant'; },
  W15: s => { delete s.withdrawals; removeEdge(s); },
};
function has(list, value, label) { assert(Array.isArray(list) && list.includes(value), `${label}: missing ${value}`); }
function records(archive) { return archive.filter(item => Number.isInteger(item.number) && typeof item.record === 'string' && item.operation === undefined); }
function recordNames(archive) { return records(archive).map(item => item.record); }
function staticBase(ctx) {
  const s = ctx.open();
  for (const event of ['setup', 'pull', 'noop']) ctx.event(s, event);
  const snapshot = s.snapshot();
  equal(wd(snapshot), [{ evidence: 'subject', because: 'reason', sequence: 2, event: 'pull' }], 'valid control withdrawal');
  assert.equal(snapshot.bindings.hud.withdrawn, true);
  const parsed = JSON.parse(s.save());
  assert.equal(parsed.graph.relations.filter(edge).length, 1, 'valid control has exactly one withdrawn edge');
  return s;
}
function predicateProjection(snapshot) {
  return { withdrawals: wd(snapshot), value: snapshot.qualified_values.reason_read, grounds: snapshot.value_grounds.reason_read, hud: snapshot.bindings.hud };
}

async function probe(test, ctx) {
  if (mutations[test.id]) {
    const s = staticBase(ctx);
    const base = s.save();
    const valid = ctx.restore(base, 'positive-control');
    equal(valid.snapshot(), s.snapshot(), 'valid save positive control');
    const changed = JSON.parse(base);
    mutations[test.id](changed);
    assert.notEqual(JSON.stringify(changed), JSON.stringify(JSON.parse(base)), 'mutation changes input');
    if (test.classification === 'malformed-save-rejected') {
      let thrown = null;
      try { ctx.restore(JSON.stringify(changed), 'mutated'); } catch (error) { thrown = error; }
      assert(thrown, 'malformed withdrawal save was accepted');
      assert.equal(thrown.kind, 'restore', 'restore error classification (not fatal/load)');
      return { observedClass: 'restore-rejected', diagnostic: errorValue(thrown), projection: { outcome: 'restore-rejected' } };
    }
    const restored = ctx.restore(JSON.stringify(changed), 'edited-accepted');
    if (test.id === 'W14') {
      equal(wd(restored.snapshot()), changed.withdrawals, 'edited source-possible record preserved');
      ctx.event(restored, 'read_reason');
      const p = predicateProjection(restored.snapshot());
      has(p.grounds.evidence, 'second', 'edited predicate reason');
      assert(!p.grounds.evidence.includes('reason'), 'predicate still uses superseded edited reason');
      return { observedClass: 'source-possible-edit-accepted', projection: p };
    }
    assert.equal(wd(restored.snapshot()).length, 0);
    assert.equal(restored.snapshot().bindings.hud.withdrawn, false);
    ctx.event(restored, 'noop');
    return { observedClass: 'coordinated-omission-accepted', projection: { withdrawals: wd(restored.snapshot()), withdrawn: restored.snapshot().bindings.hud.withdrawn } };
  }
  if (test.id === 'W01') {
    const s = staticBase(ctx);
    const restored = ctx.restore(s.save());
    equal(s.snapshot(), restored.snapshot(), 'valid snapshot');
    equal(JSON.parse(s.save()), JSON.parse(restored.save()), 'valid save');
    for (const v of [s, restored]) ctx.event(v, 'read_reason');
    equal(s.snapshot(), restored.snapshot(), 'continuation');
    return { observedClass: 'valid-roundtrip', projection: predicateProjection(s.snapshot()) };
  }
  if (test.id === 'W16') {
    const s = staticBase(ctx);
    const first = clone(wd(s.snapshot()));
    ctx.event(s, 'repeat');
    equal(wd(s.snapshot()), first, 'first record is write-once');
    assert(!s.snapshot().effects.some(x => x.kind === 'withdraw'));
    const restored = ctx.restore(s.save());
    for (const v of [s, restored]) {
      ctx.event(v, 'repeat');
      equal(wd(v.snapshot()), first, 'write-once after restore');
      assert(!v.snapshot().effects.some(x => x.kind === 'withdraw'));
      ctx.event(v, 'read_reason');
      const p = predicateProjection(v.snapshot());
      has(p.grounds.evidence, 'reason', 'original reason grounds');
      has(p.grounds.caveats, 'stale', 'original reason qualification');
      assert(!p.grounds.evidence.includes('second'));
    }
    equal(s.snapshot(), restored.snapshot(), 'repeat continuation');
    return { observedClass: 'first-record-preserved', projection: predicateProjection(s.snapshot()) };
  }
  if (test.id === 'W17') {
    const s = ctx.open(); ctx.event(s, 'setup');
    const before = s.save(); const snapshot = s.snapshot();
    const outcomes = [];
    for (const event of ['refuse', 'fail']) {
      outcomes.push(ctx.event(s, event, 'rejected'));
      equal(s.snapshot(), snapshot, 'failed event snapshot rollback');
      assert.equal(s.save(), before, 'failed event exact save rollback');
      const archive = ctx.drain(s, event);
      if (archive) equal(archive, [], 'failed event archive rollback');
    }
    const restored = ctx.restore(before);
    for (const v of [s, restored]) ctx.event(v, 'pull');
    equal(s.snapshot(), restored.snapshot(), 'valid withdrawal after failures');
    return { observedClass: 'transaction-rollback', projection: { withdrawals: wd(s.snapshot()), rejected: outcomes.map(x => ({ outcome: x.outcome, origin: x.origin, code: x.code })) } };
  }
  if (test.id === 'W18') {
    const s = ctx.open();
    for (const event of ['cycle', 'hold', 'cycle', 'release_first']) ctx.event(s, event);
    const shot = s.snapshot();
    assert(shot.retired['a@2']);
    has(shot.value_grounds.second.evidence, 'a@2', 'last own-ground holder');
    const retainedRecord = clone(wd(shot).find(x => x.evidence === 'a@2'));
    assert(retainedRecord); assert.equal(retainedRecord.because, 'a@2');
    const saved = s.save(); const restored = ctx.restore(saved);
    equal(restored.snapshot(), shot, 'restore retains holder and reason');
    const beforeArchive = ctx.drain(s, 'before-release');
    if (beforeArchive) equal(beforeArchive, []);
    for (const v of [s, restored]) {
      for (const event of ['refuse', 'fail']) {
        ctx.event(v, event, 'rejected');
        assert.equal(v.save(), saved, 'last-root rollback');
        equal(v.snapshot(), shot, 'last-root snapshot rollback');
        const archive = ctx.drain(v, event); if (archive) equal(archive, []);
      }
      ctx.event(v, 'release_last');
    }
    equal(s.snapshot(), restored.snapshot(), 'same accepted release after restore');
    const archive = ctx.drain(s, 'final'); const restoredArchive = ctx.drain(restored, 'restored-final');
    {
      equal(archive, restoredArchive, 'ordered archive equality after restore');
      equal(recordNames(archive), ['a@2']);
      equal(records(archive)[0].withdrawal, retainedRecord, 'original archived withdrawal');
      assert(!s.snapshot().retired['a@2']);
    }
    ctx.restore(s.save(), 'after-release-roundtrip');
    return { observedClass: 'eligible-reason-archived', projection: { retainedRecord, ownGrounds: shot.value_grounds.second, finalFirst: s.snapshot().values.first, finalSecond: s.snapshot().values.second }, lifecycle: { activeRetiredReason: Boolean(s.snapshot().retired['a@2']), archivedRecords: archive ? recordNames(archive) : null } };
  }
  if (test.id === 'W19') {
    const s = ctx.open();
    for (const event of ['initialize', 'cycle']) ctx.event(s, event);
    assert(s.snapshot().retired['a@2']);
    const old = clone(wd(s.snapshot()).find(x => x.evidence === 'a@2'));
    assert(old); assert.equal(old.because, 'a@2');
    // Restore occurs before any later computation has copied the holder's value.
    const restored = ctx.restore(s.save(), 'live-history-holder');
    equal(restored.snapshot(), s.snapshot(), 'restore retains unread historical holder');
    const before = ctx.drain(s, 'while-held'); if (before) equal(before, []);
    let readProjection;
    for (const v of [s, restored]) {
      ctx.event(v, 'probe');
      const shot = v.snapshot();
      assert.equal(shot.bindings.hud.withdrawn, true); assert.equal(shot.bindings.hud.stale, true);
      for (const p of [shot.qualified_values.result.provenance, shot.value_grounds.result]) {
        has(p.evidence, 'a@2', 'later history provenance');
        has(p.caveats, 'withdrawn', 'later history withdrawal');
        has(p.caveats, 'stale', 'later history qualification');
      }
      readProjection = { result: shot.qualified_values.result, grounds: shot.value_grounds.result, old };
      ctx.event(v, 'clear'); ctx.event(v, 'replace');
    }
    equal(s.snapshot(), restored.snapshot(), 'same history continuation');
    const archive = ctx.drain(s, 'holder-released'); const restoredArchive = ctx.drain(restored, 'restored-holder-released');
    {
      equal(archive, restoredArchive, 'ordered archive after restored history');
      equal(recordNames(archive), ['a@2', 'kept@1']);
      equal(records(archive)[0].withdrawal, old);
      has(records(archive)[1].reading.provenance.evidence, 'a@2', 'archived reading provenance');
      assert(!s.snapshot().retired['a@2']);
    }
    ctx.restore(s.save(), 'after-holder-release');
    return { observedClass: 'read-preserved-then-archived', projection: readProjection, lifecycle: { activeRetiredReason: Boolean(s.snapshot().retired['a@2']), archivedRecords: archive ? recordNames(archive) : null } };
  }
  if (test.id === 'W20') {
    const s = ctx.open(); ctx.event(s, 'initialize');
    for (let i = 0; i < 4; i++) ctx.event(s, 'cycle');
    const original = clone(wd(s.snapshot()));
    assert.equal(original.length, 8);
    for (let i = 2; i <= 5; i++) {
      const a = original.find(x => x.evidence === `a@${i}`);
      const b = original.find(x => x.evidence === `b@${i}`);
      assert.equal(a.because, i === 2 ? 'b' : `b@${i - 1}`);
      assert.equal(b.because, `a@${i}`);
    }
    assert(s.snapshot().retired['a@2']);
    const restored = ctx.restore(s.save(), 'required-chain');
    equal(restored.snapshot(), s.snapshot(), 'required chain exact restore');
    for (const v of [s, restored]) {
      ctx.event(v, 'probe');
      const p = v.snapshot().value_grounds.reason_read;
      has(p.evidence, 'a@5', 'predicate true reason'); has(p.caveats, 'stale', 'reason qualification');
      equal(wd(v.snapshot()), original, 'predicate does not change chain');
      const archive = ctx.drain(v, 'required-chain'); if (archive) equal(archive, []);
      ctx.event(v, 'cycle'); ctx.event(v, 'probe');
      has(v.snapshot().value_grounds.reason_read.evidence, 'a@6', 'continued exact reason');
      assert.equal(wd(v.snapshot()).length, 10);
      const after = ctx.drain(v, 'continued-chain'); if (after) equal(after, []);
    }
    equal(s.snapshot(), restored.snapshot(), 'required chain continuation');
    ctx.restore(s.save(), 'continued-chain-roundtrip');
    return { observedClass: 'required-chain-preserved', projection: predicateProjection(s.snapshot()) };
  }
  throw new Error(`unimplemented probe ${test.id}`);
}


for (const item of cases) {
  test(`adversarial ${item.id}: ${item.expectation}`, async () => {
    const source = await readFile(new URL(`fixtures/adversarial/${item.fixture}`, import.meta.url), 'utf8');
    const ctx = context(source);
    try { await probe(item, ctx); }
    finally { ctx.close(); }
  });
}
