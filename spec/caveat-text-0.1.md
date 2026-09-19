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
