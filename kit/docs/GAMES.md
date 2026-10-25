# Caveat in a game

A game can keep its story's knowledge in Caveat: what the player has been
told, who told them, what the player decided on the strength of it, and what
changes when a source turns out to be wrong. The game sends what happens; the
program says which choices rested on what, and each choice keeps the reasons
it was made on. This page is for a game developer embedding the runtime. It
assumes [the authoring card](AUTHORING_CARD.md) for the language itself.

The [intel example](../examples/intel-betrayal/README.md) is the whole
pattern in one small program: informants radio in intel, the player picks a
route, and exposing an informant reopens every plan that rested on their word.

## Story concepts in Caveat

| In the game | In the program |
| --- | --- |
| A source the player hears from: an informant, a codec contact, a rumour mill, an NPC | `evidence NAME from "…"`, one per source. The text is what a debrief can show. |
| What a source says, again and again | `readings S from SOURCE limit N`, one stream per source and topic; each `sample` is one report |
| What the report bears on, such as "the north gate is clear" | a `claim`, with `supports` or `opposes` on the sample |
| A player's choice | `decisions D limit N`, committed `using` the reports it reads. Its grounds are frozen at that moment. |
| An exclusive choice (parry or dodge, north or south) | one series whose value is the choice; a second commit is refused until a rule reopens it |
| "That informant was lying" | `withdraw every SOURCE because PROOF`, then reopen each plan that `rests_on_withdrawn` |
| "That one report was wrong" | `withdraw latest(S) because PROOF`, or `reading(S, N)` for an earlier one |
| A rumour that is doubted, not disproved | `qualify latest(S) with C`, a caveat; `unqualify` takes the doubt back |
| A choice the player can't take back | `decisions D limit 1 irreversible forecloses OTHER` |
| A cutscene that must not be interrupted | `hold D because E`; reopenings wait for `release` |

To withdraw everything one source said, write
`withdraw every SOURCE because PROOF` (draft for 1.4, [untrusted
sources](reference/spec/caveat-untrusted-sources-0.1.md)): it withdraws the
source and every reading sampled from it. A plain `withdraw SOURCE` does not
reach readings already taken, since each sample is its own observation
(Withdrawal 0.1, "Predicates").

## The game loop

Send story events, not frames. A rule decides when an input becomes
evidence; sampling every frame records one reading per frame and fills the
save ([authoring guide](reference/docs/AI_AUTHORING.md#what-grows-the-save)).

- **Time.** Declare `event advance dt min 0 max 100; clock advance every 1;`
  and send the frame's milliseconds when time matters to the story; read
  `elapsed()` in rules. Do not name your own event `tick`: `tick` may only
  carry a `dt` within `0..0.1` seconds. For a countdown that must agree to
  the frame, send whole units and carry the remainder
  ([elapsed](reference/spec/caveat-elapsed-0.1.md)).
- **Cost.** Measured on caveat-lang 1.3.0 in Node 22 on one 2.8 GHz server
  core, one event took about 30 microseconds for a clock `advance` and about
  90 for an accepted intel report, reading only `bindings`. That is far
  inside a frame for story beats, choices and belief changes, and the wrong
  tool for per-entity physics or AI every frame. Measure on your own target.
- **Read only what you show.** `dispatchView(event, payload, { fields: [...] })`
  builds only the named view fields. `bindings` is usually all a HUD needs;
  add `commitments`, `decision_series` and `decision_journal` for a debrief.
- **Refusals are gameplay.** A program's `reject "…"` comes back as
  `{ outcome: 'rejected', origin: 'policy', message }` and changes nothing, so
  the message can go straight to the player.

```js
import { loadRuntimeFromDirectory } from 'caveat-lang/node';   // in a browser: loadRuntime from caveat-lang/session
import { decisionHistory } from 'caveat-lang/session';

const runtime = await loadRuntimeFromDirectory();
const session = runtime.open(source);
const out = session.dispatchView('betray', { informant: 'ocelot' },
  { fields: ['bindings', 'commitments', 'decision_series', 'decision_journal'] });
if (out.outcome === 'rejected') showToPlayer(out.message);
else for (const revision of decisionHistory(out.view, 'north_plan')) debrief(revision);
```

`caveat types PROGRAM` writes TypeScript for every event payload and
displayed value, so `{ gate: 'nrth' }` is a compile error.

## In the browser

`caveat-lang/session` needs no Node APIs. Serve the installed package's
`lib/` and `runtime/` and load the runtime from URLs, or import the runtime
module through webpack or Vite (the package README, "TypeScript and
bundlers"). A trap stops every session from that runtime instance; load the
runtime again to continue.

## Saves

`session.save({ compact: true, positional: true })` writes each accepted
event as a few digits: the intel story saves in 112 bytes. It holds about
4,096 / (2 × (parameters + 1)) events; after that the save is the full state
save, which is larger. Clock events count as events.

A save restores only with the exact source that wrote it. A patch to the
game's `.cav` file is a different program, and every earlier save is refused
as belonging to a different program. Keep each shipped source, so an old save
can still be restored with the source that wrote it. Then replay the player's
events onto the patched program, which decides each one afresh; keep your own
log of accepted events once a game outgrows the replay save.

## Explaining a choice to the player

`decisionHistory(view, series)` lists every revision with what it was
committed on and what reopened it. The names are the program's (`north_ocelot@1`);
`session.snapshot().symbols` gives each evidence's `from "…"` text, so a
debrief can say "what ocelot tells you over the radio". Build the snapshot
when the debrief opens, not every frame.
