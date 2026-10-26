# Related work

Caveat builds on ideas that predate it. This note places it beside fourteen
of them. It is not a benchmark and not a ranking. These systems solve different
problems and are not interchangeable products. Caveat claims none of the ideas
below as its own invention.

Each entry says what the cited source describes, and then how Caveat differs
in its own documented behavior. Each comparison is marked:

- **[checked]** The Caveat side is from this repository's specifications, and
  the other system's side is stated in the cited primary source.
- **[unverified]** We believe this difference holds, but we have not checked
  it against the other system's full documentation or behavior. Do not repeat
  it as established.

Sources were read on 2026-10-10, those for the W3C PROV and agent decision
record entries on 2026-10-12, and those for the TMS, Hyperon and Watson
entries and the four entries from bitemporal databases onward on 2026-10-25.
Caveat's behavior is summarized in
[What is checked, and how](VERIFICATION.md).

## Truth maintenance (Doyle, 1979; de Kleer, 1986)

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

Johan de Kleer, "An Assumption-Based TMS", *Artificial Intelligence* 28,
1986, pp. 127–162. An assumption-based TMS (ATMS) labels each conclusion with
the sets of assumptions, called environments, under which it holds, so it
can answer for many contexts at once without switching between them. It
recommends retracting through an assumption rather than by deleting a
justification (section 4.9).

- **[checked]** A Caveat value's lineage is one set of evidence and caveat
  identities, combined by union
  ([Explanations 0.2](../spec/caveat-explanations-0.2.md)). That is a single
  environment per value. Caveat does not keep several environments for one
  value, and does not answer queries across contexts the way an ATMS does.
  The ATMS's worst case, labels that grow exponentially with the number of
  contexts (de Kleer, section 4.8), does not arise.
- **[checked]** Caveat's withdrawal follows the shape de Kleer recommends. It
  removes no observation or record. It adds the `withdrawn` caveat to every
  current value whose lineage or grounds include the evidence
  ([Withdrawal 0.1](../spec/caveat-withdrawal-0.1.md)).
- **[checked]** The reverse question, what rests on a piece of evidence, is
  answered from a snapshot by
  [Dependents 0.1](../spec/caveat-dependents-0.1.md). It reports which
  decisions and values rest on the evidence. It does not run the program to
  show what a withdrawal would make it do.
- **[unverified]** McAllester's logic-based TMS (LTMS) propagates clauses
  rather than justifications (David McAllester, "Truth Maintenance", AAAI-90,
  pp. 1109–1116, surveys the family). Caveat does no logical inference over
  lineage, so we expect the comparison to be distant, but have not made it.

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

## Probabilistic Logic Networks in OpenCog Hyperon

Ben Goertzel et al., "OpenCog Hyperon: A Framework for AGI at the Human
Level and Beyond" ([arXiv 2310.18318](https://arxiv.org/abs/2310.18318), a
preprint, 2023), describes a rewrite of OpenCog around the MeTTa language.
Its Probabilistic Logic Networks (PLN) give a statement a truth value with
two parts: a strength, the estimated probability that it is true, and a
confidence, how certain that estimate is
([arXiv 2412.19524](https://arxiv.org/abs/2412.19524), a preprint).

- **[checked]** Caveat scores no belief. A value carries the evidence and
  caveats it rests on ([Explanations 0.2](../spec/caveat-explanations-0.2.md)),
  and a caveat's consequence is one of five declared levels, from
  `negligible` to `catastrophic`, not a probability. Withdrawing evidence marks what rests on it rather than lowering a number
  ([Withdrawal 0.1](../spec/caveat-withdrawal-0.1.md)).
- **[unverified]** From the sources cited here, PLN revises truth values as
  evidence accumulates and does not keep a frozen record of the basis an
  earlier decision was made on. We have not read MeTTa's or PLN's
  implementation. The two could be combined: a program could carry a PLN
  strength as an ordinary value beside Caveat's lineage;
  [Caveat alongside probabilities](PROBABILITIES.md) shows how with any score.

## Watson's DeepQA and Elemental Cognition

David Ferrucci et al., "Building Watson: An Overview of the DeepQA Project",
*AI Magazine* 31(3), 2010, pp. 59–79
([doi:10.1609/aimag.v31i3.2303](https://doi.org/10.1609/aimag.v31i3.2303)).
DeepQA generated candidate answers, gathered and scored evidence for each,
and answered with a confidence. Elemental Cognition's later work
(Jennifer Chu-Carroll, David Ferrucci et al., "Beyond LLMs: Advancing the
Landscape of Complex Reasoning", [arXiv 2402.08064](https://arxiv.org/abs/2402.08064),
a preprint, 2024) puts a symbolic reasoning engine at the core and uses
language models around it, to acquire knowledge and to talk with users.

- **[checked]** Caveat generates no answers and scores none. A program states
  its rules, and each event an agent or host sends is accepted or refused by
  them ([Dispatch 0.1](../spec/caveat-dispatch-0.1.md)). An accepted
  decision keeps the grounds and caveats it was made on
  ([decision journal](../spec/caveat-decision-journal-0.1.md)).
- **[unverified]** Caveat fits the same split at a smaller scale: a language
  model proposes an observation or a decision, and Caveat records what it
  rests on and refuses what the rules do not allow. We have not compared it
  with Elemental Cognition's engine, which also solves constraints and
  optimizes, which Caveat does not.

## W3C PROV

[PROV-DM](https://www.w3.org/TR/prov-dm/), a W3C Recommendation (2013), is a
data model for provenance: entities, the activities that use and generate
them, and the agents responsible. It defines invalidation (`wasInvalidatedBy`,
"the start of the destruction, cessation, or expiry of an existing entity by
an activity"), revision (`wasRevisionOf`) and bundles, which are named sets of
provenance descriptions that can themselves have provenance.

- **[checked]** Caveat's records cover some of the same ground inside a
  running program. A withdrawal records that an observation is no longer stood
  behind, and why, and removes nothing
  ([Withdrawal 0.1](../spec/caveat-withdrawal-0.1.md)). The decision journal
  keeps what each commitment was made on and the evidence that reopened it
  ([decision journal](../spec/caveat-decision-journal-0.1.md)).
- **[checked]** PROV is an interchange model for describing provenance after
  the fact. It does not evaluate rules, and it does not decide when a
  decision reopens. Caveat does both, and it does not export PROV.
- **[unverified]** A withdrawal resembles invalidation and a decision
  revision resembles `wasRevisionOf`. We have not defined a mapping between
  them.

## Agent decision records (2026)

Several 2026 projects record what an AI system decided and why:

- Victor Ojewale, Harini Suresh and Suresh Venkatasubramanian, "Audit Trails
  for Accountability in Large Language Models"
  ([arXiv 2601.20727](https://arxiv.org/abs/2601.20727), a preprint). It
  proposes ordered, tamper-evident, append-only records that link a model's
  technical history to governance records such as approvals and waivers.
- [Semantica](https://github.com/semantica-agi/semantica), a self-hosted
  Python platform for knowledge graphs, rules and "decision provenance".
- Agent-memory projects that add truth maintenance over stored facts. For
  example, one [opencode-memory change](https://gitlab.com/ghavenga/opencode-memory/-/merge_requests/595)
  uses delete-and-rederive maintenance (DRed; Gupta, Mumick and
  Subrahmanian, SIGMOD 1993). A retraction there closes a fact's validity
  interval instead of deleting it.

- **[checked]** The audit-trail paper treats tamper evidence as central.
  Caveat does not provide it. Caveat records the evidence supplied to it
  without authenticating it. A save is not signed, and restore checks a save's
  structure, not whether it is the original account of a session
  ([Save 0.1, host trust boundary](../spec/caveat-save-0.1.md#host-trust-boundary)).
- **[checked]** Caveat keeps a recorded decision's basis when evidence it
  rested on is withdrawn. The decision reopens only through an authored
  policy ([Withdrawal 0.1](../spec/caveat-withdrawal-0.1.md)).
- **[unverified]** From the sources cited here (the paper's abstract, the
  change's description and a third-party review of Semantica), none of these
  states that it keeps a past decision's frozen basis beside a retraction. We
  have not read their full documentation or code. Do not repeat this as a
  finding.

## Bitemporal databases

Richard T. Snodgrass and Ilsoo Ahn, "Temporal Databases", *IEEE Computer*
19(9), 1986, and the system-versioned and application-time tables of
ISO/IEC 9075:2011 (SQL:2011). A bitemporal table records two times for each
row: when a fact held (valid time) and when the database recorded it
(transaction time). A correction closes the old row's transaction time
instead of deleting it, so a query can ask what the database said as of an
earlier moment.

- **[checked]** Caveat's withdrawal also removes nothing. The observation,
  its reading archives, every decision's frozen basis and the decision
  journal stay as they were
  ([Withdrawal 0.1](../spec/caveat-withdrawal-0.1.md), step 3).
- **[checked]** Caveat orders its records by event and sequence number. A
  withdrawal records when it happened, not since when the evidence was
  wrong, so Caveat has no separate valid time
  ([Withdrawal 0.1](../spec/caveat-withdrawal-0.1.md), step 1).
- **[unverified]** A bitemporal table does not by itself mark the results
  derived from a corrected row. Caveat adds the `withdrawn` caveat to every
  current value whose lineage includes the evidence. We have not surveyed
  temporal databases with triggers or derived views that do the same.

## Legal citators

Law publishers' citators, such as Westlaw's KeyCite and LexisNexis's
Shepard's, mark how later courts treated a case. KeyCite's
[Overruling Risk](https://www.thomsonreuters.com/en-us/help/westlaw-precision/tools/keycite-overruling-risk)
warns about a point of law that has been "undermined because it relies on an
overruled or otherwise invalid prior decision". An overruled opinion is not
rewritten: it still reads as it was decided.

- **[checked]** The shape is the same as Caveat's: withdrawing a source
  flags what currently rests on it, and each past decision keeps the basis
  it had. In Caveat the runtime computes this from lineage it recorded when
  each value and decision was made, and a decision reopens only through an
  authored `reopen … because …` policy
  ([Withdrawal 0.1](../spec/caveat-withdrawal-0.1.md)).
- **[unverified]** The help page shows the warning, not how it is produced.
  We understand citator flags to come from editors reading the citing
  opinions, not from recorded dependencies, but have not confirmed it.

## Retraction notices and retraction trackers

A retracted paper stays published, marked as retracted. A nanopublication
"is persistent, you can never edit nor delete it", and retracting one
publishes a second, signed nanopublication that points at it
([nanopub documentation](https://nanopub.readthedocs.io/en/latest/publishing/retraction.html)).
Tools such as Guillaume Cabanac's Feet of Clay Detector and Nicholas
DeVito's RetractoBot find papers that cite retracted work and alert readers
or authors.

- **[checked]** Caveat records a withdrawal with its reason as a separate
  record, and removes nothing
  ([Withdrawal 0.1](../spec/caveat-withdrawal-0.1.md)). Nanopublications
  also sign the retraction. Caveat does not sign or authenticate records
  ([Save 0.1, host trust boundary](../spec/caveat-save-0.1.md#host-trust-boundary)).
- **[unverified]** The trackers work from citations, which say that a paper
  mentions another, not that a conclusion used it. Caveat's lineage records
  what an evaluation read. We have not read the trackers' methods.

## Temporal knowledge graphs for agent memory (Graphiti)

Preston Rasmussen et al., "Zep: A Temporal Knowledge Graph Architecture for
Agent Memory" ([arXiv 2501.13956](https://arxiv.org/abs/2501.13956), a
preprint). Its Graphiti library keeps a bitemporal graph of facts. When a
new fact contradicts an edge, it sets the edge's `invalid_at` time instead of
deleting it.

- **[checked]** Both keep a fact that is no longer stood behind. Caveat
  keeps the observation and adds the `withdrawn` caveat to it and to current
  values that rest on it ([Withdrawal 0.1](../spec/caveat-withdrawal-0.1.md)).
- **[unverified]** We read a summary of the paper, not the paper or the
  library. From that summary, Graphiti does not record decisions or mark
  what was derived from an invalidated edge. Do not repeat this as a
  finding.

## What this note does not claim

- It does not say Caveat is better than any of these systems, or than a
  general-purpose language. The [glowcap benchmark](../experiments/glowcap/RESULTS.md)
  compares Caveat with TypeScript on one task, and the blind rounds went to
  TypeScript.
- It does not say "almost no other language" tracks provenance. Several do.
- An entry marked [unverified] is a starting point for a reader to check,
  not a finding.
