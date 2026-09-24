// Runs one program through registered cases with the kit's session library and
// compares every step with the task model. Checks follow v2 (verify.mjs):
// admission, the full projection, refusals that change nothing, grounds within
// lineage, the session sequence, exact save/restore, and a restored session
// that keeps agreeing with an uninterrupted one. A fatal error is never a
// refusal: it fails the case.
import assert from 'node:assert/strict';

const clone = value => JSON.parse(JSON.stringify(value));

function assertGrounds(snapshot) {
  const within = (grounds, lineage, label) => {
    for (const key of ['evidence', 'caveats']) {
      for (const name of grounds[key]) assert(lineage[key].includes(name), `${label}: ${key} ${name} is outside lineage`);
    }
  };
  for (const [name, grounds] of Object.entries(snapshot.value_grounds ?? {})) {
    within(grounds, snapshot.qualified_values[name].provenance, `state ${name}`);
  }
  for (const [name, grounds] of Object.entries(snapshot.commitment_grounds ?? {})) {
    within(grounds, snapshot.commitment_bases[name].provenance, `decision ${name}`);
  }
}

function oracleStep(task, state, event) {
  const before = structuredClone(state);
  const result = task.step(state, event);
  assert.deepEqual(state, before, 'model mutated its input state');
  if (!result.accepted) assert.deepEqual(result.state, before, 'model refusal changed its state');
  return result;
}

export function runCase(runtime, source, task, test) {
  let session;
  let shadow;
  let state = task.initial();
  let step = -1;
  const counts = { events: 0, accepted: 0, rejected: 0, restores: 0 };
  const inspect = () => {
    const snapshot = session.snapshot();
    assert.deepEqual(task.projectActual(snapshot), task.project(state), 'the program shows something the task does not');
    assertGrounds(snapshot);
    assert.equal(snapshot.sequence, state.sequence, 'session sequence differs');
    assert.equal(snapshot.elapsed, 0, 'the session clock moved');
  };
  const restore = () => {
    // Round-trip the save through JSON, as a host storing it would.
    const restored = runtime.restore(source, JSON.stringify(clone(JSON.parse(session.save()))));
    try {
      assert.deepEqual(restored.view(), session.view(), 'restore changed the view');
      assert.deepEqual(restored.snapshot(), session.snapshot(), 'restore changed the snapshot');
    } catch (error) { restored.close(); throw error; }
    if (!shadow) shadow = session;
    else session.close();
    session = restored;
    counts.restores++;
  };
  const dispatch = (target, event) => {
    const before = target.save();
    const view = target.view();
    const outcome = target.dispatch(event.event, event.payload);
    const accepted = outcome.outcome === 'accepted';
    if (!accepted) {
      assert.deepEqual(JSON.parse(target.save()), JSON.parse(before), 'a refused event changed the saved state');
      assert.deepEqual(target.view(), view, 'a refused event changed the view');
    }
    return accepted;
  };
  try {
    session = runtime.open(source);
    inspect();
    for (const [index, event] of test.events.entries()) {
      step = index;
      if (event.resume === true) restore();
      else {
        const expected = oracleStep(task, state, event);
        const accepted = dispatch(session, event);
        counts.events++;
        counts[accepted ? 'accepted' : 'rejected']++;
        assert.equal(accepted, expected.accepted, `the event was ${accepted ? 'accepted' : 'refused'}; the task ${expected.accepted ? 'accepts' : 'refuses'} it`);
        state = expected.state;
        if (shadow) {
          assert.equal(dispatch(shadow, event), accepted, 'the restored session and the original disagree on admission');
          assert.deepEqual(session.view(), shadow.view(), 'the restored session diverged from the original');
        }
      }
      inspect();
    }
    restore();
    inspect();
    return { id: test.id, kind: test.kind, pass: true, ...counts };
  } catch (error) {
    return { id: test.id, kind: test.kind, pass: false, step: step + 1, event: test.events[step] ?? null,
      error: String(error?.message ?? error).split('\n').slice(0, 12).join('\n'), ...counts };
  } finally {
    try { session?.close(); } catch { /* a trapped or fatal session may refuse */ }
    try { shadow?.close(); } catch { /* likewise */ }
  }
}

// Every case for one program; a program that does not load fails them all.
export function scoreProgram(runtime, source, task, cases) {
  let loadError = null;
  try { runtime.open(source).close(); } catch (error) { loadError = String(error?.message ?? error); }
  const results = loadError ? [] : cases.map(test => runCase(runtime, source, task, test));
  const passed = results.filter(result => result.pass).length;
  return { loadError, passed, total: cases.length, pass: loadError === null && passed === cases.length, cases: results };
}
