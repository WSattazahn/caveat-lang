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

## 3. Footprint

`footprint.mjs` measures the WebAssembly size raw and gzipped, what a host ships (runtime, glue and session library, gzipped), the kit library's size, start-up (loading the runtime and opening each workload's program in a fresh Node process, median of nine), WebAssembly memory per session after its workload, and the state save's size. Each is compared with `baseline.json` and its allowed growth. A release that grows a measure past its allowance either fixes it or records the new baseline in the same pull request with `--write-baseline`, so the growth is in the diff.

## 4. The speed target

`target.json` names the measure, the release-gate workload's median time per event in WebAssembly with the whole view on CI's Ubuntu runner, and once set, the number to beat and the date it should be met by. `target.mjs` reports it from the profile with how far off it is.
