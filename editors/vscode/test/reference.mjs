// The runtime's own rules, restated for checking the grammar against them:
// - character classes (code, string, comment) from spec/caveat-text-0.1.md;
// - statement spans as runtime/src/link.rs statement_spans finds them;
// - `$` substitution in `for` blocks as runtime/src/repeat.rs substitute does.

const identifierStart = ch => /[A-Za-z_]/.test(ch);
const identifierChar = ch => /[A-Za-z0-9_]/.test(ch);

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
    if (start === null && !/\s/.test(ch)) start = index;
  }
  return spans;
}

// A statement's code characters, with strings and comments blanked out.
export function codeOf(text, classes, [start, end]) {
  let out = '';
  for (let index = start; index < end; index++) out += classes[index] === 'code' ? text[index] : ' ';
  return out;
}

// Every `for KIND as $NAME { ... }` statement: its range up to the closing
// brace, where the binding is, and the body between the braces. Like
// repeat.rs, the body ends at the last `}` of the statement.
export function forBlocks(text, classes = classify(text)) {
  const blocks = [];
  for (const span of statementSpans(text, classes)) {
    const code = codeOf(text, classes, span);
    const header = /^\s*for\s+([A-Za-z_]\w*)\s+as\s+(\$([A-Za-z_]\w*))\s*\{/.exec(code);
    if (!header) continue;
    const open = span[0] + code.indexOf('{');
    const close = span[0] + code.lastIndexOf('}');
    blocks.push({
      kind: header[1],
      binding: header[3],
      bindingAt: span[0] + header.index + header[0].lastIndexOf(header[2]),
      start: span[0],
      end: close + 1,
      bodyStart: open + 1,
      bodyEnd: close,
    });
  }
  return blocks;
}

// Each `$` in a block body, comments and quoted text included: where it is,
// and how long the substituted part is (the `$` plus the longest bound name
// that prefixes the identifier run; 0 where the runtime refuses the name).
export function substitutions(text, block) {
  const found = [];
  for (let index = block.bodyStart; index < block.bodyEnd; index++) {
    if (text[index] !== '$') continue;
    let end = index + 1;
    if (end < text.length && identifierStart(text[end])) {
      end += 1;
      while (end < text.length && identifierChar(text[end])) end += 1;
    }
    const word = text.slice(index + 1, end);
    const bound = [block.binding, 'index'].filter(name => word.startsWith(name)).sort((a, b) => b.length - a.length)[0];
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
    if (/\s/.test(ch)) continue;
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
// the positions of the FROM names of relation statements.
export function relations(text, classes = classify(text)) {
  const found = new Set();
  const fromNames = new Set();
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
      fromNames.add(at(0));
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
    if (classes[index] === 'comment' || /\s/.test(ch)) {
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
