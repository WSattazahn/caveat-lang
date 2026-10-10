# Caveat documentation

Everything you need to write, run and test a reactive Caveat program ships in
this package. Some documents link to files in the source repository; those are
background, not required reading.

## Start here

- [Getting started](GETTING_STARTED.md): install, write a first program, check
  it with a scenario file, and look inside a session.
- [Caveat on one page](REFERENCE.md): the language in brief. The commands,
  the core, the syntax, outcomes and common mistakes, with links to
  everything below.
- [Worked example](WORKED_EXAMPLE.md): one complete program taken through
  `validate`, `check`, `test` and `explain`. The package's tests follow it and
  check the output it shows.
- [Authoring guide](reference/docs/AI_AUTHORING.md): the authoring loop, numeric
  ranges, the session clock, evidence that ages and decisions that are
  reconsidered.

- [Project names and CLI commands](NAMES.md): the unambiguous `caveat-lang`
  command introduced in rc.7, the supported `caveat` shorthand and other projects with the
  same name.

## The reactive language

The language is specified as a base profile followed by additions. Read them in
order; each later profile assumes the earlier ones.

- [Reactive 0.1](reference/spec/caveat-reactive-0.1.md): claims, evidence,
  caveats, state, events, rules, commitments and reopening.
- [Reactive 0.2](reference/spec/caveat-reactive-0.2.md): pure functions,
  presentation bindings and emitted cues.
- [Reactive 0.3](reference/spec/caveat-reactive-0.3.md): numeric values that
  carry their evidence and caveats through computation.
- [Reactive 0.4](reference/spec/caveat-reactive-0.4.md): the source library
  and qualified text.
- [Reactive 0.5](reference/spec/caveat-reactive-0.5.md): readings and decision
  histories.
- [Reactive 0.6](reference/spec/caveat-reactive-0.6.md): computation over
  retained history.
- [Reactive 0.7](reference/spec/caveat-reactive-0.7.md): reusable procedures.

Features added alongside the profiles:

- [Define](reference/spec/caveat-define-0.1.md): named expressions,
  `define NAME = EXPRESSION;`.
- [Repetition](reference/spec/caveat-repetition-0.1.md): declarations repeated
  for each entity of a kind, `for KIND as $x { … };`.
- [Routed repetition](reference/spec/caveat-routed-repetition-0.1.md): a
  repetition block whose rules each run only for the member the event names,
  `for KIND as $x routed by P { … };`. Routing assistance, not isolation.
- [Procedure symbols](reference/spec/caveat-procedure-symbols-0.1.md):
  procedures that take evidence, other symbols and histories as parameters.
- [Typed parameters](reference/spec/caveat-typed-parameters-0.1.md): event
  parameters that name an entity or a member instead of a number.
- [Identifiers](reference/spec/caveat-identifiers-0.1.md): event parameters
  that carry text learned at run time, such as a commit SHA: `NAME id`.
- [Source text](reference/spec/caveat-text-0.1.md): statements, comments and
  quoting.
- [Reject](reference/spec/caveat-reject-0.1.md): refusing an event so nothing it
  did survives.
- [Explanations 0.1](reference/spec/caveat-explanations-0.1.md) and
  [0.2](reference/spec/caveat-explanations-0.2.md): `because` citations the
  runtime checks, and grounds as distinct from lineage.
- [Decision journal](reference/spec/caveat-decision-journal-0.1.md): the ordered
  record of every decision change.
- [Late qualification](reference/spec/caveat-late-qualification-0.1.md):
  caveats learned after the fact, `qualify EVIDENCE with CAVEAT;`.
- [Renewal](reference/spec/caveat-renewal-0.1.md): evidence that can be observed
  again, and caveats that arrive after a delay.
- [Withdrawal](reference/spec/caveat-withdrawal-0.1.md): recording that an
  observation is no longer stood behind, without erasing what rested on it,
  `withdraw E because R;`.
- [Permission](reference/spec/caveat-permission-0.1.md): what permitted a
  decision, recorded apart from what it rests on,
  `commit D … permitted by latest(approvals) for head;`.
- [Reopening triggers](reference/spec/caveat-reopening-triggers-0.1.md): a
  decision series declares which readings reopen it,
  `decisions merge limit 8 reopened by pushes;`.
- [State caveats](reference/spec/caveat-state-caveats-0.1.md): asking which
  caveats a retained value carries, and reopening because of them.
- [Observation order](reference/spec/caveat-observation-order-0.1.md): uses of
  evidence that are certain to fail are load-time errors.
- [Elapsed](reference/spec/caveat-elapsed-0.1.md): reading the session clock.
- [Save](reference/spec/caveat-save-0.1.md) and
  [view](reference/spec/caveat-view-0.1.md): saving and restoring a session,
  and the compact per-event view.
- [Standard functions](reference/runtime/prelude.cav): the prelude every
  program can call.

Background:

- [Caveat 0.1](reference/spec/caveat-0.1.md): the original model the reactive
  language implements.
- [Essence](reference/docs/CAVEAT_ESSENCE.md): the invariants any change to the
  language must keep.

## History and long-running sessions

- [History integration](HISTORY.md): drain and retain the host archive,
  interpret exact versus conservative reports, and budget live and historical
  storage separately.
- [Windows](reference/spec/caveat-windows-0.1.md),
  [lineage compaction](reference/spec/caveat-lineage-compaction-0.1.md) and
  [departure](reference/spec/caveat-departure-0.1.md): retirement, exact own
  grounds and records leaving a session.
- [Withdrawal collection](reference/spec/caveat-withdrawal-collection-0.1.md):
  required reasons, conservative retention, historical report scope and
  transactional collection. The
  [collector draft](reference/docs/design/withdrawal-collector-draft.md) keeps
  its evidence.

## The kit

- [Package README](../README.md): the `caveat` command, the session library,
  errors, use in a browser, and TypeScript and bundlers.
- [Scenarios 0.1](reference/spec/caveat-scenarios-0.1.md): scenario files.
- [Dispatch outcomes 0.1](reference/spec/caveat-dispatch-0.1.md): accepted,
  rejected and fatal outcomes, and rejection origins and codes.
- [Serve 0.1](reference/spec/caveat-serve-0.1.md): `caveat serve`, one session
  driven by another program through JSON lines.
- [Agent evidence](../examples/agent-evidence/README.md): a Python caller for
  `caveat serve` that sends evidence, withdraws it and judges each attempt by
  the current assessment.
- [Check 0.1](reference/spec/caveat-check-0.1.md): `caveat check`, advisory
  warnings about patterns worth a second look, and how to allow one on purpose.
- [Interface 0.1](reference/spec/caveat-interface-0.1.md): a program's events,
  payload fields, displayed values and declared names, which `caveat types`
  turns into TypeScript declarations.

## Example

- [Thermostat](reference/examples/thermostat_history.cav) and its
  [scenarios](reference/examples/thermostat_history.scenarios.json): readings kept over
  time, and a decision revised on every reading while earlier revisions keep
  their basis.

For an agent integration, follow the [agent quickstart](AGENT_START.md): choose
commands for the installed version, test a small policy, and use the official
caller to keep required-operation success separate from assessment.

This package also provides the [local MCP authoring bridge](MCP.md).
