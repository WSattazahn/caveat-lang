# The Carmack checks

Four checks that run on every release, and on every pull request that touches the runtime or the kit. The rule behind them: measure everything, hide nothing, and be exactly right. Every number is reported however it looks.

| Check | Script | Fails a build? |
| --- | --- | --- |
| 1. Where the time per event goes | `profile.mjs` and `runtime/examples/carmack_profile.rs` | No, it reports |
| 2. Byte-identical replay everywhere | `replay.mjs` and `replay-core.mjs` | Yes, on any difference |
| 3. Nothing hidden: size, start-up, memory | `footprint.mjs` against `baseline.json` | No, unless run with `--strict` |
| 4. One speed target with a deadline | `target.mjs` against `target.json` | No, a miss is printed as a miss |

CI runs them in `.github/workflows/carmack.yml`. Timing on shared runners is too noisy to gate a pull request, so checks 1, 3 and 4 write their numbers to the job summary and the `carmack-measures` artifact; a release record quotes them.

## Workloads

`workloads.mjs` fixes two programs and their events, replayed on a fresh session each repetition:

- **release-gate** (`programs/release-gate.cav`): three components with three checks each; reports are flagged, unflagged and withdrawn, and decisions reopen. 48 events, 200 repetitions.
- **ledger** (`ledger/project.cav`): the project's own decision ledger, all of `ledger/events.jsonl`.

## 1. Where the time goes

`profile.mjs` times every event in the WebAssembly runtime through `dispatchView`, the path a host uses, twice: with the whole view, and with no view fields (the dispatch alone). The difference is what building and parsing the view costs. It also times the save, the compact save, and restoring each.

With `--native`, it also runs `carmack_profile`, a native build with the fixed phase boundaries of the `event-phase-profile` feature, and splits each event into:

- **rules**: the program's handlers and bindings (phases `evaluation`, `binding_evaluation`). This includes the lineage each value carries as it is computed, which is inline and has no boundary of its own.
- **record keeping**: withdrawal collection, compaction, the archive and its provenance (`collection`, `compaction`, `archive_construction_and_removal`, `final_settlement`).
- **transaction**: checking the payload, copying the session so a refusal leaves it untouched, committing or rolling back (`transaction_preparation`, `commit_cleanup`, `rollback_cleanup`).
- **payload and view building**, and **view JSON**: outside the phases.

```sh
npm run build
cargo build --release --manifest-path runtime/Cargo.toml --features event-phase-profile --example carmack_profile
node bench/carmack/profile.mjs --native runtime/target/release/examples/carmack_profile
```

## 2. Replay

`replay-core.mjs` drives each program in `replay.mjs`'s list with seeded random events (some out of range on purpose, so refusals are compared too), saves at a random step both ways, restores both saves, and runs the rest of the sequence on all three sessions. On each platform the three must agree event by event. Every outcome and save is hashed with SHA-256; `--compare` then requires the digests from Node on Linux, Windows and macOS and from Chromium, Firefox and WebKit to be byte-identical.

```sh
node bench/carmack/replay.mjs --browsers chromium,firefox,webkit
node bench/carmack/replay.mjs --compare test-results/carmack
```

`webkit-stress.mjs` starts the runtime in 200 fresh WebKit pages and opens every replay program three times in each, failing on any WebAssembly trap. WebKit 26.0 (Playwright 1.55) trapped in about 1% of fresh pages inside session creation, with "call_indirect to a null table entry", "Out of bounds memory access" or "Unreachable code should not be executed". Our inputs reached the runtime intact, and the traps stopped when WebKit's baseline compiler or its tier-up was switched off (`JSC_useBBQJIT=0`, or its OSR entry points off), so it is an engine fault, not ours. WebKit 27.2 (Playwright 1.64.1) showed none in 800 pages against 6 in 1,200 on 26.0, so CI runs 27.2 and this check guards against a return.

```sh
node bench/carmack/webkit-stress.mjs --runtime dist/pkg-reactive --pages 200
```

## 3. Footprint

`footprint.mjs` measures the WebAssembly size raw and gzipped, what a host ships (runtime, glue and session library, gzipped), the kit library's size, start-up (loading the runtime and opening each workload's program in a fresh Node process, median of nine), WebAssembly memory per session after its workload, and the state save's size. Each is compared with `baseline.json` and its allowed growth. A release that grows a measure past its allowance either fixes it or records the new baseline in the same pull request with `--write-baseline`, so the growth is in the diff.

## 4. The speed target

`target.json` names the measure, the release-gate workload's median time per event in WebAssembly with the whole view on CI's Ubuntu runner, and once set, the number to beat and the date it should be met by. `target.mjs` reports it from the profile with how far off it is.

## Stress cases

`stress.mjs` runs the worst inputs we know of for the runtime's own limits and times them:

- **Withdrawal reach**: K decisions rest on one piece of evidence, and one event withdraws it and reopens all K. The retract event is timed for K from 100 to 500 (the history capacity bounds K), with how its time grows against linear and quadratic.
- **Lineage growth**: every event adds two renewable occurrences to one state's lineage until the provenance limit rejects the event (`provenance_limit`).
- **Evaluation limit**: a heavy reducer is folded over a growing stream until the evaluation limit rejects the event (`evaluation_limit`).

Times are reported, never gated. What must hold is that every case ends accepted or with its documented refusal, and that the session still answers the same way afterwards; anything else, a thrown error or a fatal session included, exits 1. The programs are generated by the script, so they stay out of the corpus sweeps. CI runs it in the `build` job of `carmack.yml`, where an exit 1 fails the job and the times go to the job summary.

```sh
node bench/carmack/stress.mjs
```
