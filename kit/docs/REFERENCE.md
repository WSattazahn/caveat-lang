# Caveat on one page

A Caveat program records decisions, their reasons and permissions. Events bring
readings in; rules record, decide and reopen. `limit` histories retain their
items and refuse additions at capacity. `window` histories retire older records;
eligible records can depart into the [host archive](HISTORY.md).

This page is the language in brief. The [worked example](WORKED_EXAMPLE.md)
takes one complete program through every command, and the package's tests run
its files and check the output it shows. The excerpts below are checked to
appear in that program. The tables summarize the specifications they link to,
which are authoritative.

## Commands

Run each as `npx --no-install caveat …` in a project with the package installed.

| Command | What it answers |
| --- | --- |
| `validate PROGRAM` | Does the program load? Lists its events, histories and displayed values. |
| `check PROGRAM` | Are there patterns worth a second look? Advisory warnings with a code and a line ([check](reference/spec/caveat-check-0.1.md)). With `--strict` it exits 1 when there is a warning; `--unused` also reports declarations nothing else names; `--records` also reports reads of a decision's record that a later `qualify` reaches from 1.1, and with `--save SAVE` the values that save holds without such a caveat. |
| `types PROGRAM` | What does a host send and read? TypeScript declarations for its events, payloads and displayed values, from its [interface](reference/spec/caveat-interface-0.1.md). |
| `test SCENARIOS…` | Did each scenario get the outcomes and snapshot values it expects ([scenarios](reference/spec/caveat-scenarios-0.1.md))? An expected scalar must be equal, an array must have the same length and match element by element in order, and an object matches only the members it lists; `{"$set": [...]}` ignores order. It prints `check`'s warnings for each program and runs on. |
| `explain PROGRAM [EVENTS]` | After these events, what does each decision rest on, which event (with its payload) observed each of its grounds, and why is each displayed value what it is? |
| `dependents PROGRAM NAME [EVENTS]` | What rests on this evidence, reading stream or caveat? Includes [withdrawal records](reference/spec/caveat-dependents-0.1.md). Draft for 1.4: `undo-list PROGRAM EVENTS` lists every revision, current or past, resting on withdrawn evidence, irreversible ones marked; it reopens nothing ([untrusted sources](reference/spec/caveat-untrusted-sources-0.1.md)). |
| `replay PROGRAM EVENTS` | What was the snapshot after each event? |
| `serve [--state FILE] PROGRAM` | Keeps a session open over standard input and output; with `--state`, in a file across runs ([serve](reference/spec/caveat-serve-0.1.md)). |
| `init [DIRECTORY]` | Writes the getting-started program, its scenarios and an events file. |

Each answers only its own question. A program that loads can still do the
wrong thing, and a clean check is not a passing test.

## The core

This example keeps `limit` readings; a decision is a series of revisions:

<!-- excerpt: frost.cav -->
```caveat
readings soil from probe limit 12;
readings aerial from drone limit 12;
readings approvals from grower limit 4;
decisions cover limit 4 reopened by soil opposing frost_risk, aerial opposing frost_risk;
```

Each `sample` into `soil` adds a reading, `soil@1`, `soil@2` and so on. None is
overwritten. `cover@1` is the first revision of the decision. A new revision
can be made only after the current one is reopened. `reopened by` reopens the
current revision when a matching reading is taken.

A decision rests on what `using` reads, and a grant can permit it:

<!-- excerpt: frost.cav -->
```caveat
on decide when latest(soil) <= 2
    commit cover because enough using latest(soil) permitted by latest(approvals);
```

This one decision has three kinds of reason, and `explain` names them:

- **Grounds** ("based on"): what `using` read, here `soil@1` with its
  caveats. They are frozen when the decision is made.
- **Grant** ("permitted by"): what `permitted by` found. Without one, the
  commit is refused as `policy/not_permitted`. A grant is not grounds.
- **Lineage** ("could also have been influenced by"): everything else that
  touched it, such as the guard and the grant.

A reading found to be wrong is withdrawn, not contradicted, and nothing is
reopened unless a rule or a `reopened by` declaration says so:

<!-- excerpt: frost.cav -->
```caveat
on misread withdraw latest(soil) because recheck;
on misread when committed(cover) and not reopened(cover) and rests_on_withdrawn(cover)
    reopen cover because recheck;
```

## Syntax

Statements end with `;` and may span lines. Comments are `#` or `//` to the
end of the line; `--` is not a comment.

### Declarations

| Form | Declares |
| --- | --- |
| `claim NAME;` | Something evidence bears on. |
| `evidence NAME from "SOURCE";` | Evidence. The source is a label, not authentication. |
| `caveat NAME consequence LEVEL;` | A caveat. LEVEL is `negligible`, `low`, `material`, `high` or `catastrophic`. The level is reported in the snapshot's `symbols` and changes no outcome; a rule that reads the caveat, such as `when has_caveat(STATE, NAME) reject "…"`, makes it matter. |
| `CAVEAT qualifies EVIDENCE;` | The caveat goes with that evidence and everything computed from it. |
| `readings NAME from EVIDENCE limit N;` | A stream of up to N readings. |
| `decisions NAME limit N [reopened by [changed] STREAM [supporting\|opposing CLAIM], …];` | A decision with up to N revisions ([triggers](reference/spec/caveat-reopening-triggers-0.1.md)); `changed` fires only on a reading whose value differs from the stream's previous one (from 1.1.0, [changed triggers](reference/spec/caveat-changed-triggers-0.1.md)). A series has no `window`, so every revision stays in the save; cap recommits ([guide](reference/docs/AI_AUTHORING.md#what-grows-the-save)). |
| `decisions NAME limit 1 irreversible [forecloses ACTION, …] [reopened by …];` | A decision that can't be undone. Its commit must `retain` each `material` or worse caveat it rests on, and once made, a commit of any `ACTION` it forecloses is refused. Reopening it records doubt and keeps the foreclosure ([foreclosure](reference/spec/caveat-foreclosure-0.1.md)). |
| `state NAME = EXPRESSION [min A max B];` | A number. Bounds default to, and may not exceed, `-1e12..1e12`. |
| `event NAME [PARAMETER, …];` | A parameter is `NAME min A max B`, `NAME in A B …`, `NAME kind KIND` or `NAME id` ([identifiers](reference/spec/caveat-identifiers-0.1.md)). |
| `identifiers limit N;` | Required before any `id` parameter. |
| `define NAME = EXPRESSION;` | A named expression, inlined where read ([define](reference/spec/caveat-define-0.1.md)). |
| `fn NAME(A, B) = EXPRESSION;` | A pure function. It cannot read state or evidence. |
| `proc NAME(PARAMETER, …) { STEP; … };` | Steps run by `call`. A parameter is a number, or `NAME readings`, `decisions`, `evidence`, `claim` or `caveat`. Symbol arguments are bare declared names; pass `S` to `s readings` and use `latest(s)` inside, not `latest(S)` to `e evidence` ([paired examples](../examples/procedure-handles/README.md), [spec](reference/spec/caveat-procedure-symbols-0.1.md)). |
| `bind TARGET.PROPERTY = EXPRESSION [when CONDITION] [because CITATION, …];` | A displayed value. The last binding whose condition holds wins. |
| `clock EVENT every STEP;` | The event whose one parameter, `dt`, advances `elapsed()` ([elapsed](reference/spec/caveat-elapsed-0.1.md)). A countdown from a fractional clock can show a second too many; one that must agree to the tick uses an [integer clock](reference/spec/caveat-elapsed-0.1.md#integer-clocks). |
| `renewable EVIDENCE limit N;` | Evidence whose name can move to up to N distinct occurrences ([renewal](reference/spec/caveat-renewal-0.1.md)). |
| `for KIND as $NAME { STATEMENT; … };` | The body once per member of KIND: each `entity NAME kind KIND at PLACE;`, in order, after `place PLACE kind KIND;`. `state $m_eaten = 0;` declares `cave_eaten`, `pool_eaten` and so on, and `when target == target.$m` selects the member an event parameter `target kind KIND` names. Every body statement ends with `;` ([repetition](reference/spec/caveat-repetition-0.1.md)). It also serves candidates that get the same readings, such as hosts or packages: one stream and one routing rule per member ([guide](reference/docs/AI_AUTHORING.md#compare-several-candidates)). From 1.2.0 a body may hold a `for` block, which expands in each copy and sees both names; `$index` is the innermost member's ([nested repetition](reference/spec/caveat-nested-repetition-0.1.md)). |

### Rules

A rule is `on EVENT [when CONDITION] EFFECT;`, a procedure step `[when CONDITION] EFFECT;`.
An event's rules run in source order, and each sees what the earlier ones did.

| Effect | Does |
| --- | --- |
| `set STATE = EXPRESSION [because CITATION, …]` | Sets a number. |
| `sample STREAM = EXPRESSION [supports\|opposes CLAIM] [noting ID]` | Records a reading; without a stance it bears on no claim (from 1.1.0). `noting` keeps the text of an `id` value with the reading, such as the version string a number was read from (from 1.1.0, [reading notes](reference/spec/caveat-reading-notes-0.1.md)). |
| `reveal EVIDENCE [supports\|opposes CLAIM]` | Observes evidence, optionally with a stance ([neutral reveal](reference/spec/caveat-neutral-reveal-0.1.md)). |
| `commit SERIES because enough\|budget\|deadline [using EXPRESSION \| citing EXPRESSION, …] [permitted by GRANT [for EXPRESSION]] [retaining CAVEAT [because EVIDENCE], …]` | Makes a decision. Its grounds are what `using` or `citing` reads; `citing` records no value ([citing](reference/spec/caveat-citing-0.1.md)). `retaining C because E` keeps a caveat on purpose with its reason, which must be observed and joins the grounds ([retention reasons](reference/spec/caveat-retention-reasons-0.1.md)). Draft for 1.4: `because once E` refuses the commit as `reason_spent` when E's current occurrence is already a reason another commitment holds, so one confirmation backs one action ([untrusted sources](reference/spec/caveat-untrusted-sources-0.1.md)). GRANT is evidence or `latest(STREAM)`. `for X` requires the grant's value to equal X ([permission](reference/spec/caveat-permission-0.1.md)). To keep "not ready" with its grounds, commit a second series `citing` what it rests on ([guide](reference/docs/AI_AUTHORING.md#record-not-ready-with-its-reasons)). |
| `reopen SERIES because EVIDENCE\|latest(STREAM)\|caveated(STATE, CAVEAT)` | Reopens the current revision. |
| `hold SERIES because …`, `release SERIES because …` | From 1.2.0: keeps the current revision in force. A reopening while it is on hold, by a rule or a trigger, is deferred; `release` ends the hold and applies every deferred reopening. A reopening of a revision that rests on withdrawn evidence is never held. `on_hold(SERIES)` asks whether it is on hold ([holds](reference/spec/caveat-holds-0.1.md)). |
| `resolve SERIES = VALUE because … [materialized CAVEAT, …]` | From 1.3.0: records how the current revision turned out, or `resolve revision(SERIES, K) = …` the Kth's. It changes no basis, reopens nothing and takes no stance; `materialized` names retained caveats that came true. A second outcome is refused as `already_resolved` unless the first one's evidence is withdrawn. `resolved(…)` asks whether a revision has one, `outcome(…)` reads it ([outcomes](reference/spec/caveat-outcomes-0.1.md)). |
| `withdraw EVIDENCE\|latest(STREAM) because EVIDENCE` | Marks it wrong. Later reads carry a `withdrawn` caveat ([withdrawal](reference/spec/caveat-withdrawal-0.1.md)). Draft for 1.4: `withdraw every NAME because EVIDENCE` withdraws every held, observed reading of a stream, or every occurrence of evidence and every reading sampled from it, in one step ([untrusted sources](reference/spec/caveat-untrusted-sources-0.1.md)). A `withdrawn(...)` read carries the reason as evidence, so a decision whose `using` reads it is grounded on the reason as well as the readings ([guide](reference/docs/AI_AUTHORING.md#what-a-withdrawal-adds-to-a-decisions-grounds)). |
| `qualify EVIDENCE with CAVEAT [after SECONDS]`, `unqualify EVIDENCE with CAVEAT` | `qualify` attaches a caveat now, or after that much session time. `unqualify` takes back what `qualify` added, on every held occurrence. The take-back is recorded, not erased, and decisions keep their basis ([take-back](reference/spec/caveat-take-back-0.1.md)). `latest(STREAM)` in place of `EVIDENCE` names a stream's current reading, and `unqualify latest(EVIDENCE)` only the current occurrence (from 1.1.0, [latest](reference/spec/caveat-latest-0.1.md)). |
| `renew EVIDENCE` | Moves the name to a new, **unobserved** occurrence. `observed(EVIDENCE)` is false again until a `reveal`. Earlier occurrences keep what they were about. `occurrence(EVIDENCE, N)` names the Nth, from 1, in `withdraw`, `qualify`, `unqualify`, `reopen … because`, `observed`, `withdrawn` and `carries`; an N that names no held occurrence refuses an effect as `expression` and makes a predicate false ([earlier occurrence](reference/spec/caveat-earlier-occurrence-0.1.md)). `reading(STREAM, N)` names a stream's Nth reading, `STREAM@N`, in the same places (from 1.1.0, [reading index](reference/spec/caveat-reading-index-0.1.md)). |
| `reject "MESSAGE"` | Refuses the whole event. |
| `call PROCEDURE(ARGUMENT, …)` | Runs a procedure's steps. The call's guard and every argument pass their evidence and caveats to everything the steps do, samples included, even an argument the body never reads ([procedures](reference/spec/caveat-reactive-0.7.md#evaluation-order-and-qualification)). Pass only what the steps read; a flag passed to a procedure that samples reaches every new reading. |

### Expressions

| Kind | Forms |
| --- | --- |
| Values | numbers, `true`, `false`, `"text"`, state and parameter names, `PARAMETER.MEMBER` for an `in` parameter |
| Operators | `+ - * /` (`+` also joins text), `== != < <= > >=`, `and or not` |
| Choice | `if(CONDITION, A, B)`, `require(CONDITION, VALUE)` |
| Decisions | `committed(SERIES)`, `reopened(SERIES)`, `rests_on_withdrawn(SERIES)`, `permission_withdrawn(SERIES)`; `rests_on_withdrawn(revision(SERIES, K))` asks it of the Kth revision, from 1, current or past, and is false when K names none (from 1.1.0, [past revisions](reference/spec/caveat-past-revisions-0.1.md)); `in_basis(SERIES, E)` asks whether the current revision's grounds hold `E`, `latest(STREAM)` or `reading(STREAM, N)`, and `in_basis(revision(SERIES, K), …)` asks it of the Kth (from 1.1.0, [in basis](reference/spec/caveat-in-basis-0.1.md)); `resolved(SERIES)` and `resolved(revision(SERIES, K))` ask whether a revision has an outcome (from 1.3.0, [outcomes](reference/spec/caveat-outcomes-0.1.md)) |
| Evidence | `observed(E)`, `withdrawn(E)`, `withdrawn(latest(STREAM))`, `carries(E, CAVEAT)`, `carries(latest(NAME), CAVEAT)` (the current reading or occurrence, from 1.1.0, [latest](reference/spec/caveat-latest-0.1.md)), `has_caveat(STATE, CAVEAT)`, `qualified(VALUE, E[, CAVEAT…])` |
| Histories | `has_sample(STREAM)`, `latest(H)`, `history_count(H)`, `history_at(H, INDEX)`, `fold_history(H, INITIAL, FUNCTION)`, where H is a stream or a series; `outcome(SERIES)` and `outcome(revision(SERIES, K))` read a revision's outcome, refusing as `expression` when it has none (from 1.3.0) |
| Functions | `abs min max clamp floor ceil round sqrt sin cos wrap`; text: `number_text percent_text time_text time_second_text time_total_text text id_text`; `elapsed()` |

A value carries the evidence and caveats it was computed from, and a guard
passes its own to what it guards.

## Outcomes

An event is **accepted**, and everything it did is kept, or **rejected**, and
nothing it did is kept:

| Origin | Codes |
| --- | --- |
| `policy` | `reject` (the program's own), `not_permitted` |
| `input` | `unknown_event`, `payload_invalid`, `bound_exceeded` |
| `evaluation` | `bound_exceeded` (a state outside its range), `decision_in_force` (a commit before the current decision is reopened), `already_committed` (a second commit of a plain commitment, which has no `decisions` declaration), `foreclosed` (a commit of an action an irreversible decision has closed off), `caveated_irreversible` (an irreversible commit resting on a `material` or worse caveat it does not retain), `ungrounded_citation` (a `because` citing what the value never read), `empty_caveated_selection` (no observed witness for a `caveated()` reopening), `unobserved_evidence` (qualifying, citing with `qualified(…)`, withdrawing or reopening on evidence no event has observed), `not_committed` (reopening a decision that was never committed), `reason_spent` (draft for 1.4: a `because once` reason another commitment already holds), `expression` (division by zero, a nonfinite result, a negative `sqrt`, a history index out of range, `latest` of an empty history, or a decision with no numeric `using` value), `requirement_failed` (a false `require(…)`, including inside `clamp` and `wrap`) |
| `limit` | `attention_limit` (examination exceeds the remaining budget), `history_limit`, `identifier_limit`, `renewal_limit`, `scheduled_limit` (more than 4,096 pending `qualify … after`), `work_limit`, `depth_limit`, `provenance_limit` (from 1.4: a value's lineage past 1,024 identifiers or 65,536 name bytes), `evaluation_limit` (from 1.4: one expression past 65,536 evaluated nodes) |

Anything else is **fatal**, for example a type error in an expression. A fatal outcome never counts as a
rejection, and the session must be discarded
([dispatch](reference/spec/caveat-dispatch-0.1.md)). From code,
`session.dispatch(event, payload)` returns an accepted event's snapshot, and
`session.dispatchView(event, payload)` the same outcome with the view in
place of the snapshot, which it does not build: use it when the host reads only the view. With `{ fields: [...] }` as a third argument it builds only the named view fields ([view](reference/spec/caveat-view-0.1.md#selected-fields)); name the fields the host reads, since parts of the view grow with the session. `session.save({ compact: true })` writes a small [replay save](reference/spec/caveat-replay-save-0.1.md). `session.save({ compact: true, positional: true })` writes a [positional](reference/spec/caveat-replay-save-0.2.md) one, which holds several times as many events. In a scenario, a bare
`"rejected": true` matches any `policy` refusal; name the code to tell the program's own `reject` from `not_permitted`.

## Common mistakes

Fresh agents writing programs from these documents, in the authoring trials,
made the first four:

- `--` comments. Use `#` or `//`.
- Bounds beyond `-1e12..1e12`, or a time counter that overflows its bounds.
  Read `elapsed()` instead of accumulating time.
- The same rules written out once per instrument. Use a `proc`;
  `caveat check` points these out.
- Lineage taken for grounds. Grounds are what `using` read; run
  `caveat explain` when unsure.
- Expecting a new reading to reconsider a decision by itself.
- Expecting `renew` to observe evidence. It makes a new, unobserved
  occurrence.

An agent integrating Caveat into an application made these three:

- An event of its own named `tick`. Any `tick` may take only `dt`, within
  `0..0.1` seconds, even with a `clock` declaration: "tick must declare only dt
  with bounds inside 0..0.1 seconds". Name it `advance` and declare
  `clock advance every 1`.
- One parameter name listing a member at different positions in two events:
  "origin.external_text already names a different value". Each `NAME.MEMBER`
  is one program-wide constant. Rename one, such as to `requester_origin`.
- An event parameter named like a state: "parameter limitations shadows state
  or a coordinate". Rename one of them.

## More

- [Worked example](WORKED_EXAMPLE.md): one program through every command.
- [Getting started](GETTING_STARTED.md): a slower first walk through.
- [Authoring guide](reference/docs/AI_AUTHORING.md): time, evidence that ages,
  and running a session from a script.
- The profiles, in order: [reactive 0.1](reference/spec/caveat-reactive-0.1.md)
  through [0.5 histories](reference/spec/caveat-reactive-0.5.md),
  [0.6](reference/spec/caveat-reactive-0.6.md) and
  [0.7 procedures](reference/spec/caveat-reactive-0.7.md), with the
  [documentation index](README.md) for everything else.
- Not covered here: connections, cues and controls in worlds and games,
  attention budgets (`budget`, `examine`, `defer`), modules and bundles.
