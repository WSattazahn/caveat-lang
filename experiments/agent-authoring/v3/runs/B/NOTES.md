# Notes: run B, `pond.cav`

## Documentation read

All of it is inside `node_modules/caveat-lang/`:

- `README.md`, `docs/README.md` and `docs/GETTING_STARTED.md`.
- `docs/reference/docs/AI_AUTHORING.md` and `CAVEAT_ESSENCE.md`.
- `docs/reference/examples/thermostat_history.cav` and its `.scenarios.json`, plus `docs/reference/runtime/prelude.cav`.
- The specs in `docs/reference/spec/`, all read in full: `caveat-0.1`, `reactive-0.1` through `0.7`, `decision-journal`, `explanations-0.1` and `0.2`, `reject`, `state-caveats`, `late-qualification`, `define`, `save`, `view`, `text`, `dispatch`, `observation-order`, `procedure-symbols`, `renewal`, `repetition`, `typed-parameters`, `elapsed` and `scenarios`.
- The first 60 lines of `bin/caveat.mjs`, for its usage text.

## How the program works

- The declarations are `claim ice_safe`, `evidence auger` and `evidence crack_report`, plus the caveats `single_hole` (material, qualifying `auger`) and `secondhand` (low, qualifying `crack_report`). The stream is `readings thickness from auger limit 8` and the decision series is `decisions rink limit 3`.
- `measure`:
  - An explicit `reject` fires once 8 readings exist. Without it, the ninth sample hits the stream limit, which is a fatal `unclassified` error rather than a refusal (see Testing).
  - Two `sample` rules follow, one per side of 10 cm. Their guards read only the payload, so each reading's provenance is exactly `{thickness@N, single_hole}`.
  - If the rink is open, a reading under 10 cm then runs `reopen rink because latest(thickness)`.
- `crack`: a `reject` fires if `crack_report` is already observed. Otherwise the rule reveals `crack_report opposes ice_safe`. If the rink is open, it also runs `reopen rink because crack_report`.
- `decide` rejects in three cases:
  - fewer than 3 readings;
  - a decision in force (`committed and not reopened`);
  - 3 decisions already made. This last check is defensive: the policy already makes a fourth decision impossible.

  Otherwise it runs `commit rink because enough using fold_history(thickness, latest(thickness), min)`.
- "Rink open" is defined as `committed(rink) and not reopened(rink) and latest(rink) >= 10`. So a closed decision is never reopened, and a reopened one gains no further reopenings.
- The program keeps no state of its own. The `hud` bindings read `history_count`, `fold_history`, `observed`, `committed`, `reopened` and `latest(rink)` directly.

## Testing

- **Scenario file.** `npx --no-install caveat test pond.scenarios.json` runs `pond.scenarios.json`, whose 10 scenarios all pass. They cover:
  - the initial HUD, including that `hud` is the only binding target;
  - reading provenance and supports/opposes relations;
  - the 8-reading cap;
  - input refusals: out of range, missing or extra parameter, wrong type, unknown event, and payloads on `crack` and `decide`;
  - the three-reading minimum and the decision-in-force rule;
  - reopening by a thin reading and by a crack, with exact journal entries and exact `commitment_grounds`, which never include `crack_report`;
  - no second reopening, and a closed decision that is never reopened;
  - the full three-decision path;
  - the boundary values 10 and 9.99;
  - `resume` steps at several points.

  The runner also checks atomicity of refusals and agreement after a restore on its own.
- **Random model check.** `node fuzz.mjs [runs] [seed]` compares the program with an independent JavaScript model of TASK.md. It sends random valid and invalid events, and saves and restores at random points, dispatching every event to all the live sessions. After each event it checks:
  - accept or refuse, and the origin of any refusal;
  - that a refused event leaves the save unchanged;
  - that restored sessions agree;
  - the exact `hud`;
  - the journal (commitment, change, because, caveats, value);
  - the grounds of every decision.

  Two runs of 1,500 sequences (seeds 12345 and 987) came to about 56,000 events and 5,400 restores, with 0 failures. Paths with all three decisions, and reopenings by both a crack and a reading, were exercised.
- **Mutation check (files deleted afterwards).** I ran the scenario file against 9 hand-mutated copies of the program:
  - `>=` changed to `>`;
  - the `not reopened` guard dropped;
  - the stream limit changed to 9;
  - the ninth-measure `reject` removed;
  - the three-reading minimum changed to 2;
  - the thinnest replaced by the latest reading;
  - the crack reopening allowed on a closed rink;
  - the `secondhand` qualification removed;
  - `opposes` changed to `supports`.

  Every mutant failed at least one scenario.
- **Exploring by hand.** `node explore.mjs measure:12 decide crack ...` prints the HUD, grounds, journal and relations.

## Unsure of

- **Lineage of later decisions.** The runtime's lineage for a later revision (`commitment_bases`, `relies_on` edges and `retains`) includes the predecessor's reopening witness. For example, `rink@2` relies on `crack_report` and retains `secondhand` after a crack reopening. The spec says this is intended decision-history lineage. The *grounds*, which is what TASK.md asks about, contain only the thickness readings and `single_hole`.
- **Value types.** `hud.cracked` is bound as the number 1/0, and `decision` as text. I read TASK.md's `1`/`0` as numbers.
- **Extra labels.** I added `scene` and two `display` caveat labels. They appear in the snapshot's labels, not as bindings, so `hud` still has exactly the six properties.
- **Refusal types.** A `measure` payload such as `{"cm": "12"}` is refused by the runtime with origin `input`. I did not pin its exact code.

## Departures from the rules

None that I know of. I read and wrote only inside the run-B directory and its installed `node_modules/caveat-lang`, did not use the web, git or other agents, and followed the install steps as given. The scratch files `explore.mjs` and `fuzz.mjs` stay in the directory. A temporary `mutants/` subdirectory held the mutated copies and was deleted.
