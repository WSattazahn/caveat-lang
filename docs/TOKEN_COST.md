# What Caveat costs an agent in tokens

Status: a measurement, 2026-10-31. Nothing here changes the language, the kit
or any promise. The cuts it proposes are listed at the end, each for its own
pull request.

An agent pays for Caveat in tokens four ways: what it reads before it writes,
the program and tests it writes, the tool replies it reads back, and the
records it reads later. This page counts each one on files in this repository.

## How it was measured

[`experiments/token-cost/measure.mjs`](../experiments/token-cost/measure.mjs)
counts every number below and writes them to
[`results.json`](../experiments/token-cost/results.json). It runs this
checkout's kit (main at `9185f46`) against the caveat-lang 1.4.1 WebAssembly
runtime from npm:

```sh
npm install --no-save --prefix /tmp/tok js-tiktoken@1.0.21 @anthropic-ai/tokenizer@0.0.4 caveat-lang@1.4.1
node experiments/token-cost/measure.mjs --deps /tmp/tok \
  --runtime /tmp/tok/node_modules/caveat-lang/runtime \
  --output experiments/token-cost/results.json
```

The tokenizer of current Claude models is not public, so three stand-ins
count every file: OpenAI's o200k_base and cl100k_base, and the legacy Claude
tokenizer. They agree within 5% on prose and JSON; on Caveat source the legacy
Claude tokenizer counts 10 to 16% more. Numbers below are o200k_base. Read
them as sizes to within about a fifth, and comparisons between them as
reliable.

## Short answer

- **Writing a program costs about 650 to 1,100 tokens** for the trial tasks
  (12 per line). The agents' own scenario tests cost 2 to 9 times as much as
  the program.
- **Reading first costs 1,500 to 27,500 tokens**, depending on how far down
  the documents an agent goes. The MCP tool list adds 3,951 tokens to every
  session.
- **Tool replies are 190 to 5,500 tokens.** Nearly half of every
  `caveat_dispatch` reply is the save, which the agent must also write back,
  as output, on its next call.
- **`explain` text grows with grounds times revisions.** A decision resting on
  60 readings and revised 16 times explains in 25,207 tokens; 57% of that
  repeats the same observations under each revision.

## Writing a program

| Programs | Count | Tokens (min–max, mean) | Tokens per line |
| --- | ---: | --- | ---: |
| `examples/*.cav` | 10 | 72–1,839, 415 | 9 |
| `kit/examples/**/*.cav` | 10 | 209–1,993, 817 | 12 |
| Authoring trial v3, agents' programs | 4 | 643–1,141, 858 | 10 |
| Authoring trial v4, agents' programs | 8 | 645–1,094, 852 | 12 |

Where the tokens go in the twelve trial programs (v3 and v4 together):

- **`on` rules and `bind` lines are about 60 to 70% of the code.** Each one names
  its event or binding again: `on gust when ...`, `bind hud.decision = ...`.
  Heads written a second time for the same event or binding are about 10% of
  the code. The usual case is a default and its overrides (`bind hud.decision
  = "none";` then `bind hud.decision = "go" when ...`).
- **String literals are 15 to 19%**: source descriptions, refusal messages,
  scene text.
- **Comments are 7% in v4 and 27% in v3**, by the agent's choice.
- **Tests cost more than the program.** The scenario files the v4 agents wrote
  are 1,983 to 4,216 tokens, 2.3 to 5.5 times their program; in v3, 4.5 to 9.1
  times. A test suite in any language costs tokens, so this is not a cost of
  Caveat as such, but it is where most of an author's output goes.
- **No agent used a procedure.** v4's ferry task has parallel rules for each
  instrument; every author wrote each instrument's rules out
  ([trial v4 results](../experiments/agent-authoring/v4/RESULTS.md),
  "Procedures and full histories"). Neither [AGENT_START](../kit/docs/AGENT_START.md)
  nor the [authoring card](../kit/docs/AUTHORING_CARD.md) shows `proc`, and
  AGENT_START does not show `for ... as` templates either.

## Reading before writing

| File | Tokens |
| --- | ---: |
| `skills/caveatist/SKILL.md` | 1,497 |
| `llms.txt` | 1,294 |
| `kit/docs/AGENT_START.md` | 2,250 |
| `kit/docs/AUTHORING_CARD.md` | 3,332 |
| `kit/docs/MCP.md` | 2,522 |
| `kit/docs/WORKED_EXAMPLE.md` | 3,536 |
| `kit/docs/REFERENCE.md` | 6,420 |
| `docs/AI_AUTHORING.md` | 13,916 |
| MCP `tools/list` (7 tools) | 3,951 |

AGENT_START sends an agent to the card, then the one-page reference, then the
authoring guide: 25,918 tokens if read in full. The tool list is in context on
every turn of an MCP session. By tool: `caveat_find` 823, `caveat_dependents`
716, `caveat_dispatch` 671, `caveat_test` 578, `caveat_explain` 524,
`caveat_check` 361, `caveat_validate` 276.

## Tool replies

Two programs: v4 run A's ferry program with six events, and the retractions
kit example with its fourteen events.

| Tool | Ferry | Retractions |
| --- | ---: | ---: |
| `caveat_check` | 191 | 189 |
| `caveat_validate` | 317 | 405 |
| `caveat_dependents` | 713 | |
| `caveat_explain` | 1,278 | 2,874 |
| `caveat_dispatch` | 2,441 | 5,547 |

Every reply's text is the JSON of its structured result. The CLI's `explain`
text for the same ferry history is 571 tokens, under half the MCP reply.

What a `caveat_dispatch` reply holds, by tokens removed when each part is
left out:

| Part | Ferry | Retractions |
| --- | ---: | ---: |
| The save | 1,145 (47%) | 2,655 (48%) |
| Source hashes and runtime identity, repeated | 262 (11%) | 583 (11%) |
| `observed_by` (repeats the evidence list) | 49 | 176 |
| A schema tag on every event's outcome | 72 | 168 |
| `lineage`, where it equals `grounds` | 26 | 97 |

**The save costs twice.** The agent reads it in the reply, then writes it
back unchanged as the next call's `save` argument, and output tokens are
priced several times higher than input. It grows by 150 to 175 tokens an
event. Keeping the retractions ledger one event per call writes 17,658
tokens of save back across 14 calls, for 130 tokens of events.

The library already has smaller saves. For the same two histories:

| Save | Ferry | Retractions |
| --- | ---: | ---: |
| `session.save()`, what `caveat_dispatch` returns today | 1,143 | 2,653 |
| `session.save({ compact: true })` | 132 | 188 |
| `session.save({ compact: true, positional: true })` | 78 | 60 |
| The events themselves | 58 | 130 |

## Reading a record later

`explain` prints, under every revision of a decision, one "observed at" line
for each reading it rests on, then lists them again in the revision's
"committed because" line. The [probe program](../experiments/token-cost/growth.cav)
records G readings, then decides K times with a reopening between each:

| Grounds | Revisions | Events | `explain` text | Without repeated "observed at" lines |
| ---: | ---: | ---: | ---: | ---: |
| 4 | 1 | 5 | 351 | 291 |
| 4 | 16 | 35 | 2,751 | 1,791 |
| 16 | 4 | 23 | 2,331 | 1,371 |
| 16 | 16 | 47 | 7,563 | 3,723 |
| 60 | 1 | 61 | 3,487 | 2,587 |
| 60 | 4 | 67 | 7,831 | 4,231 |
| 60 | 16 | 91 | 25,207 | 10,807 |

The project's own ledger (`ledger/project.cav`, 242 events) explains in 8,139
tokens, because its decisions rest on few readings each. Records where many
answers rest on many reports, as in the agent tests, grow much faster.

## Compared with another way of keeping the record

This repository holds no task done both in Caveat and in another form, so no
public comparison is fair yet. The private agent tests compared Caveat
records with plain notes for the same jobs; their numbers stay private unless
the owner releases them.

## The three cheapest cuts

None changes what any program means, and all three fit 1.x.

1. **Return the compact save from `caveat_dispatch`.** The tool ships first in
   1.5.0, so its save can change before release at no compatibility cost.
   Measured: 1,143 to 78 tokens on the ferry history and 2,653 to 60 on the
   retractions history, on every call, read once and written once. The
   compact save restores like any save and falls back to the full save when
   the events do not fit ([Replay save 0.1](../spec/caveat-replay-save-0.1.md),
   [0.2](../spec/caveat-replay-save-0.2.md)).
   About half of every dispatch reply goes away.
2. **Print each observation once in `explain` text.** List observations in
   the Evidence section and name them by id under each revision, and drop a
   "committed because" list that repeats "based on". Measured on the probe:
   25,207 to 10,807 tokens at 60 grounds and 16 revisions. `explain`'s text
   is not promised ([Compatibility](COMPATIBILITY.md), "Text written for
   people"); `--json` and the MCP report are unchanged.
3. **Show templates and procedures where agents start.** Put one
   `for ... as` template and one `proc` into AGENT_START and the authoring
   card, with when to reach for each: the same rules for several items. No
   trial agent used a procedure, and nothing they read first shows one. Docs
   only. The saving is not measured yet: the next authoring trial should count
   program tokens against v4's.

Worth doing later, at more cost:

- **Shorter tool descriptions.** 3,951 tokens on every turn. A release may
  reword them ([Compatibility](COMPATIBILITY.md), "MCP tool descriptions"),
  but each one carries rules agents rely on, so this needs a trial, not a
  trim.
- **A brief report.** Hashes, runtime identity, schema tags and repeated
  lists are 17 to 19% of a dispatch reply, and about a third of what is left
  once the save is gone. Removing them
  from `caveat-explain/0.1` changes a promised format, so it starts as a 1.x
  opt-in; making it the default is [2.0 candidate 9](2.0-CANDIDATES.md#9-every-report-repeats-hashes-and-schema-tags).

## Limits

- **Stand-in tokenizers.** See "How it was measured".
- **Files, not sessions.** These are the sizes of what an agent reads and
  writes. A session also spends tokens on its own reasoning, retries and
  error messages, and on start-up that does not depend on Caveat.
- **Caching.** A client may cache the tool list and documents between turns,
  which makes re-reading them cheaper than these counts. Output, such as a
  save written back, a program or a scenario file, is never cached.
- **Two tasks.** The trial programs come from two small tasks of the same
  shape, written by agents of one model family.
