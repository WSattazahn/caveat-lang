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
  'identifiers', 'define', 'fn', 'proc', 'bind', 'cue', 'control', 'clock', 'budget', 'entity', 'display', 'module', 'use'];
const control = ['on', 'when', 'for', 'as', 'if', 'then', 'otherwise', 'when_committed'];
const effects = ['set', 'sample', 'reveal', 'examine', 'commit', 'reopen', 'emit', 'call', 'reject', 'qualify', 'renew', 'withdraw', 'defer'];
const modifiers = ['from', 'limit', 'because', 'using', 'retaining', 'after', 'with', 'cost', 'consequence', 'kind',
  'id', 'permitted', 'by', 'at', 'to', 'via', 'through', 'toward'];
// Epistemic predicates, expression functions and callable primitives: names
// that are functions only where a call follows.
export const builtins = ['observed', 'committed', 'reopened', 'examined', 'has_caveat', 'caveated', 'carries', 'qualified',
  'latest', 'has_sample', 'history_at', 'history_count', 'fold_history', 'elapsed', 'require', 'text', 'abs', 'atan2',
  'ceil', 'floor', 'round', 'sqrt', 'sin', 'cos', 'clamp', 'wrap', 'number_text', 'percent_text', 'time_second_text',
  'time_total_text', 'time_text', 'min', 'max', 'id_text', 'withdrawn', 'rests_on_withdrawn',
  'permission_withdrawn'];
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

// Statement rules come before the template rule, so a statement that starts
// with a template (`$b_sounding supports charted_$b;`) is still read whole.
const blockBody = [
  { include: '#block-comments' },
  { include: '#block-strings' },
  { include: '#block-relation-statement' },
  { include: '#block-rule-statement' },
  { include: '#block-profile-statement' },
  { include: '#template' },
  { include: '#nested-block' },
  { include: '#block-code' },
];
// What a rule refers to in each context: at the top level, or in a `for` body,
// where names may be built from templates and `$` is substituted. `referred`
// is a name a statement refers to, which a module may qualify (`glow::lamp`).
const blockName = String.raw`[A-Za-z_$][A-Za-z0-9_$]*`;
const top = {
  comments: '#comments',
  strings: '#strings',
  dollar: '#stray-template',
  code: '#code',
  declarations: '#declaration-names',
  relations: '#relation-effects',
  declared: name,
  referred: String.raw`${name}(?:::${name})*`,
};
const block = {
  comments: '#block-comments',
  strings: '#block-strings',
  dollar: '#template',
  code: '#block-code',
  declarations: '#block-declaration-names',
  relations: '#block-relation-effects',
  declared: blockName,
  referred: String.raw`${blockName}(?:::${blockName})*`,
};

// Where a statement starts: a line, or after `;`, `{` or `}` on the same line.
const statementStart = String.raw`(?:^|(?<=[;{}]))\s*`;

// Relations, in the forms the parsers accept (runtime/src/parser.rs and the
// effects in runtime/src/reactive.rs). Elsewhere the three words are names:
// `claim qualifies;`, `place supports kind dock;`. The parsers split words
// on any whitespace, so a relation may be wrapped onto following lines, and
// a program's last statement may omit its `;`. TextMate matches one line at
// a time, so the wrapped forms are regions that carry the context across.
const relationWords = String.raw`(?:supports|opposes|qualifies)\b`;
const evidential = String.raw`(?:supports|opposes)\b`;
const relation = () => ({ name: 'keyword.operator.relation.caveat' });
const reference = context => ({ patterns: [{ include: context.dollar }] });
// Where a line's code ends: at the line's end, or where a comment begins.
const lineEnd = String.raw`(?:$|#|//)`;
// A relation word followed by its one last name and the end of the statement,
// or by the end of the line's code, where what follows cannot be seen.
const relationAt = (context, relations) => ({
  match: String.raw`${notProperty}${relations}(?=\s+${context.referred}\s*(?:[;}]|${lineEnd})|\s*${lineEnd})`,
  ...relation(),
});

// First words the parsers always read as a statement of their own
// (parser.rs dispatches on `reveal` and `when_committed`), which must keep
// their region. `sample` is not one: a caveat may be named `sample`. Alone
// on its line it is left to the sample region, which reads what follows
// either way; followed by a relation word it may begin a wrapped relation.
const notRelationSource = String.raw`(?:${words([...profileHeads, 'reveal', 'when_committed'])})\b|sample\s*${lineEnd}`;

// `FROM supports|opposes|qualifies TO;`, the whole statement.
function relationStatement(context) {
  const n = context.referred;
  return {
    patterns: [
      // On one line, whatever FROM is: `camera supports door_open;`.
      {
        match: String.raw`${statementStart}(${n})\s+(${relationWords})\s+(${n})(?=\s*(?:;|${lineEnd}))`,
        captures: { 1: reference(context), 2: relation(), 3: reference(context) },
      },
      // Wrapped: FROM ends its line, or the relation does. FROM may be any
      // name a program declares, keywords included (`evidence place from
      // "sensor";`), so it keeps the color it has on its own. A profile word
      // is left to its statement, which also finds a wrapped relation, and
      // so are the words that open an effect region (`reveal` alone on its
      // line).
      {
        begin: String.raw`${statementStart}(?!${notRelationSource})(${n})(?=\s+${relationWords}\s*${lineEnd}|\s*${lineEnd})`,
        beginCaptures: { 1: { patterns: [{ include: context.dollar }, { include: context.code }] } },
        end: String.raw`(?=\S)`,
        applyEndPatternLast: true,
        patterns: [
          { include: context.comments },
          relationAt(context, relationWords),
        ],
      },
    ],
  };
}

// Statements and effects whose relation may sit on any of their lines: each is
// a region from its keyword to the end of the statement.
function relationEffects(context) {
  const region = (begin, scope, relations) => ({
    begin,
    beginCaptures: { 1: { name: scope } },
    end: '(?=[;}])',
    patterns: [
      { include: context.comments },
      { include: context.strings },
      { include: context.dollar },
      relationAt(context, relations),
      { include: context.code },
    ],
  });
  return {
    patterns: [
      // `reveal CAVEAT then FROM REL TO` (core), `reveal EVIDENCE REL CLAIM`.
      region(String.raw`${notProperty}(reveal)\b`, 'keyword.other.effect.caveat', evidential),
      // `when_committed ACTION FROM REL TO`
      region(String.raw`${statementStart}(when_committed)\b`, 'keyword.control.caveat', evidential),
      // `sample STREAM = EXPRESSION REL CLAIM`
      region(String.raw`${notProperty}(sample)\b(?=\s+${context.referred}\s*=)`, 'keyword.other.effect.caveat', evidential),
      // `sample` ending its line: the next line decides between that effect
      // and a relation from a caveat or evidence named `sample`
      // (`caveat sample consequence high;`), so any relation word counts.
      region(String.raw`${notProperty}(sample)\b(?=\s*${lineEnd})`, 'keyword.other.effect.caveat', relationWords),
    ],
  };
}

// `rule NAME when A, B => C;` (core profile).
const ruleStatement = context => ({
  match: String.raw`${statementStart}(rule)\s+(${context.declared})`,
  captures: {
    1: { name: 'keyword.other.statement.caveat' },
    2: { name: 'entity.name.function.rule.caveat', patterns: [{ include: context.dollar }] },
  },
});

const profileStatement = context => ({
  begin: String.raw`${statementStart}(${words(profileHeads)})\b(?!\s*\()`,
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
    // A profile word may also name evidence: `place` on one line, then
    // `supports c;` on the next.
    relationAt(context, relationWords),
    { include: context.code },
  ],
});

const code = context => ({
  patterns: [
    { include: context.declarations },
    { include: context.relations },
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
    { include: '#relation-statement' },
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
    'relation-statement': relationStatement(top),
    'block-relation-statement': relationStatement(block),
    'relation-effects': relationEffects(top),
    'block-relation-effects': relationEffects(block),
    'rule-statement': ruleStatement(top),
    'block-rule-statement': ruleStatement(block),
    'profile-statement': profileStatement(top),
    'block-profile-statement': profileStatement(block),
    code: code(top),
    'block-code': code(block),
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
