// Bounded source fixtures for the Lean/Rust guard-and-citation comparison.
// This module validates and renders syntax; it does not evaluate Caveat semantics.
export const SCHEMA = 'caveat-guard-citation/0.2';
export const STATES = Object.freeze(['a', 'b', 'g', 'x', 'y']);
export const GENERATOR_SEED = 0x05eedca7;
export const MAX_CASE_BYTES = 64 * 1024;

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

function integer(value, minimum, maximum, path) {
  if (!Number.isSafeInteger(value) || Object.is(value, -0) || value < minimum || value > maximum) {
    fail(path, `expected an integer in ${minimum}..${maximum}, excluding negative zero`);
  }
}

function reference(value, path) {
  if (!STATES.includes(value)) fail(path, 'expected one of a, b, g, x, y');
}

function arrayLength(value, minimum, maximum, path) {
  if (!Array.isArray(value) || value.length < minimum || value.length > maximum) {
    fail(path, `expected an array of length ${minimum}..${maximum}`);
  }
}

function references(value, path) {
  arrayLength(value, 0, 4, path);
  for (const [index, entry] of value.entries()) reference(entry, `${path}[${index}]`);
}

// One nonnested conditional expression; external statement guards remain separate.
function bodyWidth(value, path) {
  if (Array.isArray(value)) { references(value, path); return value.length; }
  objectFields(value, ['condition', 'then', 'else'], path);
  reference(value.condition, path + '.condition');
  references(value.then, path + '.then');
  references(value.else, path + '.else');
  return Math.max(value.then.length, value.else.length);
}

const renderSum = names => names.length === 0 ? '0' : names.join(' + ');
function renderBody(body) {
  return Array.isArray(body) ? renderSum(body)
    : 'if(' + body.condition + ' != 0, ' + renderSum(body.then) + ', ' + renderSum(body.else) + ')';
}

/** Validate the bounded JSON protocol; return the original case without mutation. */
export function validateCase(value) {
  objectFields(value, ['schema', 'id', 'seed', 'steps'], 'case');
  if (value.schema !== SCHEMA) fail('case.schema', `expected ${SCHEMA}`);
  if (typeof value.id !== 'string' || !/^[a-z][a-z0-9-]{0,63}$/.test(value.id)) {
    fail('case.id', 'expected 1..64 lowercase identifier characters');
  }
  objectFields(value.seed, ['a', 'b', 'g'], 'case.seed');
  integer(value.seed.a, -1000, 1000, 'case.seed.a');
  integer(value.seed.b, -1000, 1000, 'case.seed.b');
  integer(value.seed.g, 0, 1, 'case.seed.g');
  arrayLength(value.steps, 1, 8, 'case.steps');
  let totalActions = 0;
  let magnitudeBound = Math.max(1, Math.abs(value.seed.a), Math.abs(value.seed.b));
  for (const [stepIndex, step] of value.steps.entries()) {
    const stepPath = `case.steps[${stepIndex}]`;
    objectFields(step, ['actions'], stepPath);
    arrayLength(step.actions, 1, 4, `${stepPath}.actions`);
    totalActions += step.actions.length;
    if (totalActions > 8) fail('case.steps', 'at most 8 total actions are admitted');
    for (const [actionIndex, action] of step.actions.entries()) {
      const path = `${stepPath}.actions[${actionIndex}]`;
      objectFields(action, ['target', 'body', 'guard', 'citations'], path);
      reference(action.target, `${path}.target`);
      const width = bodyWidth(action.body, `${path}.body`);
      if (action.guard !== null) reference(action.guard, `${path}.guard`);
      if (action.citations !== null) references(action.citations, `${path}.citations`);
      magnitudeBound *= Math.max(1, width);
      if (magnitudeBound > 1_000_000_000) fail(path, 'conservative numeric bound exceeds 1e9');
    }
  }
  // The host must separately bound the original input bytes before parsing.
  if (Buffer.byteLength(JSON.stringify(value), 'utf8') > MAX_CASE_BYTES) {
    fail('case', 'compact JSON exceeds 64 KiB');
  }
  return value;
}

/** Render only validated names/numbers. No expected numeric or metadata outputs. */
export function renderCase(value) {
  validateCase(value);
  const events = ['seed', ...value.steps.map((_, index) => `step${index}`)];
  const lines = [
    'evidence ea from "conformance a";',
    'evidence eb from "conformance b";',
    'evidence eg from "conformance guard";',
    'caveat ca consequence material;',
    'caveat cb consequence material;',
    'caveat cg consequence material;',
    'ca qualifies ea;',
    'cb qualifies eb;',
    'cg qualifies eg;',
    ...STATES.map((name) => `state ${name} = 0 min -1000000000 max 1000000000;`),
    ...events.map((name) => `event ${name};`),
    'on seed reveal eg;',
    'on seed reveal ea;',
    'on seed reveal eb;',
    `on seed set a = qualified(${value.seed.a}, ea);`,
    `on seed set b = qualified(${value.seed.b}, eb);`,
    `on seed set g = qualified(${value.seed.g}, eg);`,
  ];
  for (const [index, step] of value.steps.entries()) {
    for (const action of step.actions) {
      const guard = action.guard === null ? '' : ` when ${action.guard} != 0`;
      const body = renderBody(action.body);
      const citation = action.citations === null ? ''
        : ` because ${action.citations.length === 0 ? 'nothing' : action.citations.join(', ')}`;
      lines.push(`on step${index}${guard} set ${action.target} = ${body}${citation};`);
    }
  }
  return { source: `${lines.join('\n')}\n`, events };
}

function action(target, body, guard = null, citations = null) {
  return { target, body, guard, citations };
}

function fixture(id, seed, ...steps) {
  return { schema: SCHEMA, id, seed, steps: steps.map((actions) => ({ actions })) };
}

function selection(condition, yes, no) { return { condition, then: yes, else: no }; }

function branchCases() {
  const seed = () => ({ a: 3, b: -2, g: 1 });
  const choose = () => selection('g', ['a'], ['b']);
  return [
    fixture('branch-true', seed(), [action('x', choose())]),
    fixture('branch-false', { a: 3, b: -2, g: 0 }, [action('x', choose())]),
    fixture('branch-equal-values', { a: 7, b: 7, g: 1 }, [action('x', choose())]),
    fixture('branch-equal-values-false', { a: 7, b: 7, g: 0 }, [action('x', choose())]),
    fixture('branch-zero-selected', { a: 0, b: -2, g: 1 }, [action('x', choose())]),
    fixture('branch-negative-condition', seed(), [action('x', selection('b', ['a'], ['g']))]),
    fixture('branch-empty-selected', seed(), [action('x', selection('g', [], ['b']))]),
    fixture('branch-duplicate-reads', seed(), [action('x', selection('g', ['a', 'a', 'b'], []))]),
    fixture('branch-condition-narrowed', seed(),
      [action('g', ['g'], null, [])], [action('x', choose())]),
    fixture('branch-selected-narrowed', seed(),
      [action('a', ['a'], null, [])], [action('x', choose())]),
    fixture('branch-condition-narrowed-false', { a: 3, b: -2, g: 0 },
      [action('g', ['g'], null, [])], [action('x', choose())]),
    fixture('branch-selected-narrowed-false', { a: 3, b: -2, g: 0 },
      [action('b', ['b'], null, [])], [action('x', choose())]),
    fixture('branch-outer-guard', seed(), [action('x', selection('a', ['b'], []), 'g')]),
    fixture('branch-skipped-outer-guard', { a: 3, b: -2, g: 0 },
      [action('x', ['b'])], [action('x', selection('a', ['a'], ['b']), 'g', ['a'])]),
    fixture('branch-cite-condition', seed(), [action('x', choose(), null, ['g'])]),
    fixture('branch-cite-selected', seed(), [action('x', choose(), null, ['a'])]),
    fixture('branch-cite-unselected', seed(), [action('x', choose(), null, ['b'])], [action('y', ['b'])]),
    fixture('branch-late-rejection', seed(),
      [action('x', ['b']), action('y', choose(), null, ['b'])], [action('y', ['x'])]),
    fixture('branch-sequential-selection', { a: 3, b: -2, g: 0 },
      [action('g', ['a']), action('x', choose())], [action('g', []), action('y', choose())]),
  ];
}

function fixedCases() {
  const seed = () => ({ a: 3, b: -2, g: 1 });
  return [
    fixture('eager-cancellation', { a: 2, b: -2, g: 1 }, [action('x', ['a', 'b'])]),
    fixture('eager-duplicate-read', seed(), [action('x', ['a', 'a', 'b'])]),
    fixture('successful-guard', seed(), [action('x', ['a'], 'g')]),
    fixture('skipped-guard', { a: 3, b: -2, g: 0 },
      [action('x', ['b'])], [action('x', ['a'], 'g', ['b'])]),
    fixture('invalid-citation', seed(), [action('x', ['a'], null, ['b'])], [action('y', ['b'])]),
    fixture('late-rejection', seed(),
      [action('x', ['a']), action('y', ['b'], null, ['a'])], [action('y', ['x', 'b'])]),
    fixture('self-citation-prewrite', seed(),
      [action('x', ['b']), action('x', ['a'], null, ['x'])], [action('x', ['b'], null, ['b'])]),
    fixture('valid-self-citation', seed(),
      [action('x', ['a'])], [action('x', ['x', 'b'], null, ['x'])]),
    fixture('guard-only-citation', seed(), [action('x', ['a'], 'g', ['g'])]),
    fixture('citation-reads-grounds', seed(),
      [action('x', ['a'], 'g')], [action('y', ['a'], null, ['x'])]),
    fixture('guard-reads-lineage', seed(),
      [action('g', ['b'], null, [])], [action('x', ['a'], 'g')]),
    fixture('narrowed-grounds-flow', seed(),
      [action('x', ['a', 'b'], null, [])], [action('y', ['x'])]),
    fixture('citation-union-duplicate', seed(), [action('x', ['a', 'b'], null, ['a', 'a', 'b'])]),
    fixture('fresh-assignment', seed(), [action('x', ['a'])], [action('x', [])]),
    fixture('negative-guard', { a: -2, b: 3, g: 0 }, [action('x', ['b'], 'a')]),
    fixture('sequential-guard', { a: 3, b: -2, g: 0 },
      [action('g', ['a']), action('x', ['b'], 'g')]),
    fixture('sequential-body', seed(), [action('x', ['a']), action('y', ['x', 'b'])]),
    fixture('empty-citation', seed(), [action('x', ['a', 'b'], null, [])]),
    fixture('empty-body', seed(), [action('x', [])]),
    fixture('late-rejection-retains-prior-step', seed(), [action('x', ['a'])],
      [action('y', ['b']), action('x', ['b'], null, ['a'])], [action('y', ['x'])]),
    fixture('zero-valued-evidence', { a: 0, b: 0, g: 1 },
      [action('x', ['a', 'b']), action('y', ['x'], null, ['a'])]),
  ];
}

/** xorshift32: all choices derive from the recorded seed, never Math.random. */
function generator(seed) {
  let state = seed >>> 0;
  return (bound) => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    return (state >>> 0) % bound;
  };
}

function generatedCases() {
  const pick = generator(GENERATOR_SEED);
  const refs = () => Array.from({ length: pick(5) }, () => STATES[pick(STATES.length)]);
  return Array.from({ length: 32 }, (_, index) => {
    const seed = { a: pick(2001) - 1000, b: pick(2001) - 1000, g: pick(2) };
    let remaining = 1 + pick(8);
    const steps = [];
    while (remaining > 0) {
      const count = 1 + pick(Math.min(4, remaining));
      const actions = Array.from({ length: count }, () => action(
        STATES[pick(STATES.length)], refs(), pick(3) === 0 ? null : STATES[pick(STATES.length)],
        pick(3) === 0 ? null : refs(),
      ));
      steps.push(actions);
      remaining -= count;
    }
    return fixture(`generated-${String(index).padStart(2, '0')}`, seed, ...steps);
  });
}

/** Return fresh fixtures each time. Rejections stay in the corpus. */
export function cases() {
  const values = [...fixedCases(), ...branchCases(), ...generatedCases()];
  for (const value of values) validateCase(value);
  return values;
}
