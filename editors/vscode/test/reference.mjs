// The runtime's own rules, restated for checking the grammar against them:
// - character classes (code, string, comment) from spec/caveat-text-0.1.md;
// - whitespace as Rust's char::is_whitespace reads it, which the runtime
//   splits words at and skips between statements;
// - statement spans as runtime/src/link.rs statement_spans finds them, and
//   statements as link.rs statements_of reads them, a last statement without
//   its `;` included, which is how runtime/src/repeat.rs reads them;
// - `$` substitution in `for` blocks as runtime/src/repeat.rs substitute does,
//   with `$Q` in a routed rule whose code or quoted text names a member by Q
//   (spec/caveat-routed-repetition-0.1.md section 10), and the events a
//   routed block reads, a `for` block's own once expanded, as repeat.rs
//   Part::events reads them.

const identifierStart = ch => /[A-Za-z_]/.test(ch);
const identifierChar = ch => /[A-Za-z0-9_]/.test(ch);

// Rust's whitespace, the Unicode White_Space property, which link.rs,
// repeat.rs and parser.rs read with char::is_whitespace, split_whitespace and
// trim. JavaScript's \s, split(/\s+/) and trim() read another set: U+0085 is
// whitespace to Rust only, and U+FEFF to JavaScript only. So the reference
// uses no \s. fixtures/substitutions.json lists the set, checked against Rust.
const spaces = String.raw`\t\n\v\f\r \u0085\u00a0\u1680\u2000-\u200a\u2028\u2029\u202f\u205f\u3000`;
const space = `[${spaces}]`;
const oneSpace = new RegExp(`^${space}$`);
const spaceRun = new RegExp(`${space}+`);
export const isWhitespace = ch => oneSpace.test(ch);
// The words of a text, split at whitespace, as Rust's split_whitespace splits
// them: none empty.
const wordsOf = text => text.split(spaceRun).filter(Boolean);
// A statement without the `;` that ends it (repeat.rs without_terminator).
const withoutTerminator = code => (code.endsWith(';') ? code.slice(0, -1) : code);

// One class per character: 'code', 'string' or 'comment'. A string runs from
// its opening quote to its closing quote (a backslash takes the next character,
// line breaks included); a comment runs from `#` or `//` to the line end.
export function classify(text) {
  const classes = new Array(text.length).fill('code');
  for (let index = 0; index < text.length;) {
    const ch = text[index];
    if (ch === '"') {
      let end = index + 1;
      while (end < text.length && text[end] !== '"') end += text[end] === '\\' ? 2 : 1;
      end = Math.min(end + 1, text.length);
      classes.fill('string', index, end);
      index = end;
    } else if (ch === '#' || (ch === '/' && text[index + 1] === '/')) {
      let end = index;
      while (end < text.length && text[end] !== '\n') end += 1;
      classes.fill('comment', index, end);
      index = end;
    } else {
      index += 1;
    }
  }
  return classes;
}

// [start, end) of each statement, ending at a `;` outside strings, comments
// and braces.
export function statementSpans(text, classes = classify(text)) {
  const spans = [];
  let start = null;
  let braces = 0;
  for (let index = 0; index < text.length; index++) {
    const ch = text[index];
    if (classes[index] === 'string') { if (start === null) start = index; continue; }
    if (classes[index] === 'comment') continue;
    if (ch === '{') braces += 1;
    else if (ch === '}') braces = Math.max(0, braces - 1);
    if (ch === ';' && braces === 0) {
      if (start !== null) spans.push([start, index + 1]);
      start = null;
      continue;
    }
    if (start === null && !isWhitespace(ch)) start = index;
  }
  return spans;
}

// The statement spans, and what follows the last of them, from its first
// character that is not whitespace, a comment or a `;`, to the end of the
// text, when there is one: a last statement without its `;`, which the loader
// reads too (link.rs statements_of and tail).
export function statementsOf(text, classes = classify(text)) {
  const spans = statementSpans(text, classes);
  for (let index = spans.at(-1)?.[1] ?? 0; index < text.length; index++) {
    if (classes[index] === 'comment' || text[index] === ';' || isWhitespace(text[index])) continue;
    spans.push([index, text.length]);
    break;
  }
  return spans;
}

// A statement's code characters, with strings and comments blanked out.
export function codeOf(text, classes, [start, end]) {
  let out = '';
  for (let index = start; index < end; index++) out += classes[index] === 'code' ? text[index] : ' ';
  return out;
}

// A `for` block's header, up to the `{` that opens its body: its KIND, its
// binding with and without the `$`, and P or nothing.
const blockHeader = new RegExp(String.raw`^${space}*for${space}+([A-Za-z_]\w*)${space}+as${space}+(\$([A-Za-z_]\w*))`
  + String.raw`(?:${space}+routed${space}+by${space}+([A-Za-z_]\w*))?${space}*\{`);

// Every `for KIND as $NAME { ... }` statement, routed by P or not, a last one
// without its `;` too, as repeat.rs expand finds them: its range up to the
// closing brace, where the binding is, P or null, and the body between the
// braces. Like repeat.rs, the body ends at the last `}` of the statement.
export function forBlocks(text, classes = classify(text)) {
  const blocks = [];
  for (const span of statementsOf(text, classes)) {
    const code = codeOf(text, classes, span);
    const header = blockHeader.exec(code);
    if (!header) continue;
    const open = span[0] + code.indexOf('{');
    const close = span[0] + code.lastIndexOf('}');
    blocks.push({
      kind: header[1],
      binding: header[3],
      route: header[4] ?? null,
      bindingAt: span[0] + header.index + header[0].lastIndexOf(header[2]),
      start: span[0],
      end: close + 1,
      bodyStart: open + 1,
      bodyEnd: close,
    });
  }
  return blocks;
}

const longestPrefix = (word, names) => names.filter(name => word.startsWith(name)).sort((a, b) => b.length - a.length)[0];

// A `$`, and the name after it, if any: a `$` word as repeat.rs reads one.
const dollarWord = /\$([A-Za-z_][A-Za-z0-9_]*)?/g;

// `text` with each `$` word replaced as repeat.rs substitute replaces it, by
// the longest name in `bindings` (a Map of name to value) that begins it, the
// rest of the word kept; null when a word begins with none of them.
function substitute(text, bindings) {
  let unbound = false;
  const out = text.replace(dollarWord, (written, word = '') => {
    const name = longestPrefix(word, [...bindings.keys()]);
    if (name === undefined) {
      unbound = true;
      return written;
    }
    return bindings.get(name) + word.slice(name.length);
  });
  return unbound ? null : out;
}

// The members of each kind, in declaration order: every top-level
// `entity NAME kind KIND at PLACE` statement, a last one without its `;` too
// (repeat.rs entity_kinds).
function entityKinds(text, classes = classify(text)) {
  const kinds = new Map();
  for (const span of statementsOf(text, classes)) {
    const words = wordsOf(withoutTerminator(codeOf(text, classes, span)));
    if (words.length !== 6 || words[0] !== 'entity' || words[2] !== 'kind' || words[4] !== 'at') continue;
    if (!kinds.has(words[3])) kinds.set(words[3], []);
    kinds.get(words[3]).push(words[1]);
  }
  return kinds;
}

// Each `event` statement's name and its `kind` parameters, as [name, kind]
// pairs, the first declaration of a name only: the top-level ones, and those
// a `for` block declares once expanded, in the order the part declares them
// (repeat.rs Part::events). The part and a block's body are read as
// statementsOf reads them, so a last statement without its `;` counts, a
// last block too. A block's copies are its statements other than its `on`
// rules, each member's with `$NAME` and `$index` substituted. A block of a
// kind with no member, with a nested `for`, or with a `$` word in those
// statements that neither binds, declares none.
export function eventKinds(text, classes = classify(text)) {
  const events = new Map();
  // As repeat.rs event_declaration reads a statement: its words, the first
  // `event`, then the name, then the parameters, split at commas.
  const declare = code => {
    const [first, name, ...rest] = wordsOf(withoutTerminator(code));
    if (first !== 'event' || name === undefined || events.has(name)) return;
    events.set(name, rest.join(' ').split(',')
      .map(wordsOf)
      .filter(words => words.length === 3 && words[1] === 'kind')
      .map(([parameter, , kind]) => [parameter, kind]));
  };
  const members = entityKinds(text, classes);
  for (const span of statementsOf(text, classes)) {
    const code = codeOf(text, classes, span);
    const header = blockHeader.exec(code);
    if (!header) {
      declare(code);
      continue;
    }
    const body = text.slice(span[0] + code.indexOf('{') + 1, span[0] + code.lastIndexOf('}'));
    const bodyClasses = classify(body);
    const statements = statementsOf(body, bodyClasses).map(statement => ({
      text: body.slice(...statement),
      first: wordsOf(withoutTerminator(codeOf(body, bodyClasses, statement)))[0],
    }));
    if (statements.some(({ first }) => first === 'for')) continue;
    const declarations = statements.filter(({ first }) => first !== 'on').map(statement => statement.text);
    const copies = (members.get(header[1]) ?? []).map((member, position) => declarations.map(declaration =>
      substitute(declaration, new Map([[header[3], member], ['index', String(position + 1)]]))));
    if (copies.length === 0 || copies.flat().includes(null)) continue;
    for (const copy of copies.flat()) declare(codeOf(copy, classify(copy), [0, copy.length]));
  }
  return events;
}

// In a routed block, each routed rule's range in the text, and the `kind`
// parameter Q its code or quoted text names a member by, or null: an `on
// EVENT` statement of the body, EVENT written without `$`, whose event
// declares `P kind KIND`. Q is read as repeat.rs parameter_rules reads it,
// from the rule with its comments blanked and its quoted text kept: the
// longest `kind` parameter of the event that begins a `$` word neither
// `$NAME` nor `$index` begins, when it is not P and the rule names a member
// by no other.
function routedRules(text, block) {
  if (!block.route) return [];
  const events = eventKinds(text);
  const body = text.slice(block.bodyStart, block.bodyEnd);
  const classes = classify(body);
  const rules = [];
  for (const [start, end] of statementsOf(body, classes)) {
    const [first, event] = wordsOf(withoutTerminator(codeOf(body, classes, [start, end])));
    const parameters = (first === 'on' && events.get(event)) || [];
    if (!parameters.some(([name, kind]) => name === block.route && kind === block.kind)) continue;
    let uncommented = '';
    for (let index = start; index < end; index++) uncommented += classes[index] === 'comment' ? ' ' : body[index];
    const named = new Set();
    for (const [, word = ''] of uncommented.matchAll(dollarWord)) {
      if (longestPrefix(word, [block.binding, 'index'])) continue;
      const parameter = longestPrefix(word, parameters.map(([name]) => name));
      if (parameter !== undefined && parameter !== block.route) named.add(parameter);
    }
    rules.push({
      start: block.bodyStart + start,
      end: block.bodyStart + end,
      parameter: named.size === 1 ? [...named][0] : null,
    });
  }
  return rules;
}

// Each `$` in a block body, comments and quoted text included: where it is,
// and how long the substituted part is (the `$` plus the longest bound name
// that prefixes the identifier run; 0 where the runtime refuses the name).
// The names bound are the block's own and, in a routed rule whose code or
// quoted text names a member by Q, Q, in its comments too (section 10).
export function substitutions(text, block) {
  const rules = routedRules(text, block);
  const found = [];
  for (let index = block.bodyStart; index < block.bodyEnd; index++) {
    if (text[index] !== '$') continue;
    let end = index + 1;
    if (end < text.length && identifierStart(text[end])) {
      end += 1;
      while (end < text.length && identifierChar(text[end])) end += 1;
    }
    const word = text.slice(index + 1, end);
    const rule = rules.find(({ start, end: ruleEnd }) => index >= start && index < ruleEnd);
    const bound = longestPrefix(word, [block.binding, 'index', ...(rule?.parameter ? [rule.parameter] : [])]);
    found.push({ index, length: bound ? bound.length + 1 : 0, word });
  }
  return found;
}

// The first word of every statement, at any depth: after the start of the
// text, a `;`, `{` or `}` in code.
export function statementHeads(text, classes = classify(text)) {
  const heads = [];
  let expectHead = true;
  for (let index = 0; index < text.length; index++) {
    if (classes[index] === 'comment') continue;
    if (classes[index] === 'string') { expectHead = false; continue; }
    const ch = text[index];
    if (ch === ';' || ch === '{' || ch === '}') { expectHead = true; continue; }
    if (isWhitespace(ch)) continue;
    if (expectHead && identifierStart(ch)) {
      let end = index + 1;
      while (end < text.length && identifierChar(text[end])) end += 1;
      heads.push({ index, word: text.slice(index, end) });
    }
    expectHead = false;
  }
  return heads;
}

// Where a relation word is a relation, in the forms the parsers accept: the
// statement `FROM REL TO;` (runtime/src/parser.rs), `reveal CAVEAT then FROM
// REL TO` and `when_committed ACTION FROM REL TO` (parser.rs), and the
// effects `reveal EVIDENCE REL CLAIM` and `sample STREAM = EXPRESSION REL
// CLAIM` (runtime/src/reactive.rs). Words are split on any whitespace, line
// breaks included, as the parsers split them, within each piece of code
// between `;`, `{` and `}`; the last statement may omit its `;`
// (scan_statements in parser.rs). Returns the relation words' positions, and
// for each relation statement, where its FROM name starts and its TO name ends.
export function relations(text, classes = classify(text)) {
  const found = new Set();
  const fromNames = new Map();
  const any = new Set(['supports', 'opposes', 'qualifies']);
  const evidential = new Set(['supports', 'opposes']);
  let words = [];
  let word = null;
  const endWord = () => {
    if (word) words.push(word);
    word = null;
  };
  const endPiece = terminator => {
    endWord();
    const w = words.map(entry => entry.text);
    const at = position => words[position].index;
    const n = w.length;
    const statementEnd = terminator === ';' || terminator === null;
    if (n === 3 && any.has(w[1]) && statementEnd) {
      found.add(at(1));
      fromNames.set(at(0), at(2) + w[2].length);
    }
    if (n === 6 && w[0] === 'reveal' && w[2] === 'then' && evidential.has(w[4])) found.add(at(4));
    if (n === 5 && w[0] === 'when_committed' && evidential.has(w[3]) && statementEnd) found.add(at(3));
    for (let i = 0; i < n; i++) {
      if (w[i] === 'reveal' && n === i + 4 && evidential.has(w[i + 2])) found.add(at(i + 2));
      if (w[i] === 'sample' && w[i + 2] === '=' && n - (i + 3) >= 3 && evidential.has(w[n - 2])) found.add(at(n - 2));
    }
    words = [];
  };
  for (let index = 0; index < text.length; index++) {
    const ch = text[index];
    if (classes[index] === 'comment' || isWhitespace(ch)) {
      endWord();
    } else if (classes[index] === 'code' && ';{}'.includes(ch)) {
      endPiece(ch);
    } else {
      word ??= { index, text: '' };
      word.text += ch;
    }
  }
  endPiece(null);
  return { found, fromNames };
}

// Every identifier in code: where it starts, the word, and the character
// before it.
export function identifiers(text, classes = classify(text)) {
  const found = [];
  for (let index = 0; index < text.length; index++) {
    if (classes[index] !== 'code' || !identifierStart(text[index])) continue;
    if (index > 0 && (identifierChar(text[index - 1]) || text[index - 1] === '$')) continue;
    let end = index + 1;
    while (end < text.length && identifierChar(text[end])) end += 1;
    found.push({ index, word: text.slice(index, end), before: index > 0 ? text[index - 1] : '' });
    index = end - 1;
  }
  return found;
}

// Numeric literals in code, as runtime/src/reactive_expr.rs tokenize reads
// them: digits with an optional fraction, or a fraction alone, then an
// optional exponent. Digits that continue a name or follow `$` are not one.
export function numbers(text, classes = classify(text)) {
  const found = [];
  const pattern = /(?<![\w$])(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?/g;
  for (const match of text.matchAll(pattern)) {
    if (classes[match.index] === 'code') found.push({ index: match.index, length: match[0].length });
  }
  return found;
}
