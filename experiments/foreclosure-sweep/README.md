# Foreclosure sweep and prototype

Evidence for [Foreclosure 0.1](../../spec/caveat-foreclosure-0.1.md), the
1.2 draft that lets a decision declare that it is irreversible and what it
closes off. The spec decides; this folder keeps what it was drawn from.

## The design note

A design note of 2026-10-13 explored the idea, from idea A of a review of
Wissner-Gross and Freer's causal entropic forces (Physical Review Letters 110,
168702, 2013). The paper is about physics, not languages; what it lends is the
question an agent that keeps its options open needs to ask: which commitments
spend options, and what did they spend? The note proposed four tiers: a guide
note, a declared `irreversible`, foreclosure as a recorded relation, and option
accounting. The owner approved foreclosure for 1.2 on 2026-10-13. The spec
takes the second and third tiers and settles the note's questions in its
"Questions settled" section.

## The prototype

The note's prototype models a migration whose drop of the old table can't be
undone, written with what 1.0 has. It ran on `caveat-lang@0.1.0-rc.19`; its
two scenarios passed (X01: a sampled dry run can't ground the drop, so
rollback stays open; X02: after the drop, rollback is refused). It is kept
here as text, not as a `.cav` file, so that the corpus the sweep reads stays
the repository's own programs.

```caveat
scene "Dropping the old table cannot be undone, so it needs uncaveated grounds and closes the rollback option.";

claim migration_safe;
evidence full_check from "full migration dry run";
evidence sample_check from "sampled migration dry run";
caveat sampled_only consequence high;
sampled_only qualifies sample_check;
evidence approvals from "owner";

readings full_runs from full_check limit 8;
readings sampled_runs from sample_check limit 8;
readings grants from approvals limit 4;
decisions drop_old_table limit 1;
decisions rollback limit 4;

event dry_run rows_ok min 0 max 1, full in sampled complete;
event approve;
event drop;
event roll_back;

on dry_run when full == full.complete and rows_ok == 1 sample full_runs = 1 supports migration_safe;
on dry_run when full == full.complete and rows_ok == 0 sample full_runs = 0 opposes migration_safe;
on dry_run when full == full.sampled and rows_ok == 1 sample sampled_runs = 1 supports migration_safe;
on dry_run when full == full.sampled and rows_ok == 0 sample sampled_runs = 0 opposes migration_safe;
on approve sample grants = 1 supports migration_safe;

// Foreclosure by convention: an irreversible commit accepts only a complete
// dry run, never one carrying sampled_only.
on drop when not has_sample(full_runs) reject "irreversible: needs a complete dry run";
on drop commit drop_old_table because enough using latest(full_runs) permitted by latest(grants);
on roll_back when committed(drop_old_table) reject "foreclosed by drop_old_table";
on roll_back commit rollback because enough using 1;
```

`explain` after a complete dry run, an approval, the drop and a rollback:

```text
foreclose.cav after 4 events (sequence 3)

Events
    1  dry_run {"rows_ok":1,"full":"complete"}  accepted
    2  approve  accepted
    3  drop  accepted
    4  roll_back  refused (policy): foreclosed by drop_old_table

Decisions
  made under source sha256:3d8fc80e0aa4ab1f1aeb00b6d75004c6a32f6420311f2551be757fcdb5a3ffd5
  drop_old_table: 1 of at most 1
    drop_old_table@1 = 1  in force
      based on full_runs@1
      full_runs@1: observed at #1 dry_run {"rows_ok":1,"full":"complete"}
      permitted by grants@1
      could also have been influenced by grants@1
      #3 drop: committed because full_runs@1
  rollback: 0 of at most 4

Evidence
  full_runs@1 = 1 supports migration_safe  (#1 dry_run)
  grants@1 = 1 supports migration_safe  (#2 approve)

Displayed
  nothing bound
```

The refused `roll_back` and `drop_old_table@1` are not linked, and nothing
marks the drop as the commitment that can't be undone.

## The sweep

`node experiments/foreclosure-sweep/sweep.mjs` reads every tracked Caveat
program's source and prints a summary; `--write` also rewrites
[`RESULTS.json`](RESULTS.json). It runs no program. The spec's "Corpus sweep"
section reads the results.
