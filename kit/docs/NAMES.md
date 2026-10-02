# Project names and CLI commands

**CAVEAT Language — Programs that remember why.** The npm package is
`caveat-lang`, a programming language for evidence-bearing values and decisions.

## Which command should I use?

Starting with **0.1.0-rc.7**, prefer `caveat-lang`. Both executable names point
to the same CLI, and `caveat` remains supported shorthand:

```sh
npx --no-install caveat-lang help
npx --no-install caveat-lang --version
npx --no-install caveat-lang init
npx --no-install caveat-lang test umbrella.scenarios.json
npx --no-install caveat-lang serve umbrella.cav
```

The frozen rc.7 candidate is
[available on GitHub](https://github.com/WSattazahn/caveat-lang/releases/tag/v0.1.0-rc.7)
and is not published on npm yet; these commands require its installed tarball.
This checkout is unpublished rc.8 development with the same command names. The published rc.6 has only `caveat`; use that spelling
with the [rc.6 installation instructions](GETTING_STARTED.md).

Other packages can also install an executable named `caveat`. Which one that
short command launches depends on the installation and PATH. `caveat-lang`
selects this language CLI when both `caveat-lang` and `caveat-cli` are installed
in the same npm project. Its help identifies **CAVEAT Language**, and
`--version` prints that name plus the installed package version.

The Python [agent-evidence caller](../examples/agent-evidence/README.md) keeps
its compatible default, `npx --no-install caveat`. For rc.7 in a shared npm
project, set its `CAVEAT_COMMAND` environment variable to this JSON array:

```json
["npx", "--no-install", "caveat-lang"]
```

On Windows, use `"npx.cmd"` instead of `"npx"` in that array, or give the
absolute path to the npx launcher.

## Is this the coding-agent memory tool?

No. CAVEAT Language (`caveat-lang`) is not affiliated with the
[`caveat-cli` coding-agent memory tool](https://github.com/kitepon/Caveat) or
other projects using the name CAVEAT. These projects serve different purposes.
The language's repository name, package name, `.cav` extension and syntax are
unchanged.
