# CAVEAT

**Code that knows why.**

Caveat is a programming language for programs that act on what they know and keep track of how they know it:

- **Values carry their evidence.** Every number computed from an observation carries that evidence and its caveats through every sum, comparison and label.
- **Explanations can't lie.** Anything a program shows can say *why*, and the runtime rejects an explanation that cites something that never counted.
- **Decisions remember.** A decision records what it was made on, reopens when the world disagrees, and keeps a journal.
- **Late knowledge is one line.** When you learn something after the fact, such as a bad reading or a faded memory, one line qualifies everything built on it. Decisions already made keep what they knew.

**[Play the glowcap explainer](https://wsattazahn.github.io/caveat-lang/glowcap.html).** Four look-alike mushrooms, a belief, a trust decision that is made, doubted and remade, and a "why?" under everything on the page. The rules and explanations all live in [`game/glowcap.cav`](game/glowcap.cav); the page only renders them.

**Trail Rescue** is a new, complete mechanic: spend three scout tokens, weigh
fallible reports, choose a tunnel and reconsider when its evidence changes or
ages. Its [requirements and 24 scenarios](experiments/trail-rescue/PROTOCOL.md)
were committed before implementation. The [Caveat program](game/trail_rescue.cav)
owns the rules; the [browser page](web/trail-rescue.html) displays its decisions
and frozen explanations. Build and run it with `npm run build`, `npm run serve`,
then open `http://127.0.0.1:4173/trail-rescue.html`.

The mechanic added [state caveat queries](spec/caveat-state-caveats-0.1.md):
`has_caveat(plan_basis, stale)` checks the actual grounds of a saved basis, and
`reopen route because caveated(plan_basis, stale)` cites the precise observations
that have gone stale. The journal now preserves elapsed time and the chosen
numeric value, and restore checks its consistency with commitments and evidence.
See the [implementation record](experiments/trail-rescue/RESULTS.md).

## What only Caveat does

| | Caveat | A general-purpose language |
| --- | --- | --- |
| **Explanations** | `bind label = "Could be a duskcap" when … because contradiction;` The runtime checks the citation: it may leave dependencies out, never add one. | A list you maintain by hand. Nothing checks it. |
| **Caveats through computation** | `qualified(1, taste, tasted_in_dark)` flows through every sum, condition, decision and label that uses it. | Thread a caveat list through every function. |
| **Late caveats** | `qualify taste with taste_faded;` reaches every current value built on the taste. Decisions made earlier keep theirs. | Find every derived value by hand, and remember to freeze the ones that were decisions. |
| **Decisions that remember** | `commit`, `reopen`, and revisions of a decision series. Each revision is grounded on exactly what it used, and `decision_journal` records them in order. | History arrays, basis snapshots and bookkeeping code. |
| **Grounds and lineage** | *Lineage* is everything that could have influenced a value. *Grounds* is what it is based on. Guards are control, never content, and grounds ⊆ lineage always. | One or the other, by convention. |
| **Atomic events** | `reject "already absorbed";` No partial update survives a rejected event. | Validate everything before the first write, by discipline. |

## The evidence: the glowcap benchmark

The [glowcap benchmark](experiments/glowcap/RESULTS.md) implements the same game beat twice, in TypeScript and in Caveat:

- a frozen, pre-registered scenario suite;
- twelve change requests, eight committed before either implementation changed for those rounds;
- seeded differential fuzz comparing accepted and rejected events, views, and save/resume behavior.

| Round | Change cost, Caveat vs TypeScript | Result |
| --- | --- | --- |
| 1 · first version | 55 vs 57 (CR1–4) | Parity. The weak spots became the language's work list. |
| 3 · blind requests | 97 vs 53 (CR5–8) | TypeScript. Caveat's lineage fought the designer's meaning. |
| 5 · replay after language changes | **71 vs 110** (CR1–8) | Caveat, with 134 lines against 164. Written knowing the requests. |
| 6 · blind requests | **92 vs 103** (CR9–12) | TypeScript. Caveat passed two of four phases on the first run, capped regrowth at eight lives, and exceeded the 1 ms event budget. |
| 6 · replay after language changes | **88 vs 103** (CR9–12 plus correction) | Lower change cost; mixed size (168 vs 233 lines, 12,701 vs 11,586 bytes). Three of four phases passed on the first run. Written knowing the requests. |

The earlier losses led to [grounded explanations](spec/caveat-explanations-0.1.md), [grounds](spec/caveat-explanations-0.2.md), [reject](spec/caveat-reject-0.1.md), [define](spec/caveat-define-0.1.md), [typed parameters](spec/caveat-typed-parameters-0.1.md), [the view](spec/caveat-view-0.1.md), [late qualification](spec/caveat-late-qualification-0.1.md) and the [decision journal](spec/caveat-decision-journal-0.1.md).

Round 6 led to [incremental evaluation](spec/caveat-incremental-evaluation-0.1.md), a load-time [observation-order check](spec/caveat-observation-order-0.1.md), [procedures that take evidence](spec/caveat-procedure-symbols-0.1.md), [renewable evidence and timed caveats](spec/caveat-renewal-0.1.md), and [save/restore without event replay](spec/caveat-save-0.1.md). The replay passes all 38 scenarios and a separate twelve-life regrowth check beyond the old eight-life cap. Its evidence still has a declared limit of 1,024 lives per mushroom: the request for unlimited regrowth remains unmet. Its first save-format attempt exceeded the size limit; that failed run remains in the results.

The replay's first 2,000-sequence fuzz found five disagreements in about 7.3 million events. JSON parsing changed some floating-point values by one unit in the last place, moving timed behavior across a boundary. Exact round-trip parsing fixed those timing differences. Final review then found the fuzz was hiding a decision-basis ordering bug by sorting that list. The adapter now uses the journal's ordered evidence, and an order-sensitive comparator checks it. That correction adds two changed lines to the original 86. The five captured sequences and both stricter 2,000-sequence fuzz runs now agree: 14,443,471 fuzz events with zero divergences. The blind round remains a TypeScript win, and this replay does not establish an adoption win under the same rule: size is mixed and only change cost is clearly better.

What Caveat still costs, honestly:

- 51.6 µs median per event plus view for the replay program, against 2.5 µs for TypeScript, measured after the fuzz;
- 482,716 gzipped bytes (about 483 KB) for the Caveat policy, adapter and runtime;
- rounds 4, 5 and the round-6 replay were written knowing the requests, so another blind round is needed to test generalization.

## A taste

```caveat
event absorb target kind mushroom, sort in glowcap duskcap;

for mushroom as $m {
    on absorb when $m_here and $m_consumed == 1 reject "already absorbed";
    on absorb when $m_here and sort == sort.glowcap
        set support = support + qualified(1, absorb_$m);
    on absorb when $m_here and sort == sort.duskcap and committed(trust)
        reopen trust because absorb_$m;
    on tick when $m_tasted_at >= 0 and now - $m_tasted_at >= 60
        qualify taste_$m with taste_faded;

    bind $m.label = "Probably a glowcap" when $m_unknown and probably_safe because support;
    bind $m.label = "Could be a duskcap — taste first" when $m_unknown and uncertain because contradiction;
};
```

Read [Why Caveat](docs/WHY_CAVEAT.md) for side-by-side code and the Caveatist way of working. To write your first program, start from the [authoring guide](docs/AI_AUTHORING.md).

## Status

CAVEAT 0.3 is executable, and the **0.4 game profile** adds executable knowledge requirements, evidence-dependent outcomes, and source-authored spatial presentation. The Rust reference runtime parses source, evaluates the epistemic graph, exposes the CAVEAT Map, and runs the same game program in a terminal or a WebAssembly browser session. The new features are specified in [Draft 0.4](spec/caveat-0.4-draft.md).

Other playable examples include **Light the Way**, a direct-control ferry rescue powered by the new reactive CAVEAT profile. Movement, scouting, collisions, damage, score, and outcomes are source rules. **The Last Beacon** remains a separate 3D island mystery whose available decisions, evidence, retained uncertainty, nine outcomes, locations, cameras, and paths are authored in `game/the_last_beacon.cav`. **Moon Garden** remains a short mobile-first mystery with both a 2D presentation and a separate **Moon Garden 3D** presentation driven by the same CAVEAT session. The earlier **The Door** scenario remains the world/action stress test.

**CAVEAT 3D 0.2** now consumes normalized Rust action-runtime executions (`Move`, `Inspect`, `Operate`, `Open`, `Observe`, `Stay`) and maps those commands to semantic place/entity/symbol presentation bindings. Moon Garden no longer needs normal per-action camera choreography. See `spec/caveat3d-0.2.md`. The remaining graphics problem is art-direction automation: semantic identifiers can now drive the scene, but attractive camera composition and assets still require authored presentation bindings.

## Light the Way — play immediately

[Play Light the Way](https://last-beacon-caveat.w4ltr0n.chatgpt.site). Hold and drag the lighthouse beam, or use the arrow keys. Steer the ferry around reefs into the green harbor. Hold the light over the current marker to take a reading while the ferry keeps moving. When the storm shifts, decide whether to correct the old plan manually or spend another second scouting. Damage, sampling progress, and rescued passengers appear directly in the scene. Retry immediately. No download or reading panels are required.

The new [reactive language profile](spec/caveat-reactive-0.1.md) adds bounded numeric state, typed events, arithmetic, ordered conditional rules, and atomic event execution. The same rules can read and change CAVEAT's actual evidence graph. In [the game source](game/light_the_way.cav), spotting a reef records evidence, pays to examine the associated caveat, reopens the initial course commitment, and makes the ferry slow near the known danger. The original chart claim and contrary evidence both remain in the graph.

```caveat
state boat_x = ferry.x min -9.5 max 13;
event tick dt min 0 max 0.1;
on tick when phase == 1 set boat_z = boat_z - boat_speed * dt;
```

The [reactive 0.2 extension](spec/caveat-reactive-0.2.md) adds reusable numeric functions, evaluated presentation bindings, explicit sound/visual cues, source-declared controls, and a source-declared clock. The rescue's screen transitions, feedback, visual state, and keyboard steering now live in the Caveat program. Cues publish only when their event commits; a failed calculation rolls back its graph changes and feedback together.

[Reactive 0.3](spec/caveat-reactive-0.3.md) carries evidence and caveats through numeric values, arithmetic, function calls, comparisons, and conditional decisions. A commitment made `using` a derived value automatically retains its caveats and records a frozen numeric basis plus `relies_on` evidence edges. The crosscurrent's source-authored observation and steering functions exercise these semantics; the browser continues consuming ordinary rendering values.

The crosscurrent demonstrates why that matters beyond moving code: a qualified forecast supports an initial navigation commitment; an opposing observation reopens it and leads to counter-steering while retaining the unresolved caveat. Observation changes the navigator's response, not the physical current. Both evidence histories remain inspectable. [Preserving Caveat's essence](docs/CAVEAT_ESSENCE.md) records the invariants and the remaining Rust/browser boundaries.

The current shifts twice during the crossing. A previously measured steering correction keeps its old value until a fresh sample supports a revised plan. Bright arrows show the surface flow; faint arrows preserve the last reading, and the HUD shows its age. Rescouting has an immediate cost: the same light must stay over the marker while the moving ferry still needs steering.

[Reactive 0.4](spec/caveat-reactive-0.4.md) adds bounded text expressions and conditional expressions. The standard functions `abs`, `min`, `max`, and `clamp` are implemented in [Caveat source](runtime/prelude.cav), replacing their Rust algorithms. Source functions also format the game's clock, numbers, and percentages; source state owns notice lifetimes and hull indicators. Rust still implements the evaluator and browser bridge. This is a first source-defined standard library, not a self-hosted compiler.

[Reactive 0.5](spec/caveat-reactive-0.5.md) adds bounded reading streams and decision series. Each `sample` creates a distinct evidence occurrence, even when the measured number repeats. `latest(flow)` reads its qualified value; a decision series records a new frozen basis after its previous decision is explicitly reopened. Old occurrences remain addressable in snapshots and graph relations. The game uses one sampling procedure and one navigation series across repeated weather changes, replacing its separate first-reading and storm-reading implementations.

The same capability is exercised by a [thermostat program](examples/thermostat_history.cav): cold, warm, and cold readings revise the heating command while preserving every reading and the unresolved calibration caveat. Run `cargo test --manifest-path runtime/Cargo.toml --test thermostat_history` to exercise its input history and rollback behavior. This is a separate source consumer of the generic language feature, with no thermostat logic in the interpreter.

[Reactive 0.6](spec/caveat-reactive-0.6.md) makes retained history available to source computation through `history_count`, zero-based `history_at`, and `fold_history`. Reducers are ordinary pure Caveat functions: summing, calculating a range, or choosing a history-based policy requires no corresponding Rust algorithm. Results retain the observations, caveats, and selection dependencies involved. Invalid indexes, reducer errors, and bounded-work failures roll back the whole event. The thermostat calculates mean/range/trend in source, while the game compares archived readings to report a measured flow reversal. This is bounded history computation, not yet arbitrary collections or self-hosting.

[Reactive 0.7](spec/caveat-reactive-0.7.md) adds reusable effect procedures. A `proc` can share input handling or a sequence of observation and revision steps across source events. Calls freeze their numeric arguments and entry guard, retain every argument's qualifications, and run within the calling event's atomic transaction. The game shares keyboard cleanup, held-input activation, and aim movement; the thermostat records and revises through a source procedure. Rust provides bounded calls and validation, while the behavior remains Caveat source.

[Explanations 0.1](spec/caveat-explanations-0.1.md) lets a binding say what it cites: `bind label.text = "…" when … because contradiction;`. The runtime checks the citation against the binding's lineage, so an explanation may leave dependencies out but can never cite evidence or a caveat the value and its conditions did not read. The full lineage stays available for audit.

[Explanations 0.2](spec/caveat-explanations-0.2.md) separates what a value is *based on* (its grounds) from everything that could have influenced it (its lineage). A rule’s guard, a skipped rule, the guard that revealed evidence, and a decision’s predecessor all stay in lineage but never enter grounds. Citations read grounds, `set x = e because c` narrows them, and grounds are always a subset of lineage.

[Reject 0.1](spec/caveat-reject-0.1.md) adds `reject "MESSAGE"`: an event that is not allowed fails atomically with that message, without a dummy state or a `require` trick.

[Define 0.1](spec/caveat-define-0.1.md) adds `define NAME = EXPRESSION;`, a named expression over state and the graph that is inlined wherever it is read, including per member inside a `for` block.

[View 0.1](spec/caveat-view-0.1.md) adds `dispatch_view`: the same transaction as `dispatch`, returning only what a host redraws after an event (bindings and their citations, cues, effects, commitments and their grounds, relations) as compact JSON.

[Typed Parameters 0.1](spec/caveat-typed-parameters-0.1.md) lets an event take an entity by name (`target kind mushroom`) or one of a list of names (`sort in glowcap duskcap`). The host sends names; the source reads positions, with `sort.duskcap` and `target.pool` as constants.

[Late Qualification 0.1](spec/caveat-late-qualification-0.1.md) adds `qualify EVIDENCE with CAVEAT`: a caveat learned after the fact reaches every current value built on that evidence, while decisions already made keep what they were made on.

[Decision Journal 0.1](spec/caveat-decision-journal-0.1.md) publishes every commitment and reopening, in order, with the evidence each was based on in the order it was observed.

[Observation Order 0.1](spec/caveat-observation-order-0.1.md) rejects, when a program loads, a rule that qualifies a value with evidence only a later rule of the same event can reveal: a use that can only fail. Uses that could succeed on some dispatch are left to the runtime.

[Procedure Symbols 0.1](spec/caveat-procedure-symbols-0.1.md) lets a procedure take evidence, a claim or a caveat by name (`proc learn(e evidence, sort)`), so every way of observing something can share one set of rules. Each call is specialized for the names it passes when the program loads.

[Renewal 0.1](spec/caveat-renewal-0.1.md) gives evidence an identity that events create: `renewable taste_cave limit 256;` and `renew taste_cave` make the name mean a new, unobserved occurrence while earlier ones keep what they were about. `qualify taste_cave with taste_faded after 60` fades that occurrence on its own clock, and `carries(taste_cave, taste_faded)` asks whether it has.

[Save 0.1](spec/caveat-save-0.1.md) saves a session and restores it without replaying events: `WebReactiveSession.save()` and `WebReactiveSession.restore(source, saved)`. Restoring costs what loading costs plus the size of the save. Restore validates the saved names, values and histories; mutation tests check that an altered save is either refused or remains playable without a crash.

[Incremental Evaluation 0.1](spec/caveat-incremental-evaluation-0.1.md) makes an event cost what it touches: a binding is evaluated again only when something it reads changed, and a transaction copies only what its effects write. Round 6's 32-entity program went from 1.1 ms to 0.16 ms per event with identical results, checked against full evaluation after every event in the test suite.

Water time and accumulated rain travel now come from Caveat state too. The source-defined `wrap` function keeps travel bounded; the renderer maps those values onto its existing wave shader and seeded rain geometry. Pausing or replaying a session preserves the corresponding weather pose.

The browser sends input and elapsed time to generic `WebReactiveSession`, then draws its snapshot. It does not calculate the ferry's movement, collisions, damage, route rules, or rescue result. Rust implements the language interpreter; the game-specific rules are Caveat. [Design notes](docs/LIGHT_THE_WAY.md) explain the controls and the source/runtime/renderer boundary.

Keyboard policy also lives in Caveat: source-declared physical-key controls
retain independent presses and releases, combine aliases and opposing keys,
and steer during source ticks. The browser forwards those input facts. A
source-only remapping changes the playable controls; observing, examining,
and reopening still follow the same qualified event rules.

Pure Caveat functions are callable through the [source library API](spec/source-library-0.1.md) from Rust and WebAssembly. The Door's movement timing and turning policy now live in its Caveat source, and The Last Beacon's feedback classification and copy use source functions. The same compiled evaluator serves both hosts. Full program validation and host transaction boundaries remain explicit.

For headless authoring, `caveat-reactive validate` checks a reactive program and `caveat-reactive replay` executes JSONL event histories with qualified snapshots. The [authoring guide](docs/AI_AUTHORING.md) includes commands and a thermostat input fixture. [Development insights](docs/DEVELOPMENT_INSIGHTS.md) separates demonstrated behavior from hypotheses about broader usefulness and AI adoption.

The [Slime glow ability policy](docs/SLIME_GLOW_ABILITY.md) is a small host-integration
consumer: a reported mushroom absorption creates one qualified learning receipt,
and ordinary ability toggles retain that basis without growing a per-input
history. Glow controls require a separate host-verified renderer capability;
without it, the source reports the mushroom discovery without claiming light.
Native and real WebAssembly tests exercise 10,001 ready toggles, early
discovery, duplicate absorption, reset and source-only policy variation. Vessel's
world contact, inventory transaction and visible glow remain host responsibilities.

## The Last Beacon — story experiment

On stormbound Saint Orin, thirty-two ferry passengers are approaching a failing lighthouse. Spend three watches investigating uncertain evidence, try provisional plans, reopen them when the world disagrees, and choose between restoring the light, sending the island pilot, or holding the ferry offshore until daylight. Six interactions produce 324 legal decision sequences and nine outcomes across three final destinations. Earlier investigations unlock targeted preparations; performing those preparations changes what the same final order accomplishes. See the [game design and action contracts](docs/THE_LAST_BEACON.md).

The game also develops the language runtime. The generic Rust `GameSession` and WebAssembly `WebGameSession` apply world actions and epistemic effects atomically, expose the graph reached by the player's actual decisions, and restore saves by replaying source-checked selections. The parser now supports quoted semicolons, escaped and Unicode text, line comments, and source locations in errors. These capabilities are available to any CAVEAT program; see the [game-session specification](spec/game-session-0.1.md) and [source-text specification](spec/caveat-text-0.1.md).

These are actual game-source statements:

```caveat
require bridge_reserve observed reserve_charge;
resolve relight_beacon as beacon_guided when observed measured_pulses;
resolve relight_beacon as beacon_limited when observed hot_cable;
resolve relight_beacon as beacon_limited when observed split_beam;
resolve relight_beacon as beacon_unverified otherwise;
```

The runtime checks the reached evidence graph before an action. Declared but undiscovered evidence cannot unlock it; contradictory evidence remains recorded. The browser receives available choices, blocked reasons, and the selected outcome from CAVEAT. It supplies reusable graphics and controls. [Presentation declarations](spec/presentation.md) also place the world and compose its views without changing JavaScript.

[Play the earlier story](https://last-beacon-caveat.w4ltr0n.chatgpt.site/last-beacon.html). This hosted copy opens directly in your browser; downloading an HTML file is optional.

### Build and play

With rustup and Node.js 20 or newer installed, run from the repository root. `rust-toolchain.toml` pins the compiler, and rustup installs it with the WebAssembly target on first use:

```sh
cargo install wasm-bindgen-cli --version 0.2.104 --locked
npm ci
npm run build
npm run serve
```

The build also writes `dist/pkg-reactive/`: the same runtime without the sequential, graphics and 3D sessions, for hosts that only run reactive programs (`cargo build --no-default-features`). It is about a fifth smaller.

Open [Light the Way locally](http://127.0.0.1:4173/rescue.html). The earlier [story experiment](http://127.0.0.1:4173/last-beacon.html) remains available. The build compiles the Rust runtime to WebAssembly and assembles `dist/` with game sources, browser assets, and a local copy of Three.js. Existing games remain available in the same build.

To play the earlier Last Beacon story in a terminal, without a browser or JavaScript:

```sh
cargo run --manifest-path runtime/Cargo.toml --bin caveat -- --game game/the_last_beacon.cav
```

For an optional offline copy, run `npm run package:game` after building. Open `dist/Light-the-Way.html` in a full modern browser (`dist/The-Last-Beacon.html` contains the earlier story): it embeds the actual CAVEAT WebAssembly runtime, story, and graphics and works without a server or network connection. Some file-preview applications disable scripts or module imports; those previews cannot run the game. The launch screen now includes browser guidance, a retry action, and a timeout for failed or stalled imports. In the earlier story game, device-local saves preserve your watch. Choose with the buttons or keys **1–3**, open the journal with **J**, and toggle ambient sound with **M**.

The [workflow notes](docs/WORKFLOW.md) explain how the video's visual inspection and testing loop informed this implementation.

### Verify

```sh
cargo test --manifest-path runtime/Cargo.toml
cargo clippy --manifest-path runtime/Cargo.toml --all-targets -- -D warnings
npx playwright install chromium
npm run test:beacon
npm run test:rescue
npm run test:slime-glow
```

Reactive runtime tests check atomic rollback and malformed input, and rescue simulations verify a complete crossing, damage, timing, and the direct effect of observed evidence on motion. Browser rescue tests use real mouse, keyboard, and touch input. The story tests exhaust legal Beacon routes, checking unavailable-action rejection, investigation costs, retained caveats, reopening, physical destinations, evidence-dependent outcomes, and save restoration. Browser tests exercise the WebAssembly game and its presentation, starting their own local server. Run `npm run build` first; a separate `npm run serve` process is not required for tests. After `npm run package:game`, `npm run test:launch` also exercises blocked imports, a stalled module, and disabled-script previews. `npm run test:slime-glow` needs no browser: it drives the installed WebAssembly runtime directly and checks the host-integration policy's bounded toggles, verified renderer capability, qualified escape, atomic failure, and malformed input. It runs in the same continuous integration job that assembles `dist/`.

## Repository layout

- `spec/` — versioned language specifications
- `runtime/` — reference runtime
- `runtime/prelude.cav` — source-defined standard math and display functions
- `tests/` — canonical semantic tests
- `examples/` — example CAVEAT programs
- `game/` — playable CAVEAT scenarios, including The Last Beacon, Moon Garden, and The Door
- `web/` — browser presentations and generic world rendering
- `scripts/` — reproducible browser build, local server, and game verification

> Do not save CAVEAT by redefining it. If the computational model collapses into an existing paradigm, record the result.
