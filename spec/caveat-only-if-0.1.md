# CAVEAT Only If 0.1 — a value chosen by a condition that is not its grounds

**Status:** stable from 1.4.0 under the [compatibility commitment](../docs/COMPATIBILITY.md); drafted for 1.4. It adds to [Reactive 0.7](caveat-reactive-0.7.md) and [Untrusted Sources 0.1](caveat-untrusted-sources-0.1.md), and it changes no program that loads today.

A decision keeps two records of what it read: its grounds, the evidence it
was made on, and its lineage, everything that could have influenced it. A
rule's `when` condition is lineage. What its `using` expression reads is
grounds, and that includes everything an `if(...)` in it reads, its condition
too.

That makes one common decision awkward to write. Several sources vote, and
the decision should rest on the votes that agreed and only on those. With
`using if(m1_yes, latest(m1_vote), 0) + if(m2_yes, latest(m2_vote), 0) + ...`
the dissenting votes join the grounds, because each condition read them. The
only way to keep them out was one rule per set of agreeing sources, each with
its own `when` and `using`: 4 rules for two of three sources, and 16 for three
of five ([the model-votes experiment](../experiments/model-votes/README.md)).
Before this profile the direct form did not load:

```text
votes.cav does not load: unknown expression function only_if
```

(`caveat-lang@1.3.0`, `caveat validate`.)

## Syntax

```caveat
only_if(CONDITION, VALUE)
```

CONDITION is a boolean expression and VALUE a numeric one. It may appear
wherever a numeric expression may.

## Meaning

The value is VALUE when CONDITION holds, and 0 otherwise. VALUE is evaluated
only when CONDITION holds.

- **Grounds.** The reads of CONDITION are not grounds. Where the runtime
  computes grounds (a commit's `using` and `citing`, a state's grounds), an
  `only_if` adds VALUE's reads when CONDITION holds and nothing otherwise.
  This holds inside a nested `only_if` too: anything read while a condition is
  evaluated stays out of the grounds.
- **Lineage.** CONDITION's reads are lineage, as a rule's `when` condition's
  are, and so are VALUE's when it is evaluated. `explain` lists the
  condition's evidence under "could also have been influenced by".

So a decision grounded on votes reads:

```caveat
on decide when yes >= 2 commit approve because enough using
    only_if(m1_yes, latest(m1_vote)) + only_if(m2_yes, latest(m2_vote)) + only_if(m3_yes, latest(m3_vote));
```

With m1 and m2 for and m3 against, `approve@1` rests on `m1_vote@1` and
`m2_vote@1`. `m3_vote@1` is in its lineage only. Withdrawing m3's vote leaves
the decision without a withdrawn ground. Withdrawing m2's makes
`rests_on_withdrawn(approve)` true.

Everything that depends on grounds follows from this, with no change of its
own: `rests_on_withdrawn`, `in_basis`, the pins that keep a ground's record,
the undo list, and the caveats a decision carries. A caveat on evidence that
only a condition read is not on the decision's grounds, so an `irreversible`
series does not ask for it to be retained. A rule's `when` condition already
behaves this way. `only_if` gives the same choice inside an expression and
gives a program no new way around a check.

A program that defines `fn only_if` keeps its function: its calls are that
function's, with the arguments as written. Without such a function, the form
did not load before this profile.

## Not in this profile

- A sum over the members of a kind, such as `sum(for model as $m: ...)`. A
  program writes one `only_if` term per member, which is one term per source
  rather than one rule per set of sources.
- A choice between two values (`if` with the condition as lineage only).

## Proofs

`runtime/tests/only_if.rs`: grounds and lineage of a commit and of a state, a
false condition, a condition that would fail if VALUE were evaluated, nesting,
withdrawal of a vote in the grounds and of one only in the lineage, save and
restore, and `fn only_if` kept. `experiments/model-votes/` writes its decision
as three `only_if` terms. Its scenarios pass, and the undo list reopens only the
decision a withdrawn vote tipped.

## Corpus sweep

No tracked program mentions `only_if`
([`experiments/only-if-sweep/`](../experiments/only-if-sweep/README.md)).
Every tracked program and scenario file gives the same `validate --json` and
`test --json` output under `caveat-lang@1.3.0` and this checkout, apart from
the files that use the 1.4 drafts.

## Changes

- 0.1 (draft): first version.
- 2026-11-14 (D088): in a bundle, a module that declares `only_if` as
  something other than a function reads `only_if(…)` as the form, as from
  1.4.0. Before 1.4.0 the name was the module's, so the call reached the
  bundle program's `fn MODULE__only_if`, and a module that gave it a number
  for its condition loaded. A bundle that only releases before 1.4.0 could
  load is read as they linked it, so such a call reaches that function again
  ([Caveat 0.5](caveat-0.5-draft.md)). Test:
  `runtime/tests/d088_module_names_older_releases_loaded.rs`.
- 2026-11-14 (D093): 1.4.0 and 1.4.1 refused a function body that gives
  `only_if(…)` a number for its condition, reading the form there, though the
  program declares `fn only_if`. A bundle with such a body is read as the
  releases before 1.4.0 linked it, where they could load it: a module's
  `only_if(…)` reaches `fn MODULE__only_if`, in its rules and its functions
  alike, as in 1.3.0. Where they could not, the body calls `fn only_if`.
  Test: `runtime/tests/d093_module_only_if_beside_a_program_only_if.rs`.
