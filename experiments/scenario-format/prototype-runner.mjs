// Throwaway prototype of the Scenarios 0.1 runner, used only to execute the
// spec's examples. Not the kit implementation.
//   node run-scenarios.mjs <runtime-dir> <file.scenarios.json> [--json]
import { readFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import path from 'node:path';

const [runtimeDir, file] = process.argv.slice(2);
const asJson = process.argv.includes('--json');
const { default: init, WebReactiveSession } = await import(pathToFileURL(path.join(runtimeDir, 'caveat_runtime.js')).href);
await init({ module_or_path: await readFile(path.join(runtimeDir, 'caveat_runtime_bg.wasm')) });

const ORIGINS = new Set(['policy', 'input', 'evaluation', 'limit']);
const STEP_KINDS = ['send', 'expect', 'same_as', 'checkpoint', 'resume', 'size'];
class Invalid extends Error {}
class Failure extends Error { constructor(detail) { super(detail.message ?? 'failed'); this.detail = detail; } }

// ---- file validation (fail closed) ----
function only(object, allowed, where) {
  if (!object || typeof object !== 'object' || Array.isArray(object)) throw new Invalid(`${where}: expected an object`);
  for (const key of Object.keys(object)) if (!allowed.includes(key)) throw new Invalid(`${where}: unknown field ${key}`);
}
function pointer(p, where) {
  if (typeof p !== 'string' || (p !== '' && !p.startsWith('/'))) throw new Invalid(`${where}: ${JSON.stringify(p)} is not a JSON Pointer`);
  if (/~[^01]|~$/.test(p)) throw new Invalid(`${where}: invalid escape in ${p}`);
}
function validate(doc) {
  only(doc, ['schema', 'source', 'scenarios', 'note'], 'file');
  if (doc.schema !== 'caveat-scenarios/0.1') throw new Invalid(`file: unsupported schema ${JSON.stringify(doc.schema)}`);
  if (typeof doc.source !== 'string') throw new Invalid('file: source must be a path');
  if (!Array.isArray(doc.scenarios) || !doc.scenarios.length) throw new Invalid('file: scenarios must be a non-empty array');
  const ids = new Set();
  for (const [i, scenario] of doc.scenarios.entries()) {
    const where = `scenario ${i + 1}`;
    only(scenario, ['id', 'title', 'source', 'steps', 'note'], where);
    if (typeof scenario.id !== 'string' || !scenario.id || ids.has(scenario.id)) throw new Invalid(`${where}: id must be a unique non-empty string`);
    ids.add(scenario.id);
    if (typeof scenario.title !== 'string') throw new Invalid(`${scenario.id}: title is required`);
    if (!Array.isArray(scenario.steps) || !scenario.steps.length) throw new Invalid(`${scenario.id}: steps must be a non-empty array`);
    const names = new Set(['initial', 'before']);
    for (const [j, step] of scenario.steps.entries()) {
      const at = `${scenario.id} step ${j + 1}`;
      const kinds = STEP_KINDS.filter(k => k in (step ?? {}));
      if (kinds.length !== 1) throw new Invalid(`${at}: a step needs exactly one of ${STEP_KINDS.join(', ')}`);
      const kind = kinds[0];
      if (kind === 'send') {
        only(step, ['send', 'payload', 'rejected', 'repeat', 'note'], at);
        if (typeof step.send !== 'string' || !step.send) throw new Invalid(`${at}: send names an event`);
        if ('repeat' in step && !(Number.isInteger(step.repeat) && step.repeat >= 1 && step.repeat <= 10000)) throw new Invalid(`${at}: repeat must be an integer 1..10000`);
        if ('rejected' in step) {
          const r = step.rejected;
          if (r === false) throw new Invalid(`${at}: omit rejected to expect acceptance`);
          if (r !== true && typeof r !== 'string') {
            only(r, ['origin', 'code', 'message'], `${at} rejected`);
            if (!ORIGINS.has(r.origin)) throw new Invalid(`${at}: origin must be one of ${[...ORIGINS].join(', ')}`);
            if ('code' in r && typeof r.code !== 'string') throw new Invalid(`${at}: code must be a string`);
            if ('message' in r && (r.origin !== 'policy' || typeof r.message !== 'string')) throw new Invalid(`${at}: only a policy rejection has a stable message`);
          }
        }
      } else if (kind === 'expect') {
        only(step, ['expect', 'note'], at);
        only(step.expect, Object.keys(step.expect ?? {}), at);
        if (!Object.keys(step.expect).length) throw new Invalid(`${at}: expect needs at least one path`);
        for (const [p, value] of Object.entries(step.expect)) { pointer(p, at); matchers(value, `${at} ${p}`, true); }
      } else if (kind === 'same_as') {
        only(step, ['same_as', 'paths', 'note'], at);
        if (!names.has(step.same_as)) throw new Invalid(`${at}: unknown state ${JSON.stringify(step.same_as)}`);
        if (!Array.isArray(step.paths) || !step.paths.length) throw new Invalid(`${at}: paths must be a non-empty array`);
        for (const p of step.paths) pointer(p, at);
      } else if (kind === 'checkpoint') {
        only(step, ['checkpoint', 'note'], at);
        if (typeof step.checkpoint !== 'string' || !step.checkpoint || names.has(step.checkpoint)) throw new Invalid(`${at}: checkpoint names must be new and not initial/before`);
        names.add(step.checkpoint);
      } else if (kind === 'resume') {
        only(step, ['resume', 'note'], at);
        if (step.resume !== true) throw new Invalid(`${at}: resume must be true`);
      } else if (kind === 'size') {
        only(step, ['size', 'note'], at);
        only(step.size, ['save', 'snapshot'], at);
        for (const [doc, bound] of Object.entries(step.size)) {
          only(bound, ['max', 'max_growth', 'since'], `${at} ${doc}`);
          if (('max' in bound) === ('max_growth' in bound)) throw new Invalid(`${at}: give max or max_growth`);
          if ('max_growth' in bound && (doc !== 'snapshot' || !names.has(bound.since))) throw new Invalid(`${at}: max_growth applies to the snapshot and needs a known since`);
        }
      }
    }
  }
}

const MATCHERS = new Set(['$exact', '$set', '$includes', '$absent']);
function matchers(value, where, member) {
  if (Array.isArray(value)) { value.forEach(v => matchers(v, where, false)); return; }
  if (!value || typeof value !== 'object') return;
  const keys = Object.keys(value);
  const dollar = keys.filter(k => k.startsWith('$'));
  if (!dollar.length) { for (const k of keys) matchers(value[k], where, true); return; }
  if (keys.length !== 1) throw new Invalid(`${where}: a matcher must be its object's only key; write a literal object with $exact`);
  const [op] = dollar;
  if (!MATCHERS.has(op)) throw new Invalid(`${where}: unknown matcher ${op}`);
  if (op === '$absent' && (value.$absent !== true || !member)) throw new Invalid(`${where}: $absent is true and names a path or object member`);
  if ((op === '$set' || op === '$includes') && !Array.isArray(value[op])) throw new Invalid(`${where}: ${op} takes an array`);
  if (op !== '$exact' && op !== '$absent') matchers(value[op], where, false);
}

// ---- matching ----
function get(doc, p) {
  if (p === '') return { found: true, value: doc };
  let value = doc;
  for (const raw of p.split('/').slice(1)) {
    const key = raw.replace(/~1/g, '/').replace(/~0/g, '~');
    if (value === null || typeof value !== 'object' || !Object.hasOwn(value, key)) return { found: false };
    value = value[key];
  }
  return { found: true, value };
}
const isMatcher = v => v && typeof v === 'object' && !Array.isArray(v) && Object.keys(v).length === 1 && Object.keys(v)[0].startsWith('$');
const canon = v => JSON.stringify(v, (k, x) => (x && typeof x === 'object' && !Array.isArray(x) ? Object.fromEntries(Object.keys(x).sort().map(key => [key, x[key]])) : x));
const exact = (a, b) => canon(a) === canon(b);
function match(actual, expected, at) {
  if (isMatcher(expected)) {
    const [[op, arg]] = Object.entries(expected);
    if (op === '$exact') return exact(actual, arg) ? null : { path: at, expected: arg, actual };
    if (op === '$set') {
      if (!Array.isArray(actual)) return { path: at, expected, actual };
      const a = actual.map(canon).sort(); const b = arg.map(canon).sort();
      return a.length === b.length && a.every((x, i) => x === b[i]) ? null : { path: at, expected, actual };
    }
    if (op === '$includes') return Array.isArray(actual) && arg.every(x => actual.some(y => exact(x, y))) ? null : { path: at, expected, actual };
    throw new Invalid(`unknown matcher ${op} at ${at}`);
  }
  if (Array.isArray(expected)) {
    if (!Array.isArray(actual) || actual.length !== expected.length) return { path: at, expected, actual };
    for (let i = 0; i < expected.length; i++) { const m = match(actual[i], expected[i], `${at}/${i}`); if (m) return m; }
    return null;
  }
  if (expected && typeof expected === 'object') {
    if (!actual || typeof actual !== 'object' || Array.isArray(actual)) return { path: at, expected, actual };
    for (const [k, v] of Object.entries(expected)) {
      const child = `${at}/${k.replace(/~/g, '~0').replace(/\//g, '~1')}`;
      if (isMatcher(v) && '$absent' in v) { if (Object.hasOwn(actual, k)) return { path: child, expected: v, actual: actual[k] }; continue; }
      if (!Object.hasOwn(actual, k)) return { path: child, expected: v, actual: '(absent)' };
      const m = match(actual[k], v, child); if (m) return m;
    }
    return null;
  }
  return actual === expected ? null : { path: at, expected, actual };
}
function expectAt(doc, p, expected) {
  const { found, value } = get(doc, p);
  if (isMatcher(expected) && '$absent' in expected) return found ? { path: p, expected, actual: value } : null;
  if (!found) return { path: p, expected, actual: '(path not found)' };
  return match(value, expected, p);
}

// ---- sessions ----
function groundsWithinLineage(snapshot) {
  const pairs = [
    ...Object.entries(snapshot.value_grounds ?? {}).map(([n, g]) => [`value ${n}`, g, snapshot.qualified_values?.[n]?.provenance]),
    ...Object.entries(snapshot.commitment_grounds ?? {}).map(([n, g]) => [`commitment ${n}`, g, snapshot.commitment_bases?.[n]?.provenance]),
  ];
  for (const [label, grounds, lineage] of pairs) {
    for (const key of ['evidence', 'caveats']) {
      for (const name of grounds[key] ?? []) {
        if (!lineage?.[key]?.includes(name)) throw new Failure({ kind: 'runtime-invariant', message: `grounds outside lineage: ${label} ${key} ${name}` });
      }
    }
  }
}
function dispatch(session, event, payloadText) {
  let text;
  try { text = session.dispatch_outcome(event, payloadText); } catch (error) { return { outcome: 'fatal', message: String(error?.message ?? error) }; }
  let o;
  try { o = JSON.parse(text); } catch { return { outcome: 'fatal', message: 'unparseable outcome' }; }
  if (o.schema !== 'caveat-dispatch/0.1' || !['accepted', 'rejected'].includes(o.outcome) || (o.outcome === 'rejected' && !ORIGINS.has(o.origin))) {
    return { outcome: 'fatal', message: `unrecognised outcome ${text.slice(0, 200)}` };
  }
  return o;
}
function outcomeMatches(o, expected) {
  if (expected === undefined) return o.outcome === 'accepted';
  if (o.outcome !== 'rejected') return false;
  if (expected === true) return o.origin === 'policy';
  if (typeof expected === 'string') return o.origin === 'policy' && o.message === expected;
  return o.origin === expected.origin && (!('code' in expected) || o.code === expected.code) && (!('message' in expected) || o.message === expected.message);
}
const brief = o => (o.outcome === 'rejected' ? { outcome: o.outcome, origin: o.origin, code: o.code, message: o.message } : { outcome: o.outcome, ...(o.message ? { message: o.message } : {}) });
const bytes = doc => Buffer.byteLength(JSON.stringify(doc));

function runScenario(defaultSource, scenario, sources) {
  const source = sources.get(scenario.source ?? defaultSource);
  let stepNumber = 0;
  let sessions = [];
  const states = new Map();
  const primary = () => sessions[sessions.length - 1];
  const snap = s => JSON.parse(s.snapshot());
  const counts = { dispatches: 0, accepted: 0, rejected: 0, resumes: 0 };
  try {
    try { sessions.push(new WebReactiveSession(source)); }
    catch (error) { throw new Failure({ kind: 'load', message: String(error?.message ?? error) }); }
    states.set('initial', snap(primary()));
    groundsWithinLineage(states.get('initial'));
    for (const [index, step] of scenario.steps.entries()) {
      stepNumber = index + 1;
      const current = () => snap(primary());
      if ('send' in step) {
        states.set('before', current());
        const payloadText = JSON.stringify('payload' in step ? step.payload : {});
        for (let r = 0; r < (step.repeat ?? 1); r++) {
          const results = sessions.map((session, i) => {
            const before = { save: session.save(), view: session.view(), snapshot: session.snapshot() };
            const o = dispatch(session, step.send, payloadText);
            const who = i === sessions.length - 1 ? 'primary' : `shadow ${i + 1}`;
            if (o.outcome === 'fatal') throw new Failure({ kind: 'send', session: who, message: 'fatal outcome', outcome: o, repetition: r + 1 });
            if (o.outcome === 'rejected') {
              for (const part of ['save', 'view', 'snapshot']) {
                if (session[part]() !== before[part]) throw new Failure({ kind: 'send', session: who, message: `rejected event changed the ${part}`, outcome: brief(o), repetition: r + 1 });
              }
            } else groundsWithinLineage(o.snapshot);
            return { o, who, snapshot: o.snapshot ? JSON.stringify(o.snapshot) : session.snapshot() };
          });
          const main = results[results.length - 1];
          counts.dispatches++; counts[main.o.outcome]++;
          if (!outcomeMatches(main.o, step.rejected)) throw new Failure({ kind: 'send', session: 'primary', message: 'unexpected outcome', expected: step.rejected ?? 'accepted', outcome: brief(main.o), repetition: r + 1 });
          for (const other of results.slice(0, -1)) {
            if (canon(brief(other.o)) !== canon(brief(main.o))) throw new Failure({ kind: 'send', session: other.who, message: 'resumed session disagreed on the outcome', expected: brief(main.o), outcome: brief(other.o), repetition: r + 1 });
            if (other.o.outcome === 'accepted' && canon(JSON.parse(other.snapshot)) !== canon(JSON.parse(main.snapshot))) throw new Failure({ kind: 'send', session: other.who, message: 'resumed session diverged', repetition: r + 1 });
          }
        }
      } else if ('expect' in step) {
        const doc = current();
        for (const [p, expected] of Object.entries(step.expect)) {
          const m = expectAt(doc, p, expected);
          if (m) throw new Failure({ kind: 'expect', session: 'primary', ...m });
        }
      } else if ('same_as' in step) {
        const doc = current(); const then = states.get(step.same_as);
        if (!then) throw new Failure({ kind: 'same_as', message: `no ${step.same_as} state yet` });
        for (const p of step.paths) {
          const a = get(then, p); const b = get(doc, p);
          if (!a.found || !b.found || !exact(a.value, b.value)) throw new Failure({ kind: 'same_as', session: 'primary', path: p, expected: a.found ? a.value : '(path not found)', actual: b.found ? b.value : '(path not found)' });
        }
      } else if ('checkpoint' in step) {
        states.set(step.checkpoint, current());
      } else if ('resume' in step) {
        const saved = primary().save();
        let restored;
        try { restored = WebReactiveSession.restore(source, saved); } catch (error) { throw new Failure({ kind: 'resume', message: `restore failed: ${String(error?.message ?? error)}` }); }
        sessions.push(restored); counts.resumes++;
        const prior = sessions[sessions.length - 2];
        if (canon(snap(restored)) !== canon(snap(prior)) || canon(JSON.parse(restored.view())) !== canon(JSON.parse(prior.view()))) throw new Failure({ kind: 'resume', message: 'restore changed the snapshot or view' });
        groundsWithinLineage(snap(restored));
      } else if ('size' in step) {
        for (const [doc, bound] of Object.entries(step.size)) {
          const n = doc === 'save' ? bytes(JSON.parse(primary().save())) : bytes(current());
          const limit = 'max' in bound ? bound.max : bytes(states.get(bound.since)) + bound.max_growth;
          if (n > limit) throw new Failure({ kind: 'size', path: doc, expected: `at most ${limit} bytes`, actual: `${n} bytes` });
        }
      }
    }
    stepNumber = scenario.steps.length + 1;
    const final = WebReactiveSession.restore(source, primary().save());
    try { if (canon(snap(final)) !== canon(snap(primary()))) throw new Failure({ kind: 'final-resume', message: 'final restore changed the snapshot' }); }
    finally { final.free(); }
    return { id: scenario.id, pass: true, ...counts };
  } catch (error) {
    if (!(error instanceof Failure)) throw error;
    return { id: scenario.id, pass: false, failure: { step: stepNumber, ...error.detail }, ...counts };
  } finally {
    for (const s of sessions) { try { s.free(); } catch { /* a trapped session is discarded */ } }
  }
}

let doc;
try {
  doc = JSON.parse(await readFile(file, 'utf8'));
  validate(doc);
} catch (error) {
  console.error(`INVALID ${file}: ${error.message}`);
  process.exit(2);
}
const dir = path.dirname(file);
const sources = new Map();
for (const rel of new Set([doc.source, ...doc.scenarios.map(s => s.source).filter(Boolean)])) sources.set(rel, await readFile(path.join(dir, rel), 'utf8'));
const results = doc.scenarios.map(s => runScenario(doc.source, s, sources));
if (asJson) {
  console.log(JSON.stringify({ schema: 'caveat-scenario-report/0.1', file: path.basename(file),
    sources: Object.fromEntries([...sources].map(([k, v]) => [k, createHash('sha256').update(v).digest('hex')])), scenarios: results }, null, 2));
} else {
  for (const r of results) {
    if (r.pass) console.log(`PASS ${r.id}  (${r.dispatches} events, ${r.rejected} rejected, ${r.resumes} resumes)`);
    else console.log(`FAIL ${r.id} step ${r.failure.step} ${r.failure.kind}${r.failure.session ? ` [${r.failure.session}]` : ''}: ${JSON.stringify(Object.fromEntries(Object.entries(r.failure).filter(([k]) => !['step', 'kind', 'session'].includes(k))))}`);
  }
  console.log(`${results.filter(r => r.pass).length}/${results.length} scenarios passed`);
}
process.exitCode = results.every(r => r.pass) ? 0 : 1;
