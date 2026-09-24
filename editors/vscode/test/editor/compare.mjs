// Compares the tokens VS Code reports with the scopes the tests expect.
//
// VS Code reports each line's tokens as [text, scopes] pieces. Both it and
// vscode-textmate index a line by UTF-16 code unit, so 😀 is two positions,
// and so are the tests' per-position scopes. Counting code points instead
// would shift every position after the first character outside the BMP.

// The scopes of each code unit, from VS Code's pieces.
export function editorScopes(tokens) {
  const scopes = [];
  for (const [content, scope] of tokens) {
    for (let unit = 0; unit < content.length; unit++) scopes.push(scope);
  }
  return scopes;
}

// Where VS Code and the tests disagree about `text`, or null. `expected` holds
// the tests' scopes for every code unit of `text`, line breaks included.
export function difference(text, expected, tokens) {
  const lines = text.replace(/\n/g, '');
  const reported = tokens.map(([content]) => content).join('');
  if (reported !== lines) return { message: 'VS Code tokenized different text' };
  const wanted = expected.filter((_, index) => text[index] !== '\n').map(scopes => scopes.join(' '));
  const got = editorScopes(tokens);
  const at = wanted.findIndex((scopes, index) => scopes !== got[index]);
  if (at < 0) return null;
  return { at, message: `code unit ${at} is "${got[at]}" in VS Code, "${wanted[at]}" in the tests` };
}
