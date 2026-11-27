# The npm release gate, in Caveat

`publish-npm.yml` publishes only the package that a successful Runtime run on
main tested for the release tag. Which run and which package that is used to
be decided by JavaScript in `scripts/release-publish.mjs`. It is now decided
by [`release-gate.cav`](release-gate.cav): the script fetches what GitHub says
about the Runtime runs and artifacts for the tagged revision and sends each
fact to the program as an event (`run`, `choose_run`, `artifact`, `decide`).
The program refuses, or commits `publish` resting on the newest run's reading
and its one package, and the script publishes nothing it did not allow.

- **Same answers as before.** The previous checks still run beside the
  program (`selectRuntimeCandidateLegacy`), and any difference between the two
  answers, refusal messages included, refuses the release.
  `release-gate.test.mjs` replays every release the workflow has published
  ([`../fixtures/release-gate-history.json`](../fixtures/release-gate-history.json),
  captured with `capture-history.mjs`), each of them with one thing changed,
  and 400 randomized run histories through both, and requires equal answers.
- **A record of why.** `select candidate` writes `candidate/release-gate/`
  (`events.jsonl` and `explain.txt`), which goes into the workflow's
  `npm-publish-candidate` artifact and is printed in the log. Replay it with
  `npx caveat explain scripts/release-gate/release-gate.cav candidate/release-gate/events.jsonl`.
- **Withdrawn runs.** `withdraw_run` marks the newest run as untrustworthy; a
  release resting on it is reopened and refused. The script never sends it
  today: it is there for a later check, and the scenarios test it.
- **Not judged by itself.** The workflow runs the program on a released kit
  pinned by SHA256 in [`pin.json`](pin.json), never on the runtime being
  released. Changing the pin is a verification change.

```
npm run test:release-gate
```
