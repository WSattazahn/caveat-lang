# Change requests for the Glowcap beat

`BEAT.md` describes a small game feature: a slime, look-alike mushrooms, what
the slime learns about them, and what the game tells the player and why. It
was built up by sixteen change requests (CR1–CR16) on top of a base behaviour.
`existing-scenarios.mjs` (base to CR12) and `existing-scenarios-r7.mjs` (CR13–CR16) hold its acceptance scenarios.

The game's designer will ask for four more changes. Write them: **CR17, CR18,
CR19 and CR20**, applied in that order, each cumulative on everything before
it. Write them as the designer would, about what the player sees and what the
game allows, with no reference to how anything is implemented.

## What a request must be

- Precise enough that two people implementing it independently produce the
  same view and the same accepted and rejected events. Name every new view
  field, string, event, payload field and evidence id exactly, and say what
  happens at each boundary.
- Testable through the view and through accepted and rejected events, as the
  existing scenarios test.
- **Bounded.** Anything that grows (lists, counts, lives, history, stored
  data) has a stated bound in the request, and the request says what happens
  when the bound is reached. Nothing may require unbounded memory.
- Compatible with every earlier request unless the request says exactly which
  earlier behaviour it replaces.
- A plausible next step for this game. Choose them on your own judgement of
  what the feature needs; there is no quota of any kind of request.

## What to produce, in this directory

1. `REQUESTS.md`: the four requests, written like the CR bullets in
   `BEAT.md` (one bullet each, as long as the request needs, tables allowed).
2. `scenarios-r8.mjs`: an ES module exporting `SCENARIOS`, an array in the
   format of `existing-scenarios.mjs` and `existing-scenarios-r7.mjs`. Ids continue from S51. Each scenario
   has `since: 'cr17'` (or `cr18`, `cr19`, `cr20`). If a request deliberately
   changes an existing scenario's expectation, say so in REQUESTS.md and add a
   replacement scenario; existing scenarios cannot be edited, so such a change
   is only possible where the old scenario's `until` already ends it, and you
   should otherwise avoid it. The module must import nothing; define any
   helper constants inside it.
   - Steps: `['absorb', id, kind]`, `['taste', id, kind]`, `['witness', id, kind]`,
     `['tick', dt, times]` (dt at most 0.1; 0.0625 makes 16 ticks one second),
     `['expect', partialView]`, `['reject', event]`, `['resume']`. A new event
     with other payload fields is written `['eventName', { field: value, … }]`
     and dispatched as `{ type: 'eventName', field: value, … }`.
   - `expect` checks only the fields listed; lists are compared as sets
     unless a request says order matters, and then your request must say so.
   - Aim for about three scenarios per request, covering its boundaries and
     its interaction with earlier requests (resume included).
3. Work every expectation out by hand from the requests, step by step. There
   is no implementation to run them against; a wrong expectation costs every
   implementer. Keep scenarios short enough to trace.

`node --check scenarios-r8.mjs` must pass, and
`node -e "import('./scenarios-r8.mjs').then(m => console.log(m.SCENARIOS.length))"`
must print the count.

Your final answer: a few lines naming the four requests.
