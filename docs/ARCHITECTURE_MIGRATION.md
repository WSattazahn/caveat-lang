# CAVEAT architecture migration plan

Status: approved implementation plan. This document is the gate for the refactor; scenario-specific renderer/Rust patches should not bypass it.

## Goal

Make **CAVEAT programs authoritative** for reasoning, world structure, and consequences. Rust remains a small portable reference runtime; the browser remains a generic renderer/player. The Door becomes a consumer/stress-test of the language, not hard-coded engine behavior.

Target pipeline:

```
.cav source
  -> parse + semantic validation
  -> CAVEAT program model
  -> epistemic state + world state + action plan
  -> generic runtime event stream
  -> CLI / tests / browser renderer
```

No layer may contain a special case for `open`, `stairwell`, `retreat`, or The Door.

## Architectural boundaries

### CAVEAT source owns
- claims, evidence, provenance, supports/opposes/qualifies;
- caveats, consequence and attention cost;
- investigations and revealed evidence;
- choices, commitments, retained uncertainty and reopening;
- world entities and their semantic kinds;
- containment/parenting and connections;
- named locations;
- action sequences and their physical preconditions/effects;
- observations produced by actions;
- player-facing labels/outcome feedback.

### Rust reference runtime owns
- parsing and AST construction;
- symbol resolution and type/semantic validation;
- epistemic graph evaluation;
- attention budget;
- commitment/reopening state;
- validation of world/action references;
- conversion of valid action plans into **generic** events.

Rust must not own coordinates or behavior for a named scenario.

### Renderer owns
- rendering generic entity kinds;
- transforms/hierarchy;
- generic animation of events such as rotate, move path, look, visibility;
- camera/presentation;
- input/UI generated from pending CAVEAT actions.

Renderer must not infer that an action named `stairwell` means a particular camera position.

## Language surface: staged design

Do not implement all syntax at once.

### Phase A — declarative world graph

Add source forms sufficient to declare:
- `place <id> kind <kind>`
- `entity <id> kind <kind> at <place>`
- `contains <place> <entity-or-place>`
- `connect <from> to <to> via <entity>`
- entity transform/presentation properties where needed.

Semantic validation:
- every referenced symbol exists;
- entity/place identifiers are unique;
- containment has no cycles;
- connections reference compatible places/entities;
- interactive barriers used as connectors exist.

Milestone: The Door's topology is represented in `.cav`, but behavior remains unchanged.

### Phase B — generic physical action plans

Add reusable verbs/action-plan forms for:
- `look`
- `move` / `move_path`
- `operate`
- `open`
- `close`
- `through`
- `observe`
- ordered `sequence`.

The compiler resolves these to generic runtime events. Scenario action names bind to plans in source.

Validation examples:
- `through door_a` requires a connection using `door_a`;
- an `open` target must have an openable/door capability;
- movement endpoints must be reachable in the declared topology;
- an action cannot claim a stair destination unless the route reaches a place of kind `stairwell`;
- sequential traversal through a closed barrier must include opening/clearance first.

Milestone: `open`, `continue`, `retreat`, and `stairwell` disappear from Rust match statements.

### Phase C — reasoning-to-world binding

Allow action plans to:
- reveal evidence/observations;
- create or reopen commitments;
- retain caveats;
- expose revised choices only after required world/epistemic state.

The same action is responsible for both its reasoning effect and physical plan, preventing UI text/world behavior divergence.

Milestone: opening Door A physically exposes `smoke_report`, which reopens the existing commitment without resetting Door A.

### Phase D — source-authored presentation

Move scenario-specific:
- labels;
- signs/text;
- environment entity definitions;
- material/presentation hints;
out of `web/3d.html` and into CAVEAT/world data.

Renderer may have generic visual templates for kinds such as corridor, fire_door, stair, handrail, sign, camera. A scenario selects/configures templates; it does not receive handwritten JS geometry.

Milestone: another CAVEAT scenario can create a door/corridor/stairwell without editing JS or Rust.

## The Door source model

Required topology:

```
start_corridor
  -- door_a -->
cross_corridor
  --> continuation_corridor
  -- stair_door_b --> stair_landing --> stair_flight_down
  -- door_a --> start_corridor

start_corridor --> alternate_route   # for reroute
```

Required branch contracts:

| Choice | Epistemic effect | Physical sequence | End state |
|---|---|---|---|
| inspect latch | reveal maintenance evidence | no movement | start corridor |
| check camera | reveal coverage evidence | no movement | start corridor |
| wait | retain uncertainty / receive timed info if specified | no teleport | start corridor |
| reroute | commit alternate route | turn + traverse modeled alternate route | alternate route |
| open | provisional commitment; action exposes new evidence | operate bar -> open A -> cross threshold | cross corridor |
| continue | retain/revise commitment | forward traversal | continuation corridor |
| retreat | revise commitment | turn -> approach A -> traverse A | start corridor |
| stairwell | revise commitment | approach B -> operate/open B -> cross -> landing -> descend | stair flight |

## Compiler invariants

A program is invalid if:
1. a player-facing action has no semantic effect or no action plan;
2. feedback names a destination/action not represented by its plan;
3. a route endpoint is unreachable from the action's precondition;
4. a traversal crosses a closed barrier without an opening step/capability;
5. an action claims a stairwell but its endpoint is not in a stairwell topology;
6. two distinct routes accidentally alias an endpoint unless explicitly declared to converge;
7. an observation references evidence not declared in the epistemic graph;
8. a reopen references a commitment that cannot exist on that execution path.

These are language errors, not visual QA discoveries.

## Test strategy

### Parser tests
Each new source form has success and failure fixtures.

### Semantic validator tests
Test every invariant above with minimal programs. Invalid programs must fail before runtime/rendering.

### Runtime conformance
No test should inspect The Door-specific Rust branches. Tests assert generic plans/events from source.

### Canonical scenario matrix
The Door must exercise every exposed route independently:
- inspect latch
- check camera
- wait
- reroute
- open -> continue
- open -> retreat
- open -> stairwell

For each route assert:
- epistemic state;
- commitment state;
- current logical place;
- barrier state;
- expected ordered events;
- player feedback.

### Visual QA
Only after semantic/runtime tests pass:
- screenshot meaningful stages for every route;
- inspect destinations, barriers, signs and spatial coherence;
- image variance alone is never a pass condition.

## Migration order

1. Restore green baseline after schema-4 syntax/compiler failures.
2. Add this plan and freeze new scenario-specific behavior.
3. Introduce world AST/data types independent of renderer.
4. Parse Phase-A world declarations.
5. Add semantic validator + negative fixtures.
6. Migrate The Door topology into `.cav`.
7. Introduce generic action-plan AST and parser.
8. Compile plans to generic runtime events.
9. Migrate The Door actions from `world3d.rs` to source.
10. Bind observations/reopening to action plans.
11. Move scenario-specific environment/presentation from HTML to source/templates.
12. Delete The Door-specific Rust and JS branches.
13. Run full semantic + branch + visual matrix.
14. Only then call the canonical demo ready.

## Change discipline

Before each implementation phase:
1. state the invariant/milestone;
2. add or update tests that define it;
3. implement the smallest general capability;
4. run strict format/lint/tests;
5. do not proceed if the gate is red;
6. inspect visual output only after semantic correctness.

No empty commits, no weakening lint/tests, no hard-coded patch whose only justification is making The Door look right.

## Definition of architectural success

We are finished with this migration when:
- The Door's meaningful nouns, topology, reasoning and action consequences live in CAVEAT source;
- changing a route does not require Rust/JS edits;
- adding a second comparable scenario does not require Rust/JS edits;
- invalid physical/semantic combinations are rejected before rendering;
- Rust is mostly parser/evaluator/validator/event infrastructure;
- the browser is mostly generic rendering/interaction infrastructure;
- CAVEAT itself is visibly responsible for the distinctive game behavior.
