// Bounded source fixtures for the Lean/Rust late-qualification comparison.
// This module validates and renders syntax; it does not evaluate Caveat semantics.
export const LATE_SCHEMA = 'caveat-late-qualification/0.1';
export const LATE_STATES = Object.freeze(['a', 'b', 'g', 'x']);
export const COMMITMENTS = Object.freeze(['plan', 'hold', 'keep']);
export const EVIDENCE = Object.freeze(['ea', 'eb', 'eg', 'eu']);
// Each late caveat with the caveats that qualify it, as the renderer declares them:
// `meta qualifies late; deep qualifies meta;`. `other` has no qualifiers.
export const LATE_CAVEATS = Object.freeze({
  late: Object.freeze(['meta', 'deep']),
  meta: Object.freeze(['deep']),
  other: Object.freeze([]),
});
const CAVEATS = Object.freeze(['ca', 'cb', 'cg', ...Object.keys(LATE_CAVEATS), 'deep']);

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

function guard(value, path) {
  if (value !== null && !LATE_STATES.includes(value)) fail(path, 'expected null or a state');
}

/** Validate the bounded fixture protocol; return the original case without mutation. */
export function validateLateCase(value) {
  objectFields(value, ['schema', 'id', 'seed', 'writes', 'commits', 'qualifications'], 'case');
  if (value.schema !== LATE_SCHEMA) fail('case.schema', `expected ${LATE_SCHEMA}`);
  if (typeof value.id !== 'string' || !/^[a-z][a-z0-9-]{0,63}$/.test(value.id)) {
    fail('case.id', 'expected 1..64 lowercase identifier characters');
  }
  objectFields(value.seed, ['a', 'b', 'g'], 'case.seed');
  for (const name of ['a', 'b']) {
    const n = value.seed[name];
    if (!Number.isSafeInteger(n) || Object.is(n, -0) || Math.abs(n) > 1000) fail(`case.seed.${name}`, 'expected an integer in -1000..1000');
  }
  if (value.seed.g !== 0 && value.seed.g !== 1) fail('case.seed.g', 'expected 0 or 1');
  if (!Array.isArray(value.writes)) fail('case.writes', 'expected an array');
  if (value.writes.length > 2) fail('case.writes', 'at most 2 writes');
  for (const [index, write] of value.writes.entries()) {
    const path = `case.writes[${index}]`;
    objectFields(write, ['body', 'guard', 'citations'], path);
    list(write.body, 3, ['a', 'b', 'g'], `${path}.body`);
    guard(write.guard, `${path}.guard`);
    if (write.citations !== null) list(write.citations, 3, ['a', 'b', 'g'], `${path}.citations`);
  }
  if (!Array.isArray(value.commits) || value.commits.length < 1 || value.commits.length > COMMITMENTS.length) {
    fail('case.commits', `expected 1..${COMMITMENTS.length} commitments`);
  }
  for (const [index, commit] of value.commits.entries()) {
    const path = `case.commits[${index}]`;
    objectFields(commit, ['using', 'guard', 'retaining'], path);
    if (commit.using !== null) list(commit.using, 3, LATE_STATES, `${path}.using`);
    if (commit.using !== null && commit.using.length === 0) fail(`${path}.using`, 'expected null or names');
    guard(commit.guard, `${path}.guard`);
    list(commit.retaining, 3, CAVEATS, `${path}.retaining`);
  }
  if (!Array.isArray(value.qualifications) || value.qualifications.length < 1 || value.qualifications.length > 4) {
    fail('case.qualifications', 'expected 1..4 qualifications');
  }
  for (const [index, q] of value.qualifications.entries()) {
    const path = `case.qualifications[${index}]`;
    objectFields(q, ['evidence', 'caveat', 'guard'], path);
    if (!EVIDENCE.includes(q.evidence)) fail(`${path}.evidence`, `expected one of ${EVIDENCE.join(', ')}`);
    if (!Object.hasOwn(LATE_CAVEATS, q.caveat)) fail(`${path}.caveat`, 'expected a late caveat');
    guard(q.guard, `${path}.guard`);
  }
  return value;
}

const sum = names => names.join(' + ');
const when = name => name === null ? '' : ` when ${name} != 0`;

/** Render only validated names/numbers. No expected numeric or metadata outputs. */
export function renderLateCase(value) {
  validateLateCase(value);
  const qualifyEvents = value.qualifications.map((_, index) => `learn${index}`);
  const events = ['seed', 'prepare', 'decide', ...qualifyEvents];
  const lines = [
    'evidence ea from "conformance a";',
    'evidence eb from "conformance b";',
    'evidence eg from "conformance guard";',
    'evidence eu from "conformance unobserved";',
    ...CAVEATS.map(name => `caveat ${name} consequence material;`),
    'ca qualifies ea;',
    'cb qualifies eb;',
    'cg qualifies eg;',
    'meta qualifies late;',
    'deep qualifies meta;',
    ...LATE_STATES.map(name => `state ${name} = 0 min -1000000000 max 1000000000;`),
    ...[...events, 'never'].map(name => `event ${name};`),
    'on seed reveal eg;',
    'on seed reveal ea;',
    'on seed reveal eb;',
    'on never reveal eu;',
    `on seed set a = qualified(${value.seed.a}, ea);`,
    `on seed set b = qualified(${value.seed.b}, eb);`,
    `on seed set g = qualified(${value.seed.g}, eg);`,
  ];
  for (const write of value.writes) {
    const body = write.body.length === 0 ? '0' : sum(write.body);
    const citation = write.citations === null ? ''
      : ` because ${write.citations.length === 0 ? 'nothing' : write.citations.join(', ')}`;
    lines.push(`on prepare${when(write.guard)} set x = ${body}${citation};`);
  }
  for (const [index, commit] of value.commits.entries()) {
    const using = commit.using === null ? '' : ` using ${sum(commit.using)}`;
    const retaining = commit.retaining.length === 0 ? '' : ` retaining ${commit.retaining.join(', ')}`;
    lines.push(`on decide${when(commit.guard)} commit ${COMMITMENTS[index]} because enough${using}${retaining};`);
  }
  for (const [index, q] of value.qualifications.entries()) {
    lines.push(`on learn${index}${when(q.guard)} qualify ${q.evidence} with ${q.caveat};`);
  }
  return { source: `${lines.join('\n')}\n`, events, qualifyFrom: 3 };
}

function late(id, seed, writes, commits, ...qualifications) {
  return { schema: LATE_SCHEMA, id, seed, writes, commits, qualifications };
}
const write = (body, guardName = null, citations = null) => ({ body, guard: guardName, citations });
const commit = (using, guardName = null, retaining = []) => ({ using, guard: guardName, retaining });
const qualify = (evidence, caveat, guardName = null) => ({ evidence, caveat, guard: guardName });

/** Return fresh fixtures each time. A refusal stays in the corpus. */
export function lateCases() {
  const seed = () => ({ a: 3, b: -2, g: 1 });
  const values = [
    // The decision and the current values both rest on ea; only the values learn.
    late('late-commitment-basis', seed(), [write(['a', 'b'], 'g', ['a'])],
      [commit(['a', 'b'], null, ['cb']), commit(null)], qualify('ea', 'late', 'g')),
    // x reads ea but is grounded on eb: its lineage learns, its grounds do not.
    late('late-lineage-not-grounds', seed(), [write(['a', 'b'], null, ['b'])],
      [commit(['x'])], qualify('ea', 'other')),
    // eg reaches x only as a guard: lineage only, and the commit guarded by g keeps its record.
    late('late-guard-evidence', seed(), [write(['a'], 'g')],
      [commit(['x'], 'g'), commit(['g'])], qualify('eg', 'meta')),
    late('late-skipped-guard', { a: 3, b: -2, g: 0 }, [write(['a'])],
      [commit(['a'])], qualify('ea', 'late', 'g')),
    late('late-unobserved-refused', seed(), [write(['a'])],
      [commit(['a'])], qualify('eu', 'late')),
    late('late-repeated-and-chained', seed(), [write(['a', 'b'])],
      [commit(['x'], null, ['ca']), commit(['b'])],
      qualify('ea', 'late'), qualify('ea', 'late'), qualify('eb', 'meta'), qualify('eu', 'other')),
    late('late-unrelated-commitment', seed(), [],
      [commit(['b']), commit(null, null, ['cb'])], qualify('ea', 'late')),
    late('late-retained-same-caveat', seed(), [write(['a'], null, [])],
      [commit(['a'], null, ['late']), commit(['x'])], qualify('ea', 'late')),
    late('late-every-commitment', seed(), [write(['a', 'b', 'g'])],
      [commit(['a']), commit(['x'], 'g', ['cg']), commit(['a', 'b'])],
      qualify('eb', 'other', 'g'), qualify('ea', 'late', 'g'), qualify('eg', 'late')),
  ];
  for (const value of values) validateLateCase(value);
  return values;
}
