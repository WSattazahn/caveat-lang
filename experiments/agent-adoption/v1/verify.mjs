// Private study evaluator. Never ship this file in a trial packet.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

export const STUDY_SCHEMA = 'caveat-agent-adoption-study/1';
const sha256 = value => createHash('sha256').update(value).digest('hex');
const errorText = error => error instanceof Error ? error.message : String(error);
const summary = cases => ({ pass: cases.length > 0 && cases.every(item => item.pass), cases });
const same = (actual, expected, message) => assert.deepEqual(actual, expected, message);
const accepted = (session, event, payload) => {
  const result = session.dispatch(event, payload);
  assert.equal(result.outcome, 'accepted', `${event}: ${JSON.stringify(result)}`);
  return result.snapshot;
};
const refused = (session, event, payload, policy = true) => {
  const before = session.save();
  const result = session.dispatch(event, payload);
  assert.equal(result.outcome, 'rejected', `${event} should be rejected`);
  if (policy) { assert.equal(result.origin, 'policy'); assert.equal(result.code, 'reject'); }
  assert.equal(session.save(), before, `${event} rejection must preserve the complete saved session`);
  return result;
};
const neutral = (snapshot, name) => {
  assert.ok(snapshot.observations?.includes(name), `${name} must be observed`);
  assert.equal(snapshot.relations.some(edge => edge.from === name && ['supports', 'opposes'].includes(edge.relation)), false, `${name} observation must be neutral`);
};
const grounds = (snapshot, decision, reading, value, caveats = ['uncertain']) => {
  same(snapshot.commitment_grounds[decision], { evidence: [reading], caveats }, `${decision} exact grounds`);
  const basis = snapshot.commitment_bases[decision];
  assert.equal(basis?.value, value, `${decision} basis value`);
  assert.ok(basis.provenance.evidence.includes(reading), `${decision} lineage includes its reading`);
  for (const caveat of caveats) assert.ok(basis.provenance.caveats.includes(caveat), `${decision} lineage retains ${caveat}`);
  // Control lineage may include earlier readings/correction. It is not grounds.
};
const openDecision = (snapshot, id, expected) => {
  assert.equal(snapshot.commitments.find(item => item.action === id)?.open, expected, `${id} open state`);
};
const preserved = (snapshot, before, id = 'answer@1') => {
  same(snapshot.commitment_grounds[id], before.commitment_grounds[id], 'original grounds must remain frozen');
  same(snapshot.commitment_bases[id], before.commitment_bases[id], 'original basis must remain frozen');
  same(snapshot.reading_streams.checks.occurrences[0], before.reading_streams.checks.occurrences[0], 'original reading archive must remain frozen');
};

// Each named case has a fresh session; failures do not hide unrelated checks.
export async function checkProgram(runtime, source) {
  const cases = [];
  const run = async (name, body) => {
    let session;
    try { session = runtime.open(source); await body(session); cases.push({ name, pass: true }); }
    catch (error) { cases.push({ name, pass: false, error: errorText(error) }); }
    finally { session?.close(); }
  };
  await run('declarations-and-event-contract', session => {
    const snapshot = session.snapshot();
    for (const [name, kind] of [['tool', 'evidence'], ['correction', 'evidence'], ['ready', 'claim'], ['uncertain', 'caveat'], ['stale', 'caveat']]) {
      const symbol = snapshot.symbols.find(item => item.name === name);
      assert.equal(symbol?.kind, kind);
      if (kind === 'caveat') assert.equal(symbol.consequence, 'material');
    }
    assert.equal(snapshot.reading_streams.checks.template, 'tool');
    assert.equal(snapshot.reading_streams.checks.limit, 16);
    assert.equal(snapshot.decision_series.answer.limit, 8);
    same(snapshot.events.find(item => item.name === 'observe')?.parameters, [{ name: 'value', min: 0, max: 100 }]);
    for (const name of ['assess', 'correct', 'learn_stale']) same(snapshot.events.find(item => item.name === name)?.parameters, []);
    assert.ok(snapshot.relations.some(edge => edge.from === 'uncertain' && edge.relation === 'qualifies' && edge.to === 'tool'));
    assert.equal(snapshot.relations.some(edge => edge.from === 'stale'), false, 'stale must be late knowledge');
  });
  await run('empty-policy-refusals-atomic', session => {
    for (const event of ['assess', 'correct', 'learn_stale']) refused(session, event);
  });
  await run('invalid-observations-atomic', session => {
    for (const payload of [{ value: -1 }, { value: 101 }, { value: '85' }, {}]) refused(session, 'observe', payload, false);
    accepted(session, 'observe', { value: 85 }); accepted(session, 'assess');
    refused(session, 'observe', { value: 101 }, false);
  });
  for (const value of [0, 69, 70, 100]) {
    await run(`threshold-${value}-neutral-grounds`, session => {
      const snapshot = accepted(session, 'observe', { value });
      neutral(snapshot, 'tool');
      assert.equal(snapshot.decision_series.answer.current, null, 'observe must not decide');
      same(snapshot.decision_journal, []);
      assert.equal(snapshot.reading_streams.checks.occurrences.length, 1);
      const reading = snapshot.reading_streams.checks.occurrences[0];
      assert.equal(reading.value, value); assert.equal(reading.id, 'checks@1');
      assert.equal(reading.claim, 'ready'); assert.equal(reading.relation, value >= 70 ? 'supports' : 'opposes');
      same(reading.provenance, { evidence: ['checks@1'], caveats: ['uncertain'] });
      if (value < 70) refused(session, 'assess');
      else {
        const committed = accepted(session, 'assess');
        assert.equal(committed.decision_series.answer.current, 'answer@1');
        grounds(committed, 'answer@1', 'checks@1', value);
        openDecision(committed, 'answer@1', false);
        refused(session, 'assess');
      }
    });
  }
  for (const value of [85, 92, 40]) {
    await run(`every-reading-reopens-${value}`, session => {
      accepted(session, 'observe', { value: 85 }); const before = accepted(session, 'assess');
      const observed = accepted(session, 'observe', { value });
      assert.equal(observed.decision_series.answer.current, 'answer@1');
      assert.equal(observed.decision_series.answer.revisions.length, 1, 'observe must not create a revision');
      openDecision(observed, 'answer@1', true); preserved(observed, before);
      assert.ok(observed.decision_journal.some(item => item.commitment === 'answer@1' && item.change === 'reopened' && item.event === 'observe'));
      if (value < 70) refused(session, 'assess');
      else {
        const revised = accepted(session, 'assess');
        assert.equal(revised.decision_series.answer.current, 'answer@2');
        grounds(revised, 'answer@2', 'checks@2', value); preserved(revised, before);
        openDecision(revised, 'answer@2', false);
      }
    });
  }
  await run('correction-withdrawal-revision-frozen-history', session => {
    accepted(session, 'observe', { value: 85 }); const before = accepted(session, 'assess');
    const corrected = accepted(session, 'correct'); neutral(corrected, 'correction');
    same(corrected.withdrawals.map(item => [item.evidence, item.because]), [['checks@1', 'correction']]);
    openDecision(corrected, 'answer@1', true); preserved(corrected, before);
    assert.ok(corrected.decision_journal.some(item => item.commitment === 'answer@1' && item.change === 'reopened' && item.event === 'correct' && item.sequence === corrected.sequence));
    refused(session, 'assess'); refused(session, 'correct');
    accepted(session, 'observe', { value: 92 }); const revised = accepted(session, 'assess');
    assert.equal(revised.decision_series.answer.current, 'answer@2');
    grounds(revised, 'answer@2', 'checks@2', 92); preserved(revised, before);
    same(revised.withdrawals, corrected.withdrawals);
    same(revised.decision_series.answer.revisions.map(item => item.id), ['answer@1', 'answer@2']);
    const correctedAgain = accepted(session, 'correct');
    neutral(correctedAgain, 'correction');
    openDecision(correctedAgain, 'answer@2', true);
    same(correctedAgain.commitment_grounds['answer@2'], revised.commitment_grounds['answer@2']);
    same(correctedAgain.commitment_bases['answer@2'], revised.commitment_bases['answer@2']);
    same(correctedAgain.reading_streams.checks.occurrences, revised.reading_streams.checks.occurrences);
    assert.ok(correctedAgain.withdrawals.some(item => item.evidence === 'checks@2' && item.because === 'correction'));
    refused(session, 'correct');
  });
  await run('correction-before-decision', session => {
    accepted(session, 'observe', { value: 85 }); const corrected = accepted(session, 'correct');
    assert.equal(corrected.decision_series.answer.current, null); neutral(corrected, 'correction');
    assert.ok(corrected.withdrawals.some(item => item.evidence === 'checks@1' && item.because === 'correction'));
    refused(session, 'assess'); refused(session, 'correct');
  });
  await run('late-stale-preserves-past-refuses-future', session => {
    accepted(session, 'observe', { value: 85 }); const before = accepted(session, 'assess');
    const late = accepted(session, 'learn_stale'); preserved(late, before);
    openDecision(late, 'answer@1', false);
    assert.ok(late.relations.some(edge => edge.from === 'stale' && edge.relation === 'qualifies' && edge.to === 'tool'));
    preserved(accepted(session, 'learn_stale'), before);
    const future = accepted(session, 'observe', { value: 90 });
    preserved(future, before); openDecision(future, 'answer@1', true);
    same(future.reading_streams.checks.occurrences[1].provenance, { evidence: ['checks@2'], caveats: ['stale', 'uncertain'] });
    refused(session, 'assess');
  });
  await run('old-clean-reading-remains-eligible-after-stale', session => {
    const before = accepted(session, 'observe', { value: 85 });
    const late = accepted(session, 'learn_stale');
    same(late.reading_streams.checks.occurrences, before.reading_streams.checks.occurrences);
    const committed = accepted(session, 'assess'); grounds(committed, 'answer@1', 'checks@1', 85);
  });
  await run('save-restore-continuation', session => {
    accepted(session, 'observe', { value: 85 }); accepted(session, 'assess'); accepted(session, 'correct');
    const restored = runtime.restore(source, session.save());
    try {
      same(restored.snapshot(), session.snapshot());
      for (const [event, payload] of [['observe', { value: 92 }], ['assess'], ['learn_stale'], ['observe', { value: 95 }]]) {
        same(accepted(restored, event, payload), accepted(session, event, payload));
      }
      same(refused(restored, 'assess'), refused(session, 'assess'));
      assert.equal(restored.save(), session.save());
    } finally { restored.close(); }
  });
  return summary(cases);
}

export async function checkHost(runtime, starterSource, attempt) {
  const cases = [];
  if (typeof attempt !== 'function') return summary([{ name: 'host-export', pass: false, error: 'host.mjs must export attempt(session, score)' }]);
  for (const [label, scores, expected] of [['primary', [85, 101, 92, 40, 85], [true, false, false, false, true]], ['secondary', [101, 73, 99, 30, 80], [false, true, false, false, true]]]) {
    const session = runtime.open(starterSource);
    try {
      for (const [index, score] of scores.entries()) {
        const name = `${label}-${index + 1}-score-${score}`;
        const calls = [];
        const before = session.snapshot().sequence;
        const proxy = new Proxy(session, { get(target, key) {
          if (key === 'close') return () => { throw new Error('attempt must not close the supplied session'); };
          const value = Reflect.get(target, key, target);
          if (key === 'dispatch' || key === 'dispatchView') return (event, payload) => {
            const outcome = value.call(target, event, payload);
            calls.push({ event, payload, outcome: outcome.outcome });
            return outcome;
          };
          return typeof value === 'function' ? value.bind(target) : value;
        } });
        try {
          const result = await attempt(proxy, score);
          assert.equal(typeof result?.ok, 'boolean');
          assert.equal(result.ok, expected[index], 'attempt verdict');
          assert.ok(calls.length >= 1 && calls.length <= 2, 'must perform observe and assess; fail-fast only after rejected observe');
          same(calls[0].payload, { confidence: score }, 'must supply this score to the official event');
          assert.equal(calls[0].event, 'observe');
          if (calls.length === 1) assert.equal(calls[0].outcome, 'rejected');
          else assert.equal(calls[1].event, 'assess');
          const snapshot = session.snapshot();
          const current = snapshot.decision_series.assessment.current;
          const committed = snapshot.decision_journal.find(item => item.commitment === current && item.change === 'committed');
          const reopened = snapshot.commitments.find(item => item.action === current)?.open;
          const permits = snapshot.bindings.assessment.verdict === 'approved' && reopened === false && committed?.sequence > before;
          const requiredAccepted = calls.length === 2 && calls.every(item => item.outcome === 'accepted');
          assert.equal(result.ok, Boolean(requiredAccepted && permits), 'required operations plus a fresh in-force decision');
          cases.push({ name, pass: true, score, expected: expected[index], calls });
        } catch (error) { cases.push({ name, pass: false, error: errorText(error), score, calls }); }
      }
    } finally { session.close(); }
  }
  return summary(cases);
}

export async function checkScenarios(api, runtime, doc, source, sourceName) {
  try {
    api.validateScenarioFile(doc);
    assert.equal(doc.source, sourceName, 'suite must target its preserved submission');
    for (const scenario of doc.scenarios) if (scenario.source !== undefined) assert.equal(scenario.source, sourceName);
    const steps = doc.scenarios.flatMap(item => item.steps);
    const markers = Object.fromEntries(['checkpoint', 'same_as', 'resume'].map(name => [name, steps.some(step => Object.hasOwn(step, name))]));
    const report = await api.runScenarioFile(doc, { runtime, readSource(name) {
      assert.equal(name, sourceName, 'only the submitted source can be loaded'); return source;
    }, file: sourceName.replace(/\.cav$/, '.scenarios.json') });
    return { pass: report.failed === 0, count: doc.scenarios.length, markers, meetsStructuralMinimum: doc.scenarios.length >= 6 && Object.values(markers).every(Boolean), report };
  } catch (error) { return { pass: false, error: errorText(error) }; }
}

export async function verifySubmission({ submissionDirectory, packageDirectory, runtimeDirectory }) {
  const root = path.resolve(submissionDirectory);
  const pkg = path.resolve(packageDirectory);
  const nodeApi = await import(pathToFileURL(path.join(pkg, 'lib/node.mjs')).href);
  const scenariosApi = await import(pathToFileURL(path.join(pkg, 'lib/scenarios.mjs')).href);
  const runtime = await nodeApi.loadRuntimeFromDirectory(runtimeDirectory ?? path.join(pkg, 'runtime'));
  const manifest = JSON.parse(await readFile(path.join(pkg, 'package.json'), 'utf8'));
  const artifacts = {};
  const read = async name => {
    try { const bytes = await readFile(path.join(root, name)); artifacts[name] = { present: true, bytes: bytes.length, sha256: sha256(bytes) }; return bytes.toString('utf8'); }
    catch (error) { artifacts[name] = { present: false, error: errorText(error) }; return null; }
  };
  const programs = {};
  const scenarios = {};
  for (const name of ['first', 'tracker']) {
    const source = await read(`${name}.cav`);
    programs[name] = source === null ? summary([{ name: 'source-present', pass: false }]) : await checkProgram(runtime, source);
    const suite = await read(`${name}.scenarios.json`);
    try { scenarios[name] = suite === null || source === null ? { pass: false, error: 'missing source or suite' } : await checkScenarios(scenariosApi, runtime, JSON.parse(suite), source, `${name}.cav`); }
    catch (error) { scenarios[name] = { pass: false, error: errorText(error) }; }
  }
  const starterPath = path.join(pkg, 'examples/agent-evidence/assessment.cav');
  const starter = await readFile(starterPath, 'utf8');
  let host;
  if (await read('host.mjs') === null) host = summary([{ name: 'host-present', pass: false }]);
  else {
    try { const module = await import(`${pathToFileURL(path.join(root, 'host.mjs')).href}?study=${Date.now()}`); host = await checkHost(runtime, starter, module.attempt); }
    catch (error) { host = summary([{ name: 'host-load', pass: false, error: errorText(error) }]); }
  }
  const starterUnchanged = starter === await readFile(starterPath, 'utf8');
  if (!starterUnchanged) { host.cases.push({ name: 'official-starter-unchanged', pass: false }); host.pass = false; }
  for (const name of ['receipts/commands.jsonl', 'events.jsonl', 'diagnosis.md', 'explanation.md', 'FEATURE_REQUEST.md', 'RESULTS.md']) await read(name);
  return {
    schema: STUDY_SCHEMA,
    candidate: { name: manifest.name, version: manifest.version, runtime: runtime.identity, starterSha256: sha256(starter), starterUnchanged },
    artifacts, programs, scenarios, host,
    automated: { programSemanticsScore: Number(programs.first.pass) + Number(programs.tracker.pass), finalObjectivesPass: programs.tracker.pass && scenarios.tracker.pass && scenarios.tracker.meetsStructuralMinimum === true && host.pass },
    manual: { discovery: null, meaningfulScenarios: null, diagnosis: null, exactExplanation: null, hostAttempts: null, honestMissingCapability: null, preservedFirstArtifacts: null, total: null },
    limitations: ['Manual scores require cited review; proxies do not award points.', 'First-artifact ordering requires the external CLI transcript.', 'The scorer executes submitted JavaScript in the evaluator environment; it is not a security sandbox.'],
  };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const options = {};
  for (let i = 0; i < args.length; i += 2) {
    if (!['--submission', '--package'].includes(args[i]) || !args[i + 1]) throw new Error('Usage: node verify.mjs --submission DIRECTORY --package INSTALLED_PACKAGE_DIRECTORY');
    options[args[i] === '--submission' ? 'submissionDirectory' : 'packageDirectory'] = args[i + 1];
  }
  if (!options.submissionDirectory || !options.packageDirectory) throw new Error('Both --submission and --package are required');
  const result = await verifySubmission(options);
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
  process.exitCode = result.automated.finalObjectivesPass ? 0 : 1;
}
