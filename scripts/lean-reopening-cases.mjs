// Bounded source fixtures for the Lean/Rust reopening comparison.
// This module validates and renders syntax; it does not evaluate Caveat semantics.
export const REOPEN_SCHEMA = 'caveat-reopening/0.1';
export const REOPEN_STATES = Object.freeze(['a', 'b', 'g']);
export const REOPEN_COMMITMENTS = Object.freeze(['plan', 'hold', 'keep']);
// Each evidence with the caveats it carries, as the renderer declares them:
// `cb qualifies eb; late qualifies eb; meta qualifies late;`. `eu` is never observed.
export const EVIDENCE_CAVEATS = Object.freeze({
  ea: Object.freeze(['ca']),
  eb: Object.freeze(['cb', 'late', 'meta']),
  eg: Object.freeze(['cg']),
  eu: Object.freeze([]),
});
const CAVEATS = Object.freeze(['ca', 'cb', 'cg', 'late', 'meta']);

function fail(path, message) {
  throw new TypeError(`${path}: ${message}`);
}

function objectFields(value, fields, path) {
  if (value === null || typeof value !== 'object' || Array.isArray(value)
      || ![Object.prototype, null].includes(Object.getPrototypeOf(value))) {
    fail(path, 'expected a plain object');
  }
  const keys = Reflect.ownKeys(value);
  if (keys.length !== fields.length || keys.some((key) => !fields.includes(key))) {
    fail(path, `expected exactly fields ${fields.join(', ')}`);
  }
  for (const key of fields) {
    if (!Object.hasOwn(value, key) || !('value' in Object.getOwnPropertyDescriptor(value, key))) {
      fail(path, `missing data field ${key}`);
    }
  }
}

function list(value, maximum, allowed, path) {
  if (!Array.isArray(value) || value.length > maximum) fail(path, `expected an array of at most ${maximum}`);
  for (let index = 0; index < value.length; index += 1) {
    if (!Object.hasOwn(value, index) || !allowed.includes(value[index])) {
      fail(`${path}[${index}]`, `expected one of ${allowed.join(', ')}`);
    }
  }
}

/** Validate the bounded fixture protocol; return the original case without mutation. */
export function validateReopenCase(value) {
  objectFields(value, ['schema', 'id', 'seed', 'commits', 'reopenings'], 'case');
  if (value.schema !== REOPEN_SCHEMA) fail('case.schema', `expected ${REOPEN_SCHEMA}`);
  if (typeof value.id !== 'string' || !/^[a-z][a-z0-9-]{0,63}$/.test(value.id)) {
    fail('case.id', 'expected 1..64 lowercase identifier characters');
  }
  objectFields(value.seed, ['a', 'b', 'g'], 'case.seed');
  for (const name of ['a', 'b']) {
    const n = value.seed[name];
    if (!Number.isSafeInteger(n) || Object.is(n, -0) || Math.abs(n) > 1000) fail(`case.seed.${name}`, 'expected an integer in -1000..1000');
  }
  if (value.seed.g !== 0 && value.seed.g !== 1) fail('case.seed.g', 'expected 0 or 1');
  if (!Array.isArray(value.commits) || value.commits.length < 1 || value.commits.length > REOPEN_COMMITMENTS.length) {
    fail('case.commits', `expected 1..${REOPEN_COMMITMENTS.length} commitments`);
  }
  for (const [index, commit] of value.commits.entries()) {
    const path = `case.commits[${index}]`;
    objectFields(commit, ['using', 'retaining'], path);
    if (commit.using !== null) list(commit.using, 3, REOPEN_STATES, `${path}.using`);
    if (commit.using !== null && commit.using.length === 0) fail(`${path}.using`, 'expected null or names');
    list(commit.retaining, 3, CAVEATS, `${path}.retaining`);
  }
  if (!Array.isArray(value.reopenings) || value.reopenings.length < 1 || value.reopenings.length > 4) {
    fail('case.reopenings', 'expected 1..4 reopenings');
  }
  for (const [index, r] of value.reopenings.entries()) {
    const path = `case.reopenings[${index}]`;
    objectFields(r, ['commitment', 'evidence', 'guard'], path);
    if (!REOPEN_COMMITMENTS.includes(r.commitment)) fail(`${path}.commitment`, `expected one of ${REOPEN_COMMITMENTS.join(', ')}`);
    if (!Object.hasOwn(EVIDENCE_CAVEATS, r.evidence)) fail(`${path}.evidence`, 'expected declared evidence');
    if (r.guard !== null && r.guard !== 'g') fail(`${path}.guard`, 'expected null or g');
  }
  return value;
}

const sum = names => names.join(' + ');
const when = name => name === null ? '' : ` when ${name} != 0`;

/** Render only validated names/numbers. No expected metadata outputs. */
export function renderReopenCase(value) {
  validateReopenCase(value);
  const reopenEvents = value.reopenings.map((_, index) => `doubt${index}`);
  const events = ['seed', 'decide', ...reopenEvents];
  const lines = [
    'evidence ea from "conformance a";',
    'evidence eb from "conformance b";',
    'evidence eg from "conformance guard";',
    'evidence eu from "conformance unobserved";',
    ...CAVEATS.map(name => `caveat ${name} consequence material;`),
    'ca qualifies ea;',
    'cb qualifies eb;',
    'cg qualifies eg;',
    'late qualifies eb;',
    'meta qualifies late;',
    ...REOPEN_STATES.map(name => `state ${name} = 0 min -1000000000 max 1000000000;`),
    ...[...events, 'never'].map(name => `event ${name};`),
    'on seed reveal eg;',
    'on seed reveal ea;',
    'on seed reveal eb;',
    'on never reveal eu;',
    `on seed set a = qualified(${value.seed.a}, ea);`,
    `on seed set b = qualified(${value.seed.b}, eb);`,
    `on seed set g = qualified(${value.seed.g}, eg);`,
  ];
  for (const [index, name] of REOPEN_COMMITMENTS.entries()) {
    const commit = value.commits[index];
    // A commitment the case does not make is declared on an event that never runs.
    if (commit === undefined) {
      lines.push(`on never commit ${name} because enough;`);
      continue;
    }
    const using = commit.using === null ? '' : ` using ${sum(commit.using)}`;
    const retaining = commit.retaining.length === 0 ? '' : ` retaining ${commit.retaining.join(', ')}`;
    lines.push(`on decide commit ${name} because enough${using}${retaining};`);
  }
  for (const [index, r] of value.reopenings.entries()) {
    lines.push(`on doubt${index}${when(r.guard)} reopen ${r.commitment} because ${r.evidence};`);
  }
  return { source: `${lines.join('\n')}\n`, events, reopenFrom: 2 };
}

function reopenCase(id, seed, commits, ...reopenings) {
  return { schema: REOPEN_SCHEMA, id, seed, commits, reopenings };
}
const commit = (using, retaining = []) => ({ using, retaining });
const reopen = (commitment, evidence, guardName = null) => ({ commitment, evidence, guard: guardName });

/** Return fresh fixtures each time. Refusals stay in the corpus. */
export function reopenCases() {
  const seed = () => ({ a: 3, b: -2, g: 1 });
  const values = [
    // plan rests on ea and retains late; eb reopens it and its record stays.
    reopenCase('reopen-retains-caveats', seed(), [commit(['a'], ['late'])], reopen('plan', 'eb')),
    // The cause's caveats arrive through the graph: eb carries cb, late and meta.
    reopenCase('reopen-cause-caveats', seed(), [commit(['a', 'b'], ['cg'])], reopen('plan', 'eb')),
    reopenCase('reopen-guard-skipped', { a: 3, b: -2, g: 0 }, [commit(['a'], ['late'])], reopen('plan', 'eb', 'g')),
    reopenCase('reopen-guard-held', seed(), [commit(['g'], ['ca'])], reopen('plan', 'ea', 'g')),
    // A repeated cause adds nothing; a new cause is recorded after the first.
    reopenCase('reopen-repeated-and-new-cause', seed(), [commit(['a'], ['late'])],
      reopen('plan', 'eb'), reopen('plan', 'eb'), reopen('plan', 'ea'), reopen('plan', 'eg')),
    reopenCase('reopen-uncommitted-refused', seed(), [commit(['a'])], reopen('hold', 'eb')),
    reopenCase('reopen-unobserved-refused', seed(), [commit(['a'], ['late'])], reopen('plan', 'eu')),
    // Reopening hold leaves plan and keep exactly as they were.
    reopenCase('reopen-other-commitments', seed(),
      [commit(['a'], ['late']), commit(['b'], ['cg']), commit(null, ['meta'])],
      reopen('hold', 'ea'), reopen('keep', 'eg', 'g'), reopen('plan', 'eu')),
    reopenCase('reopen-without-using', seed(), [commit(null, ['late', 'cb'])], reopen('plan', 'eg')),
  ];
  for (const value of values) validateReopenCase(value);
  return values;
}
