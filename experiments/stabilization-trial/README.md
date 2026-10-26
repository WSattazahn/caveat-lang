# Stabilization trial

The predefined trial [Path to 1.0](../../docs/releases/path-to-1.0.md) asks
for in stabilization (E2), as the
[rc.19 plan](../../docs/releases/rc19-development.md) describes it (S3). An
installed `caveat-lang` drives [`trial.cav`](trial.cav), a deploy gate,
through the integration starter with a file store:

| Step | What it shows |
| --- | --- |
| `policy`, `input_*`, `evaluation_*`, `limit_identifier_limit` | A refusal from each rejection origin. Each one leaves the sequence and the view unchanged |
| `decision` | A deploy in force on a passing CI run, permitted by the current assessment |
| `restart` | A new host resumes from the store at the same sequence and assessment |
| `doubt`, `policy_in_doubt` | A doubt on the runner reopens the deploy under the program's authored policy; deploying again is refused |
| `take_back` | `unqualify` takes the doubt back; a new deploy rests on the same run without the caveat |
| `history` | After more runs than the window holds and a second restart, `explain` over the snapshot and the kept archive shows `deploy@1` committed, then reopened with `flaky_runner`; the take-back is recorded |
| `edited_source` | The store refuses to resume under edited source, even a comment |

The trial's host budgets are `BUDGETS` in [`run.mjs`](run.mjs): the slowest
send, the checkpoint size and the archive size. They are the budgets a host
sets for itself, as COMPATIBILITY.md's "Limits" asks; they are not promises
of the package.

The send budget is wall-clock time, and a shared CI runner can stall one
synced write, so the whole trial runs again in a fresh store, up to three
times, and passes when one trial keeps every send within budget. A real
slowdown is slow in every trial and still fails.

```sh
node experiments/stabilization-trial/run.mjs --package=node_modules/caveat-lang
```

CI runs it from the packed tarball's fresh install in
`scripts/test-kit-package.mjs`.

## What it found

Run against the published `caveat-lang@0.1.0-rc.18`, the second restart
failed: restore refused the store's save with "reopening cites caveats
unrelated to its witnesses". A reopening recorded because of a caveat, followed
by a take-back of that caveat, left a save that no runtime would restore,
though Take-back 0.1 leaves the journal as it was. rc.19 fixes the restore
check (`runtime/src/reactive_save.rs`), with the regression
`stabilization_trial_a_take_back_after_a_caveated_reopening_restores` in
`runtime/tests/take_back.rs`.
