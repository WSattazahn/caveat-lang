# Engineering readiness: an approval that keeps its reasons

<!-- caveat-package:identity -->
Package: **`caveat-lang@0.1.0-rc.17`**.
<!-- /caveat-package:identity -->

A release approval is only as good as the test result it rests on. This
example ties the approval of one exact revision to that revision's test
result, records later that the test ran under the wrong configuration, and
lets the program's own policy decide what that means. An unrelated decision
stays as it was. The approval's original grounds can still be read after a
restart and from the host's archive. Every input is illustrative.

- `readiness.cav`: the program.
- `readiness.scenarios.json`: four scenarios that state its expected behavior.
- `run.mjs`: the whole story through the [integration starter](../../docs/STARTER.md),
  with a restart, a refused event, a failed storage write and the archive.

## Run it

In a project with this package installed, copy the folder out of
`node_modules/caveat-lang/examples/readiness` and run:

```sh
npx --no-install caveat-lang check readiness.cav
npx --no-install caveat-lang test readiness.scenarios.json
node run.mjs
```

`run.mjs` writes its store to a new temporary directory and prints where.
Pass an empty directory to choose it, and `--json` for the facts it checked.

## What happens

1. `propose rev-a1b2c3`, then a passing `test_reported` for that revision.
   `approve_release` commits `release@1` because of `test_result`. The program
   refuses approval when the test result is for another revision or failed.
2. A dependency audit with no findings, and `sign_off_dependencies` commits
   `dependency_signoff@1` because of `dependency_audit`.
3. The host closes and reopens from its store. The approval is still in force.
4. `test_configuration_wrong` qualifies the test result with
   `wrong_configuration`. Qualification only records the caveat. What
   reopens the approval is the authored policy in `readiness.cav`:
   `reopen release because caveated(tests_passed, wrong_configuration)`.
   A caveat with no such rule reopens nothing; scenario P03 qualifies the audit
   with the low-consequence `mirror_lag`, and the sign-off stays in force.
5. `dependency_signoff@1` is unchanged, because nothing connects it to the
   test result.
6. Approving again on the same test result is refused, and the refused event
   changes nothing. A storage failure leaves the next event accepted but not
   durable until `flush()` stores it.
7. Ten more runs for `rev-d4e5f6` are new occurrences of the evidence,
   `test_result@2` to `test_result@11`, without the caveat. A passing one
   earns `release@2`. Runs that leave the window of eight go to the host's
   archive. `test_result` stays, because `release@1` rests on it.
8. After another restart, `explain` with the archive shows `release@1`,
   superseded, committed on `test_result` and reopened with
   `wrong_configuration`.

## Four identities

These are different things, and the example keeps them apart:

| Identity | Here | Where it is shown |
| --- | --- | --- |
| Application revision | `rev-a1b2c3`, `rev-d4e5f6` | An `id` payload the program compares |
| Caveat policy (source) | SHA-256 of `readiness.cav` | The host's `sourceSha256` and the store's checkpoint |
| Runtime | Runtime revision and WebAssembly SHA-256 | `runtime.identity`, recorded in the checkpoint |
| Evidence | `test_result`, `test_result@2`, … | The decision journal and `explain` |

The source digest identifies the program text. It authenticates nothing. A
save restores only under the exact source it was made under; the starter
refuses a store made under other source text.

## Limits

`test_configuration_wrong` qualifies the most recent test run, so report it
before a newer run arrives. The program decides whether a revision is ready;
taking the release is up to the application, and only while
`host.permits('release')` is true.
