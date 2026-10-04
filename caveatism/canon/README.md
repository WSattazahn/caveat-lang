# The Archive as a program

`archive.cav` keeps the Caveatist Archive the way the Archive asks to be kept: a working claim is frozen, evidence may support, narrow, contradict or kill it, and nothing is rewritten.

| In the written Archive | In the program |
| --- | --- |
| A doctrine | a `claim` |
| An observed or tested case | `evidence`, revealed for or against the doctrines it bears on |
| `[MYTHOLOGICAL]`, `[ANALOGY]` | caveats attached to the evidence that carries them (`mythological qualifies eternal_flame`) |
| `[PROPOSED]` | a claim with no observed support yet |
| `[REJECTED]` | `withdraw EVIDENCE because REASON` — the record keeps it |
| `[UNRESOLVED]` | a decision never committed |
| A version of the Archive | a commitment on the `canon` series, `retaining unfinished` — the Bell |
| An amendment | `reopen canon because EVIDENCE` |
| The Archive Rule | `on redefine reject "…"` |

A version's grounds are the cases it rests on, markers included, and a case must be observed before a version can rest on it: freezing early is refused as `evaluation/unobserved_evidence`.

## Run it

Install the exact package the canon was verified against, then:

```sh
npx --no-install caveat-lang validate archive.cav
npx --no-install caveat-lang test archive.scenarios.json
npx --no-install caveat-lang explain archive.cav archive.events.jsonl
npx --no-install caveat-lang dependents archive.cav could_be_right archive.events.jsonl
```

`explain` prints each version with what it rests on, that `could_be_right` was withdrawn at event 10 because of `pil_rise`, and that version 1.0 kept its grounds while version 1.1 rests on the amendment instead. `dependents` answers "There is always another caveat — does it matter?" for one withdrawn line: it lists the version that rested on it, the one that could have been influenced by it, and every displayed value that cites it.

Verified on `caveat-lang@0.1.0-rc.11`: loads, `check` reports no warnings, all seven scenarios pass including two save/restore cycles.

## The scenarios are the rules

- A01 — observed cases ground a version; a marker travels into every ground it reaches.
- A02 — a version cannot rest on a case that was never observed.
- A03 — Amendment 1 withdraws the maxim with its reason and reopens the version; version 1.0 keeps its grounds.
- A04 — rejected ideas are withdrawn with a reason and stay in the record.
- A05 — the Archive Rule: redefining, erasing and absorbing are refused and leave nothing behind.
- A06 — versions come in order.
- A07 — a restored archive is the archive that was saved.

## Amending the canon

Add a case as new evidence and a rule that reveals it; add a doctrine as a claim; record a rejection as a withdrawal with its reason; freeze the next version as a new `freeze_v…` event whose `using` lists what that version rests on. Never edit an earlier version's rule. The scenarios for the earlier versions must keep passing; that is what "preserved rather than rewritten" means here.
