# Start an agent integration

Use Caveat when your application needs to keep a decision's evidence, carry its
caveats, and reconsider it while retaining the original grounds. Supplied inputs
and authored policies remain your responsibility; a Caveat result grants no
permission to act outside the host's authorization.

## Choose commands for the installed version

With Node 20 or later, try the published **0.1.0-rc.6** preview in an empty
directory:

```sh
npm init -y
npm install caveat-lang@0.1.0-rc.6
npx --no-install caveat init
npx --no-install caveat test umbrella.scenarios.json
npx --no-install caveat explain umbrella.cav events.jsonl
```

The two scenarios pass. The explanation retains `rain_chance@1` and its
`forecast_is_old` caveat as `umbrella@1`'s grounds after `sky` reopens it.
[Getting started](GETTING_STARTED.md) explains each file and command.

This documentation also covers **rc.7 release preparation**, whose npm publication is pending.
Once a verified rc.7 tarball is installed, use its unambiguous command name:

```sh
npx --no-install caveat-lang --version
npx --no-install caveat-lang doctor --json
npx --no-install caveat-lang demo agent
```

`doctor` checks the installed files and a synthetic runtime session and reports
PATH executable ownership. `demo agent` runs the shipped assessment program:
observe 85, assess, withdraw the reading after a correction, observe 92, assess
again. Both revisions remain inspectable and the original grounds stay frozen.
These are illustrative scores. The demo writes no files. rc.6 has neither
command; [CLI names](NAMES.md) explains the version and naming boundary.

## Build one small, testable integration

1. Read [Caveat on one page](REFERENCE.md) before inventing syntax. The
   [authoring guide](reference/docs/AI_AUTHORING.md) covers lifecycle and
   qualification details; specifications linked there define the contracts.
2. State required outcomes in [scenarios](reference/spec/caveat-scenarios-0.1.md).
   Include a refusal, a correction and an assertion that earlier grounds remain
   unchanged. `validate` answers whether source loads; `check` offers warnings;
   `test` checks the stated outcomes. They answer different questions.
3. Use `explain PROGRAM EVENTS` to inspect exact grounds and revision history.
   Use `dependents PROGRAM NAME EVENTS` to see what relies on evidence that
   changed. Each command supports `--json`; prefix it with the executable for
   your installed version as above.
4. Start host integration with the [official Python client and assessment
   program](../examples/agent-evidence/README.md). Its `CaveatServer` and `Attempt`
   separate request handling, accepted operations and the current assessment.
   An attempt succeeds only when all required operations succeed and a fresh
   assessment permits its result. An earlier approval cannot cover a refused
   observation. Preserve those checks when adapting the example.

Observing is separate from assessing. Reassessing requires reopening the current
revision. Withdrawal preserves the archive, and late qualification leaves prior
decision grounds and archived readings unchanged. The starter's
[qualification examples](../examples/agent-evidence/QUALIFICATION.md) show the
difference between archived readings and future values.

## When the integration needs something missing

Check the relevant guide and
[related issues](https://github.com/WSattazahn/caveat-lang/issues?q=is%3Aissue), then
prepare an [agent feature request](https://github.com/WSattazahn/caveat-lang/issues/new?template=agent-feature-request.md).
Include the exact package/build identity, minimal source and event history or
host call, command, expected and actual output, alternatives considered, and a
small acceptance check. Distinguish a missing capability from invalid input,
authored policy, a bug or host permissions. Identify what you could not verify.

Share only permitted, sanitized material. Follow the user's and host's posting
permissions; keep a local draft when posting is not authorized. The invitation
to report a need supplies a feedback route, not permission to publish it.
