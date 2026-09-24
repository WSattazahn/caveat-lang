# Notes: run A2, adding the sonar gauge to pond.cav

## Documentation read

Everything is under `node_modules/caveat-lang/`:

- `README.md`, `docs/README.md`, `docs/GETTING_STARTED.md`
- `docs/reference/docs/AI_AUTHORING.md`, `docs/reference/docs/CAVEAT_ESSENCE.md`
- Specs in `docs/reference/spec/`:
  - read in full: reactive 0.1 to 0.7, define, reject, explanations 0.1 and 0.2, decision journal, state caveats, dispatch, save, view, scenarios, observation order, source text, typed parameters, late qualification, renewal, repetition, procedure symbols, elapsed
  - not opened: `caveat-0.1.md`
- `docs/reference/runtime/prelude.cav` and `docs/reference/examples/thermostat_history.cav`
- The first 60 lines of `bin/caveat.mjs`, to see the CLI's usage text

## Was the original program correct?

I found no way in which the original `pond.cav` failed `TASK.md`, so I made no fixes to the original behaviour. I kept an unchanged copy as `pond.original.cav` and checked it three ways:

- by hand-driven runs;
- by 1,500 randomized sessions compared against an independent JavaScript model of `TASK.md` (see Testing);
- by reasoning through each rule:
  - 10 cm or more supports the claim; thinner opposes it.
  - A ninth measurement is refused with a policy `reject`. Without that rule it would be a fatal history-capacity error.
  - `decide` is refused before three measurements and while a decision is in force.
  - The decision's value is a fold-minimum over all readings, and its grounds are every reading plus `single_hole`.
  - Only an open, not-yet-reopened decision can be reopened, which covers both "no further reopening" and "a closed decision is never reopened".
  - `hud` has exactly its six properties.
  - The runner confirmed that save and restore work.

## What I changed, and why (CHANGE.md)

All changes are in `pond.cav`:

1. **New evidence and caveat.** I declared `evidence sonar from "a sonar gauge on the ice"`, `caveat uncalibrated_sonar consequence material` and `uncalibrated_sonar qualifies sonar`. Every scan therefore carries `uncalibrated_sonar`.
2. **New reading stream and event.** I added `readings sonar_thickness from sonar limit 8` and `event scan cm min 0 max 60`. The core refuses bad input to `scan` the same way it refuses bad input to `measure`.
3. **Scan rules, mirroring the measure rules.**
   - A ninth scan is refused with a policy `reject`.
   - A scan of 10 cm or more supports `ice_safe`; a thinner scan opposes it.
   - A thin scan reopens an open rink `because latest(sonar_thickness)`, the scan just taken.
   - Scans and measurements are counted separately. Each stream has its own limit and its own guard.
   - Procedures cannot take a reading stream as a parameter, so I could not share one procedure between `measure` and `scan`. The rules are written out twice.
4. **Decide gate.** A new `define reading_total` adds the two counts, and `decide` is refused while `reading_total < 3`. I reworded the refusal message to mention both instruments.
5. **Thinnest reading.** `thinnest_so_far` is now the minimum over both streams:
   - With auger readings, it folds the sonar stream starting from the auger minimum.
   - Without them, it folds the sonar stream alone.
   - It is only evaluated when at least one reading exists, through the `has_reading` guard or because `decide` needs three readings.

   Because `commit rink ... using thinnest_so_far` reads every record of both folds, the runtime grounds each decision on every reading from both instruments and their caveats. The crack report is not included. I kept the runtime's own decisions: the commitment, `commitment_grounds` and `decision_journal`.
6. **Display.**
   - I added `hud.scans = history_count(sonar_thickness)`.
   - `hud.readings` still counts auger measurements only.
   - `hud.thinnest` now covers both instruments, and is shown `when has_reading`.

   `hud` has exactly the seven properties the change request lists.

Decision capacity stays at three, and three is still enough:

- A thin reading from either instrument makes every later decision "closed", and a closed decision can never be reopened.
- The crack can reopen a decision only once.
- So at most there is open, then crack, then open, then a thin reading, then closed: three revisions. A fourth `decide` is always refused as "in force". The tests check this.

## How I tested

- **`pond.scenarios.json`: 17 scenarios, all pass.** Run with `npx --no-install caveat test pond.scenarios.json`. They cover:
  - the exact initial `hud`;
  - core refusals: out of range, missing, extra or string `cm`, payloads on `crack` and `decide`, a null payload, and an unknown event;
  - the 10 cm boundary for both instruments;
  - stream, relation and caveat records;
  - the separate eight-reading limits;
  - the three-readings-in-total gate;
  - decisions made from scans only, measurements only, or both, with exact grounds and journal entries;
  - reopening by a thin scan, a thin measurement or the crack, with no further reopening afterwards;
  - a closed decision never reopening;
  - the three-revision limit;
  - the minimum being taken over old readings;
  - fractional values;
  - `checkpoint`/`same_as` on frozen records, and `resume` at several points.

  The runner also checks atomicity of every refusal, agreement after restore, and grounds staying within lineage.
- **`fuzz.mjs`: randomized differential test.** Usage is `node fuzz.mjs [runs] [seed] [file]`.
  - Each run sends a random sequence of valid and invalid events and compares every accepted snapshot against a plain-JS model: `hud`, sequence, journal order, causes and values, grounds evidence and caveats, and journal caveats.
  - It checks that every refusal leaves the save byte-identical, and that it came from `policy` or `input` as expected.
  - It saves and restores at random points and requires identical snapshots.
  - Results: `pond.cav` 6,000 runs (seed 424242) and 1,500 runs (seed 12345), 0 failures. `pond.original.cav` against the original-task model: 1,500 runs, 0 failures.
  - Coverage counters show all three reopen causes and third revisions are reached.
- **Mutation check (`mutants/make.mjs`).** I made six deliberate mistakes:
  - no scan reopening;
  - counting only auger readings for the gate;
  - no scan limit;
  - thinnest over the auger only;
  - scans reopening closed decisions;
  - one limit shared between the instruments.

  Every mutant failed both the scenario file and the fuzz test.
- `explore.mjs` and `dump.mjs` are the ad-hoc scripts I used to look at snapshots.

## What I am unsure of

- **Grounds caveats.** "Its grounds are every reading from both instruments taken so far, with their caveats": I read this as the caveats the included readings actually carry. A decision made on scans alone is grounded with only `uncalibrated_sonar`, and one made on measurements alone with only `single_hole`.
- **Lineage versus grounds.** From `rink@2` on, a decision's lineage (`commitment_bases` provenance) includes `crack_report`/`secondhand` whenever the predecessor was reopened by the crack. This is the runtime's documented rule (Reactive 0.5, Explanations 0.2). The grounds and the journal's `committed` entries exclude them, which is what the task asks for.
- **Refusal messages.** The wording is my own. I changed the `decide` message and added a new one for the ninth scan.
- **Ordering.** Journal `because` lists evidence in observation order, which interleaves the two instruments. `commitment_grounds` arrays come back sorted by name. My tests compare grounds as sets.
- **Scenario-runner behaviour.** Matchers such as `$set` nested inside `$exact` are compared literally. My first test draft made that mistake, and I fixed the tests, not the program.

## Departures from the rules and other notes

- **Interruption.** My session was interrupted once by a service interruption (an API usage limit), not by anything in the task. The coordinator asked me to continue, and I resumed from where I stopped with the same rules.
- **Working directory.** Everything I did was inside the A2 directory: install, reading, edits and tests. I read nothing outside it. I did not use the web, git or other agents.
- **Injected context.** The harness automatically placed some unrelated project instructions and memory notes from elsewhere on the machine into my context. I did not open those files and did not act on them.
- **Files I created.** Beyond `package.json`, `package-lock.json` and `node_modules/` from the install, I created only these, all in A2:
  - `pond.original.cav`, an untouched copy of the original;
  - `pond.scenarios.json`;
  - `fuzz.mjs`, `explore.mjs` and `dump.mjs`;
  - the `mutants/` folder;
  - this file.
