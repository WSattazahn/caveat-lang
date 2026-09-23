# Trail Rescue implementation record

Trail Rescue is a new mechanic authored in Caveat after its
[protocol](PROTOCOL.md) and [24 scenarios](scenarios.json) were registered in
commit `a03af45`. The requirements author had not read the runtime or game
implementations. The author did know the project's overview; this is not a
blind-author comparison, and there is no competing implementation or winner.
The registered requirements were not amended.

## What is playable

[The program](../../game/trail_rescue.cav) owns the hidden physical conditions,
three-token scouting budget, two fallible reports, all observations, thirty-second
aging, disputed assessments, frozen route plans, reopening, and rescue outcomes.
[The page](../../web/trail-rescue.html) provides the controls, ordered notebook
and decision history, local save/resume, and optional hidden-condition controls.

The [shared adapter](../../web/trail-rescue-policy.js) validates and translates
the input envelope, then projects source bindings and runtime explanations.
It has no resource, freshness, route eligibility or outcome calculation. It
translates the protocol's `tick` into the source-declared `advance`, preserving
one binary64 addition; the runtime reserves the literal event name `tick` for
steps no larger than 0.1 seconds. Projection orders set-valued grounds by the
source's observation ordinal and uses the journal's order for decisions.

Run `npm run build`, `npm run serve`, and open
`http://127.0.0.1:4173/trail-rescue.html`.

## Language and runtime improvements

- [`has_caveat(STATE, CAVEAT)`](../../spec/caveat-state-caveats-0.1.md) checks
  the grounds a state actually rests on. A caveat present only in a control
  guard's lineage cannot make it true. Bindings update when grounds change,
  even if the state's number and the graph do not.
- `reopen ACTION because caveated(STATE, CAVEAT)` cites the exact observed
  evidence occurrences in those grounds that currently carry the caveat.
  It handles older occurrences after renewal and groups simultaneous causes
  into one ordered journal entry. An empty selection rejects atomically.
- [Journal entries](../../spec/caveat-decision-journal-0.1.md) preserve the
  source clock's elapsed time and the commitment's frozen numeric value. A
  host no longer needs a parallel history to display when and what was chosen.
  These fields remain optional for older saves.
- [Restore](../../spec/caveat-save-0.1.md) checks journal records against
  commitments, frozen grounds, revision order, event/sequence relationships,
  observed witnesses, caveats, and reopening graph edges. Coordinated edits
  to an unsigned save can still be accepted: consistency is not authenticity.
- A custom clock that allows negative time can now restore a save made below
  zero. Previously the runtime accepted the event and save, then refused to
  restore it. Nonfinite time, and negative time on a forward-only clock,
  remain invalid.

Module linking, procedure specialization, dependency tracking, native execution,
JSON save/restore, the WASM bridge, and browser rendering all exercise these
changes. Game-specific rules were not added to the interpreter.

## Verification record

The [first full scenario run](evidence/first-run.json) passed all 24 scenarios
(162 registered steps). Its invariant run used seed `0x7a11beef`, 200 sequences
of 80 events: **16,000 events**, 7,325 accepted and 8,675 rejected, with
**3,110 restores**. Restored sessions were compared against uninterrupted
shadows. Rejections preserved both the complete view and complete saved state;
every array was compared in order.

Additional checks cover non-JSON inputs, hidden-condition leakage, immutable
history, observation identity, budget limits, and source-only changes to the
budget, aging delay, report caveat and physical outcomes. Final verification
also exercises a negative custom clock and its pending caveat timer through
the actual WASM adapter. The results are retained in
[final-run.json](evidence/final-run.json).

The browser test plays both 1180×1000 desktop and 390×844 mobile layouts using
real buttons. It covers successful and failed rescue, stale frozen grounds
despite newer support, replanning, local save/reload/resume, malformed saves,
both report choices and hidden physical changes. Both layouts were visually
inspected; checks found no browser errors or horizontal overflow. Screenshots
are produced under `test-results/trail-rescue-*.png` and retained by CI.

Final verification passes **440 Rust tests**, format and zero-warning Clippy,
the full browser build, all 24 scenarios, five additional policy checks, and
both browser layouts. Check counts and build/source hashes are in
[verification.json](evidence/verification.json). Commands:

```sh
cargo fmt --manifest-path runtime/Cargo.toml --all -- --check
cargo clippy --manifest-path runtime/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path runtime/Cargo.toml --all-targets
npm run build
npm run test:trail-rescue
npm run test:trail-rescue-page
npm run test:slime-glow
npm run test:glowcap
npm run test:glowcap-page
node --test experiments/glowcap/compare-views.test.mjs
node experiments/glowcap/replay-divergences.mjs
node experiments/glowcap/harness.mjs --phase=cr12 --impl=caveat5
```

The existing Glowcap suite passes 38/38 scenarios; its five saved fuzz failures
still agree over 16,011 events and 13 restores. Slime Glow's 10,001-toggle
graph-size check also passes. The previous 14.4-million-event differential run
was not repeated for this change and is not claimed as fresh evidence.

## Failures kept on the record

Before the first scenario run, source validation caught two authoring errors:
a claim name collided with a derived definition, and a procedure read an event
parameter without receiving it as an argument. The claim was renamed and the
parameter passed explicitly. No scenario requirement changed.

The new state-caveat regression failed before its implementation. Restore
review then demonstrated two additional failures before their fixes: reversing
two same-event commitment journal entries was accepted, and restoring a legal
below-zero custom clock was rejected. Both have permanent regression tests.
The first browser run passed; no visual correction was needed.

## Scope

This is one finite rescue beat, with at most five acquired observations and a
bounded decision history. It demonstrates source authoring, runtime improvements
and an end-to-end playable consumer. It does not establish unlimited history,
general-purpose collections, external evidence authenticity, or broad adoption.
This run includes correctness checks, not a new timing benchmark.
