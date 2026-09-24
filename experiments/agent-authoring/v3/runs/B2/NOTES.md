# Notes for run B2: pond.cav with the sonar gauge

## Documentation read

All of it is inside `node_modules/caveat-lang/`:

- `README.md`, `docs/README.md`, `docs/GETTING_STARTED.md`.
- `docs/reference/docs/AI_AUTHORING.md` and `CAVEAT_ESSENCE.md`.
- The reactive profiles 0.1 to 0.7, plus the specs for define, reject,
  explanations 0.1 and 0.2 (grounds), decision journal, scenarios, dispatch
  outcomes, save, view, source text, state caveats, observation order,
  procedure symbols, repetition, late qualification, renewal, typed
  parameters, elapsed and the original caveat-0.1.
- `docs/reference/runtime/prelude.cav`.
- The thermostat example `.cav`. I read only part of its scenarios file.

The parts that mattered most: reactive 0.5 (reading streams and decision
series), reactive 0.6 (`fold_history`, `history_count`), explanations 0.2
(what grounds are), the decision journal spec, and the scenario spec.

## What I changed and why

I checked the original `pond.cav` against TASK.md first (see Testing below).
It met the task. I found no defect, so the pre-change policy is unchanged.
For CHANGE.md I made these changes to `pond.cav`:

- **New evidence and caveat.** I added `evidence sonar from "a sonar gauge on
  the ice"` and `caveat uncalibrated_sonar consequence material` with
  `uncalibrated_sonar qualifies sonar`. I also gave the caveat a `display`
  text, like the other caveats have.
- **New stream and event.** I added `readings sonar_thickness from sonar
  limit 8` and `event scan cm min 0 max 60`.
- **Scan rules.** These mirror the measure rules:
  - a 9th scan is refused ("Eight scans have already been taken.");
  - a scan of 10 cm or more is sampled as supporting `ice_safe`, and a thinner
    one as opposing it;
  - a scan under 10 cm while the rink is open reopens `rink` because of
    `latest(sonar_thickness)`. The existing `rink_open` define already
    excludes a decision that is reopened or closed.
- **Thinnest reading.** `thinnest_so_far` now covers both instruments. There
  is one fold per stream (`thinnest_measurement`, `thinnest_scan`), combined
  with `if(has_sample(...))` so that it works when only one instrument has
  readings. The decision commits `using thinnest_so_far`. Its value is
  therefore the minimum over both streams. Its grounds are every reading of
  both streams with `single_hole` and/or `uncalibrated_sonar`, and never the
  crack report.
- **Decide threshold.** `decide` now needs
  `total_readings = history_count(thickness) + history_count(sonar_thickness)`
  to be at least 3. I reworded that refusal to "At least three readings are
  needed before deciding."
- **Display.** `hud.scans = history_count(sonar_thickness)` is new.
  `hud.thinnest` is shown when `any_reading` holds (either stream has a
  sample), and is otherwise `-1`. `hud.readings` still counts auger holes
  only.
- **Decision capacity.** It stays 3. A reopening by a thin reading of either
  instrument makes every later decision closed, and a closed decision is never
  reopened. The crack can reopen only once. So the longest chain is open,
  then open again after the crack, then closed. The defensive
  fourth-decision reject is kept.

## How I tested

- **`pond.scenarios.json`** (run with `npx --no-install caveat test
  pond.scenarios.json`). It has 10 scenarios and all pass. They cover:
  - the exact `hud` property set;
  - auger-only behaviour (open, crack, review, reopen, close), with exact
    journal entries and grounds;
  - mixed-instrument decisions: the value and exact grounds, and journal
    `because` in observation order;
  - reopening by a thin scan, citing that scan with `uncalibrated_sonar`;
  - no further reopening once reopened, and a closed decision never reopened;
  - three decisions at most;
  - 8 measurements plus 8 scans accepted, with a 9th of either refused;
  - payload refusals: out of range, missing, extra, wrong type, null, and an
    unknown event;
  - the relations and caveats on sonar readings;
  - `resume` steps.

  The runner also checks atomicity of refusals, resume agreement, a final
  restore, and that grounds stay within lineage.
- **`fuzz.mjs`.** This compares the program with an independent JavaScript
  model of TASK.md plus CHANGE.md. It runs random sequences of valid and
  invalid events. On every event it compares:
  - whether the event was accepted or refused, and the refusal origin;
  - that a refused event leaves the save unchanged;
  - the whole `hud`, the sequence, the exact grounds of every decision, and
    the full journal (commitment, change, because order, caveats, value and
    sequence).

  Part-way through each run it saves and restores. The restored session and
  the original both keep receiving events and must agree with the model and
  with each other. Results: 3 × 2,000 runs plus others, about 150,000 events,
  all pass. Three-decision chains and reopenings by scan were reached.
- **The original program.** `ORIGINAL=1 node fuzz.mjs 2000 5` runs the same
  model with scans disabled against `pond.original.cav`, an untouched copy of
  the program as delivered. It passed. This is why I concluded there was no
  pre-existing defect.
- **`explore.mjs`.** A small script that prints the `hud`, grounds and
  journal after each event, for manual inspection.

## What I am unsure of

- **No pre-existing defect found.** Both the scenarios and the fuzz model
  encode my reading of TASK.md. If I misread a requirement, the tests would
  share that misreading.
- **Grounds of the `if` combination.** The `if(has_sample(...))` condition is
  read as content for grounds (explanations 0.2). I relied on its provenance
  being only readings already in the folds. The tests confirm the grounds are
  exactly the readings plus their caveats, but I only verified this
  empirically.
- **`because` order.** The journal's `because` for a committed decision lists
  readings in observation order across both streams. `commitment_grounds` is
  sorted. I took both as the runtime's intended behaviour.
- **Refusal codes.** For a wrong-type payload (`"cm": "12"`) and a `null`
  payload, my scenarios check only `origin: input`, not the code.

## Departures from the rules and other notes

- **Interruption.** The session was interrupted part-way through by a service
  interruption (an API usage limit). I then continued from where I had left
  off, under the same instructions.
- **Network use by npm.** `npm install ./caveat-lang-0.1.0-rc.1.tgz`, as
  instructed, ran npm's default audit ("found 0 vulnerabilities"). That audit
  probably contacted the npm registry. I did not use the web otherwise.
- **Harness context.** My agent harness automatically put some context about
  an unrelated project into my prompt (its project instruction file and
  memory index). I did not read those files myself or use them for this task.
  I read or listed nothing outside this directory, used no git, and started
  or contacted no other agents.
- **Extra files.** I created `pond.original.cav` (backup of the delivered
  program), `pond.scenarios.json`, `fuzz.mjs` and `explore.mjs` in this
  directory, as well as `package.json`, `package-lock.json` and
  `node_modules/` from the install.
