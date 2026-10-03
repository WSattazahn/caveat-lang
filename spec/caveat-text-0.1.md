# CAVEAT source text — 0.1

The Rust reference parser reads semicolon-separated statements. A semicolon inside
a double-quoted string is ordinary text. Strings support Unicode and preserve
spaces and line breaks; this applies to `scene`, `display`, and quoted evidence
provenance.

```caveat
# Mission briefing
scene "The beacon is silent; the sky is full of voices.";
display archive "The log says \"return\".\nTrust is provisional.";
evidence archive from "https://station.example/log;revision=2#signal";
claim investigate; // Keep the search open.
```

This example contains declarations only. To validate it with the reactive kit,
append `event inspect;`: a reactive program must declare an event. In
`display archive "…"`, `archive` names the declared evidence; leave that
name unquoted so the display text attaches to that symbol.

Outside strings, `#` and `//` start line comments. Comments end at a newline or the
end of the file and count as whitespace, so comments cannot merge adjacent tokens.
Quotes, semicolons, and other comment markers inside a comment have no effect.
Inside strings, `#` and `//` are literal text.

Supported escapes are `\"` (double quote), `\\` (backslash), `\n` (newline),
`\r` (carriage return), and `\t` (tab). Unknown escapes and incomplete strings are
errors. A literal backslash must be written as `\\`.

Diagnostics include one-based `line N, column M` source locations. Columns count
Unicode characters, with a tab counting as one character. Invalid statements
point to their first token; unterminated strings point to their opening quote;
unknown escapes point to their backslash. LF and CRLF source lines are supported.

For compatibility, the runtime continues to accept empty statements, an omitted
semicolon on the final statement, and unquoted evidence provenance. Semicolons
between statements are still required. Use quoted provenance when it contains
comment markers or when exact whitespace matters. These are lexical improvements;
the language's evidence, caveat, commitment, and reopening semantics are unchanged.

## Declared names

The statements of drafts 0.1 to 0.3 that declare a name take it as a word:
the name in `claim`, `evidence` and `caveat`, and the name and kind in
`place` and `entity`, with the place an `entity` is `at`. The loader splits
the statement at whitespace, with each comment read as whitespace, and takes
the word in that position whatever characters it holds: a whitespace-free
word, outside quoted text and comments, where a `;` ends the statement and
`#` or `//` begins a comment. `claim 1a;`, `evidence log{ from notes;`,
`place fi-eld kind fi.eld;` and `entity no-rth kind plot at fi-eld;` all
load. A relation, or an entity's `at`, names a declared name by the same
word. In the body of a `for` block, a brace in unquoted text, a name's
included, pairs only within its statement, so `evidence $p_log{ from notes;`
there refuses the block ([repetition](caveat-repetition-0.1.md) section 2).

The reactive layer requires an identifier wherever it reads a name: an ASCII
letter or `_`, then ASCII letters, digits and `_`, and not `true`, `false`,
`and`, `or` or `not`. That covers the names it declares, such as a state's
or an event's, the kind in an event parameter `P kind KIND`, and the names an
expression reads, such as `target.north`. A declared name that is not an
identifier loads where only the statements above use it, and not where the
reactive layer needs it. When a `for` block writes `$NAME` into a state name,
with `entity north{ kind plot at field;` as its member, `state $p_n` becomes
`state north{_n`, and the program does not load: "invalid reactive identifier
north{_n" ([repetition](caveat-repetition-0.1.md)).

`grammar/caveat-0.1.ebnf` gives the recommended form, `identifier`: a letter,
then letters, digits and `_`. Apart from the five words above, a name of that
form is also a reactive identifier. The loader does not check a declared name
against it.
