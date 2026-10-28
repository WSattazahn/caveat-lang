# Is this pull request done?

A thread used to judge by hand whether its pull request was ready to merge.
[`done.cav`](done.cav) now decides it, from what GitHub says about the pull
request, and keeps "ready" or "not ready" as a decision with what it rests on.

```
node scripts/done-check/done.mjs 327
node scripts/done-check/done.mjs 327 --explain   # what the verdict rests on
```

In a Claude session, run it with
`NODE_USE_ENV_PROXY=1 NODE_EXTRA_CA_CERTS=/root/.ccr/ca-bundle.crt` and no
`GITHUB_TOKEN`; elsewhere `GITHUB_TOKEN` is used when set.

| Fact | Event | Ready needs |
| --- | --- | --- |
| GitHub's comparison of main with the head | `head contains_main` | the head contains current main (nothing to merge in) |
| each check run on the head, the newest of its name | `check outcome, sweep` | none failed (cancelled and timed out count as failed), none still running, at least one |
| the files the PR changes | `files runtime` | where a file under `runtime/` or `spec/` changes, the corpus and save sweep (the `rc.15 compatibility and departure gates` job) passed on this head |
| PRs merged to main since the PR that last changed `ledger/events.jsonl` | `ledger owed` | nothing: owed events never block, but a ready verdict carries the `ledger_owed` caveat and the script names the PRs |

It is advisory: it exits 0 whatever the verdict, unless `--strict` is given.
Any later fact reopens the verdict, so a session kept open across a PR's life
decides again on what it now knows. `npm run test:done-check` runs the
program's scenarios and the host's tests.
