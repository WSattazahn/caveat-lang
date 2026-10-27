# Design note: claims that rest on a Lean proof

Status: a design note with one working example. It adds no syntax and changes
nothing the language means; everything below uses features of 1.4.1. The
example is `examples/lean-bridge/`, its check `scripts/check-lean-grounds.mjs`.

## The idea

Some claims rest on mathematics. An agent that uses a lemma should be able to
say which kind of lemma it used: one a proof checker has accepted, or one a
model or a paper asserted. And when either kind turns out wrong, everything
built on it should be found, not just the lemma.

Caveat already does the second half. Evidence can carry a caveat, a decision
keeps what it rested on, and withdrawing evidence lets rules reopen exactly
the decisions whose grounds held it. The bridge supplies the first half: a
check, outside the runtime, that turns "this Lean theorem exists, its proof
is complete and its statement is the one we pinned" into an ordinary event.

## How it fits together

1. **The program names the theorem.** The evidence's source label is
   `lean:` followed by the theorem's full name:

   ```caveat
   evidence lean_lemma from "lean:LeanBridgeDemo.lt_two_pow";
   ```

   As with any source, the label is not authentication. What ties it to a
   proof is the check in step 3.

2. **A manifest pins the statement.** `lean-grounds.json` lists, for each
   Lean ground, the evidence name, the Lean source, the theorem, its statement
   as a person reads it, the SHA-256 of the statement as Lean elaborates it,
   and the two events to send: one when the check passes, one when it fails.

3. **CI checks it.** For each entry the check compiles the source with the
   pinned Lean (4.34.1, the same release the model proofs use), finds the
   theorem, refuses any axiom beyond `propext`, `Classical.choice` and
   `Quot.sound` (so `sorry`, custom axioms and `native_decide` are refused),
   and compares the statement hash. It writes the events a host sends the
   program: `lean_checked`, or `lean_failed` with `why` set to `broken`,
   `changed` or `missing`. The job fails on any failure.

4. **The program does the rest.** `lean_checked` reveals the lemma.
   Results are committed `citing` it, and a result that also cites an
   unchecked lemma carries that lemma's `unchecked` caveat (consequence
   `material`). `lean_failed` withdraws the Lean lemma and reopens every
   result whose grounds held it; a counterexample to the unchecked lemma
   withdraws only that one and reopens only the results that used it. A
   fixed proof renews the evidence, so new results rest on the new
   occurrence and the old ones keep what they rested on.

```
caveat explain examples/lean-bridge/grounds.cav EVENTS
  checked_result@1  based on lean_lemma
  combined_result@1 based on lean_lemma, model_lemma (caveats: unchecked)
```

## What the check does and does not establish

It establishes that, under the pinned Lean, a theorem by that name exists,
its proof is complete using only the standard axioms, and its statement
elaborates to the same term as when the hash was recorded.

It does not establish:

- **That the statement says what the claim means.** A checked proof removes
  the need to review the proof, not the statement. A person reads the
  statement once, when its hash is recorded; changing the hash is a reviewed
  change to the manifest, and until then the check reports `changed`.
- **More than Lean's own trust base.** The kernel, the compiler and the
  imported library are trusted, as in the model proofs
  ([VERIFICATION.md](../VERIFICATION.md)).
- **Stability across Lean releases.** The hash is of the elaborated term as
  this Lean release prints it. An upgrade may change the printing; the check
  then reports `changed`, which errs toward reopening, and the hash is
  re-pinned after reading the statement again.
- **Anything about the event's sender.** The runtime records the events it
  is given. Only the CI job's own output ties `lean_checked` to a check.

The four failure controls in the check break the demo theorem on purpose (a
`sorry`, a changed statement, a renamed theorem, a proof that does not
compile) and require each to be refused for its intended reason, so a green
run shows the check can fail.

## Choices made

- **Host-side, not in the language.** A Lean ground is ordinary evidence and
  the check is ordinary host code, so nothing in the 1.x promise moves. A
  later form in the language (a declared proof source, say) would be its own
  specification PR with a corpus sweep.
- **One event pair per ground.** The example has one theorem, so it has one
  `lean_checked` and one `lean_failed`. A program with several would use an
  `id` parameter or a procedure per ground; the manifest already names each
  entry's events.
- **No Mathlib.** The example uses only Lean's core library, so CI needs no
  package download beyond the pinned Lean release it already fetches. A
  project whose lemmas need Mathlib would compile them in its own Lake
  project and point the manifest at the built module.

## Where it could go

The case for more is the AI safety reader: a claim whose mathematical ground
a checker accepted needs less review than one resting on a model's sketch,
and the ledger says which is which and finds everything downstream when
either fails. Very few real tasks have Lean proofs today, so this stops at a
note and an example until a user's task needs more.
