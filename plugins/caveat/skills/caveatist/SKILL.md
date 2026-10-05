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

2. **State grounds, not just lineage.** [TESTED] For every conclusion, list what it is based on (grounds) separately from everything that could have influenced it (lineage). Never cite more than the conclusion actually read; cite less only if the omission is deliberate and the full lineage stays available.

3. **Price each caveat with one question: what changes if it is true?** [PROPOSED] Name the decision it would change, the consequence, and the stake. Spend attention in proportion. Severe consequence at low probability can outrank the reverse. Record caveats you chose not to pursue as such, with the reason.

4. **Decide with the caveat attached.** [TESTED] When you commit to a course, write the unresolved caveats into the decision itself ("retaining: …"). Stopping is allowed; silent stopping is not.

5. **Withdraw; never erase.** [TESTED] When something you relied on turns out wrong, record the withdrawal with its reason, keep the original record, and list every conclusion that rested on it. Reopen those; do not edit them in place. A reopened decision keeps its original grounds; a new decision gets new ones.

6. **Keep objecting and halting separate.** [PROPOSED] Raise objections freely; halt work only with explicit authority. In a multi-agent setting, name who may do which.

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
