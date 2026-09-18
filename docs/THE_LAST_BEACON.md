# The Last Beacon

The Last Beacon is a complete six-decision CAVEAT mystery set on the stormbound island of Saint Orin. A ferry with thirty-two people is approaching a failing lighthouse. The player must decide which evidence to examine, which provisional plan to try, and which cost to accept before dawn.

The authoritative game is [`game/the_last_beacon.cav`](../game/the_last_beacon.cav). Its claims, caveats, investigations, commitments, reopening, world topology, action plans, and player-facing narrative all live in that file. Rust evaluates the source and executes its generic world commands. The browser presents the resulting state and animates the commands.

## Play contract

There are three watches of attention. Each watch permits one investigation costing one unit, followed by one commitment. The six interactions each offer three options, for 729 complete playthroughs. Attention is a discrete investigation budget, not a real-time countdown. Reading the journal or rotating the island does not spend it.

The first two commitments lead to a concrete action and reveal a reason to reopen that commitment. Returning to the keeper's court is an explicit journey in the source. It does not erase evidence or undo a decision. The final commitment retains all nine caveats: three examined and six unexamined.

An examined caveat is not automatically a resolved caveat. For example, the reserve test establishes available voltage while leaving endurance uncertain. Supporting evidence can coexist with opposing evidence. The player sees that history rather than a synthetic certainty percentage.

| Interaction | Player's question | Source options |
| --- | --- | --- |
| `first_watch` | Which old assurance needs checking? | `lens_salt`, `chart_age`, `radio_echo` |
| `first_signal` | Which assurance will I put to the test? | `trust_beam`, `trust_chart`, `trust_radio` |
| `changed_water` | What can the current conditions support? | `tide_lag`, `reserve_unknown`, `bearing_drift` |
| `revised_plan` | Which solution should I prepare? | `bridge_reserve`, `mark_channel`, `establish_contact` |
| `final_watch` | Which remaining uncertainty most changes my decision? | `beam_heat`, `shoal_depth`, `anchor_hold` |
| `final_signal` | Which cost will I accept? | `relight_beacon`, `launch_pilot`, `hold_offshore` |

## The island

All connections are bidirectional. Every journey starts at the keeper's court until the final decision. The lighthouse route includes the tower base and the elevated lantern room; the breakwater route passes through the harbor. There are no implied teleports or closed barriers.

| Place | Kind | Direct neighbors |
| --- | --- | --- |
| `keeper_court` | `courtyard` | Lighthouse base, observatory, tidal archive, harbor |
| `lighthouse_base` | `lighthouse` | Keeper's court, lantern room |
| `lantern_room` | `lantern_room` | Lighthouse base |
| `observatory` | `observatory` | Keeper's court |
| `tidal_archive` | `archive` | Keeper's court |
| `harbor` | `harbor` | Keeper's court, breakwater |
| `breakwater` | `breakwater` | Harbor |

The browser may assign coordinates and visual templates to these semantic kinds. Source action plans determine where the player travels, what is inspected or operated, what is observed, and where the action ends. The renderer must show a continuous ascent between the lighthouse base and lantern room.

## Evidence and reopening

The initial evidence is plausible but stale: the log's safe sector, an old channel chart, and a recorded radio call. An investigation exposes only the chosen flaw. No unselected inspection silently runs in the background.

The first commitment exposes a second observation through action. Sighting the beam reveals its second sector on a reef. Walking the charted route reveals a submerged marker. Challenging the radio call exposes relay feedback. Each observation opposes its corresponding assurance and reopens the selected commitment because of a retained caveat.

The second watch finds a viable direction: a strip of water for the shallow launch, a short reserve for the lamp, or room beyond the shoal for the ferry to hold. Preparation then exposes a limitation: cable heat, moving sand, or engine failure. The revised commitment reopens with that limitation still recorded.

The final investigation can support one solution more strongly without certifying all of them. The source deliberately does not turn unexamined caveats into automatic death rolls. It models a choice among different sacrifices, with an honest record of the information available when the player acted.

## Three endings

| Action | Physical contract | Resolution and cost |
| --- | --- | --- |
| `relight_beacon` | Walk to the tower, operate the reserve, ascend, clean the lens, operate the beacon. End in `lantern_room`. | Meridian enters the harbor under manually controlled pulses. All thirty-two disembark. The keeper remains at the breaker and spends the reserve. |
| `launch_pilot` | Walk to the harbor, inspect the tide gauge, release the pilot launch. End in `harbor`. | The pilot transfers the passengers to the sheltered beach. The beacon stays dark; the ship and cargo await salvage. |
| `hold_offshore` | Walk to the observatory, check the telescope, transmit the hold. End in `observatory`. | The ferry anchors offshore under a bearing watch. The crossing is canceled and a daylight tow is arranged. |

These are authored outcomes with distinct destinations, operations, claims, and consequences. There is no hidden numerical win score. Earlier choices determine what the player knows and what their decision record can justify. The final choice determines the outcome. This distinction is intentional and visible in the source.

The first two choices explicitly declare `converge` because all their routes return to the court. The final choice does not converge, and the compiler validates its three distinct endpoints.

## Presentation contract

All interaction titles, descriptions, action labels, tradeoffs, feedback, place labels, and ending prose are `display` declarations. The browser reads these conventions:

| Label | Meaning |
| --- | --- |
| `game_title`, `game_subtitle`, `game_intro` | Opening presentation |
| `<interaction>` and `<interaction>_body` | Current decision heading and situation |
| `<action>` and `<action>_hint` | Option label and tradeoff |
| `<action>_result` | Narrative after that action actually executes |
| `<final_action>_title` | Ending heading |
| `<caveat>_uncertainty` | The uncertainty preserved in the journal |
| `<place>`, `<entity>`, `<evidence>`, `<claim>` | Source-authored names and descriptions |

Only the final `relight_beacon` action operates the beacon, only `launch_pilot` operates the boat, and only `hold_offshore` operates the transmitter. Interim actions inspect those systems or prepare their supporting equipment. Generic renderer effects can therefore follow `operate` commands without inventing ending logic.

Full label metadata is available to the renderer at initialization. That does not make future evidence discovered: the live epistemic snapshot and observed world commands are authoritative for what has happened.

## Verification

Run the scenario tests from the repository root:

```sh
cargo test --manifest-path runtime/Cargo.toml --test the_last_beacon
cargo run --manifest-path runtime/Cargo.toml --bin caveat-map -- validate game/the_last_beacon.cav
```

The route matrix exhausts all 729 complete sequences and checks:

- Every offered action remains physically available and every route completes.
- Investigation costs produce exactly three spent attention units.
- The first five journeys return to the court through valid source paths.
- Both provisional commitments remain reopened in the history.
- The final commitment preserves all nine caveats, including six unexamined ones.
- Exactly one of the three outcome claims receives supporting evidence.
- Each ending has its own destination and observation, with 243 routes reaching each.

Additional tests check that initialization does not run future investigations, each first investigation reveals only its selected evidence, save restoration preserves the decision record and completed world, and every visible option has narrative feedback and a physical action plan.

The automated route matrix verifies language and simulation behavior. Visual review must also inspect the tower ascent, breakwater journey, all three final destinations, and their corresponding operated objects.
