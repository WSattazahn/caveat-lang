# Related work

Caveat builds on ideas that predate it. This note places it beside six of
them. It is not a benchmark and not a ranking. These systems solve different
problems and are not interchangeable products. Caveat claims none of the ideas
below as its own invention.

Each entry says what the cited source describes, and then how Caveat differs
in its own documented behavior. Each comparison is marked:

- **[checked]** The Caveat side is from this repository's specifications, and
  the other system's side is stated in the cited primary source.
- **[unverified]** We believe this difference holds, but we have not checked
  it against the other system's full documentation or behavior. Do not repeat
  it as established.

Sources were read on 2026-10-10. Caveat's behavior is summarized in
[What is checked, and how](VERIFICATION.md).

## Truth maintenance (Doyle, 1979)

Jon Doyle, "A Truth Maintenance System", *Artificial Intelligence* 12(3),
1979. A truth maintenance system records a justification for each belief and
revises which beliefs hold when a justification is withdrawn or contradicted.
It is the clearest ancestor of "keep the reasons with the conclusion".

- **[checked]** Caveat records grounds and lineage for each value, and a late
  qualification reaches current values that read the evidence
  ([Late Qualification 0.1](../spec/caveat-late-qualification-0.1.md)).
- **[checked]** Caveat separates current belief from a recorded decision. A
  committed decision keeps the basis it was made on. New evidence reopens it
  only through an authored `reopen … because …` policy, and the journal keeps
  both the original grounds and the reason it reopened
  ([decision journal](../spec/caveat-decision-journal-0.1.md)). Classic truth
  maintenance updates which beliefs hold. It does not keep a frozen record of
  an earlier decision beside the revised beliefs.
- **[unverified]** Later TMS variants (assumption-based, de Kleer 1986) track
  several contexts at once. We have not compared Caveat with them.

## Datalog provenance (provenance semirings)

Todd J. Green, Grigoris Karvounarakis and Val Tannen, "Provenance Semirings",
PODS 2007. Query results are annotated with values from a commutative
semiring, so one framework covers why-provenance, lineage, counting and
probabilities over positive relational and Datalog queries.

- **[checked]** Caveat's provenance is two fixed channels of evidence and
  caveat identities, combined by set union and checked against citations
  ([Explanations 0.2](../spec/caveat-explanations-0.2.md)). It is not
  parameterized by a semiring, and it computes no probabilities.
- **[checked]** Caveat programs are event-driven state machines with
  commitments, rejection and rollback, not queries over a database.
- **[unverified]** Caveat's lineage resembles why-provenance under set
  semantics. We have not established a formal correspondence.

## Rego (Open Policy Agent)

[Rego](https://www.openpolicyagent.org/docs/policy-language) is OPA's policy
language. OPA's [decision logs](https://www.openpolicyagent.org/docs/management-decision-logs)
record, for each decision, the `input`, the `result`, the revision of each
policy bundle used, a `decision_id` and a `timestamp`.

- **[checked]** Both keep a decision's inputs and the policy revision that
  produced it. Caveat additionally keeps, inside the runtime, which evidence
  and caveats each decision rested on, and the decision's later reopenings.
- **[unverified]** Rego decisions are stateless evaluations of one query.
  Caveat sessions carry state from one event to the next. Rego's evaluation
  tracing may cover some of what Caveat's explanations do; we have not
  compared them.

## Cedar

Joseph W. Cutler et al., "Cedar: A New Language for Expressive, Fast, Safe,
and Analyzable Authorization", OOPSLA 2024
([arXiv 2403.04651](https://arxiv.org/abs/2403.04651)). Cedar is an
authorization policy language. Applications pass access decisions to its
engine. Its design is modeled and proved in Lean, and its logical encoding
supports policy analysis such as checking that a refactored policy set allows
the same requests.

- **[checked]** Both use Lean for a model of the design. Cedar's paper
  reports a sound and complete encoding for analysis. Caveat's Lean work is a
  smaller model with sampled conformance, and covers less of the language
  ([proofs/lean](../proofs/lean/README.md)).
- **[unverified]** Cedar answers one authorization request at a time.
  Caveat's decisions persist and can be reopened by later evidence. Caveat
  checks that an approval was observed and not withdrawn, but it is not an
  authorization system for outside actions.

## Differential dataflow

Frank McSherry, Derek G. Murray, Rebecca Isaacs and Michael Isard,
"Differential dataflow", CIDR 2013. It incrementally updates the results of
data-parallel computations, including iterative ones, as their inputs change.

- **[checked]** Caveat also updates dependent values incrementally when
  evidence changes
  ([incremental evaluation](../spec/caveat-incremental-evaluation-0.1.md)).
  Its aim is to keep reasons and frozen decision records, not to maintain
  large data-parallel results.
- **[unverified]** We have made no performance comparison. Caveat's own
  measurements are in its release records.

## Scallop

Ziyang Li, Jiani Huang and Mayur Naik, "Scallop: A Language for
Neurosymbolic Programming", PLDI 2023
([arXiv 2304.04812](https://arxiv.org/abs/2304.04812)). A Datalog-based
language with recursion, aggregation and negation, whose provenance-semiring
framework makes logical reasoning differentiable so it can be trained with
neural networks.

- **[checked]** Both carry provenance through computation. Scallop uses it to
  compute differentiable or probabilistic results. Caveat uses it to record
  and check reasons, and computes no probabilities.
- **[unverified]** Scallop has no decision journal or event rollback, as far
  as we know from the paper.

## What this note does not claim

- It does not say Caveat is better than any of these systems, or than a
  general-purpose language. The [glowcap benchmark](../experiments/glowcap/RESULTS.md)
  compares Caveat with TypeScript on one task, and the blind rounds went to
  TypeScript.
- It does not say "almost no other language" tracks provenance. Several do.
- An entry marked [unverified] is a starting point for a reader to check,
  not a finding.
