# CAVEAT

An experimental programming language for computation with claims, evidence, caveats, provisional commitments, and reopening.

CAVEAT explores a persistent epistemic graph rather than reducing every computation immediately to a single settled value. Contradictory positions may coexist; evidence carries provenance; commitments can retain unresolved caveats; later evidence can reopen earlier commitments.

## Status

CAVEAT 0.3 is executable, and the **0.4 game profile** adds executable knowledge requirements, evidence-dependent outcomes, and source-authored spatial presentation. The Rust reference runtime parses source, evaluates the epistemic graph, exposes the CAVEAT Map, and runs the same game program in a terminal or a WebAssembly browser session. The new features are specified in [Draft 0.4](spec/caveat-0.4-draft.md).

The newest playable proof-of-use is **The Last Beacon**, a 3D island mystery whose available decisions, evidence, retained uncertainty, nine outcomes, locations, cameras, and paths are authored in `game/the_last_beacon.cav`. **Moon Garden** remains a short mobile-first mystery with both a 2D presentation and a separate **Moon Garden 3D** presentation driven by the same CAVEAT session. The earlier **The Door** scenario remains the world/action stress test.

**CAVEAT 3D 0.2** now consumes normalized Rust action-runtime executions (`Move`, `Inspect`, `Operate`, `Open`, `Observe`, `Stay`) and maps those commands to semantic place/entity/symbol presentation bindings. Moon Garden no longer needs normal per-action camera choreography. See `spec/caveat3d-0.2.md`. The remaining graphics problem is art-direction automation: semantic identifiers can now drive the scene, but attractive camera composition and assets still require authored presentation bindings.

## The Last Beacon

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

[Play the hosted game](https://last-beacon-caveat.w4ltr0n.chatgpt.site). This hosted copy opens directly in your browser; downloading an HTML file is optional.

### Build and play

With stable Rust and Node.js 20 or newer installed, run from the repository root:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.104 --locked
npm ci
npm run build
npm run serve
```

Open [The Last Beacon locally](http://127.0.0.1:4173/last-beacon.html). The build compiles the Rust runtime to WebAssembly and assembles `dist/` with game sources, browser assets, and a local copy of Three.js. Existing games remain available in the same build.

To play the same Caveat program in a terminal, without a browser or JavaScript:

```sh
cargo run --manifest-path runtime/Cargo.toml --bin caveat -- --game game/the_last_beacon.cav
```

For an optional offline copy, run `npm run package:game` after building. Open `dist/The-Last-Beacon.html` in a full modern browser: it embeds the actual CAVEAT WebAssembly runtime, story, and graphics and works without a server or network connection. Some file-preview applications disable scripts or module imports; those previews cannot run the game. The launch screen now includes browser guidance, a retry action, and a timeout for failed or stalled imports. Device-local saves preserve your watch. Choose with the buttons or keys **1–3**, open the journal with **J**, and toggle ambient sound with **M**.

The [workflow notes](docs/WORKFLOW.md) explain how the video's visual inspection and testing loop informed this implementation.

### Verify

```sh
cargo test --manifest-path runtime/Cargo.toml
cargo clippy --manifest-path runtime/Cargo.toml --all-targets -- -D warnings
npx playwright install chromium
npm run test:beacon
```

The Rust tests exhaust legal Beacon routes, checking unavailable-action rejection, investigation costs, retained caveats, reopening, physical destinations, evidence-dependent outcomes, and save restoration. Browser tests exercise the WebAssembly game and its presentation, starting their own local server. Run `npm run build` first; a separate `npm run serve` process is not required for tests. After `npm run package:game`, `npm run test:launch` also exercises blocked imports, a stalled module, and disabled-script previews.

## Repository layout

- `spec/` — versioned language specifications
- `runtime/` — reference runtime
- `tests/` — canonical semantic tests
- `examples/` — example CAVEAT programs
- `game/` — playable CAVEAT scenarios, including The Last Beacon, Moon Garden, and The Door
- `web/` — browser presentations and generic world rendering
- `scripts/` — reproducible browser build, local server, and game verification

> Do not save CAVEAT by redefining it. If the computational model collapses into an existing paradigm, record the result.
