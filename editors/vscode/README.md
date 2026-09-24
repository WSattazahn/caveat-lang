# Caveat for Visual Studio Code

Syntax highlighting for [Caveat](https://github.com/WSattazahn/caveat-lang)
programs (`.cav` files). There is no language server: nothing is checked or
run, and the extension adds no commands.

## Install

The extension is not on the Visual Studio Marketplace. Install the packaged
`.vsix`:

```sh
code --install-extension caveat-0.1.0.vsix
```

or, in VS Code, open the Extensions view, choose **…** › **Install from
VSIX…**, and pick the file. To remove it:

```sh
code --uninstall-extension caveat-lang.caveat
```

CI builds and tests the `.vsix` on every push and pull request and keeps it
for 90 days as the `caveat-vscode-extension` artifact of the `runtime`
workflow run, with its `SHA256SUMS` and a report of the files inside. To
build it yourself, with Node 22 or later, from a repository checkout:

```sh
npm ci --prefix editors/vscode
npm test --prefix editors/vscode
npm run package --prefix editors/vscode
```

The package lands in `test-results/vscode/`.

## What is highlighted

- Comments (`#` and `//` to the end of the line) and quoted text, following
  [Caveat text 0.1](https://github.com/WSattazahn/caveat-lang/blob/main/spec/caveat-text-0.1.md):
  `;`, `#` and `//` inside quotes are text, the escapes are `\" \\ \n \r \t`,
  and any other escape is marked invalid.
- Keywords, by what they do: declarations (`state`, `evidence`, `bind`…),
  control (`on`, `when`, `for`…), effects (`set`, `reveal`, `commit`…),
  relations (`supports`, `opposes`, `qualifies`), clauses, booleans and
  logical operators.
- Words that mean something only in one place: stop reasons after `because`,
  consequence levels after `consequence`, cue kinds after `cue NAME`, `min`
  and `max` as bounds before a number (and as functions elsewhere), `every` in
  `clock`, `reset` in `control`, and an entity's `kind`, which is a name even
  when it is a word like `camera`.
- Statements of the core, sequential, map and presentation profiles
  (`action`, `choice`, `observe`, `position`, `camera`…) where they begin a
  statement, in a `for` body too. Reactive programs use several of these
  words as names (`event observe`, `retaining camera`), so elsewhere they are
  names, as they are when they begin a relation (`camera supports
  door_open`).
- Declared names, calls to built-in functions, other calls, numbers as the
  expression tokenizer reads them (`2`, `2.`, `.5`, `1e-3`), and names after
  a dot, which are always properties.
- Repetition: in a `for KIND as $NAME { … }` body, every substituted `$NAME`
  and `$index`, in code, quoted text and comments alike. A `$` outside a
  `for` body is marked invalid, since the runtime refuses it.

## How it is tested

The tests run the grammar through vscode-textmate and vscode-oniguruma, the
tokenizer and regular-expression engine VS Code itself uses.

- Every `.cav` file tracked in the repository, found with `git ls-files`, is
  compared character by character with a reference reading of the same text
  that follows the runtime: where strings and comments begin and end, which
  `$` names a `for` block substitutes and how much of each name, statement
  heads, relations, built-in calls, properties and numbers.
- The runtime's own word lists (`RESERVED` and `CALLABLE` in
  `runtime/src/link.rs`, `POSITIONAL` and `OUTSIDE_MODULES` in
  `runtime/tests/modules.rs`) must each be accounted for, so a keyword added
  to the language fails the tests until the grammar decides what it is. Each
  group of words is checked where it is a keyword and where it must stay a
  name.
- `test/fixtures/scopes.cav` asserts the scopes of the cases above, one caret
  line at a time.
- Packaging checks that the `.vsix` holds exactly the tested grammar,
  manifest, language configuration, README and license.
- `npm run test:editor` then checks the package in VS Code itself. It installs
  the `.vsix` into a fresh profile with VS Code's command line, opens every
  tracked `.cav` file and the fixture, and requires that each opens as Caveat
  and that VS Code's tokens match the tests' at every UTF-16 code unit, the
  unit VS Code counts positions in, so text beyond ASCII is compared too. It
  downloads the oldest VS Code the manifest supports, or another version named
  by `CAVEAT_VSCODE_VERSION`. To use an installed VS Code instead, set
  `CAVEAT_VSCODE` to its executable. On Linux without a display, run it under
  `xvfb-run -a`.

The grammar is written by `scripts/build-grammar.mjs`; edit that and run
`npm run build:grammar`. The tests fail if the committed JSON differs.

## Compatibility

The manifest declares VS Code 1.85 or later. CI runs the editor check in VS
Code 1.85.0 and in the current stable release. Other TextMate hosts are not
validated. GitHub Linguist in particular compiles grammars with a different
regular-expression engine, so this grammar needs its own validation there
before it could be proposed to Linguist.

## Limits

- A grammar cannot see which name a `for` block binds, so a highlighted
  template ends at the first character that is not a letter or digit:
  `$m_known` is `$m` followed by `_known`. The runtime substitutes the
  longest bound name instead. The two agree unless a bound name contains `_`,
  or letters or digits follow a shorter bound name directly (`$rx` with `$r`
  bound). The corpus test checks every tracked program agrees.
- A `for` header must be on one line, up to its `{`.
- A statement starts at the beginning of a line or after `;`, `{` or `}`. A
  statement that continues onto a line beginning with a profile word, such
  as `observe`, has that word read as a new statement.
- Highlighting is lexical. Apart from a stray `$` and an unknown escape, it
  does not report what the runtime would refuse.

## License

MIT, the same as the rest of the repository.
