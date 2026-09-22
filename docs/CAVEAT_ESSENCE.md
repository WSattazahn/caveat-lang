# Preserving Caveat while expanding the language

Caveat's identity is the persistent epistemic graph described in the [original specification](../spec/caveat-0.1.md): claims, evidence with provenance, qualified uncertainty, provisional commitments, and reopening without erased history. Replacing host-language code is useful only when that model remains intact.

## What must remain true

- Supporting and opposing evidence may coexist. A new reading does not silently delete the old forecast.
- Evidence retains its source. Declaring evidence does not make it observed.
- A commitment selects an action without asserting that its assumptions are certainly true.
- Retained caveats remain attached when a commitment reopens. The cause of reopening remains inspectable.
- Examination spends the program's attention budget. Presentation cannot bypass that cost.
- A failed event publishes none of its numeric changes, graph changes, or presentation cues.
- Knowledge can change an actor's response. It must not retroactively change the external physical condition being observed.
- An explanation may cite less than a value depended on, never more. The runtime rejects a citation its binding did not read; the complete lineage remains inspectable.
- Lineage records what could have influenced a value; grounds record what it is based on. Control (guards, skipped rules, revealing guards, predecessor decisions) belongs to lineage only, and grounds never exceed lineage.

`observed(...)` answers whether an evidence edge was reached, not whether a claim is true. `committed(...)` and `reopened(...)` describe the history and current status of a decision. These queries do not turn the graph into a single confidence score.

## Ordinary computation still has a place

Coordinates, geometry, audio frequencies, and interface layout can be ordinary deterministic values. Pure arithmetic functions and presentation bindings support the epistemic program. They do not make every number uncertain, claim to implement a truth oracle, or replace the graph with application flags.

Bindings display the state reached by the program. Emitted cues describe the effects of a successfully committed event. Neither form may mutate the evidence graph from browser code.

Physical key presses are ordinary input facts. Light the Way now interprets
their aliases, held states, opposing directions, and steering in Caveat source.
The browser forwards declared press/release events; it does not decide what a
key means. Holding Space still requires a complete source-timed observation
before a reading exists. Releasing input or pausing clears held keys without
erasing observations, retained caveats, or earlier commitment bases. This moves
application policy into Caveat without treating every input as epistemic evidence.

## Qualifications survive computation

[Reactive 0.3](../spec/caveat-reactive-0.3.md) makes qualified numeric values part of execution. Arithmetic and reusable functions carry their evidence and caveats. Comparisons preserve those dependencies, and conditional writes retain the qualifications involved in choosing the result. Multiplying by zero or passing an argument to a function that ignores it cannot silently erase its evaluated dependency.

A commitment made `using` such a value records the number and its provenance at that moment, automatically retains its caveats, and adds `relies_on` evidence edges. A later assignment can replace the current estimate without rewriting the earlier decision. The original evidence source remains in the graph. Ordinary browser values are projections of this state; they are not fed back as stripped authoritative values.

Conditional graph effects preserve their dependencies too. Revealing, examining, reopening, or deciding to skip one of those effects cannot be used as an intermediate step to erase a qualified condition. Query metadata records dependencies on absence without falsely marking unseen evidence as observed.

This is dependency preservation, not a truth oracle. The author chooses how evidence informs a measurement. The runtime enforces observation availability and retained qualifications; it does not independently certify sensor accuracy or evidence quality.

## Repeated observations remain distinct

[Reactive 0.5](../spec/caveat-reactive-0.5.md) gives each successful sample a fresh occurrence identity and frozen numeric value. A declared sensor is a source template; declaring it does not make it an observation. Equal-valued samples remain separate occurrences. Taking another reading never relabels the provenance already attached to a saved value or an earlier decision.

A decision series can select a new revision only after its current decision has explicitly reopened. Every revision retains its numeric basis, caveats, predecessor, and actual evidence links. Reading the latest value selects a particular occurrence; it does not rewrite the archived record. A new decision can depend on the reason its predecessor reopened as well as its fresh measurement. Those are computational dependencies, not a statement that every earlier reading is still physically current.

Skipped sampling and revision guards also affect which record remains current. Their dependencies belong to the current selection, separate from immutable archived values. A missing sample is still missing. Failed transactions consume no occurrence IDs and publish no partial history. Capacity limits reject an event atomically rather than evicting evidence silently.

## Computation over archives

[Reactive 0.6](../spec/caveat-reactive-0.6.md) allows source-defined computation
over these archives. A fold's reducer is ordinary Caveat code; Rust supplies
bounded traversal. Counting and selecting records preserve the qualifications
behind membership and indexing, and every visited record remains a dependency
even if a reducer ignores it. Means, ranges, and trends describe authored
interpretations of recorded observations. They do not discharge calibration
caveats, turn past observations into present truth, or erase conflicting edges.
The thermostat tests demonstrate that changing only a Caveat policy can change
the control decision without altering the observations it was based on.

## Qualifications survive reusable effects

[Reactive 0.7](../spec/caveat-reactive-0.7.md) extends reuse to effect procedures.
A call freezes its entry arguments and guard, retaining all their evidence and
caveats through descendant effects even when an argument is ignored. Body
guards can respond to earlier effects, but cannot shed the call's inherited
basis. A skipped call retains the evaluated guard's reasons on possible effect
targets without evaluating untaken arguments, inventing observations, or
spending attention. Nested calls share one event transaction and work budget.
Any late failure rolls back history, graph changes, cues, and occurrence IDs.

The game uses this facility for ordinary control policy; the thermostat uses
it for observation, archive computation, and explicit decision revision. The
same mechanism serves both without an application-specific Rust operation.
It does not independently determine whether an authored policy is warranted.

## The crosscurrent example

The rescue source defines a morning forecast with provenance and a caveat about an unmeasured surge. The initial navigation decision retains that caveat. A later observation can oppose the forecast and reopen the earlier decision. The revised counter-steering commitment also retains uncertainty.

The current's physical field is calculated from actual strength, position, and time. Observing it supplies a qualified measurement that source functions convert into a retained steering plan. The forecast, opposing observation, original commitment, revised commitment, and unresolved qualification all remain inspectable. A later change in actual strength need not match the retained estimate. The player sees the consequence through steering and brief visual feedback rather than a required reading panel.

The regression test compares legitimate input histories, checks the persistent graph, and separates external force from navigation compensation. It must fail if observation merely changes a journal flag or rewrites physical reality.

The storm crossing makes the distinction playable. Visible weather changes can reopen the current steering commitment without supplying perfect new measurements. The player may keep correcting an old plan manually, or hold the light over the current to take a fresh sample while the ferry keeps moving. One sampling procedure appends readings and one navigation series retains successive decisions. Faint last-reading arrows can disagree with the bright surface-flow arrows. Earlier numbers and decision bases remain intact, and later readings still retain the caveat that conditions can change again.

This separation is a design invariant enforced by the game's source and regression tests. The language does not independently decide which authored numeric variables represent external physics.

## Honest boundaries

The [source function library](../spec/source-library-0.1.md) lets native Rust
and browser hosts call the same compiled Caveat functions. All numeric inputs
are checked, and native tracked inputs retain their qualifications even when
ignored. Caller-supplied provenance is not automatically authenticated evidence.
The library's function validation also does not validate an entire surrounding
game; each program still needs its corresponding session checks.

Host transactions remain explicit. Door motion policy is inside its staged
action, so a failed call publishes no partial movement. Last Beacon feedback
is display policy applied after a committed action. If that display fails,
the interface must preserve and accurately report the recorded action instead
of replaying it or claiming that it rolled back.

The reactive runtime still uses Rust for parsing, validation, expression execution, transactions, and the graph. JavaScript still connects browser input, graphics, audio, and DOM elements to generic program outputs. Adding functions and source-authored presentation does not make Caveat self-hosting.

The source-defined prelude moves the algorithms for `abs`, `min`, `max`, and `clamp` out of Rust and into Caveat functions. Formatting policies for clocks, rounded numbers, and percentages also live in Caveat. A lazy conditional retains the condition and the branch actually evaluated; ordinary function arguments still contribute their dependencies even when the function body ignores them. Turning a qualified number into display text must preserve its qualification metadata.

Supporting text, conditional expressions, and history records extends the Rust evaluator, so moving source algorithms does not imply a net reduction in Rust lines. Parsing, transactions, graph storage, primitive numeric/text operations, and WebAssembly remain Rust. General modules and a compiler written in Caveat remain future work.

The thermostat history example tests the same observation and revision facilities outside the game. It demonstrates reuse, but it does not establish that Caveat is broadly useful in production. That requires more applications, user experience, and measurement.

The success criterion is that a new behavior and its feedback can be authored in Caveat while preserving the invariants above. More `.cav` lines, fewer JavaScript lines, or a Rust-like surface syntax are not evidence of epistemic correctness by themselves.

## The Atlas as a design reference

The user-supplied *The Caveatist Atlas v1* distinguishes fictional cultural invention from doctrine. It reinforces a useful operational distinction: recording a qualification does not disprove it, and producing an objection does not by itself establish authority to stop a task. Examination concerns material consequences; stopping can permit action while uncertainty remains recorded. These ideas inform the checks above without converting fictional institutions or rituals into compiler requirements.
