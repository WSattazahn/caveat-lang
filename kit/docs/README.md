# Caveat documentation

Everything you need to write, run and test a reactive Caveat program ships in
this package. Some documents link to files in the source repository; those are
background, not required reading.

## Start here

- [Getting started](GETTING_STARTED.md): install, write a first program, check
  it with a scenario file, and look inside a session.
- [Authoring card](AUTHORING_CARD.md): for an agent writing a program, one
  tested program with the usual parts, its scenarios, the mistakes that cost
  the most and a small host.
- [Caveat on one page](REFERENCE.md): the language in brief. The commands,
  the core, the syntax, outcomes and common mistakes, with links to
  everything below.
- [Worked example](WORKED_EXAMPLE.md): one complete program taken through
  `validate`, `check`, `test` and `explain`. The package's tests follow it and
  check the output it shows.
- [Authoring guide](../../docs/AI_AUTHORING.md): the authoring loop, numeric
  ranges, the session clock, evidence that ages and decisions that are
  reconsidered.

- [Caveat in a game](GAMES.md): for a game developer, story knowledge as
  evidence, a player's choices as decisions, a betrayal as withdrawal, and the
  game loop, saves and cost, with [the intel example](../examples/intel-betrayal/README.md).

- [Project names and CLI commands](NAMES.md): the unambiguous `caveat-lang`
  command, the supported `caveat` shorthand and other projects with the
  same name.

## The reactive language

The language is specified as a base profile followed by additions. Read them in
order; each later profile assumes the earlier ones.

- [Reactive 0.1](../../spec/caveat-reactive-0.1.md): claims, evidence,
  caveats, state, events, rules, commitments and reopening.
- [Reactive 0.2](../../spec/caveat-reactive-0.2.md): pure functions,
  presentation bindings and emitted cues.
- [Reactive 0.3](../../spec/caveat-reactive-0.3.md): numeric values that
  carry their evidence and caveats through computation.
- [Reactive 0.4](../../spec/caveat-reactive-0.4.md): the source library
  and qualified text.
- [Reactive 0.5](../../spec/caveat-reactive-0.5.md): readings and decision
  histories.
- [Reactive 0.6](../../spec/caveat-reactive-0.6.md): computation over
  retained history.
- [Reactive 0.7](../../spec/caveat-reactive-0.7.md): reusable procedures.

Features added alongside the profiles:

- [Define](../../spec/caveat-define-0.1.md): named expressions,
  `define NAME = EXPRESSION;`.
- [Repetition](../../spec/caveat-repetition-0.1.md): declarations repeated
  for each entity of a kind, `for KIND as $x { … };`.
- [Routed repetition](../../spec/caveat-routed-repetition-0.1.md): a
  repetition block whose rules each run only for the member the event names,
  `for KIND as $x routed by P { … };`. Routing assistance, not isolation.
- [Procedure symbols](../../spec/caveat-procedure-symbols-0.1.md):
  procedures that take evidence, other symbols and histories as parameters.
- [Typed parameters](../../spec/caveat-typed-parameters-0.1.md): event
  parameters that name an entity or a member instead of a number.
- [Identifiers](../../spec/caveat-identifiers-0.1.md): event parameters
  that carry text learned at run time, such as a commit SHA: `NAME id`.
- [Source text](../../spec/caveat-text-0.1.md): statements, comments and
  quoting.
- [Reject](../../spec/caveat-reject-0.1.md): refusing an event so nothing it
  did survives.
- [Explanations 0.1](../../spec/caveat-explanations-0.1.md) and
  [0.2](../../spec/caveat-explanations-0.2.md): `because` citations the
  runtime checks, and grounds as distinct from lineage.
- [Decision journal](../../spec/caveat-decision-journal-0.1.md): the ordered
  record of every decision change.
- [Late qualification](../../spec/caveat-late-qualification-0.1.md):
  caveats learned after the fact, `qualify EVIDENCE with CAVEAT;`.
- [Renewal](../../spec/caveat-renewal-0.1.md): evidence that can be observed
  again, and caveats that arrive after a delay.
- [Withdrawal](../../spec/caveat-withdrawal-0.1.md): recording that an
  observation is no longer stood behind, without erasing what rested on it,
  `withdraw E because R;`.
- [Permission](../../spec/caveat-permission-0.1.md): what permitted a
  decision, recorded apart from what it rests on,
  `commit D … permitted by latest(approvals) for head;`.
- [Reopening triggers](../../spec/caveat-reopening-triggers-0.1.md): a
  decision series declares which readings reopen it,
  `decisions merge limit 8 reopened by pushes;`.
- [State caveats](../../spec/caveat-state-caveats-0.1.md): asking which
  caveats a retained value carries, and reopening because of them.
- [Observation order](../../spec/caveat-observation-order-0.1.md): uses of
  evidence that are certain to fail are load-time errors.
- [Elapsed](../../spec/caveat-elapsed-0.1.md): reading the session clock.
- [Save](../../spec/caveat-save-0.1.md),
  [replay saves 0.1](../../spec/caveat-replay-save-0.1.md) &
  [0.2](../../spec/caveat-replay-save-0.2.md),
  [view 0.1](../../spec/caveat-view-0.1.md),
  [view 0.2](../../spec/caveat-view-0.2.md) and
  [view patch 0.1](../../spec/caveat-view-patch-0.1.md): saving and restoring
  a session, compact positional replay saves, per-event views and incremental
  reused views.
- [Neutral reveal](../../spec/caveat-neutral-reveal-0.1.md) and
  [neutral sample](../../spec/caveat-neutral-sample-0.1.md): observing or
  sampling evidence without taking a stance on a claim.
- [Take-back](../../spec/caveat-take-back-0.1.md): `unqualify`, taking back a
  caveat while recording the take-back and keeping prior decision bases.
- [Citing](../../spec/caveat-citing-0.1.md) and
  [retention reasons](../../spec/caveat-retention-reasons-0.1.md): decisions
  with grounds but no numeric value (`citing`), and `retaining C because E`.
- [Reading notes](../../spec/caveat-reading-notes-0.1.md),
  [reading index](../../spec/caveat-reading-index-0.1.md),
  [earlier occurrence](../../spec/caveat-earlier-occurrence-0.1.md),
  [latest](../../spec/caveat-latest-0.1.md),
  [past revisions](../../spec/caveat-past-revisions-0.1.md),
  [in basis](../../spec/caveat-in-basis-0.1.md) and
  [changed triggers](../../spec/caveat-changed-triggers-0.1.md): addressing
  specific readings, occurrences and revisions, noting text on readings, and
  value-change reopening triggers.
- [Nested repetition](../../spec/caveat-nested-repetition-0.1.md),
  [holds](../../spec/caveat-holds-0.1.md),
  [foreclosure 0.1](../../spec/caveat-foreclosure-0.1.md) &
  [0.2](../../spec/caveat-foreclosure-0.2.md) and
  [strict irreversible](../../spec/caveat-strict-irreversible-0.1.md): nested
  `for` blocks, deferring reopenings with `hold`/`release`, irreversible and
  reversible foreclosure, and strict lineage caveat checks.
- [Outcomes](../../spec/caveat-outcomes-0.1.md): recording how a decision
  turned out (`resolve`, `resolved`, `outcome`, `materialized`).
- [Agents](../../spec/caveat-agents-0.1.md),
  [untrusted sources](../../spec/caveat-untrusted-sources-0.1.md),
  [carries any](../../spec/caveat-carries-any-0.1.md) and
  [only if](../../spec/caveat-only-if-0.1.md): event sender declarations,
  `withdraw every`, single-use `because once`, consequence-threshold caveat
  checks (`carries_any`), and lineage-only guards (`only_if`).
- [Across members](../../spec/caveat-across-members-0.1.md),
  [grounds of](../../spec/caveat-grounds-of-0.1.md),
  [reading caveats](../../spec/caveat-reading-caveats-0.1.md),
  [thresholds](../../spec/caveat-thresholds-0.1.md),
  [routed work](../../spec/caveat-routed-work-0.1.md) and
  [unset states](../../spec/caveat-unset-states-0.1.md): `sum`/`any`/`all`
  across members, `grounds_of(SERIES)`, conditional per-reading caveats,
  named thresholds, `work skips other routes;`, and `state … unset` / `is_set`.
- [Standard functions](../../runtime/prelude.cav): the prelude every
  program can call.

Background:

- [Caveat 0.1](../../spec/caveat-0.1.md): the original model the reactive
  language implements.
- [Essence](../../docs/CAVEAT_ESSENCE.md): the invariants any change to the
  language must keep.

## Agents handing work to agents

- [Handoff example](../examples/handoff/README.md): a main agent hands work
  to sub-agents with the ledger, each in a session scoped to its own events,
  and merges their work by replay.

## Retracted evidence

- [Retracted studies](../examples/retractions/README.md): the 2020
  Surgisphere retractions, where withdrawing each study flags every conclusion
  drawn from it and decisions already made keep what they rested on.

## History and long-running sessions

- [Integration starter](STARTER.md): a host for Node and browsers that
  checkpoints every accepted event, keeps the archive through store failures
  and restarts, and keeps a send's answers apart from the current assessment.
- [History integration](HISTORY.md): drain and retain the host archive,
  interpret exact versus conservative reports, and budget live and historical
  storage separately.
- [Windows](../../spec/caveat-windows-0.1.md),
  [lineage compaction](../../spec/caveat-lineage-compaction-0.1.md) and
  [departure](../../spec/caveat-departure-0.1.md): retirement, exact own
  grounds and records leaving a session.
- [Withdrawal collection](../../spec/caveat-withdrawal-collection-0.1.md):
  required reasons, conservative retention, historical report scope and
  transactional collection. The
  [collector draft](../../docs/design/withdrawal-collector-draft.md) keeps
  its evidence.

## The kit

- [Package README](../README.md): the `caveat` command, the session library,
  errors, use in a browser, and TypeScript and bundlers.
- [Scenarios 0.1](../../spec/caveat-scenarios-0.1.md): scenario files.
- [Dispatch outcomes 0.1](../../spec/caveat-dispatch-0.1.md): accepted,
  rejected and fatal outcomes, and rejection origins and codes.
- [Serve 0.1](../../spec/caveat-serve-0.1.md): `caveat serve`, one session
  driven by another program through JSON lines.
- [Agent evidence](../examples/agent-evidence/README.md): a Python caller for
  `caveat serve` that sends evidence, withdraws it and judges each attempt by
  the current assessment.
- [Engineering readiness](../examples/readiness/README.md): a release approval
  tied to one revision's test result, reopened by an authored policy when that
  result is qualified, run through the integration starter.
- [Check 0.1](../../spec/caveat-check-0.1.md): `caveat check`, advisory
  warnings about patterns worth a second look, and how to allow one on purpose.
- [Interface 0.1](../../spec/caveat-interface-0.1.md): a program's events,
  payload fields, displayed values and declared names, which `caveat types`
  turns into TypeScript declarations.
- [Dependents 0.1](../../spec/caveat-dependents-0.1.md),
  [Review 0.1](../../spec/caveat-review-0.1.md),
  [Ledger Sets 0.1](../../spec/caveat-ledger-sets-0.1.md) and
  [Find 0.1](../../spec/caveat-find-0.1.md): impact tracing, decision review
  across single or multiple ledgers, and searching past decisions by plain
  words (`caveat find` and `caveat_find`).

## Example

- [Thermostat](../../examples/thermostat_history.cav) and its
  [scenarios](../../examples/thermostat_history.scenarios.json): readings kept over
  time, and a decision revised on every reading while earlier revisions keep
  their basis.

For an agent integration, follow the [agent quickstart](AGENT_START.md): choose
commands for the installed version, test a small policy, and use the official
caller to keep required-operation success separate from assessment.

This package also provides the [local MCP authoring bridge](MCP.md).
