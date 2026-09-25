# A worked example: covering seedlings

One complete program, taken through the commands: load it, check it, test it
with a scenario file, and ask why its decision is what it is. The package's
tests follow this page in a fresh directory. They save each file shown and run
each command shown, and they require the output printed here. For the language
itself on one page, read [Caveat on one page](REFERENCE.md).

## A complete program

Two instruments read the soil temperature. When the probe says it is cold and
the grower has said go ahead, the program decides to cover the seedlings. A
warmer reading reopens that decision, and so does finding out that the reading
it rested on was taken wrongly.

<!-- file: frost.cav -->
```caveat
scene "Decide whether to cover the seedlings tonight.";

# What the program reasons about.
claim frost_risk;
claim cover_approved;
claim misreading;
evidence probe from "a soil probe";
evidence drone from "a survey drone";
evidence grower from "the grower's go-ahead";
evidence recheck from "a second look at the probe";
caveat uncalibrated consequence material;
uncalibrated qualifies drone;

# Histories: every reading is kept, and each decision is a new revision.
identifiers limit 64;
readings soil from probe limit 12;
readings aerial from drone limit 12;
readings approvals from grower limit 4;
decisions cover limit 4 reopened by soil opposing frost_risk, aerial opposing frost_risk;
state covered = 0 min 0 max 1;

# What can happen, and what each event carries.
event probe_read celsius min -40 max 60;
event drone_read celsius min -40 max 60;
event approve by id;
event decide;
event misread;
event lay_cover;

# One set of rules for both instruments.
proc record(s readings, celsius) {
    when celsius <= 2 sample s = celsius supports frost_risk;
    when celsius > 2 sample s = celsius opposes frost_risk;
};
on probe_read call record(soil, celsius);
on drone_read call record(aerial, celsius);

on approve sample approvals = by supports cover_approved;

on decide when not has_sample(soil) reject "No probe reading yet.";
on decide when committed(cover) and not reopened(cover) reject "Already decided.";
on decide when latest(soil) <= 2
    commit cover because enough using latest(soil) permitted by latest(approvals);

# A reading taken wrongly is withdrawn, not contradicted.
on misread when not has_sample(soil) reject "No probe reading to withdraw.";
on misread when not observed(recheck) reveal recheck supports misreading;
on misread withdraw latest(soil) because recheck;
on misread when committed(cover) and not reopened(cover) and rests_on_withdrawn(cover)
    reopen cover because recheck;

on lay_cover when not committed(cover) or reopened(cover) reject "Nothing decided to lay.";
on lay_cover set covered = 1;

bind advice.text = "Waiting for a decision";
bind advice.text = "Cover the seedlings" when committed(cover) and not reopened(cover);
bind advice.text = "Think again" when reopened(cover);
bind advice.soil = "no reading";
bind advice.soil = number_text(latest(soil)) + " C" when has_sample(soil);
bind advice.approved_by = "nobody yet";
bind advice.approved_by = id_text(latest(approvals)) when has_sample(approvals);
```

What each part does:

- **Evidence and caveats.** `uncalibrated qualifies drone` attaches the caveat
  to the drone. Every drone reading, and every value computed from one, carries
  `uncalibrated`. A source label such as `"a soil probe"` describes the
  evidence; it does not authenticate it.
- **Readings.** `readings soil from probe limit 12` keeps up to 12 probe
  readings, named `soil@1`, `soil@2` and so on. Each one is a separate
  occurrence of `probe` evidence, and old ones are never overwritten. A
  `sample` beyond the limit refuses its event as `limit/history_limit`.
- **Decisions.** `decisions cover limit 4` holds up to four revisions,
  `cover@1` onward. A new revision can be made only after the current one is
  reopened. `reopened by soil opposing frost_risk` reopens the revision in
  force whenever a probe reading that opposes frost risk is taken.
- **Identifiers.** `event approve by id` receives text such as `"sam"` and
  holds it as a number, so a stream can record it. `id_text` gives the text
  back. `identifiers limit 64` is required before any `id` parameter.
- **Procedures.** `proc record(s readings, celsius)` states the rules once;
  each `call` passes a stream and a number.
- **Commitments.**
  `commit cover because enough using latest(soil) permitted by latest(approvals)`
  makes a decision.
  - Its **grounds** are what `using` reads: here `soil@1` and its caveats.
  - The **grant** is recorded apart, as what permitted it. Without one, the
    commit is refused as `policy/not_permitted`.
  - The guard (`when latest(soil) <= 2`) is in its **lineage**, what could have
    influenced it, but not in its grounds.
- **Withdrawal.** `withdraw latest(soil) because recheck` marks a reading as
  wrong. Later reads of it carry a `withdrawn` caveat, and the decision keeps
  the grounds it was made on. It is reopened only because the next rule says
  so.
- **Refusal.** `reject "…"` refuses the whole event, so nothing an earlier rule
  in it did remains. Rules run in source order.
- **Displayed values.** `bind advice.text = … when …` displays a value, and
  the last binding whose condition holds wins.

<!-- run: npx --no-install caveat validate frost.cav -->
```text
frost.cav loads.
  events: approve (by id), decide, drone_read (celsius -40..60), lay_cover, misread, probe_read (celsius -40..60)
  reading streams: aerial from drone, limit 12, approvals from grower, limit 4, soil from probe, limit 12
  decision series: cover, limit 4
  displayed: advice.approved_by, advice.soil, advice.text
```

<!-- run: npx --no-install caveat check frost.cav -->
```text
frost.cav: no warnings.
```

`check` would have warned had the two `on … read` events each spelled out the
same rules with a different stream (`C001 repeated-rules`). It would also have
warned had nothing reopened `cover` (`C002 no-reopening-path`). Both warnings
are advice, silenced by `# caveat check: allow NAME` on the line above when the
pattern is intended.

## Test it

A scenario file states what the program must do. Each scenario starts a fresh
session and runs its steps in order.

<!-- file: frost.scenarios.json -->
```json
{
  "schema": "caveat-scenarios/0.1",
  "source": "frost.cav",
  "scenarios": [
    {
      "id": "F01",
      "title": "A cold reading and a go-ahead decide; a warm reading reopens the decision, which keeps its grounds",
      "steps": [
        { "send": "probe_read", "payload": { "celsius": 1 } },
        { "send": "approve", "payload": { "by": "sam" } },
        { "send": "decide" },
        { "expect": {
            "/bindings/advice/text": "Cover the seedlings",
            "/bindings/advice/approved_by": "sam",
            "/commitment_grounds/cover@1": { "evidence": ["soil@1"], "caveats": [] },
            "/commitment_permissions/cover@1": { "grant": "approvals@1" }
        } },
        { "checkpoint": "decided" },
        { "resume": true },
        { "send": "probe_read", "payload": { "celsius": 6 } },
        { "expect": {
            "/bindings/advice/text": "Think again",
            "/decision_journal/1": { "change": "reopened", "because": ["soil@2"] }
        } },
        { "same_as": "decided", "paths": ["/decision_journal/0", "/commitment_grounds/cover@1"] }
      ]
    },
    {
      "id": "F02",
      "title": "Without a go-ahead the decision is refused",
      "steps": [
        { "send": "probe_read", "payload": { "celsius": 1 } },
        { "send": "decide", "rejected": { "origin": "policy", "code": "not_permitted" } },
        { "expect": { "/bindings/advice/text": "Waiting for a decision" } }
      ]
    },
    {
      "id": "F03",
      "title": "A misread probe reading is withdrawn, and the decision resting on it reopens",
      "steps": [
        { "send": "probe_read", "payload": { "celsius": 1 } },
        { "send": "approve", "payload": { "by": "sam" } },
        { "send": "decide" },
        { "checkpoint": "decided" },
        { "send": "misread" },
        { "expect": {
            "/bindings/advice/text": "Think again",
            "/decision_journal/1": { "change": "reopened", "because": ["recheck"] }
        } },
        { "same_as": "decided", "paths": ["/commitment_grounds/cover@1"] }
      ]
    },
    {
      "id": "F04",
      "title": "Refused events leave nothing behind",
      "steps": [
        { "send": "probe_read", "payload": { "celsius": 99 }, "rejected": { "origin": "input", "code": "bound_exceeded" } },
        { "send": "decide", "rejected": "No probe reading yet." },
        { "send": "lay_cover", "rejected": { "origin": "policy", "code": "reject" } },
        { "expect": { "/bindings/advice/soil": "no reading" } }
      ]
    }
  ]
}
```

<!-- run: npx --no-install caveat test frost.scenarios.json -->
```text
PASS F01  A cold reading and a go-ahead decide; a warm reading reopens the decision, which keeps its grounds  (4 events, 0 rejected, 1 resumes)
PASS F02  Without a go-ahead the decision is refused  (2 events, 1 rejected, 0 resumes)
PASS F03  A misread probe reading is withdrawn, and the decision resting on it reopens  (4 events, 0 rejected, 0 resumes)
PASS F04  Refused events leave nothing behind  (3 events, 3 rejected, 0 resumes)
4 passed, 0 failed (frost.scenarios.json)
```

The step types:

- **`send`** sends an event, with an optional `payload`. Without `rejected`,
  the event must be accepted.
- **`expect`** matches JSON Pointer paths into the snapshot. Objects match
  partially and arrays match in order.
- **`checkpoint` and `same_as`** require records not to change.
- **`resume`** saves the session and continues from the save.

An expected rejection takes one of these forms:

| `rejected` | Matches |
| --- | --- |
| `true` | Any `policy` refusal: the program's own `reject`, or a commit refused as `not_permitted`. |
| `"text"` | A `policy` refusal with exactly this message. |
| `{"origin": "policy", "code": "reject"}` | The program's own `reject`, with any message. |
| `{"origin": "policy", "code": "not_permitted"}` | A commit whose `permitted by` found no grant. |
| `{"origin": "input", "code": "bound_exceeded"}` | A payload outside its declared range. Codes for each origin are in [dispatch](reference/spec/caveat-dispatch-0.1.md). |

Name the code whenever two kinds of refusal could both happen. Without being
asked, the runner also checks four things:

- every refused event changed nothing;
- a resumed session agrees with the one it came from;
- the final save restores;
- grounds stay within lineage.

## See why

An events file holds one event per line:

<!-- file: events.jsonl -->
```json
{"event": "probe_read", "payload": {"celsius": 1}}
{"event": "approve", "payload": {"by": "sam"}}
{"event": "decide"}
{"event": "drone_read", "payload": {"celsius": 0}}
{"event": "probe_read", "payload": {"celsius": 6}}
```

<!-- run: npx --no-install caveat explain frost.cav events.jsonl -->
```text
frost.cav after 5 events (sequence 5)

Events
    1  probe_read {"celsius":1}  accepted
    2  approve {"by":"sam"}  accepted
    3  decide  accepted
    4  drone_read {"celsius":0}  accepted
    5  probe_read {"celsius":6}  accepted

Decisions
  cover: 1 of at most 4
    cover@1 = 1  reopened
      based on soil@1
      permitted by approvals@1
      could also have been influenced by approvals@1
      #3 decide: committed because soil@1
      #5 probe_read: reopened because soil@2

Evidence
  soil@1 = 1 supports frost_risk  (#1 probe_read)
  approvals@1 = 1 supports cover_approved  (#2 approve)
  aerial@1 = 0 supports frost_risk  (#4 drone_read)  caveats: uncalibrated
  soil@2 = 6 opposes frost_risk  (#5 probe_read)

Displayed
  advice.approved_by = "sam"  because approvals@1
  advice.soil = "6 C"  because soil@2
  advice.text = "Think again"  because approvals@1, soil@1, soil@2
```

Three words in this output mean different things:

- **Based on** is the decision's **grounds**: what its `using` read, fixed when
  it was made. A later reading never changes them.
- **Permitted by** is the grant its `permitted by` clause found. That grant is
  not grounds.
- **Could also have been influenced by** is the rest of its **lineage**:
  guards, conditions and grants that touched it.

The cold drone reading (`aerial@1`) is not in the grounds, because `using`
read only `soil`. A displayed value cites everything it could have been
influenced by, unless its binding names less with `because`. To ask the
question the other way round, run
`npx --no-install caveat dependents frost.cav soil events.jsonl`.
