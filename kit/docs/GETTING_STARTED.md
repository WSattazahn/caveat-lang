# Getting started with Caveat

This guide starts from an empty directory and ends with a program that makes a
decision, reopens it when it learns something new, and still remembers why it
chose. It takes about fifteen minutes. You need Node 20 or later; you do not
need Rust or a browser.

## 1. Install

Make an empty directory, give it a `package.json`, and install the release
candidate. Once it is published on npm, with the tag `next`:

```sh
npm init -y
npm install caveat-lang@next
```

To pin this exact candidate, install `caveat-lang@0.1.0-rc.3`. Before
publication, or to use a verified tarball you were given, pass the tarball's
path or URL to `npm install` instead.

Run `npm init -y` first: in a directory without its own `package.json`, npm
installs into the nearest parent project instead.

Check that the `caveat` command works:

```sh
npx --no-install caveat help
```

## 2. Write a program

A Caveat program says what it knows, what that knowledge rests on, and what it
decides. Save this as `umbrella.cav`. (`npx --no-install caveat init` writes
this program and the two other files this guide uses, if you would rather not
copy them.)

<!-- file: umbrella.cav -->
```caveat
scene "Decide whether to carry an umbrella.";

claim rain_likely;
evidence forecast from "the morning forecast";
evidence sky from "a look at the sky";
caveat forecast_is_old consequence material;
forecast_is_old qualifies forecast;

readings rain_chance from forecast limit 4;
decisions umbrella limit 4;
state gone = 0 min 0 max 1;

event read_forecast chance min 0 max 100;
event clear_sky;
event leave;

on read_forecast sample rain_chance = chance supports rain_likely;
on read_forecast when latest(rain_chance) >= 50 and not committed(umbrella)
  commit umbrella because enough using latest(rain_chance);
on clear_sky when not observed(sky) reveal sky opposes rain_likely;
on clear_sky when committed(umbrella) and not reopened(umbrella)
  reopen umbrella because sky;
on leave when gone == 1 reject "You have already left.";
on leave set gone = 1;

bind advice.text = "No decision yet";
bind advice.text = "Take the umbrella" when committed(umbrella) and not reopened(umbrella);
bind advice.text = "Think again: the sky has cleared" when reopened(umbrella);
```

Reading it from the top:

- `claim`, `evidence` and `caveat` name what the program can believe, where
  its knowledge comes from, and what could be wrong with it. The forecast might
  be old, so `forecast_is_old qualifies forecast`.
- `readings rain_chance from forecast` keeps a history of forecast values.
  `decisions umbrella` is a decision the program can make, reopen and make again.
- `event read_forecast chance min 0 max 100` declares an input and its allowed
  range. Anything outside it is refused before a rule runs.
- Each `on EVENT when CONDITION EFFECT;` rule reacts to an event. `sample`
  records a reading as evidence for or against a claim; `commit … because
  enough using …` decides on a value, and the decision is grounded on exactly
  the evidence that value came from; `reopen … because …` reconsiders it;
  `reject` refuses the whole event, so nothing it did survives.
- `bind` says what to show. The last binding whose condition holds wins.

Comments start with `#` or `//`.

## 3. Say what should happen

A scenario file sends events and states what should follow. Save this as
`umbrella.scenarios.json`:

<!-- file: umbrella.scenarios.json -->
```json
{
  "schema": "caveat-scenarios/0.1",
  "source": "umbrella.cav",
  "scenarios": [
    {
      "id": "U01",
      "title": "A rainy forecast decides; a clear sky reopens the decision, which keeps its reasons",
      "steps": [
        { "expect": { "/bindings/advice/text": "No decision yet" } },
        { "send": "read_forecast", "payload": { "chance": 70 } },
        { "expect": {
            "/bindings/advice/text": "Take the umbrella",
            "/commitment_grounds/umbrella@1": { "evidence": ["rain_chance@1"], "caveats": ["forecast_is_old"] }
        } },
        { "checkpoint": "decided" },
        { "resume": true },
        { "send": "clear_sky" },
        { "expect": {
            "/bindings/advice/text": "Think again: the sky has cleared",
            "/decision_journal/1": { "change": "reopened", "because": ["sky"] }
        } },
        { "same_as": "decided", "paths": ["/decision_journal/0", "/commitment_grounds/umbrella@1"] }
      ]
    },
    {
      "id": "U02",
      "title": "Refused events leave nothing behind",
      "steps": [
        { "send": "read_forecast", "payload": { "chance": 120 }, "rejected": { "origin": "input", "code": "bound_exceeded" } },
        { "send": "leave" },
        { "send": "leave", "rejected": "You have already left." },
        { "expect": { "/bindings/advice/text": "No decision yet" } }
      ]
    }
  ]
}
```

Each `expect` key is a path into the session's snapshot. U01 checks that the
decision rests on the first forecast reading and carries its caveat, that the
clear sky reopens it, and, with `same_as`, that the original decision record
did not change. The `resume` step saves the session and continues from the
save. U02 checks two kinds of refusal: an input outside its declared range, and
the program's own `reject`.

## 4. Run it

<!-- run: npx --no-install caveat test umbrella.scenarios.json -->
```sh
npx --no-install caveat test umbrella.scenarios.json
```

```text
PASS U01  A rainy forecast decides; a clear sky reopens the decision, which keeps its reasons  (2 events, 0 rejected, 1 resumes)
PASS U02  Refused events leave nothing behind  (3 events, 2 rejected, 0 resumes)
2 passed, 0 failed (umbrella.scenarios.json)
```

Besides your expectations, the runner checks on every step that a refused
event changed nothing, that a resumed session behaves exactly like the one it
was saved from, and that every decision's grounds are within what it could
have depended on.

To see a failure, change `"Take the umbrella"` to `"Leave the umbrella"` in the
scenario file and run it again:

<!-- edit: umbrella.scenarios.json: "Take the umbrella" -> "Leave the umbrella" -->
<!-- run: npx --no-install caveat test umbrella.scenarios.json -->
```text
FAIL U01 step 3 expect [primary]: /bindings/advice/text expected "Leave the umbrella", actual "Take the umbrella"
PASS U02  Refused events leave nothing behind  (3 events, 2 rejected, 0 resumes)
1 passed, 1 failed (umbrella.scenarios.json)
```

The command exits with status 1 when a scenario fails and 2 when a file is
invalid. Change the text back before going on.

<!-- edit: umbrella.scenarios.json: "Leave the umbrella" -> "Take the umbrella" -->

## 5. Look inside

To see what the program knows after each event, open a session from a script.
Save this as `explore.mjs`:

<!-- file: explore.mjs -->
```js
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const source = await readFile('umbrella.cav', 'utf8');
const runtime = await loadRuntimeFromDirectory();
const session = runtime.open(source);

for (const [event, payload] of [['read_forecast', { chance: 70 }], ['clear_sky', {}]]) {
  const outcome = session.dispatch(event, payload);
  const snapshot = session.snapshot();
  console.log(`${event}: ${outcome.outcome}; advice: ${snapshot.bindings.advice.text}`);
  for (const entry of snapshot.decision_journal) {
    const caveats = entry.caveats.length ? ` (caveats: ${entry.caveats.join(', ')})` : '';
    console.log(`  ${entry.commitment} ${entry.change} because ${entry.because.join(', ')}${caveats}`);
  }
}
session.close();
```

<!-- run: node explore.mjs -->
```sh
node explore.mjs
```

```text
read_forecast: accepted; advice: Take the umbrella
  umbrella@1 committed because rain_chance@1 (caveats: forecast_is_old)
clear_sky: accepted; advice: Think again: the sky has cleared
  umbrella@1 committed because rain_chance@1 (caveats: forecast_is_old)
  umbrella@1 reopened because sky
```

`dispatch` returns `{outcome: "accepted", snapshot}` or `{outcome: "rejected",
origin, code, message}`. Anything else throws. The snapshot also holds each
binding's explanation (`binding_explanations`), every value's lineage
(`qualified_values`), each decision's basis (`commitment_bases`), relations
between evidence and claims (`relations`) and the session clock (`elapsed`).
`session.save()` returns text you can restore later with
`runtime.restore(source, saved)`.

To ask why without writing a script, list the events in a file, one JSON object
per line, and let `caveat explain` send them. Save this as `events.jsonl`:

<!-- file: events.jsonl -->
```jsonl
{"event": "read_forecast", "payload": {"chance": 70}}
{"event": "clear_sky"}
{"event": "leave"}
{"event": "leave"}
```

<!-- run: npx --no-install caveat explain umbrella.cav events.jsonl -->
```sh
npx --no-install caveat explain umbrella.cav events.jsonl
```

```text
umbrella.cav after 4 events (sequence 3)

Events
    1  read_forecast {"chance":70}  accepted
    2  clear_sky  accepted
    3  leave  accepted
    4  leave  refused (policy): You have already left.

Decisions
  umbrella: 1 of at most 4
    umbrella@1 = 70  reopened
      based on rain_chance@1 (caveats: forecast_is_old)
      #1 read_forecast: committed because rain_chance@1 (caveats: forecast_is_old)
      #2 clear_sky: reopened because sky

Evidence
  rain_chance@1 = 70 supports rain_likely  (#1 read_forecast)  caveats: forecast_is_old
  sky opposes rain_likely

Displayed
  advice.text = "Think again: the sky has cleared"  because rain_chance@1, sky (caveats: forecast_is_old)
```

Each decision shows its value, whether it is still in force, what it is based
on, and every time it was committed or reopened and why. The refused `leave`
is listed but changed nothing. Add `--json` to get the same explanation as
data, for a program or an agent to read.

`caveat explain` answers what each decision rests on. To ask the other way
round, what rests on one piece of evidence, use `caveat dependents`:

<!-- run: npx --no-install caveat dependents umbrella.cav sky events.jsonl -->
```sh
npx --no-install caveat dependents umbrella.cav sky events.jsonl
```

```text
What rests on sky in umbrella.cav after 4 events (sequence 3)

Decisions
  nothing

Decision changes
  #2 clear_sky: umbrella@1 reopened because sky

Values
  nothing

Displayed
  advice.text = "Think again: the sky has cleared"  cites sky
```

No decision is based on the sky: it reopened one, and the advice cites it.
Ask about `rain_chance` and the decision itself is listed as based on it. Ask
about a caveat such as `forecast_is_old` and you get everything based on
evidence it qualifies. Ask this before acting on something when a piece of
evidence turns out to be wrong.

Four more commands:

- `caveat validate umbrella.cav` lists what a program declares.
- `caveat check umbrella.cav` points out patterns worth a second look, such as
  rules repeated across events (see [Check 0.1](reference/spec/caveat-check-0.1.md)).
- `caveat replay umbrella.cav events.jsonl` prints each event's outcome and
  snapshot as JSON lines.
- `caveat serve umbrella.cav` keeps a session open for another program to
  drive (see [Serve 0.1](reference/spec/caveat-serve-0.1.md)).

## 6. Keep going

- [Caveat on one page](REFERENCE.md) is the language in brief: syntax,
  outcomes and common mistakes. The [worked example](WORKED_EXAMPLE.md) takes a
  larger program through every command.
- [The documentation index](README.md) lists everything in this package.
- The [authoring guide](reference/docs/AI_AUTHORING.md) covers time, evidence
  that ages, reconsidering decisions, and a checklist for testing policies.
- [Scenarios 0.1](reference/spec/caveat-scenarios-0.1.md) specifies every step
  and matcher in scenario files.
- The language profiles start at
  [Reactive 0.1](reference/spec/caveat-reactive-0.1.md); later profiles add to
  it, up to [Reactive 0.7](reference/spec/caveat-reactive-0.7.md).
- The [thermostat example](reference/examples/thermostat_history.cav) and its
  [scenarios](reference/examples/thermostat_history.scenarios.json) show readings kept
  over time and a decision revised on every reading.
