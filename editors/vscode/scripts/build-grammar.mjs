// Writes syntaxes/caveat.tmLanguage.json. Edit this file, not the JSON:
// `npm test` fails when the committed grammar differs from what this builds.
//
// Where the words come from: runtime/src/link.rs lists the words the grammar
// reserves (RESERVED) and the callable primitives (CALLABLE);
// runtime/tests/modules.rs lists the words a module may declare (POSITIONAL)
// and those used only in statements a module cannot contain (OUTSIDE_MODULES).
// test/words.test.mjs reads those lists and fails on any word this grammar
// does not account for.
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const words = list => list.join('|');
const name = '[A-Za-z_][A-Za-z0-9_]*';
// Not a property (`toast.visible`, `.ring`) and not part of a longer name.
const notProperty = String.raw`(?<![.\w$])`;

// Reserved everywhere, so safe to highlight anywhere they stand alone.
const declarations = ['scene', 'claim', 'evidence', 'caveat', 'state', 'event', 'readings', 'decisions', 'renewable',
  'define', 'fn', 'proc', 'bind', 'cue', 'control', 'clock', 'budget', 'entity', 'display', 'module', 'use'];
const control = ['on', 'when', 'for', 'as', 'if', 'then', 'otherwise', 'when_committed'];
const effects = ['set', 'sample', 'reveal', 'examine', 'commit', 'reopen', 'emit', 'call', 'reject', 'qualify', 'renew', 'defer'];
const modifiers = ['from', 'limit', 'because', 'using', 'retaining', 'after', 'with', 'cost', 'consequence', 'kind',
  'at', 'to', 'via', 'through', 'toward'];
// Epistemic predicates, expression functions and callable primitives: names
// that are functions only where a call follows.
export const builtins = ['observed', 'committed', 'reopened', 'examined', 'has_caveat', 'caveated', 'carries', 'qualified',
  'latest', 'has_sample', 'history_at', 'history_count', 'fold_history', 'elapsed', 'require', 'text', 'abs', 'atan2',
  'ceil', 'floor', 'round', 'sqrt', 'sin', 'cos', 'clamp', 'wrap', 'number_text', 'percent_text', 'time_second_text',
  'time_total_text', 'time_text', 'min', 'max'];
// Statement heads of the core, sequential, map and presentation profiles.
// Several are ordinary names in reactive programs (`event observe`,
// `retaining camera`, `reopen route`), so they are keywords only where a
// statement starts with them, and not where they name the evidence of a
// relation (`camera supports door_open`).
export const profileHeads = ['action', 'choice', 'select', 'resolve', 'investigate', 'inspect', 'converge', 'infer',
  'require', 'observe', 'open', 'operate', 'move', 'stay', 'relate', 'place', 'connect', 'position', 'camera', 'route',
  'overview', 'start_at', 'program', 'origin'];
// Clauses and action steps that mean something only inside those statements.
const profileClauses = ['options', 'requires_open', 'steps'];
const actionSteps = ['inspect', 'operate', 'open', 'through', 'move', 'observe'];
const cueKinds = ['sound', 'toast', 'flash', 'ring'];

const comments = {
  patterns: [
    { name: 'comment.line.number-sign.caveat', match: String.raw`(#).*$`, captures: { 1: { name: 'punctuation.definition.comment.caveat' } } },
    { name: 'comment.line.double-slash.caveat', match: String.raw`(//).*$`, captures: { 1: { name: 'punctuation.definition.comment.caveat' } } },
  ],
};

const string = extra => ({
  name: 'string.quoted.double.caveat',
  begin: '"',
  beginCaptures: { 0: { name: 'punctuation.definition.string.begin.caveat' } },
  end: '"',
  endCaptures: { 0: { name: 'punctuation.definition.string.end.caveat' } },
  patterns: [{ include: '#string-escapes' }, ...extra],
});

const blockComment = (marker, scope) => ({
  name: scope,
  begin: `(${marker})`,
  beginCaptures: { 1: { name: 'punctuation.definition.comment.caveat' } },
  end: '$',
  patterns: [{ include: '#template' }],
});

const blockBody = [
  { include: '#block-comments' },
  { include: '#block-strings' },
  { include: '#template' },
  { include: '#nested-block' },
  { include: '#block-rule-statement' },
  { include: '#block-profile-statement' },
  { include: '#block-code' },
];

// What a rule refers to in each context: at the top level, or in a `for` body,
// where names may be built from templates and `$` is substituted.
const top = { comments: '#comments', strings: '#strings', dollar: '#stray-template', code: '#code', declared: name };
const block = {
  comments: '#block-comments',
  strings: '#block-strings',
  dollar: '#template',
  code: '#block-code',
  declared: String.raw`[A-Za-z_$][A-Za-z0-9_$]*`,
};

// Where a statement starts: a line, or after `;`, `{` or `}` on the same line.
const statementStart = String.raw`(?:^|(?<=[;{}]))\s*`;
const notRelation = String.raw`(?!\s+(?:supports|opposes|qualifies)\b)`;

// `rule NAME when A, B => C;` (core profile).
const ruleStatement = context => ({
  match: String.raw`${statementStart}(rule)${notRelation}\s+(${context.declared})`,
  captures: {
    1: { name: 'keyword.other.statement.caveat' },
    2: { name: 'entity.name.function.rule.caveat', patterns: [{ include: context.dollar }] },
  },
});

const profileStatement = context => ({
  begin: String.raw`${statementStart}(${words(profileHeads)})\b(?!\s*\()${notRelation}`,
  beginCaptures: { 1: { name: 'keyword.other.statement.caveat' } },
  end: '(?=[;}])',
  patterns: [
    { include: context.comments },
    { include: context.strings },
    { include: context.dollar },
    // `steps move a, observe b`: a step names what it acts on.
    {
      match: String.raw`${notProperty}(steps)\s+(${words(actionSteps)})(?=\s+[A-Za-z_$])`,
      captures: { 1: { name: 'keyword.other.caveat' }, 2: { name: 'keyword.other.statement.caveat' } },
    },
    {
      match: String.raw`(?<=,)\s*(${words(actionSteps)})(?=\s+[A-Za-z_$])`,
      captures: { 1: { name: 'keyword.other.statement.caveat' } },
    },
    { name: 'keyword.other.caveat', match: String.raw`${notProperty}(?:${words(profileClauses)})\b` },
    { include: context.code },
  ],
});

const code = declarations => ({
  patterns: [
    { include: declarations },
    { include: '#clause-values' },
    { include: '#bounds' },
    { include: '#builtin-calls' },
    { include: '#keywords' },
    { include: '#function-calls' },
    { include: '#numbers' },
    { include: '#operators' },
    { include: '#punctuation' },
    { include: '#properties' },
  ],
});

// The names declarations introduce. `$` inside a name is highlighted by
// `dollar`: a template in a `for` body, an error anywhere else.
function declarationNames(declared, dollar) {
  const named = scope => ({ name: scope, patterns: [{ include: dollar }] });
  return {
    patterns: [
      {
        match: String.raw`${notProperty}(fn|proc)\s+(${declared})`,
        captures: { 1: { name: 'storage.type.function.caveat' }, 2: named('entity.name.function.caveat') },
      },
      {
        match: String.raw`${notProperty}(clock)\s+(${declared})\s+(every)\b`,
        captures: { 1: { name: 'storage.type.caveat' }, 2: named('entity.name.function.event.caveat'), 3: { name: 'keyword.other.caveat' } },
      },
      {
        match: String.raw`${notProperty}(event)\s+(${declared})`,
        captures: { 1: { name: 'storage.type.caveat' }, 2: named('entity.name.function.event.caveat') },
      },
      {
        match: String.raw`${notProperty}(on)\s+(${declared})`,
        captures: { 1: { name: 'keyword.control.caveat' }, 2: named('entity.name.function.event.caveat') },
      },
      {
        match: String.raw`${notProperty}(define)\s+(${declared})`,
        captures: { 1: { name: 'storage.type.caveat' }, 2: named('variable.other.constant.caveat') },
      },
      {
        match: String.raw`${notProperty}(cue)\s+(${declared})\s+(${words(cueKinds)})\b`,
        captures: { 1: { name: 'storage.type.caveat' }, 2: named('entity.name.function.cue.caveat'), 3: { name: 'keyword.other.cue-kind.caveat' } },
      },
      // An entity's kind is an author-chosen tag, even when it is `camera`.
      {
        match: String.raw`${notProperty}(kind)\s+(${name})`,
        captures: { 1: { name: 'keyword.other.caveat' }, 2: { name: 'entity.name.type.kind.caveat' } },
      },
    ],
  };
}

export const grammar = {
  $schema: 'https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json',
  name: 'Caveat',
  scopeName: 'source.caveat',
  fileTypes: ['cav'],
  patterns: [
    { include: '#comments' },
    { include: '#for-block' },
    { include: '#strings' },
    { include: '#stray-template' },
    { include: '#rule-statement' },
    { include: '#profile-statement' },
    { include: '#code' },
  ],
  repository: {
    comments,
    strings: string([]),
    'string-escapes': {
      patterns: [
        { name: 'constant.character.escape.caveat', match: String.raw`\\["\\nrt]` },
        { name: 'invalid.illegal.escape.caveat', match: String.raw`\\.` },
      ],
    },
    // A `for` body is a template: every $NAME in it is substituted, inside
    // quoted text and comments too (runtime/src/repeat.rs).
    'block-strings': string([{ include: '#template' }]),
    'block-comments': {
      patterns: [
        blockComment('#', 'comment.line.number-sign.caveat'),
        blockComment('//', 'comment.line.double-slash.caveat'),
      ],
    },
    // The runtime substitutes the longest bound name that prefixes the run of
    // identifier characters after `$`. A grammar cannot see the block's bound
    // name, so the name ends at the first underscore: `$m_known` is `$m` then
    // `_known`. That is exact for bindings without an underscore.
    template: {
      name: 'variable.other.template.caveat',
      match: String.raw`\$(?:index|[A-Za-z][A-Za-z0-9]*)`,
    },
    // Outside a `for` body, $ is not part of any name.
    'stray-template': {
      name: 'invalid.illegal.template.caveat',
      match: String.raw`\$(?:${name})?`,
    },
    'for-block': {
      name: 'meta.block.repeat.caveat',
      begin: String.raw`${notProperty}(for)\s+(${name})\s+(as)\s+(\$${name})\s*(\{)`,
      beginCaptures: {
        1: { name: 'keyword.control.repeat.caveat' },
        2: { name: 'entity.name.type.kind.caveat' },
        3: { name: 'keyword.control.caveat' },
        4: { name: 'variable.parameter.template.caveat' },
        5: { name: 'punctuation.section.block.begin.caveat' },
      },
      end: String.raw`\}`,
      endCaptures: { 0: { name: 'punctuation.section.block.end.caveat' } },
      patterns: blockBody,
    },
    'nested-block': {
      begin: String.raw`\{`,
      beginCaptures: { 0: { name: 'punctuation.section.block.begin.caveat' } },
      end: String.raw`\}`,
      endCaptures: { 0: { name: 'punctuation.section.block.end.caveat' } },
      patterns: blockBody,
    },
    'rule-statement': ruleStatement(top),
    'block-rule-statement': ruleStatement(block),
    'profile-statement': profileStatement(top),
    'block-profile-statement': profileStatement(block),
    code: code('#declaration-names'),
    'block-code': code('#block-declaration-names'),
    'declaration-names': declarationNames(name, '#stray-template'),
    // In a `for` body a declared name may be built from templates:
    // `cue $r_ring ring $r ...`, `on absorb_$m ...`.
    'block-declaration-names': declarationNames(String.raw`[A-Za-z_$][A-Za-z0-9_$]*`, '#template'),
    // Closed words that mean something only after their clause keyword.
    'clause-values': {
      patterns: [
        {
          match: String.raw`${notProperty}(because)\s+(enough|budget|deadline)\b`,
          captures: { 1: { name: 'keyword.other.caveat' }, 2: { name: 'constant.language.stop-reason.caveat' } },
        },
        {
          match: String.raw`${notProperty}(because)\s+(nothing)\b`,
          captures: { 1: { name: 'keyword.other.caveat' }, 2: { name: 'constant.language.nothing.caveat' } },
        },
        {
          match: String.raw`${notProperty}(consequence)\s+(negligible|low|material|high|catastrophic)\b`,
          captures: { 1: { name: 'keyword.other.caveat' }, 2: { name: 'constant.language.consequence.caveat' } },
        },
        // `control NAME = EVENT reset;`
        { name: 'keyword.other.caveat', match: String.raw`${notProperty}reset\b(?=\s*;)` },
      ],
    },
    // `min 0 max 100` declares a range; elsewhere min and max are functions.
    bounds: {
      name: 'keyword.other.bound.caveat',
      match: String.raw`${notProperty}(?:min|max)\b(?=\s+-?\d)`,
    },
    'builtin-calls': {
      name: 'support.function.builtin.caveat',
      match: String.raw`${notProperty}(?:${words(builtins)})\b(?=\s*\()`,
    },
    keywords: {
      patterns: [
        { name: 'storage.type.caveat', match: String.raw`${notProperty}(?:${words(declarations)})\b` },
        { name: 'keyword.control.caveat', match: String.raw`${notProperty}(?:${words(control)})\b` },
        { name: 'keyword.other.effect.caveat', match: String.raw`${notProperty}(?:${words(effects)})\b` },
        { name: 'keyword.operator.relation.caveat', match: String.raw`${notProperty}(?:supports|opposes|qualifies)\b` },
        { name: 'keyword.operator.logical.caveat', match: String.raw`${notProperty}(?:and|or|not)\b` },
        { name: 'constant.language.boolean.caveat', match: String.raw`${notProperty}(?:true|false)\b` },
        { name: 'keyword.other.caveat', match: String.raw`${notProperty}(?:${words(modifiers)})\b` },
      ],
    },
    'function-calls': {
      name: 'entity.name.function.call.caveat',
      match: String.raw`${notProperty}${name}(?=\s*\()`,
    },
    // As runtime/src/reactive_expr.rs tokenizes them: `2`, `2.`, `.5`, `1e-3`.
    numbers: {
      name: 'constant.numeric.caveat',
      match: String.raw`(?<![\w$])(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?`,
    },
    operators: {
      patterns: [
        { name: 'keyword.operator.implication.caveat', match: '=>' },
        { name: 'keyword.operator.comparison.caveat', match: '==|!=|>=|<=|>|<' },
        { name: 'keyword.operator.assignment.caveat', match: '=' },
        { name: 'keyword.operator.arithmetic.caveat', match: String.raw`[+\-*/]` },
      ],
    },
    punctuation: {
      patterns: [
        { name: 'punctuation.terminator.statement.caveat', match: ';' },
        { name: 'punctuation.separator.comma.caveat', match: ',' },
        { name: 'punctuation.accessor.caveat', match: String.raw`\.` },
        { name: 'punctuation.section.parens.caveat', match: String.raw`[()]` },
      ],
    },
    // A name after a dot is a property of a presentation target.
    properties: {
      name: 'variable.other.property.caveat',
      match: String.raw`(?<=\.)${name}`,
    },
  },
};

export const grammarText = `${JSON.stringify(grammar, null, 2)}\n`;

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const target = new URL('../syntaxes/caveat.tmLanguage.json', import.meta.url);
  writeFileSync(target, grammarText);
  console.log(`wrote ${fileURLToPath(target)}`);
}
