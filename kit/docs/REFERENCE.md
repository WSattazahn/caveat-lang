# Caveat on one page

A Caveat program keeps track of what each decision rests on. It declares what
it reasons about. Events bring readings in, and rules record them, make
decisions and reopen them. The runtime keeps every reading and every revision,
and can say what each decision was based on and what permitted it.

This page is the language in brief. The [worked example](WORKED_EXAMPLE.md)
takes one complete program through every command, and the package's tests run
its files and check the output it shows. The excerpts below are checked to
appear in that program. The tables summarize the specifications they link to,
which are authoritative.

## Commands

Run each as `npx --no-install caveat …` in a project with the package
installed.

| Command | What it answers |
| --- | --- |
| `validate PROGRAM` | Does the program load? Lists its events, histories and displayed values. |
| `check PROGRAM` | Are there patterns worth a second look? Advisory warnings with a code and a line ([check](reference/spec/caveat-check-0.1.md)). |
| `test SCENARIOS…` | Did each scenario get the outcomes and snapshot values it expects ([scenarios](reference/spec/caveat-scenarios-0.1.md))? |
| `explain PROGRAM [EVENTS]` | After these events, what does each decision rest on, and why is each displayed value what it is? |
| `dependents PROGRAM NAME [EVENTS]` | What rests on this evidence, reading stream or caveat? |
| `replay PROGRAM EVENTS` | What was the snapshot after each event? |
| `serve PROGRAM` | Keeps a session open over standard input and output ([serve](reference/spec/caveat-serve-0.1.md)). |
| `init [DIRECTORY]` | Writes the getting-started program, its scenarios and an events file. |

Each answers only its own question. A program that loads can still do the
wrong thing, and a clean check is not a passing test.

## The core

Readings are kept, and a decision is a series of revisions:

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
| `caveat NAME consequence LEVEL;` | A caveat. LEVEL is `negligible`, `low`, `material`, `high` or `catastrophic`. |
| `CAVEAT qualifies EVIDENCE;` | The caveat goes with that evidence and everything computed from it. |
| `readings NAME from EVIDENCE limit N;` | A stream of up to N readings. |
| `decisions NAME limit N [reopened by STREAM [supporting\|opposing CLAIM], …];` | A decision with up to N revisions ([triggers](reference/spec/caveat-reopening-triggers-0.1.md)). |
| `state NAME = EXPRESSION [min A max B];` | A number. Bounds default to, and may not exceed, `-1e12..1e12`. |
| `event NAME [PARAMETER, …];` | A parameter is `NAME min A max B`, `NAME in A B …`, `NAME kind KIND` or `NAME id` ([identifiers](reference/spec/caveat-identifiers-0.1.md)). |
| `identifiers limit N;` | Required before any `id` parameter. |
| `define NAME = EXPRESSION;` | A named expression, inlined where read ([define](reference/spec/caveat-define-0.1.md)). |
| `fn NAME(A, B) = EXPRESSION;` | A pure function. It cannot read state or evidence. |
| `proc NAME(PARAMETER, …) { STEP; … };` | Steps run by `call`. A parameter is a number, or `NAME readings`, `decisions`, `evidence`, `claim` or `caveat` ([procedures](reference/spec/caveat-procedure-symbols-0.1.md)). |
| `bind TARGET.PROPERTY = EXPRESSION [when CONDITION] [because CITATION, …];` | A displayed value. The last binding whose condition holds wins. |
| `clock EVENT every STEP;` | The event whose one parameter, `dt`, advances `elapsed()` ([elapsed](reference/spec/caveat-elapsed-0.1.md)). |
| `renewable EVIDENCE limit N;` | Evidence whose name can move to up to N distinct occurrences ([renewal](reference/spec/caveat-renewal-0.1.md)). |

### Rules

A rule is `on EVENT [when CONDITION] EFFECT;`. A procedure step is
`[when CONDITION] EFFECT;`. An event's rules run in source order, and each
sees what the earlier ones did.

| Effect | Does |
| --- | --- |
| `set STATE = EXPRESSION [because CITATION, …]` | Sets a number. |
| `sample STREAM = EXPRESSION supports\|opposes CLAIM` | Records a reading. |
| `reveal EVIDENCE supports\|opposes CLAIM` | Observes evidence. |
| `commit SERIES because enough\|budget\|deadline [using EXPRESSION] [permitted by GRANT [for EXPRESSION]] [retaining CAVEAT, …]` | Makes a decision. GRANT is evidence or `latest(STREAM)`. `for X` requires the grant's value to equal X ([permission](reference/spec/caveat-permission-0.1.md)). |
| `reopen SERIES because EVIDENCE\|latest(STREAM)\|caveated(STATE, CAVEAT)` | Reopens the current revision. |
| `withdraw EVIDENCE\|latest(STREAM) because EVIDENCE` | Marks it wrong. Later reads carry a `withdrawn` caveat ([withdrawal](reference/spec/caveat-withdrawal-0.1.md)). |
| `qualify EVIDENCE with CAVEAT [after SECONDS]` | Attaches a caveat now, or after that much session time. |
| `renew EVIDENCE` | Moves the name to a new, **unobserved** occurrence. `observed(EVIDENCE)` is false again until a `reveal`. Earlier occurrences keep what they were about. |
| `reject "MESSAGE"` | Refuses the whole event. |
| `call PROCEDURE(ARGUMENT, …)` | Runs a procedure's steps. |

### Expressions

| Kind | Forms |
| --- | --- |
| Values | numbers, `true`, `false`, `"text"`, state and parameter names, `PARAMETER.MEMBER` for an `in` parameter |
| Operators | `+ - * /` (`+` also joins text), `== != < <= > >=`, `and or not` |
| Choice | `if(CONDITION, A, B)`, `require(CONDITION, VALUE)` |
| Decisions | `committed(SERIES)`, `reopened(SERIES)`, `rests_on_withdrawn(SERIES)`, `permission_withdrawn(SERIES)` |
| Evidence | `observed(E)`, `withdrawn(E)`, `withdrawn(latest(STREAM))`, `carries(E, CAVEAT)`, `has_caveat(STATE, CAVEAT)`, `qualified(VALUE, E[, CAVEAT…])` |
| Histories | `has_sample(STREAM)`, `latest(H)`, `history_count(H)`, `history_at(H, INDEX)`, `fold_history(H, INITIAL, FUNCTION)`, where H is a stream or a series |
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
| `evaluation` | `bound_exceeded` (a state outside its range) |
| `limit` | `history_limit`, `identifier_limit`, `work_limit`, `depth_limit` |

Anything else is **fatal**, for example `require(false, …)`, arithmetic that
fails, or `latest` of an empty history. A fatal outcome never counts as a
rejection, and the session must be discarded
([dispatch](reference/spec/caveat-dispatch-0.1.md)). In a scenario, a bare
`"rejected": true` matches any `policy` refusal. Name the code to tell the
program's own `reject` from `not_permitted`.

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
- Not covered here: worlds and games (`place`, `entity`, `for` blocks, cues,
  controls), attention budgets (`budget`, `examine`, `defer`), modules and
  bundles.
