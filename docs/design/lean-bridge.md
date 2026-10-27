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
   except the assumed results its entry lists (see below), and compares the
   statement hashes. It writes the events a host sends the program: the
   entry's pass event, or its failure event with `why` set to `broken`,
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

## Results assumed for now

Large formalizations rarely start complete. The project formalizing Fermat's
Last Theorem in Lean states some established results, such as theorems from the
1980s, as assumptions to be proven later; the proof stands, but only as
strongly as what it assumes. That is a caveat, and the bridge records it as
one.

In Lean, an assumed result is a named `axiom`, never a `sorry`:

```lean
namespace Assumed
axiom two_pow_add (a b : Nat) : 2 ^ (a + b) = 2 ^ a * 2 ^ b
end Assumed
```

The manifest lists it under the theorem that uses it, with its own pinned
statement hash. The check then accepts exactly the listed assumptions as extra
axioms, refuses any other, and reports how many the proof still uses:
`product_checked` with `assumed` set to that number.

In the program, a lemma whose check reports an assumption is qualified with
`assumed_for_now` (consequence `material`), so every result resting on it
carries the caveat and `explain` says so:

```
assumed_result@1  based on product_lemma (caveats: assumed_for_now)
checked_result@1  based on lean_lemma
```

When the assumption is proven (the axiom becomes a theorem with the same
statement), the next check reports none and the program takes the caveat
back with `unqualify`. Results derived afterwards rest on the lemma without
it; results derived before keep the basis they had, as every take-back does.
If the assumption is instead disproven, `refute_assumption` withdraws the
lemma that assumed it and reopens what rested on it, and nothing else: the
fully proven lemma and its results stand.



It establishes that, under the pinned Lean, a theorem by that name exists,
its proof is complete using only the standard axioms and the assumptions its
entry lists, and its statement and each used assumption's statement elaborate
to the same terms as when their hashes were recorded.

It does not establish:

- **That an assumed result is true.** That is what the caveat is for. The
  check only makes sure every assumption is named, listed and pinned.
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

The check's seven controls change the demo source on purpose and require the
intended result: a `sorry`, a changed statement, a renamed theorem, a proof
that does not compile, an assumption the manifest does not list and a changed
assumption must each be refused for its reason, and proving the assumption
must pass with none left. A green run shows the check can fail.

## Choices made

- **Host-side, not in the language.** A Lean ground is ordinary evidence and
  the check is ordinary host code, so nothing in the 1.x promise moves. A
  later form in the language (a declared proof source, say) would be its own
  specification PR with a corpus sweep.
- **One event pair per ground.** The example has two theorems, each with its
  own pass and failure events, and the reopening rules are one procedure
  called per result. A program with many grounds would use an `id`
  parameter; the manifest already names each entry's events.
- **Each ground is checked on its own.** `add_lt_two_pow_mul` uses
  `lt_two_pow` in its proof, but the program does not model that: if
  `lt_two_pow` breaks, the file stops compiling and the check reports both.
- **Assumptions are axioms, not `sorry`.** A `sorry` has no name or
  statement to pin, so it is always refused; an assumption must be stated.
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
