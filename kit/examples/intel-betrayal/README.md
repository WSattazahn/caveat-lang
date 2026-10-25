# Intel that can betray you

Two informants radio intel about two gates, and the player picks a route on
it. When an informant is exposed, everything they said is withdrawn, every
plan that rested on their word is reopened, and each plan keeps the reasons
it was made on. The game shows "you were played" because of what the ledger
records; no route is scripted. Every input is illustrative.

- `intel.cav`: the program.
- `intel.scenarios.json`: four scenarios that state its behavior.
- `host.mjs`: the game's side. It plays the story, saves after the choice,
  resumes the save in a later session, exposes the informant and prints the
  debrief in the evidence's own words.

## Run it

In a project with this package installed, copy the folder out of
`node_modules/caveat-lang/examples/intel-betrayal` and run:

```sh
npx --no-install caveat-lang test intel.scenarios.json
node host.mjs
```

The host prints:

```text
Ocelot radios: the north gate is clear.
Eva radios: the south gate is guarded.
You try the south gate.
  (None of your intel says that gate is clear.)
You go north.
  [saved: 112 bytes]
Ocelot was working for the other side.
You confront Ocelot again.
  (Already exposed.)

The north plan: you were played
  north_plan@1 (reopened) was made on: what ocelot tells you over the radio
    and reopened by: proof that ocelot was lying
```

## What the program does

| Need | How | Scenario |
| --- | --- | --- |
| Each report carries who said it | one stream per gate and informant, `readings north_ocelot from ocelot_word` | I01 |
| A plan rests on the reports it read | `commit $g_plan because enough using $g_ocelot_ok + $g_eva_ok` | I01 |
| An exposed informant's plans are reopened, with their grounds kept | `rests_on_withdrawn($g_plan)` reopens; `commitment_grounds` still holds `north_ocelot@1` | I01 |
| Someone else's exposure leaves a plan alone | only plans whose grounds hold a withdrawn report are reopened | I02 |
| Everything the informant said is withdrawn, not just the latest | one `slot` entity per possible reading, `withdraw reading($g_$i, $index)` | I03 |
| The player can't set out on nothing, or twice | `reject` rules whose messages the host shows | I04 |

Two things are written by hand that a larger game would feel: the sum names
each informant (`$g_ocelot_ok + $g_eva_ok`), so a new informant means editing
it, and the `slot` entities cap how many reports can be withdrawn at once at
the streams' `limit`.

See [Caveat in a game](../../docs/GAMES.md) for the loop, saves and cost.
