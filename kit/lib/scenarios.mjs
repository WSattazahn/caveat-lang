// Scenarios 0.1 (spec/caveat-scenarios-0.1.md): validation, matching and the
// runner. No Node imports; the CLI supplies files and the runtime.
import { CaveatError, DISPATCH_SCHEMA, ORIGINS } from './session.mjs';

export const SCENARIO_SCHEMA = 'caveat-scenarios/0.1';
export const REPORT_SCHEMA = 'caveat-scenario-report/0.1';
const STEP_KINDS = ['send', 'expect', 'same_as', 'checkpoint', 'resume', 'size'];
const MATCHERS = ['$exact', '$set', '$includes', '$absent'];
const MAX_REPEAT = 10000;

export class ScenarioFileError extends Error {
  constructor(where, message) {
    super(`${where}: ${message}`);
    this.name = 'ScenarioFileError';
    this.where = where;
  }
}

// ---------------------------------------------------------------- validation

const isObject = value => value !== null && typeof value === 'object' && !Array.isArray(value);

function fields(value, allowed, where) {
  if (!isObject(value)) throw new ScenarioFileError(where, 'expected an object');
  for (const key of Object.keys(value)) {
    if (!allowed.includes(key)) throw new ScenarioFileError(where, `unknown field "${key}"`);
  }
}

function checkNote(value, where) {
  if ('note' in value && typeof value.note !== 'string') throw new ScenarioFileError(where, 'note must be text');
}

export function checkPointer(pointer, where) {
  if (typeof pointer !== 'string' || (pointer !== '' && !pointer.startsWith('/'))) {
    throw new ScenarioFileError(where, `${JSON.stringify(pointer)} is not a JSON Pointer`);
  }
  if (/~[^01]|~$/.test(pointer)) throw new ScenarioFileError(where, `invalid escape in ${pointer}`);
}

function checkFinite(value, where) {
  if (typeof value === 'number' && !Number.isFinite(value)) {
    throw new ScenarioFileError(where, 'a number in this file is not finite (such as 1e999); JSON would send it as null');
  }
  if (Array.isArray(value)) value.forEach(item => checkFinite(item, where));
  else if (isObject(value)) Object.values(value).forEach(item => checkFinite(item, where));
}

const matcherName = value => {
  if (!isObject(value)) return null;
  const keys = Object.keys(value);
  return keys.length === 1 && keys[0].startsWith('$') ? keys[0] : null;
};

function hasDollarKey(value) {
  if (Array.isArray(value)) return value.some(hasDollarKey);
  if (!isObject(value)) return false;
  return Object.keys(value).some(key => key.startsWith('$')) || Object.values(value).some(hasDollarKey);
}

// member: the value sits at a path or object member, where $absent is allowed.
function checkExpected(value, where, member) {
  if (Array.isArray(value)) { value.forEach(item => checkExpected(item, where, false)); return; }
  if (!isObject(value)) return;
  const keys = Object.keys(value);
  const dollars = keys.filter(key => key.startsWith('$'));
  if (!dollars.length) { for (const key of keys) checkExpected(value[key], where, true); return; }
  if (keys.length !== 1) {
    throw new ScenarioFileError(where, 'a matcher must be its object\'s only member; write a literal object with $exact');
  }
  const [name] = dollars;
  const argument = value[name];
  if (!MATCHERS.includes(name)) throw new ScenarioFileError(where, `unknown matcher ${name}`);
  if (name === '$absent') {
    if (argument !== true) throw new ScenarioFileError(where, '$absent takes true');
    if (!member) throw new ScenarioFileError(where, '$absent applies to a path or an object member');
  }
  if (name === '$set' || name === '$includes') {
    if (!Array.isArray(argument)) throw new ScenarioFileError(where, `${name} takes an array`);
    if (hasDollarKey(argument)) {
      throw new ScenarioFileError(where, `${name} elements are literal values and cannot contain $ members`);
    }
  }
}

function checkRejected(rejected, where) {
  if (rejected === true) return;
  if (rejected === false) throw new ScenarioFileError(where, 'omit rejected to expect acceptance');
  if (typeof rejected === 'string') {
    if (!rejected) throw new ScenarioFileError(where, 'an expected policy message cannot be empty');
    return;
  }
  fields(rejected, ['origin', 'code', 'message'], `${where} rejected`);
  if (rejected.origin === 'host') {
    throw new ScenarioFileError(where, 'origin "host" is invalid in 0.1: the runner sends events straight to the core');
  }
  if (!ORIGINS.includes(rejected.origin)) throw new ScenarioFileError(where, `origin must be one of ${ORIGINS.join(', ')}`);
  if ('code' in rejected && (typeof rejected.code !== 'string' || !rejected.code)) {
    throw new ScenarioFileError(where, 'code must be non-empty text');
  }
  if ('message' in rejected) {
    if (rejected.origin !== 'policy') throw new ScenarioFileError(where, 'only a policy rejection has a stable message');
    if (typeof rejected.message !== 'string') throw new ScenarioFileError(where, 'message must be text');
  }
}

function checkBoundName(name, names, where) {
  if (!names.has(name)) {
    const hint = name === 'before' ? '; before exists only after the scenario\'s first send' : '';
    throw new ScenarioFileError(where, `unknown state ${JSON.stringify(name)}${hint}`);
  }
}

function validateStep(step, where, names) {
  if (!isObject(step)) throw new ScenarioFileError(where, 'a step is an object');
  const kinds = STEP_KINDS.filter(kind => kind in step);
  if (kinds.length !== 1) throw new ScenarioFileError(where, `a step needs exactly one of ${STEP_KINDS.join(', ')}`);
  const [kind] = kinds;
  checkNote(step, where);
  if (kind === 'send') {
    fields(step, ['send', 'payload', 'rejected', 'repeat', 'note'], where);
    if (typeof step.send !== 'string' || !step.send) throw new ScenarioFileError(where, 'send names an event');
    if ('payload' in step) checkFinite(step.payload, where);
    if ('rejected' in step) checkRejected(step.rejected, where);
    if ('repeat' in step && !(Number.isInteger(step.repeat) && step.repeat >= 1 && step.repeat <= MAX_REPEAT)) {
      throw new ScenarioFileError(where, `repeat must be an integer from 1 to ${MAX_REPEAT}`);
    }
    names.add('before');
  } else if (kind === 'expect') {
    fields(step, ['expect', 'note'], where);
    if (!isObject(step.expect) || !Object.keys(step.expect).length) {
      throw new ScenarioFileError(where, 'expect maps at least one JSON Pointer to a value');
    }
    for (const [pointer, value] of Object.entries(step.expect)) {
      checkPointer(pointer, where);
      checkFinite(value, where);
      checkExpected(value, `${where} ${pointer}`, true);
    }
  } else if (kind === 'same_as') {
    fields(step, ['same_as', 'paths', 'note'], where);
    if (typeof step.same_as !== 'string') throw new ScenarioFileError(where, 'same_as names a stored state');
    checkBoundName(step.same_as, names, where);
    if (!Array.isArray(step.paths) || !step.paths.length) throw new ScenarioFileError(where, 'paths must be a non-empty array');
    for (const pointer of step.paths) checkPointer(pointer, where);
  } else if (kind === 'checkpoint') {
    fields(step, ['checkpoint', 'note'], where);
    const name = step.checkpoint;
    if (typeof name !== 'string' || !name) throw new ScenarioFileError(where, 'checkpoint needs a name');
    if (name === 'initial' || name === 'before' || names.has(name)) {
      throw new ScenarioFileError(where, `checkpoint name ${JSON.stringify(name)} is reserved or already used`);
    }
    names.add(name);
  } else if (kind === 'resume') {
    fields(step, ['resume', 'note'], where);
    if (step.resume !== true) throw new ScenarioFileError(where, 'resume must be true');
  } else if (kind === 'size') {
    fields(step, ['size', 'note'], where);
    fields(step.size, ['save', 'snapshot'], where);
    const entries = Object.entries(step.size);
    if (!entries.length) throw new ScenarioFileError(where, 'size bounds the save, the snapshot or both');
    for (const [document, bound] of entries) {
      fields(bound, ['max', 'max_growth', 'since'], `${where} ${document}`);
      const count = ('max' in bound) + ('max_growth' in bound);
      if (count !== 1) throw new ScenarioFileError(where, 'give either max or max_growth');
      const limit = 'max' in bound ? bound.max : bound.max_growth;
      if (!Number.isInteger(limit) || limit < 0) throw new ScenarioFileError(where, 'a size limit is a non-negative integer');
      if ('max_growth' in bound) {
        if (document !== 'snapshot') throw new ScenarioFileError(where, 'max_growth applies to the snapshot');
        if (typeof bound.since !== 'string') throw new ScenarioFileError(where, 'max_growth needs since');
        checkBoundName(bound.since, names, where);
      } else if ('since' in bound) {
        throw new ScenarioFileError(where, 'since goes with max_growth');
      }
    }
  }
}

export function validateScenarioFile(doc) {
  fields(doc, ['schema', 'source', 'scenarios', 'note'], 'file');
  if (doc.schema !== SCENARIO_SCHEMA) throw new ScenarioFileError('file', `schema must be "${SCENARIO_SCHEMA}"`);
  if (typeof doc.source !== 'string' || !doc.source) throw new ScenarioFileError('file', 'source names the program file');
  checkNote(doc, 'file');
  if (!Array.isArray(doc.scenarios) || !doc.scenarios.length) throw new ScenarioFileError('file', 'scenarios must be a non-empty array');
  const ids = new Set();
  doc.scenarios.forEach((scenario, index) => {
    const where = `scenario ${index + 1}`;
    fields(scenario, ['id', 'title', 'source', 'steps', 'note'], where);
    if (typeof scenario.id !== 'string' || !scenario.id) throw new ScenarioFileError(where, 'id must be non-empty text');
    if (ids.has(scenario.id)) throw new ScenarioFileError(where, `duplicate id ${scenario.id}`);
    ids.add(scenario.id);
    if (typeof scenario.title !== 'string' || !scenario.title) throw new ScenarioFileError(scenario.id, 'title is required');
    if ('source' in scenario && (typeof scenario.source !== 'string' || !scenario.source)) {
      throw new ScenarioFileError(scenario.id, 'source names the program file');
    }
    checkNote(scenario, scenario.id);
    if (!Array.isArray(scenario.steps) || !scenario.steps.length) throw new ScenarioFileError(scenario.id, 'steps must be a non-empty array');
    const names = new Set(['initial']);
    scenario.steps.forEach((step, stepIndex) => validateStep(step, `${scenario.id} step ${stepIndex + 1}`, names));
  });
  return doc;
}

export function parseScenarioFile(text) {
  let doc;
  try { doc = JSON.parse(text); } catch (error) { throw new ScenarioFileError('file', `not JSON: ${error.message}`); }
  return validateScenarioFile(doc);
}

// ---------------------------------------------------------------- matching

export function getPointer(document, pointer) {
  if (pointer === '') return { found: true, value: document };
  let value = document;
  for (const raw of pointer.slice(1).split('/')) {
    const key = raw.replace(/~1/g, '/').replace(/~0/g, '~');
    if (Array.isArray(value)) {
      if (!/^(0|[1-9]\d*)$/.test(key) || Number(key) >= value.length) return { found: false };
      value = value[Number(key)];
    } else if (isObject(value) && Object.hasOwn(value, key)) {
      value = value[key];
    } else {
      return { found: false };
    }
  }
  return { found: true, value };
}

const escapeKey = key => String(key).replace(/~/g, '~0').replace(/\//g, '~1');

export function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (isObject(value)) return `{${Object.keys(value).sort().map(key => `${JSON.stringify(key)}:${canonical(value[key])}`).join(',')}}`;
  return JSON.stringify(value);
}

export const same = (a, b) => canonical(a) === canonical(b);

// The first JSON Pointer at which two documents differ, or null.
export function firstDifference(a, b, at = '') {
  if (Array.isArray(a) && Array.isArray(b)) {
    for (let index = 0; index < Math.max(a.length, b.length); index++) {
      if (index >= a.length || index >= b.length) {
        return { path: `${at}/${index}`, expected: index < a.length ? a[index] : ABSENT, actual: index < b.length ? b[index] : ABSENT };
      }
      const found = firstDifference(a[index], b[index], `${at}/${index}`);
      if (found) return found;
    }
    return null;
  }
  if (isObject(a) && isObject(b)) {
    for (const key of [...new Set([...Object.keys(a), ...Object.keys(b)])].sort()) {
      const where = `${at}/${escapeKey(key)}`;
      if (!Object.hasOwn(a, key) || !Object.hasOwn(b, key)) {
        return { path: where, expected: Object.hasOwn(a, key) ? a[key] : ABSENT, actual: Object.hasOwn(b, key) ? b[key] : ABSENT };
      }
      const found = firstDifference(a[key], b[key], where);
      if (found) return found;
    }
    return null;
  }
  return same(a, b) ? null : { path: at, expected: a, actual: b };
}

// Marks a missing value in failures. JSON reports write it as the $absent
// matcher, so a missing value is never confused with null or dropped.
export const ABSENT = Object.freeze({ toJSON: () => ({ $absent: true }) });

function countBy(values) {
  const counts = new Map();
  for (const value of values) {
    const key = canonical(value);
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  return counts;
}

// Returns null when actual matches expected, otherwise {path, expected, actual}.
export function match(actual, expected, at = '') {
  const name = matcherName(expected);
  if (name) {
    const argument = expected[name];
    const miss = { path: at, expected, actual };
    if (name === '$exact') return same(actual, argument) ? null : miss;
    if (name === '$absent') return miss;
    if (!Array.isArray(actual)) return miss;
    const have = countBy(actual);
    const want = countBy(argument);
    if (name === '$set') {
      if (actual.length !== argument.length) return miss;
      for (const [key, count] of want) if (have.get(key) !== count) return miss;
      return null;
    }
    for (const [key, count] of want) if ((have.get(key) ?? 0) < count) return miss;
    return null;
  }
  if (Array.isArray(expected)) {
    if (!Array.isArray(actual) || actual.length !== expected.length) return { path: at, expected, actual };
    for (let index = 0; index < expected.length; index++) {
      const found = match(actual[index], expected[index], `${at}/${index}`);
      if (found) return found;
    }
    return null;
  }
  if (isObject(expected)) {
    if (!isObject(actual)) return { path: at, expected, actual };
    for (const [key, value] of Object.entries(expected)) {
      const where = `${at}/${escapeKey(key)}`;
      if (matcherName(value) === '$absent') {
        if (Object.hasOwn(actual, key)) return { path: where, expected: value, actual: actual[key] };
        continue;
      }
      if (!Object.hasOwn(actual, key)) return { path: where, expected: value, actual: ABSENT };
      const found = match(actual[key], value, where);
      if (found) return found;
    }
    return null;
  }
  return actual === expected ? null : { path: at, expected, actual };
}

export function expectAt(document, pointer, expected) {
  const { found, value } = getPointer(document, pointer);
  if (matcherName(expected) === '$absent') return found ? { path: pointer, expected, actual: value } : null;
  if (!found) return { path: pointer, expected, actual: ABSENT };
  return match(value, expected, pointer);
}

// ---------------------------------------------------------------- runner

class Failure extends Error {
  constructor(detail) {
    super(detail.message ?? detail.kind);
    this.detail = detail;
  }
}

const encoder = new TextEncoder();
const bytes = value => encoder.encode(JSON.stringify(value)).length;

function describeExpected(rejected) {
  if (rejected === undefined) return 'acceptance';
  if (rejected === true) return 'a policy rejection';
  if (typeof rejected === 'string') return `a policy rejection ${JSON.stringify(rejected)}`;
  const code = rejected.code ? `/${rejected.code}` : '';
  const message = 'message' in rejected ? ` ${JSON.stringify(rejected.message)}` : '';
  const article = /^[aeiou]/.test(rejected.origin) ? 'an' : 'a';
  return `${article} ${rejected.origin}${code} rejection${message}`;
}

const summary = outcome => (outcome.outcome === 'accepted'
  ? { outcome: 'accepted' }
  : { outcome: outcome.outcome, origin: outcome.origin, code: outcome.code, message: outcome.message });

function outcomeMatches(outcome, rejected) {
  if (rejected === undefined) return outcome.outcome === 'accepted';
  if (outcome.outcome !== 'rejected') return false;
  if (rejected === true) return outcome.origin === 'policy';
  if (typeof rejected === 'string') return outcome.origin === 'policy' && outcome.message === rejected;
  return outcome.origin === rejected.origin
    && (!('code' in rejected) || outcome.code === rejected.code)
    && (!('message' in rejected) || outcome.message === rejected.message);
}

// Grounds are within lineage: every grounded name appears in the provenance
// of the same value or commitment.
export function groundsViolation(snapshot) {
  const groups = [
    ['value', snapshot.value_grounds, name => snapshot.qualified_values?.[name]?.provenance],
    ['commitment', snapshot.commitment_grounds, name => snapshot.commitment_bases?.[name]?.provenance],
  ];
  for (const [label, grounds, lineageOf] of groups) {
    for (const [name, grounded] of Object.entries(grounds ?? {})) {
      const lineage = lineageOf(name);
      for (const part of ['evidence', 'caveats']) {
        for (const item of grounded?.[part] ?? []) {
          if (!Array.isArray(lineage?.[part]) || !lineage[part].includes(item)) {
            return { path: `/${label === 'value' ? 'value_grounds' : 'commitment_grounds'}/${escapeKey(name)}/${part}`, item, message: `${label} ${name} is grounded on ${part} ${item}, which is not in its lineage` };
          }
        }
      }
    }
  }
  return null;
}

async function runScenario(scenario, context) {
  const { source } = context;
  const sessions = [];
  const extra = [];
  const states = new Map();
  let result;
  const counts = { events: 0, rejected: 0, resumes: 0 };
  let step = 0;
  let runtime = context.runtime;
  const primary = () => sessions[sessions.length - 1];
  const label = entry => {
    if (extra.includes(entry)) return 'final restore';
    return entry === primary() ? 'primary' : `shadow ${sessions.indexOf(entry) + 1}`;
  };
  const fail = detail => { throw new Failure({ step, ...detail }); };
  const fatal = (entry, error, extra = {}) => fail({
    kind: 'fatal', category: 'fatal', session: label(entry),
    outcome: { outcome: 'fatal', code: error.report?.code ?? null, message: error.report?.message ?? error.message }, ...extra,
  });
  const checkGrounds = (entry, snapshot) => {
    const violation = groundsViolation(snapshot);
    if (violation) fail({ kind: 'grounds', category: 'runtime-invariant', session: label(entry), path: violation.path, message: violation.message });
  };
  const read = (entry, what) => {
    try { return entry.session[what](); } catch (error) {
      if (error instanceof CaveatError) fatal(entry, error, { message: `${what}() failed` });
      throw error;
    }
  };
  const compareRestored = (restored, original, kind, name) => {
    const snapshot = read(restored, 'snapshot');
    const difference = firstDifference(original.snapshot, snapshot);
    if (difference) fail({ kind, category: 'runtime-invariant', session: name, ...difference, message: 'restoring the save changed the snapshot' });
    const viewDifference = firstDifference(read(original, 'view'), read(restored, 'view'));
    if (viewDifference) fail({ kind, category: 'runtime-invariant', session: name, ...viewDifference, message: 'restoring the save changed the view' });
    restored.snapshot = snapshot;
    const violation = groundsViolation(snapshot);
    if (violation) fail({ kind: 'grounds', category: 'runtime-invariant', session: name, path: violation.path, message: violation.message });
  };
  const restore = (entry, kind) => {
    const saved = read(entry, 'save');
    try {
      return { session: runtime.restore(source, saved), snapshot: null };
    } catch (error) {
      if (!(error instanceof CaveatError)) throw error;
      return fail({ kind, category: error.kind === 'fatal' ? 'fatal' : 'runtime-invariant', session: label(entry), message: `restore failed: ${error.message}` });
    }
  };

  const dispatch = (entry, send, repetition) => {
    const before = { save: read(entry, 'save'), view: read(entry, 'viewText') };
    let outcome;
    try {
      outcome = entry.session.dispatch(send.send, 'payload' in send ? send.payload : {});
    } catch (error) {
      if (error instanceof CaveatError) fatal(entry, error, { repetition });
      throw error;
    }
    if (outcome.outcome === 'rejected') {
      const unchanged = (what, earlier, later) => {
        const difference = firstDifference(earlier, later);
        if (difference) {
          fail({ kind: 'atomicity', category: 'runtime-invariant', session: label(entry), repetition, ...difference, message: `a rejected event changed the ${what}`, outcome: summary(outcome) });
        }
      };
      unchanged('save', JSON.parse(before.save), JSON.parse(read(entry, 'save')));
      unchanged('view', JSON.parse(before.view), read(entry, 'view'));
      unchanged('snapshot', entry.snapshot, read(entry, 'snapshot'));
    } else {
      entry.snapshot = outcome.snapshot;
      checkGrounds(entry, outcome.snapshot);
    }
    return outcome;
  };

  try {
    try {
      sessions.push({ session: runtime.open(source), snapshot: null });
    } catch (error) {
      if (!(error instanceof CaveatError)) throw error;
      fail({ kind: 'load', category: error.kind === 'fatal' ? 'fatal' : 'load', message: error.message });
    }
    primary().snapshot = read(primary(), 'snapshot');
    states.set('initial', primary().snapshot);
    checkGrounds(primary(), primary().snapshot);

    for (const [index, item] of scenario.steps.entries()) {
      step = index + 1;
      if ('send' in item) {
        states.set('before', primary().snapshot);
        for (let repetition = 1; repetition <= (item.repeat ?? 1); repetition++) {
          const lead = primary();
          const outcome = dispatch(lead, item, repetition);
          counts.events += 1;
          if (outcome.outcome === 'rejected') counts.rejected += 1;
          if (!outcomeMatches(outcome, item.rejected)) {
            fail({ kind: 'send', category: 'expectation', session: 'primary', repetition, expected: describeExpected(item.rejected), outcome: summary(outcome) });
          }
          for (const shadow of sessions.slice(0, -1)) {
            const echoed = dispatch(shadow, item, repetition);
            if (!same(summary(echoed), summary(outcome))) {
              fail({ kind: 'agreement', category: 'runtime-invariant', session: label(shadow), repetition, message: 'a resumed session disagreed on the outcome', expected: summary(outcome), actual: summary(echoed) });
            }
            if (echoed.outcome === 'accepted') {
              const difference = firstDifference(lead.snapshot, shadow.snapshot);
              if (difference) fail({ kind: 'agreement', category: 'runtime-invariant', session: label(shadow), repetition, ...difference, message: 'a resumed session reached a different snapshot' });
            }
          }
        }
      } else if ('expect' in item) {
        for (const [pointer, expected] of Object.entries(item.expect)) {
          const miss = expectAt(primary().snapshot, pointer, expected);
          if (miss) fail({ kind: 'expect', category: 'expectation', session: 'primary', ...miss });
        }
      } else if ('same_as' in item) {
        const then = states.get(item.same_as);
        for (const pointer of item.paths) {
          const earlier = getPointer(then, pointer);
          const now = getPointer(primary().snapshot, pointer);
          if (!earlier.found || !now.found) {
            fail({ kind: 'same_as', category: 'expectation', session: 'primary', path: pointer, from: item.same_as,
              expected: earlier.found ? earlier.value : ABSENT, actual: now.found ? now.value : ABSENT,
              message: `path missing from ${!earlier.found ? `the ${item.same_as} snapshot` : 'the current snapshot'}` });
          }
          const difference = firstDifference(earlier.value, now.value, pointer);
          if (difference) fail({ kind: 'same_as', category: 'expectation', session: 'primary', from: item.same_as, ...difference });
        }
      } else if ('checkpoint' in item) {
        states.set(item.checkpoint, primary().snapshot);
      } else if ('resume' in item) {
        const original = primary();
        const restored = restore(original, 'resume');
        sessions.push(restored);
        counts.resumes += 1;
        compareRestored(restored, original, 'resume', 'primary');
      } else if ('size' in item) {
        for (const [document, bound] of Object.entries(item.size)) {
          const measured = bytes(document === 'save' ? JSON.parse(read(primary(), 'save')) : primary().snapshot);
          const limit = 'max' in bound ? bound.max : bytes(states.get(bound.since)) + bound.max_growth;
          if (measured > limit) fail({ kind: 'size', category: 'expectation', session: 'primary', path: document, limit, bytes: measured });
        }
      }
    }

    step = scenario.steps.length + 1;
    const final = restore(primary(), 'final-resume');
    extra.push(final);
    compareRestored(final, primary(), 'final-resume', 'final restore');
    result = { id: scenario.id, title: scenario.title, pass: true, ...counts };
  } catch (error) {
    if (!(error instanceof Failure)) throw error;
    result = { id: scenario.id, title: scenario.title, pass: false, ...counts, failure: error.detail };
  } finally {
    for (const entry of [...sessions, ...extra]) entry.session.close();
  }
  // Releasing sessions calls into the runtime too; a trap there is a failure.
  if (result.pass && runtime.trapped) {
    result = { ...result, pass: false, failure: { step, kind: 'fatal', category: 'fatal', session: null,
      outcome: { outcome: 'fatal', code: null, message: 'the runtime trapped while releasing a session' } } };
  }
  return result;
}

// options.runtime: from createRuntime/loadRuntime. options.reload: optional
// async function returning a fresh runtime after a WebAssembly trap.
// options.readSource(path): the program text for a path relative to the file.
export async function runScenarioFile(doc, { runtime, reload, readSource, file = null }) {
  validateScenarioFile(doc);
  const sources = new Map();
  const sourceFor = async path => {
    if (!sources.has(path)) sources.set(path, await readSource(path));
    return sources.get(path);
  };
  const results = [];
  let current = runtime;
  for (const scenario of doc.scenarios) {
    if (current.trapped) {
      current = reload ? await reload() : current;
    }
    let source;
    try {
      source = await sourceFor(scenario.source ?? doc.source);
    } catch (error) {
      results.push({ id: scenario.id, title: scenario.title, pass: false, events: 0, rejected: 0, resumes: 0,
        failure: { step: 0, kind: 'load', category: 'load', message: `cannot read source: ${error.message}` } });
      continue;
    }
    results.push(await runScenario(scenario, { source, runtime: current }));
  }
  return {
    file,
    sources: [...sources.keys()],
    scenarios: results,
    passed: results.filter(result => result.pass).length,
    failed: results.filter(result => !result.pass).length,
  };
}

export function report(files, runtimeIdentity = {}) {
  return {
    schema: REPORT_SCHEMA,
    dispatchSchema: DISPATCH_SCHEMA,
    runtime: runtimeIdentity,
    files,
    passed: files.reduce((total, file) => total + file.passed, 0),
    failed: files.reduce((total, file) => total + file.failed, 0),
  };
}

// ---------------------------------------------------------------- text

const shown = value => {
  if (value === ABSENT) return '(absent)';
  const text = value === undefined ? 'undefined' : JSON.stringify(value);
  return text.length > 240 ? `${text.slice(0, 237)}...` : text;
};

const outcomeText = outcome => (outcome.outcome === 'accepted' ? 'acceptance'
  : outcome.outcome === 'fatal' ? `fatal ${outcome.code ?? 'error'} ${JSON.stringify(outcome.message)}`
    : `${outcome.origin}/${outcome.code} ${JSON.stringify(outcome.message)}`);

export function failureText(failure) {
  const repeat = failure.repetition > 1 ? ` (repetition ${failure.repetition})` : '';
  const at = failure.path !== undefined ? `${failure.path || '(root)'} ` : '';
  switch (failure.kind) {
    case 'send': return `expected ${failure.expected}; got ${outcomeText(failure.outcome)}${repeat}`;
    case 'fatal': return `${outcomeText(failure.outcome)}${failure.message ? ` during ${failure.message}` : ''}${repeat}`;
    case 'expect': return `${at}expected ${shown(failure.expected)}, actual ${shown(failure.actual)}`;
    case 'same_as': return `${at}was ${shown(failure.expected)} at "${failure.from}", now ${shown(failure.actual)}${failure.message ? ` (${failure.message})` : ''}`;
    case 'size': return `${failure.path} is ${failure.bytes} bytes, limit ${failure.limit}`;
    case 'load': return failure.message;
    default: {
      const detail = 'expected' in failure || 'actual' in failure ? ` at ${failure.path || '(root)'}: expected ${shown(failure.expected)}, actual ${shown(failure.actual)}` : '';
      return `runtime invariant: ${failure.message}${detail}${repeat}`;
    }
  }
}

export function formatFileReport(fileReport) {
  const lines = [];
  for (const result of fileReport.scenarios) {
    if (result.pass) {
      lines.push(`PASS ${result.id}  ${result.title}  (${result.events} events, ${result.rejected} rejected, ${result.resumes} resumes)`);
    } else {
      const failure = result.failure;
      const session = failure.session ? ` [${failure.session}]` : '';
      lines.push(`FAIL ${result.id} step ${failure.step} ${failure.kind}${session}: ${failureText(failure)}`);
    }
  }
  lines.push(`${fileReport.passed} passed, ${fileReport.failed} failed${fileReport.file ? ` (${fileReport.file})` : ''}`);
  return lines.join('\n');
}
