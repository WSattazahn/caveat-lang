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

`observed(...)` answers whether an evidence edge was reached, not whether a claim is true. `committed(...)` and `reopened(...)` describe the history and current status of a decision. These queries do not turn the graph into a single confidence score.

## Ordinary computation still has a place

Coordinates, geometry, audio frequencies, and interface layout can be ordinary deterministic values. Pure arithmetic functions and presentation bindings support the epistemic program. They do not make every number uncertain, claim to implement a truth oracle, or replace the graph with application flags.

Bindings display the state reached by the program. Emitted cues describe the effects of a successfully committed event. Neither form may mutate the evidence graph from browser code.

## The crosscurrent example

The rescue source defines a morning forecast with provenance and a caveat about an unmeasured surge. The initial navigation decision retains that caveat. A later observation can oppose the forecast and reopen the earlier decision. The revised counter-steering commitment also retains uncertainty.

The current's physical force is calculated independently of knowledge. Observing it changes the navigator's response, while the forecast, opposing observation, original commitment, revised commitment, and unresolved qualification all remain inspectable. The player sees the consequence through steering and brief visual feedback rather than a required reading panel.

The regression test compares legitimate input histories, checks the persistent graph, and separates external force from navigation compensation. It must fail if observation merely changes a journal flag or rewrites physical reality.

## Honest boundaries

The reactive runtime still uses Rust for parsing, validation, expression execution, transactions, and the graph. JavaScript still connects browser input, graphics, audio, and DOM elements to generic program outputs. Adding functions and source-authored presentation does not make Caveat self-hosting.

The success criterion is that a new behavior and its feedback can be authored in Caveat while preserving the invariants above. More `.cav` lines, fewer JavaScript lines, or a Rust-like surface syntax are not evidence of epistemic correctness by themselves.

## The Atlas as a design reference

The user-supplied *The Caveatist Atlas v1* distinguishes fictional cultural invention from doctrine. It reinforces a useful operational distinction: recording a qualification does not disprove it, and producing an objection does not by itself establish authority to stop a task. Examination concerns material consequences; stopping can permit action while uncertainty remains recorded. These ideas inform the checks above without converting fictional institutions or rituals into compiler requirements.
