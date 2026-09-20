# Findings from developing Caveat

This is a record of observations, design judgments, and open hypotheses. Tests
support implementation claims. Proposed applications and advantages remain
hypotheses until measured against alternatives.

## Acting with unresolved qualifications is executable

Light the Way separates the physical current, a dated observation of it, and a
navigation commitment made using that observation. Weather can change while
the old correction keeps acting. Taking another reading costs steering time;
reopening and revising a decision preserves the earlier basis.

The observed result is that these distinctions change a playable simulation.
The general hypothesis is that they also help software whose decisions depend
on changing reports. The thermostat is a second source consumer, but two
programs do not establish broad practical utility.

Evidence: `runtime/tests/light_the_way.rs`, `runtime/tests/reactive_history.rs`,
and `runtime/tests/thermostat_history.rs`.

## Preserving a dependency differs from proving relevance

Arithmetic cancellation, unused function arguments, and skipped conditional
effects exposed ways an implementation could lose the qualifications of an
evaluated input. Caveat deliberately retains those dependencies. Procedure
calls extend that contract across reusable effect sequences.

This is conservative computational lineage. It does not prove that each
retained item is a necessary cause, that an observation is accurate, or that a
policy is sensible. A useful explanation interface will need to distinguish
current selection dependencies from frozen historical bases, and help readers
find relevant records without silently deleting the complete record.

Evidence: `runtime/tests/qualified_values.rs`,
`runtime/tests/history_computation.rs`, and
`runtime/tests/reactive_procedures.rs`.

## Reuse makes evaluation timing visible

Extracting repeated rules into a procedure is not automatically equivalent.
Separate event rules reevaluate their guards. A procedure accepts its outer
guard once, freezes arguments, then executes body guards against the state
reached so far. The keyboard migration kept its final key-bit write separate
because stale-release handling depends on that order.

The insight is that language-level reuse needs an explicit temporal contract.
Tests must cover when values are captured as well as their final numbers.
Frozen readings and decision bases make this especially significant in Caveat.

Evidence: commit `b2da29e`, `runtime/tests/reactive_procedures.rs`, and
`runtime/tests/light_the_way_input.rs`.

## A validation tool must state what it validates

The older map tool accepts declared structure without executing reactive
directives. That was insufficient for an agent trying to check a generated
reactive program. The new headless runner invokes the actual reactive loader
and dispatcher, publishes only successful event snapshots, and identifies the
last committed sequence on failure. Map validation now labels its narrower
scope explicitly.

The same lesson applies to a source-function library: extracting and compiling
pure functions does not validate every declaration in the containing program.
Program callers still need their own runtime's validation. Transport must also
preserve invalid duplicate fields long enough for the validator to reject them.

Evidence: `runtime/tests/reactive_cli.rs` and the
[authoring guide](AI_AUTHORING.md).

## Host integration can be a useful stage of language adoption

Rust and JavaScript now call compiled Caveat functions for authored policies.
The Door uses source functions for motion timing and turning behavior. The
Last Beacon uses them for feedback classification and copy. Existing functions
and the same evaluator serve this boundary without creating a dummy session.

Native source-variation tests change actual Door animation events, while
browser tests change feedback without changing the committed game graph. The
hosts retain transport, geometry, rendering, and event construction. Count
removed policy separately from added generic runtime code.
A larger interpreter can still move application policy into source, but a line
count alone cannot demonstrate improved correctness or usability.

Pure host arguments may carry caller-supplied provenance. Preserving that
metadata does not authenticate it. Display-only projections must not be fed
back as authoritative, unqualified evidence.

## A first documentation-only AI authoring result

A fresh agent produced a ventilation controller from the documentation, with
one candidate and no source revisions. Validation and behavioral replays
passed; exceeding its history capacity failed as intended. Independent tests
then checked all 256 four-event combinations of three readings and an outage,
including immutable histories and recovery qualifications. The source remains
the agent's first attempt. Details and limitations are in the
[pilot record](AI_AUTHORING_PILOT.md).

This supports documentation sufficiency for one specified task. It does not
measure comparative efficiency, general authoring reliability, or adoption.
The recovered decision's inherited outage reasons also make the need to
distinguish historical dependencies from present conditions more concrete.

## A learning receipt need not archive every control press

The first Slime glow policy separates one qualified acquisition from repeated
ordinary ability toggles. The learning commitment retains the absorption report
and its qualification; turning the ability off and on preserves that basis
without creating new evidence for each button press. Ten thousand and one
toggles in both native and real WebAssembly tests leave the graph at six symbols
and five relations. No history eviction or new interpreter feature was needed.

This resolves a practical integration concern: persistent evidence does not
require treating all player input as accumulating observations. The author must
still decide what genuinely constitutes new evidence. A round reset is a new
session, not deletion of inconvenient qualifications from the same history.
The [policy contract](SLIME_GLOW_ABILITY.md) distinguishes interpreter atomicity
from the host's inventory/absorption transaction. The first live Vessel consumer
now runs that source after actual authored mushroom contact: the existing
absorption animation and inventory pickup are followed by a qualified learning
receipt, keyboard/button toggles and a journal entry. Browser review also checks
pause, modal and repeated-key behavior. Execution occurs at those event
boundaries, not on every physics/render tick.

Integration exposed a second transaction boundary. Caveat can reject its own
event atomically, but cannot undo an inventory write performed by the host.
Vessel stages this one-time learning decision before committing the pickup. A
full bag discards that candidate, while a failed policy never attempts the
inventory write; the world mushroom remains available. Installed-WASM tests
exercise these boundaries and round changes during callbacks. The candidate
can reconstruct this specific source's small pre-learning context; arbitrary
program history cannot be discarded by the same argument.

This demonstrates a real source-owned progression policy inside an existing
engine. It does not yet demonstrate the complete ability's appearance: protected
light/material changes await separate approval. The policy's `active` flag and
HUD text are not evidence of emitted light, and no performance or comparative
language advantage is inferred from this integration.

## What we have not established

- That unfamiliar AI agents can author Caveat reliably across varied tasks.
- That Caveat outperforms an equivalent library in an established language.
- That its complete provenance graphs stay understandable in long programs.
- That current execution and history bounds fit production workloads.
- That the language is self-hosting or independently verifies evidence quality.

The next useful measurements are authoring/repair success, explanation accuracy,
metadata growth, and runtime cost on identical tasks. Keep failing cases as
evidence rather than reporting only the successful demo.
