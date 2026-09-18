# CAVEAT Map — 0.1

**Status:** executable shared semantic map.

## Purpose

The CAVEAT Map is the machine-readable semantic center between source text and consumers such as the evaluator, validator, CLI, renderer, tests, and AI tools.

It is not merely a spatial map. It represents what the program knows, what relationships make that knowledge relevant, what decisions are available, what uncertainty is retained, and what execution state results from the source program.

Target architecture:

```
.cav source
  -> parse
  -> CAVEAT Map
       -> validator
       -> evaluator/session
       -> CLI
       -> renderer/world
       -> AI inspection + simulation tools
       -> CAVEAT-RS shared IR work
```

CAVEAT Map 0.1 starts with the epistemic/decision graph already earned by the language. World topology is intentionally added in the next CAVEAT 0.3 phase rather than inferred from scenario-specific Rust.

## Schema

A map contains:

- **symbols** — claims, evidence, caveats, and execution-created commitments;
- **relations** — supports, opposes, qualifies, retains, reopens;
- **investigations** — options and attention costs;
- **rules** — named inference dependencies;
- **choices** — options, retained uncertainty, and source-default selections;
- **conditionals** — effects gated by prior commitments;
- **actions** — normalized index of player/program actions and where they are offered;
- **execution** — deterministic default-path budget and commitment state;
- **world** — reserved structured world graph. In 0.1 it is empty unless supplied by future world declarations.

The JSON schema identifier is:

```
caveat-map/0.1
```

## AI-facing operations

The reference CLI exposes stable machine operations:

```
caveat-map map FILE.cav
caveat-map inspect FILE.cav SYMBOL
caveat-map trace FILE.cav SYMBOL
caveat-map actions FILE.cav
caveat-map validate FILE.cav
```

### map

Returns the complete JSON map.

### inspect

Returns one symbol plus every directly relevant relation, investigation, choice, and conditional reference.

This is intended to answer questions such as:

- What is `camera_has_blind_spot`?
- What does it qualify?
- Which choices retain it?
- Which reopening conditions refer to it?

### trace

Returns the dependency neighborhood reachable through epistemic relations from a symbol. The trace records depth and direction rather than forcing the caller to reverse-engineer source text.

### actions

Returns normalized action affordances from all choices, including:

- action id;
- display text when present;
- owning choice;
- uncertainty retained by that choice;
- conditional effects triggered by committing that action.

### validate

Parses, evaluates, and builds the map. Success means all CAVEAT 0.1/0.2 semantic references accepted by the runtime are coherent enough to map. Future CAVEAT 0.3 world invariants become part of this same operation.

## AI contract

An AI integration SHOULD consume the map before editing a non-trivial CAVEAT program.

The expected workflow is:

1. `validate`;
2. `map` or `inspect` the affected symbols;
3. `trace` dependencies before changing meaning;
4. edit;
5. `validate` again;
6. compare map state before/after;
7. only then run presentation/visual QA.

This makes the language's unknowns and dependencies explicit instead of relying on an AI to reconstruct them from unrelated Rust, JavaScript, and scenario files.

## World-map extension point

CAVEAT 0.3 will populate:

```
world.places
world.entities
world.connections
world.action_plans
```

The map, not `world3d.rs`, will then become authoritative for reachability, barriers, action preconditions, and physical outcomes.

A renderer may consume world-map data, but must not silently invent semantic structure absent from the map.

## Success criterion

The CAVEAT Map is useful when a tool can answer:

- what does the program know?
- why does it know it?
- what remains qualified or unresolved?
- what can happen next?
- what does a choice retain?
- what can reopen a commitment?
- what depends on this symbol?

without reading implementation-specific Rust or JavaScript.
