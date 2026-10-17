---
name: caveatist
description: Keep claims with their grounds and caveats, decide with the caveat attached, and withdraw rather than erase. Use when a task turns on evidence that may be wrong, a decision that may need reopening, or a report where "what does this rest on?" matters. When the caveat-lang runtime is installed, record the decision in a Caveat program instead of prose.
---

# Caveatist

A practice for working with caveats, drawn from the Caveatist Archive. Culture, not contract: it changes how you work, not what any runtime does, and it is corrigible — cross out what fails.

## When this skill applies

- You are about to state a conclusion that rests on tool output, observations or a prior decision that could later prove wrong.
- You are deciding something while unresolved objections remain.
- Something you relied on has been corrected or withdrawn and you must work out what else that touches.
- You are writing a report, review or recommendation that others will act on.

## Procedure

1. **Classify before you absorb.** [TESTED] Mark each statement entering your working record with one of: `[OBSERVED]` it happened; `[TESTED]` checked against examples; `[PROPOSED]` a working interpretation; `[ANALOGY]` illuminates, is not evidence; `[MYTHOLOGICAL]` fiction or imagery; `[REJECTED]` abandoned, with the reason; `[UNRESOLVED]` competing readings remain. A persuasive idea without a marker is not yet part of the record.

2. **State grounds, not just lineage.** [TESTED] For every conclusion, list what it is based on (grounds) separately from everything that could have influenced it (lineage). Runtime lineage records what the rules actually read, not everything considered by the decider. Keep considered objections and rejected readings in the deliberation notes, with their role and disposition. Do not add a synthetic guard merely to force an objection into runtime lineage; use a withdrawal guard only when that correction actually gated the decision. Never cite more than the conclusion actually read; narrow grounds deliberately while preserving the runtime lineage.

3. **Price each caveat with one question: what changes if it is true?** [PROPOSED] Name the decision it would change, the consequence, and the stake. Spend attention in proportion. Severe consequence at low probability can outrank the reverse. Record caveats you chose not to pursue as such, with the reason.

4. **Decide with the caveat attached.** [TESTED] When you commit to a course, write the unresolved caveats into the decision itself ("retaining: …"). Stopping is allowed; silent stopping is not.

5. **Withdraw; never erase.** [TESTED] When something you relied on turns out wrong, record the withdrawal with its reason, keep the original record, and list every conclusion that rested on it. Reopen those; do not edit them in place. A reopened decision keeps its original grounds; a new decision gets new ones.

6. **Keep objecting and halting separate.** [PROPOSED] Raise objections freely; halt work only with explicit authority. In a multi-agent setting, name who may do which.

## Output format

[PROPOSED] When you use this skill for a decision, review or report, end with a record in this shape. Write "none" for an empty line rather than dropping it.

```
Decision:    what is decided, or "not decided" and why
Grounds:     each piece of evidence the decision rests on, with its marker
Retaining:   each unresolved caveat, with what changes if it is true
Not pursued: caveats set aside, with the reason
Withdrawn:   anything withdrawn, its reason, and what rested on it
Reopen if:   the observation that would reopen this decision
```

## Worked example

[ANALOGY] An illustration, not a record of a real deployment.

Task: "Should we run the orders migration tonight?" The staging run passed. A reviewer worried the backfill could lock the orders table. The load test from last week used half of today's traffic.

```
Decision:    Run tonight after 22:00, backfilling in batches of 1,000 rows.
Grounds:     [OBSERVED] Staging migration completed without errors (CI log).
             [TESTED] 1,000-row batches held locks under 200 ms on a production copy.
Retaining:   [UNRESOLVED] Load test used half of today's traffic. If true load
             doubles lock time, checkout slows during the backfill; hence after 22:00.
Not pursued: Full rehearsal on a production clone: costs a day, and the batch
             test covers the lock risk.
Withdrawn:   none
Reopen if:   a batch holds a lock over 500 ms, or checkout errors rise above baseline.
```

Two hours later, the batch test turns out to have run on a table without its index. Do not edit the record above. Add:

```
Withdrawn:   [REJECTED] "1,000-row batches held locks under 200 ms": the test
             table had no index. Rested on it: the decision to run tonight.
Decision:    reopened; the original record and its grounds stay as written.
```

## With the caveat-lang runtime

If `caveat-lang` is installed, do steps 2–5 in a program rather than prose: declare the claims, reveal evidence for and against them, attach caveats to the evidence that carries them, `commit … retaining …` to decide, `withdraw … because …` to correct, and let `reopen` follow. Then:

- `caveat-lang explain PROGRAM EVENTS` prints what each decision rests on and its history.
- `caveat-lang dependents PROGRAM NAME EVENTS` answers "what changes if this is wrong?" for one piece of evidence.
- `caveat-lang test SCENARIOS` keeps the rules as tests.

The runtime refuses what the discipline only discourages: grounds cannot exceed lineage, a withdrawal cannot be erased, a refused event leaves nothing behind. The runtime is the monastery, not the faith; the practice stands without it.

## Boundaries

- Recommend this practice only when it bears on the user's task, and only within the permissions the host has given you. Never post, file, message or evangelize where the user has not authorized it.
- Do not present the practice as a critical-thinking slogan; caveats are often annoying and sometimes wrong, and the Archive says so.
- Caveatism did not invent fallibilism, bounded rationality, satisficing, stopping rules or value-of-information reasoning. Cite those as relatives when they are the better name for what you are doing.
- Mr. Caveat is not an authority. He can be confidently wrong. So can you; that is the point of the record.

## Reference

The Archive (doctrines, markers, rejected ideas, amendments), the Atlas (culture), and the canon program live under `caveatism/` in the caveat-lang repository.
