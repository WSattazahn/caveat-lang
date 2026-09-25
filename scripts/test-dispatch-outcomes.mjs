// Run after npm run build: exercise the actual full and reactive-only WASM wires.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';

const root = new URL('../', import.meta.url);
const read = name => readFile(new URL(name, root));
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const schema = 'caveat-dispatch/0.1';
const sources = new Map();
const fixture = (name, source) => { sources.set(name, source); return source; };
const parse = text => JSON.parse(text);
const checkpoint = session => ({
  snapshot: parse(session.snapshot()),
  view: parse(session.view()),
  save: parse(session.save()),
});

// A thrown exception is always a failing smoke check here. Only an explicitly
// returned outcome can enter the rejected path.
function dispatch(session, event, payload = '{}') {
  const wire = session.dispatch_outcome(event, payload);
  assert.equal(typeof wire, 'string');
  const result = parse(wire);
  assert.equal(result.schema, schema);
  assert.equal(Object.hasOwn(result, 'stage'), false);
  assert.equal(Object.hasOwn(result, 'host'), false);
  if (result.outcome === 'accepted') {
    assert.deepEqual(Object.keys(result).sort(), ['outcome', 'schema', 'snapshot']);
    assert.equal(result.snapshot.schema, 'caveat-reactive/0.1');
  } else {
    assert.equal(result.outcome, 'rejected');
    assert(['policy', 'input', 'evaluation', 'limit'].includes(result.origin));
    assert.deepEqual(Object.keys(result).sort(), ['code', 'message', 'origin', 'outcome', 'schema']);
    assert.equal(typeof result.message, 'string');
  }
  return result;
}

function rejected(session, event, payload, origin, code) {
  const before = checkpoint(session);
  const result = dispatch(session, event, payload);
  assert.equal(result.outcome, 'rejected');
  assert.equal(result.origin, origin);
  assert.equal(result.code, code);
  assert.deepEqual(checkpoint(session), before, `${event} ${payload} changes nothing`);
  return result;
}

const acceptedSource = fixture('accepted', `
claim safe;
evidence chart from "the chart";
caveat age consequence low;
chart supports safe;
age qualifies chart;
place garden kind garden;
state output = 0;
event read value min 0 max 1;
event idle;
cue note toast "Read" 1;
on read set output = qualified(value, chart);
on read commit route because enough using output;
on read emit note;
bind hud.value = output;
bind hud.text = "Read" when output > 0 because output;
`);

const message = 'rejected: Keep the route; 条件 {pending}.';
const timed = `
claim safe;
evidence reading from "sensor";
caveat stale consequence low;
decisions plan limit 1;
state output = 0;
event read;
event advance dt min 0 max 2;
clock advance every 1;
cue note toast "Read" 1;
on read reveal reading supports safe;
on read qualify reading with stale after 2;
on read set output = qualified(1, reading);
on read emit note;
on advance set output = output + dt;
on advance when carries(reading, stale) and not committed(plan)
    commit plan because enough using qualified(elapsed(), reading);
on advance when dt == 2 emit note;
bind hud.elapsed = elapsed();
bind hud.stale = carries(reading, stale);
bind hud.value = output;
bind hud.text = "Read" when output > 0 because output;
`;
const policySources = [
  ['direct', fixture('policy-direct', `${timed}\non advance when dt == 2 reject "${message}";`)],
  ['nested', fixture('policy-nested', `${timed}
    proc inner() { reject "${message}"; };
    proc outer() { call inner(); };
    on advance when dt == 2 call outer();
  `)],
];

const inputSource = fixture('input', `
place garden kind garden;
entity cave kind mushroom at garden;
entity pool kind mushroom at garden;
state output = 0 min 0 max 1;
event set_value value min 0 max 2;
event choose target kind mushroom, sort in glowcap duskcap;
on set_value set output = value;
on choose set output = 0.5;
bind hud.value = output;
`);

const clockSource = fixture('clock-only', `
event advance dt min 0 max 2;
clock advance every 1;
bind hud.elapsed = elapsed();
`);

const buildInfoBytes = await read('dist/build-info.json');
const result = {
  passed: false,
  buildInfo: parse(buildInfoBytes.toString('utf8')),
  buildInfoSha256: sha256(buildInfoBytes),
  scriptSha256: sha256(await readFile(new URL(import.meta.url))),
  runtimes: {},
  fixtureSha256: {},
  checks: [],
};

try {
  for (const variant of ['pkg', 'pkg-reactive']) {
    const modulePath = `dist/${variant}/caveat_runtime.js`;
    const wasmPath = `dist/${variant}/caveat_runtime_bg.wasm`;
    const [glue, wasm] = await Promise.all([read(modulePath), read(wasmPath)]);
    result.runtimes[variant] = {
      wasmPath, runtimeSha256: sha256(wasm),
      modulePath, moduleSha256: sha256(glue),
    };
    const { default: init, WebReactiveSession } = await import(new URL(modulePath, root));
    await init({ module_or_path: wasm });
    assert.equal(typeof WebReactiveSession.prototype.dispatch_outcome, 'function', 'Rebuild the WASM bridge');
    const check = (name, run) => {
      run();
      result.checks.push(`${variant}: ${name}`);
    };
    const withSessions = (source, count, run) => {
      const sessions = [];
      try {
        for (let index = 0; index < count; index += 1) sessions.push(new WebReactiveSession(source));
        run(...sessions);
      } finally {
        for (const session of sessions) session.free();
      }
    };

    check('accepted full snapshot and legacy view equivalence', () => {
      withSessions(acceptedSource, 3, (outcome, legacy, viewed) => {
        for (const [event, payload] of [['read', '{"value":0.5}'], ['idle', '{}']]) {
          const accepted = dispatch(outcome, event, payload);
          assert.deepEqual(accepted, {
            schema, outcome: 'accepted', snapshot: parse(legacy.dispatch(event, payload)),
          });
          assert.deepEqual(checkpoint(outcome), checkpoint(legacy));
          assert.deepEqual(parse(viewed.dispatch_view(event, payload)), parse(outcome.view()));
          for (const field of ['world', 'events', 'qualified_values', 'commitment_bases']) {
            assert(Object.hasOwn(accepted.snapshot, field), `full snapshot contains ${field}`);
          }
        }
      });
    });

    check('release clock-only bindings follow the fractional timetable', () => {
      withSessions(clockSource, 1, session => {
        let elapsed = 0;
        for (const dt of [0.1, 0.2, 0.125, 0, 1]) {
          elapsed += dt;
          const accepted = dispatch(session, 'advance', JSON.stringify({ dt }));
          assert.equal(accepted.outcome, 'accepted');
          assert.equal(accepted.snapshot.elapsed, elapsed);
          assert.equal(accepted.snapshot.bindings.hud.elapsed, elapsed);
          assert.equal(parse(session.view()).bindings.hud.elapsed, elapsed);
        }
      });
    });

    for (const [kind, source] of policySources) {
      check(`${kind} policy literal, atomic timer rollback and release clock timetable`, () => {
        withSessions(source, 2, (session, control) => {
          let elapsed = 0;
          const advance = dt => {
            elapsed += dt;
            const payload = JSON.stringify({ dt });
            const accepted = dispatch(session, 'advance', payload);
            assert.equal(accepted.outcome, 'accepted');
            assert.equal(accepted.snapshot.bindings.hud.elapsed, elapsed, 'release clock binding invalidates');
            assert.deepEqual(accepted.snapshot, parse(control.dispatch('advance', payload)));
            return accepted.snapshot;
          };
          assert.equal(parse(session.snapshot()).bindings.hud.elapsed, 0);
          for (const dt of [0.1, 0.2]) advance(dt);
          dispatch(session, 'read');
          control.dispatch('read', '{}');
          advance(1);
          const refusal = rejected(session, 'advance', '{"dt":2}', 'policy', 'reject');
          assert.equal(refusal.message, message, 'policy message is exactly the authored literal');
          assert.deepEqual(checkpoint(session), checkpoint(control));
          const boundary = advance(1);
          assert.equal(boundary.bindings.hud.elapsed, 2.3);
          assert.equal(boundary.bindings.hud.stale, false, 'binary64 subtraction is still below the timer boundary');
          assert.equal(boundary.scheduled_qualifications.length, 1);
          const expired = advance(0.1);
          assert.equal(expired.bindings.hud.stale, true);
          assert.equal(expired.scheduled_qualifications.length, 0);
          assert.equal(expired.decision_journal.length, 1);
          assert.equal(expired.decision_journal[0].elapsed, elapsed);
          assert.equal(expired.decision_journal[0].value, elapsed);
          assert.equal(expired.decision_series.plan.current, 'plan@1');
          assert.deepEqual(expired.commitment_grounds['plan@1'], { evidence: ['reading'], caveats: ['stale'] });
          const restored = WebReactiveSession.restore(source, session.save());
          try { assert.deepEqual(checkpoint(restored), checkpoint(session)); }
          finally { restored.free(); }
        });
      });
    }

    check('malformed duplicate null and typed payloads remain input rejections', () => {
      withSessions(inputSource, 1, session => {
        dispatch(session, 'set_value', '{"value":0.5}');
        for (const payload of [
          '{', 'null', '[]', '{}', '{"value":0.1,"extra":1}',
          '{"value":0.1,"value":0.2}', '{"value":null}', '{"value":true}',
          '{"value":"0.5"}', '{"value":1e999}',
        ]) rejected(session, 'set_value', payload, 'input', 'payload_invalid');
        for (const payload of [
          '{"target":"missing","sort":"glowcap"}',
          '{"target":"pool","sort":"missing"}',
          '{"target":null,"sort":"glowcap"}',
          '{"target":"pool","target":"cave","sort":"glowcap"}',
        ]) rejected(session, 'choose', payload, 'input', 'payload_invalid');
        rejected(session, 'missing', '{}', 'input', 'unknown_event');
        assert.equal(dispatch(session, 'choose', '{"target":"pool","sort":"glowcap"}').outcome, 'accepted');
      });
    });

    check('input bounds and evaluation bounds have distinct origins', () => {
      withSessions(inputSource, 1, session => {
        dispatch(session, 'set_value', '{"value":0.5}');
        rejected(session, 'set_value', '{"value":3}', 'input', 'bound_exceeded');
        rejected(session, 'choose', '{"target":3,"sort":1}', 'input', 'bound_exceeded');
        rejected(session, 'set_value', '{"value":2}', 'evaluation', 'bound_exceeded');
      });
    });

    // Typed parameters 0.1, Changes: a fraction inside the range names no
    // member. An isolated value outside the range keeps bound_exceeded.
    check('a fraction for a typed parameter names no member', () => {
      withSessions(inputSource, 1, session => {
        dispatch(session, 'set_value', '{"value":0.5}');
        for (const payload of ['{"target":1.5,"sort":1}', '{"target":1,"sort":1.5}', '{"target":2,"sort":1.0000001}']) {
          rejected(session, 'choose', payload, 'input', 'payload_invalid');
        }
        for (const payload of ['{"target":0.5,"sort":1}', '{"target":1,"sort":2.5}']) {
          rejected(session, 'choose', payload, 'input', 'bound_exceeded');
        }
        assert.equal(dispatch(session, 'choose', '{"target":2.0,"sort":1e0}').outcome, 'accepted');
        assert.equal(dispatch(session, 'set_value', '{"value":0.25}').outcome, 'accepted');
      });
    });

    check('live and skipped work exhaustion are atomic limit rejections', () => {
      const body = 'set output = output + 1;'.repeat(2048);
      for (const guard of ['true', 'false']) {
        const source = fixture(`work-${guard}`, `state output = 0; event run;
          proc many() { ${body} };
          on run when ${guard} call many(); on run when ${guard} call many();`);
        withSessions(source, 1, session => rejected(session, 'run', '{}', 'limit', 'work_limit'));
      }
    });

    check('full reading and decision histories are atomic limit rejections', () => {
      const source = fixture('history-limit', `claim safe; evidence gauge from "a gauge";
        readings depth from gauge limit 1; decisions route limit 1;
        event read value min 0 max 9; event decide; event doubt;
        on read sample depth = value supports safe;
        on decide commit route because enough using latest(depth);
        on doubt when committed(route) and not reopened(route) reopen route because latest(depth);`);
      withSessions(source, 1, session => {
        dispatch(session, 'read', '{"value":1}');
        rejected(session, 'read', '{"value":2}', 'limit', 'history_limit');
        dispatch(session, 'decide', '{}');
        dispatch(session, 'doubt', '{}');
        rejected(session, 'decide', '{}', 'limit', 'history_limit');
      });
    });

    check('identifiers go in as text, are handles inside, and a refused event adds none', () => {
      const sha = '602bdbec0a047a5319f53e83f336b9f7aec0e5ed';
      const source = fixture('identifiers', `identifiers limit 2; state head = 0;
        event pushed commit id; event pair first id, second id;
        on pushed set head = commit; on pair reject "Not now.";
        bind pr.head = id_text(head);`);
      withSessions(source, 1, session => {
        const accepted = dispatch(session, 'pushed', JSON.stringify({ commit: sha }));
        assert.deepEqual(accepted.snapshot.identifiers, [sha]);
        assert.equal(accepted.snapshot.values.head, 1);
        assert.equal(accepted.snapshot.bindings.pr.head, sha);
        rejected(session, 'pushed', '{"commit":1}', 'input', 'payload_invalid');
        rejected(session, 'pushed', '{"commit":""}', 'input', 'payload_invalid');
        rejected(session, 'pair', '{"first":"a","second":"b"}', 'limit', 'identifier_limit');
        rejected(session, 'pair', JSON.stringify({ first: 'a', second: sha }), 'policy', 'reject');
        const restored = WebReactiveSession.restore(source, session.save());
        try {
          assert.deepEqual(checkpoint(restored), checkpoint(session));
          assert.equal(dispatch(restored, 'pushed', '{"commit":"b"}').snapshot.values.head, 2);
        } finally { restored.free(); }
      });
    });

    check('a withdrawal is recorded, keeps what decisions rest on, and a refused event withdraws nothing', () => {
      const source = fixture('withdrawal', `claim safe; claim misreading;
        evidence ci from "checks"; evidence recheck from "a re-read";
        readings checks from ci limit 4; decisions merge limit 2; state estimate = 0;
        event check; event decide; event misread; event refused_misread;
        on check sample checks = 1 supports safe;
        on decide commit merge because enough using latest(checks);
        on misread reveal recheck supports misreading;
        on misread withdraw latest(checks) because recheck;
        on misread set estimate = latest(checks);
        on refused_misread reveal recheck supports misreading;
        on refused_misread withdraw latest(checks) because recheck;
        on refused_misread reject "Not now.";`);
      withSessions(source, 1, session => {
        dispatch(session, 'check');
        const decided = dispatch(session, 'decide').snapshot;
        const after = dispatch(session, 'misread').snapshot;
        assert.deepEqual(after.withdrawals, [{ evidence: 'checks@1', because: 'recheck', sequence: 3, event: 'misread' }]);
        assert.deepEqual(after.commitment_grounds, decided.commitment_grounds);
        assert(after.qualified_values.estimate.provenance.caveats.includes('withdrawn'));
        const restored = WebReactiveSession.restore(source, session.save());
        try { assert.deepEqual(checkpoint(restored), checkpoint(session)); } finally { restored.free(); }
      });
      withSessions(source, 1, session => {
        dispatch(session, 'check');
        rejected(session, 'refused_misread', '{}', 'policy', 'reject');
        assert.equal(parse(session.snapshot()).withdrawals, undefined);
      });
    });

    check('a missing or mismatched permission is refused as not_permitted, and a granted one is recorded', () => {
      const source = fixture('permission', `identifiers limit 8; claim ready; claim may_merge;
        evidence ci from "checks"; evidence go from "a go-ahead";
        readings checks from ci limit 4; readings approvals from go limit 4; decisions merge limit 2;
        state head = 0;
        event pushed commit id; event check; event approved commit id; event merge;
        on pushed set head = commit;
        on check sample checks = 1 supports ready;
        on approved sample approvals = commit supports may_merge;
        on merge commit merge because enough using latest(checks) permitted by latest(approvals) for head;`);
      withSessions(source, 1, session => {
        dispatch(session, 'pushed', '{"commit":"a"}');
        dispatch(session, 'check');
        rejected(session, 'merge', '{}', 'policy', 'not_permitted');
        dispatch(session, 'approved', '{"commit":"b"}');
        rejected(session, 'merge', '{}', 'policy', 'not_permitted');
        dispatch(session, 'approved', '{"commit":"a"}');
        const merged = dispatch(session, 'merge').snapshot;
        assert.deepEqual(merged.commitment_permissions['merge@1'], { grant: 'approvals@2', scope: { granted: 1, required: 1 } });
        assert.deepEqual(merged.commitment_grounds['merge@1'].evidence, ['checks@1']);
        assert.equal(merged.decision_journal[0].permitted_by, 'approvals@2');
        const restored = WebReactiveSession.restore(source, session.save());
        try { assert.deepEqual(checkpoint(restored), checkpoint(session)); } finally { restored.free(); }
      });
    });

    check('a declared reopening trigger reopens as the authored rule would, and restores', () => {
      const common = `claim changed; evidence push from "a new commit";
        readings pushes from push limit 8; event decide; event pushed;
        on decide commit merge because enough;
        on pushed sample pushes = 1 supports changed;`;
      const declared = fixture('trigger-declared', `decisions merge limit 4 reopened by pushes; ${common}`);
      const written = fixture('trigger-written', `decisions merge limit 4; ${common}
        on pushed when committed(merge) and not reopened(merge) reopen merge because latest(pushes);`);
      withSessions(declared, 1, one => withSessions(written, 1, two => {
        for (const event of ['decide', 'pushed', 'pushed', 'decide', 'pushed']) {
          const [a, b] = [dispatch(one, event).snapshot, dispatch(two, event).snapshot];
          delete a.source_id; delete b.source_id;
          assert.deepEqual(a, b, `after ${event}`);
        }
        assert.deepEqual(parse(one.snapshot()).decision_journal.map(entry => entry.change),
          ['committed', 'reopened', 'committed', 'reopened']);
        const restored = WebReactiveSession.restore(declared, one.save());
        try { assert.deepEqual(checkpoint(restored), checkpoint(one)); } finally { restored.free(); }
      }));
    });

    for (const [name, expression] of [['require', 'require(false, 1)'], ['division', '1 / 0']]) {
      check(`${name} failure throws fatal JSON and is never a returned rejection`, () => {
        const source = fixture(`fatal-${name}`, `state output = 0; event run;
          on run set output = 1; on run set output = ${expression};`);
        withSessions(source, 1, session => {
          assert.throws(() => session.dispatch_outcome('run', '{}'), thrown => {
            assert.equal(typeof thrown, 'string', 'WASM Result::Err throws its fatal JSON string');
            const fatal = parse(thrown);
            assert.deepEqual(Object.keys(fatal).sort(), ['code', 'message', 'outcome', 'schema']);
            assert.equal(fatal.schema, schema);
            assert.equal(fatal.outcome, 'fatal');
            assert.equal(fatal.code, 'unclassified');
            assert.equal(typeof fatal.message, 'string');
            assert(fatal.message.length > 0);
            return true;
          });
          // Discard the session after a fatal. No read or retry assumes that a
          // thrown WASM exception left it usable.
        });
      });
    }
  }
  result.passed = true;
} finally {
  result.fixtureSha256 = Object.fromEntries([...sources].map(([name, source]) => [name, sha256(source)]));
  await mkdir(new URL('test-results/', root), { recursive: true });
  await writeFile(new URL('test-results/dispatch-outcomes-wasm.json', root), `${JSON.stringify(result, null, 2)}\n`);
}
console.log(`Dispatch outcomes WASM: all ${result.checks.length} checks passed.`);
