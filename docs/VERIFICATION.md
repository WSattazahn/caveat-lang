# What is checked, and how

Caveat is a decision ledger for agents: an embeddable decision runtime with a
small rules language. This page says what kinds of checking stand behind that,
what each kind shows, and where checking stops. It summarizes the sources it
links; when they disagree with it, they decide.

## Two distinctions the checks rely on

**Grounds and lineage.** Every value carries two dependency channels.
*Lineage* answers "what could have influenced this?": it includes the guard of
the rule that set it, even when that rule was skipped. *Grounds* answer "what
is this based on?": the evidence and caveats its expression actually read. A
guard is control, not content, so it enters lineage and never grounds. Grounds
are always a subset of lineage. Defined in
[Explanations 0.2](../spec/caveat-explanations-0.2.md).

**Live qualification and frozen grounds.** `qualify EVIDENCE with CAVEAT`
reaches the *current* values that read the evidence: they gain the caveat in
their lineage, and in their grounds when they are grounded on it. A recorded
decision (`commit … using …`) keeps exactly the basis and grounds it was made
on. Late knowledge changes what the program believes now. It does not rewrite
what an earlier decision rested on. Whether that decision should be
reconsidered is an authored policy (`reopen … because …`), not an automatic
consequence of every caveat. Defined in
[Late Qualification 0.1](../spec/caveat-late-qualification-0.1.md) and the
[decision journal](../spec/caveat-decision-journal-0.1.md).

## Five kinds of checking

| Kind | What it shows | What it does not show | Where |
| --- | --- | --- | --- |
| **Model theorems** (Lean) | Laws that hold for *every* input of a small mathematical model: dependency flow, grounds ⊆ lineage, citation checks, outcomes, one late-qualification step and one reopening step | That the Rust runtime implements the model. The model is a separate, smaller program | [proofs/lean](../proofs/lean/README.md); CI job `lean` |
| **Sampled conformance** | For registered programs, the native and WebAssembly runtimes and the Lean model's executable agree on the compared projection, step by step | Agreement on programs or states outside the registered samples; a refinement proof | `npm run verify:lean-conformance`; cases in `scripts/lean-*-cases.mjs` |
| **Mutation controls** | The checks above can fail: deliberately broken compiled runtimes and incomplete proofs must be refused for their intended reasons, and altered scenario expectations must fail | That every possible defect would be caught | `scripts/verify-lean-conformance.mjs`, `scripts/verify-lean.mjs`, `experiments/scenario-conversion/mutation-check.mjs` |
| **Runtime tests** | The production runtime's behavior on Rust unit and integration tests, kit tests, scenario suites, installed-package tests, browser checks and byte-for-byte reproducible builds | Behavior on untested programs or workloads | [runtime workflow](../.github/workflows/runtime.yml) |
| **Unproved boundaries** | Named parts that no theorem covers, tested only by the runtime tests above | — | Listed below |

### The late-qualification theorem, precisely

[Late.lean](../proofs/lean/Caveat/Late.lean) models one step of
`[when GUARD] qualify EVIDENCE with CAVEAT` over a ledger of current values and
recorded commitments. `late_qualification_preserves_basis_and_grounds` proves
that an accepted step leaves every commitment's basis and grounds exactly as
they were. Companion theorems prove the same for the session after any outcome,
refusal included, and for any accepted sequence of qualifications. Others prove
the other half: a current value that read the evidence gains the caveat and the
guard's lineage and keeps its number, the guard never enters grounds, and a
value that did not read the evidence is unchanged.

This holds of the model because its qualification step changes only current
values. The theorems make that a checked statement rather than a reading of
the definition. They do not prove that the production runtime is correct. The
runtime's agreement with the step is sampled: nine registered programs on both
runtimes, plus two compiled mutants that leak the caveat into commitment
records and must be caught. Reopening has the same structure in
[Reopen.lean](../proofs/lean/Caveat/Reopen.lean): records and retained caveats
are preserved, and the cause is appended to the journal.

### Unproved boundaries

From the [proof documentation](../proofs/lean/README.md): commitment creation
(`commit … using … retaining`, series, predecessors), scheduled qualification,
renewal and occurrence identity, graph closure, save and restore, complete
session rollback, source parsing, function evaluation, arbitrary expressions,
work budgets, effect scheduling, floating-point behavior, the JSON decoder and
Rust's bounded sets and overflow detection are not formalized. Departure,
archives and the withdrawal collector are covered by runtime tests and the
departure gate, not by the model.

## Precise claims

- **Citation checking** establishes that cited dependencies were recorded. It
  does not establish that evidence was well chosen, that evidence or
  explanation prose is true, or that an explanation is complete.
- **Caveats propagate.** They are carried through computations and decisions.
  Caveat does not compute probabilities from them.
- **Provenance-aware computation is established research.** Caveat does not
  claim to have invented it. The [related work](#related-work) note compares
  it with existing systems.
- **Correct explanations are necessary, not sufficient.** Rollback, restore,
  source identity and persistence each need their own correctness checks.
- **Changing evidence is not changing the program.** Qualifying or correcting
  evidence in a fixed program is a runtime event. Editing the rules is a
  different operation, and a save is restored under the source it was made
  with.
- **Nothing is authenticated.** Caveat records supplied evidence. Restoring a
  save checks that it is consistent with the program, not that it is genuine,
  and the journal is not a tamper-evident log.

## Limits

These are limits, not defects:

- History windows do not bound the whole save: required reason chains can
  grow (rc.16's 3,000-cycle control saved 1.24 MB). Archive storage is
  additional. See the [history guide](../kit/docs/HISTORY.md).
- Declared resource limits refuse an event rather than grow without bound.
- Synchronous bulk work can exceed a frame budget (rc.16 measured 55–60 ms for
  a bulk release). Hosts set and test their own budgets.
- Crash recovery is the host's: Caveat stores nothing by itself.
- Whether Caveat helps real decision systems is a hypothesis. The
  [glowcap benchmark](../experiments/glowcap/RESULTS.md) records every round,
  including the blind rounds TypeScript won.

## Related work

A short comparison with truth maintenance, Datalog provenance, Rego, Cedar,
differential dataflow and Scallop is planned under the
[Path to 1.0](releases/path-to-1.0.md) plan (PR C5).
