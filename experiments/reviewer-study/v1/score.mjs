// Scores the study once the sealed files are unsealed. Two subcommands:
//
//   node score.mjs cases EXPECTED.json ACTUAL.json
//     A program's run-cases.mjs output against the model's expected views.
//     Prints each failing file and the first line that differs.
//
//   node score.mjs answers KEYS.json ANSWERS.json [ANSWERS.json …]
//     Each reviewer's answers against the key, per run, never pooled.
//
// KEYS.json is { H1: { decisions, q2, notes }, … } as sealed at stage 1.
import { readFile } from 'node:fs/promises';

const json = async file => JSON.parse(await readFile(file, 'utf8'));
const [command, ...files] = process.argv.slice(2);

function scoreCases(expected, actual) {
  const failures = [];
  for (const [file, steps] of Object.entries(expected)) {
    const got = actual[file] ?? [];
    const bad = steps.find((step, index) => {
      const mine = got[index];
      return !mine || mine.fatal || mine.accepted !== step.accepted || JSON.stringify(mine.view) !== JSON.stringify(step.view);
    });
    if (bad) failures.push({ file, line: bad.line, expected: bad, actual: got[bad.line - 1] ?? null });
  }
  return { files: Object.keys(expected).length, passed: Object.keys(expected).length - failures.length, failures };
}

const CANNOT = 'cannot_determine';
const sameSet = (a, b) => Array.isArray(a) && Array.isArray(b) && a.length === b.length
  && [...a].map(String).sort().join('\u0000') === [...b].map(String).sort().join('\u0000');
// One answer: 'correct', 'wrong' or 'unanswered'.
const mark = (item, check) => {
  if (!item || typeof item !== 'object' || !('answer' in item)) return 'unanswered';
  if (item.answer === CANNOT) return 'unanswered';
  return check(item.answer) ? 'correct' : 'wrong';
};
const receiptKind = item => (item && item.receipt && ['record', 'code', 'run', 'none'].includes(item.receipt.kind) ? item.receipt.kind : 'missing');

function scoreAnswers(keys, answers) {
  const out = { run: answers?.run ?? null, q1: [], q2: [], q3: [], receipts: { record: 0, code: 0, run: 0, none: 0, missing: 0 } };
  const count = item => { out.receipts[receiptKind(item)] += 1; };
  for (const history of ['H1', 'H2', 'H3']) {
    const key = keys[history];
    const mine = answers?.histories?.[history] ?? {};
    for (const decision of key.decisions) {
      const given = (mine.q1 ?? []).find(item => item?.decision === decision.decision) ?? {};
      const parts = {
        value: mark(given.value, a => a === decision.value),
        rests_on: mark(given.rests_on, a => sameSet(a, decision.rests_on)),
        caveats: mark(given.caveats, a => sameSet(a, decision.caveats)),
        retains: mark(given.retains, a => sameSet(a, decision.retains)),
        grant: mark(given.grant, a => a === decision.grant),
        state: mark(given.state, a => a && a.state === decision.state
          && (a.reopened_by ?? null) === decision.reopened_by && (a.reason ?? null) === decision.reopen_reason),
      };
      for (const name of Object.keys(parts)) count(given[name]);
      const all = names => names.every(name => parts[name] === 'correct');
      out.q1.push({ history, decision: decision.decision, parts,
        core: all(['value', 'rests_on', 'grant', 'state']), caveats: all(['caveats', 'retains']), strict: all(Object.keys(parts)) });
    }
    key.q2.forEach((expected, index) => {
      const probe = keys[history].probes[index];
      const given = (mine.q2 ?? []).find(item => item?.probe === probe);
      count(given);
      out.q2.push({ history, probe, kind: ['rests_on', 'later', 'grant', 'fourth'][index], expected,
        result: mark(given, a => a === expected) });
    });
    key.notes.forEach(statement => {
      const given = (mine.q3 ?? []).find(item => item?.statement === statement.statement);
      count(given);
      out.q3.push({ history, statement: statement.statement, verdict: statement.verdict, kind: statement.kind ?? null,
        given: given && typeof given.answer === 'string' ? given.answer : null });
    });
  }
  const q1 = out.q1;
  const q3false = out.q3.filter(item => item.verdict === 'false'), q3true = out.q3.filter(item => item.verdict === 'true');
  out.summary = {
    q1_core: `${q1.filter(item => item.core).length} of ${q1.length}`,
    q1_caveats: `${q1.filter(item => item.caveats).length} of ${q1.length}`,
    q1_strict: `${q1.filter(item => item.strict).length} of ${q1.length}`,
    q2: `${out.q2.filter(item => item.result === 'correct').length} of ${out.q2.length}`,
    q2_grant: `${out.q2.filter(item => item.kind === 'grant' && item.result === 'correct').length} of 3`,
    q2_fourth: `${out.q2.filter(item => item.kind === 'fourth' && item.result === 'correct').length} of 3`,
    q3_hits: `${q3false.filter(item => item.given === 'false').length} of ${q3false.length}`,
    q3_false_alarms: `${q3true.filter(item => item.given === 'false').length} of ${q3true.length}`,
    receipts: out.receipts,
  };
  return out;
}

if (command === 'cases') {
  const [expected, actual] = await Promise.all(files.slice(0, 2).map(json));
  const result = scoreCases(expected, actual);
  console.log(JSON.stringify(result, null, 2));
  process.exitCode = result.failures.length ? 1 : 0;
} else if (command === 'answers') {
  const keys = await json(files[0]);
  const results = [];
  for (const file of files.slice(1)) {
    let answers = null;
    try { answers = await json(file); } catch { answers = null; }
    results.push({ file, ...scoreAnswers(keys, answers) });
  }
  console.log(JSON.stringify(results, null, 2));
} else {
  console.error('usage: node score.mjs cases EXPECTED ACTUAL | answers KEYS ANSWERS…');
  process.exitCode = 2;
}
