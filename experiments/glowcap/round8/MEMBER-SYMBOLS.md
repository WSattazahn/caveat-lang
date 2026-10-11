# Round 8 against Member Symbols §11.6

Written 2026-10-11 for the [rc.18 plan](../../../docs/releases/rc18-development.md)
(P8, round 8 work-list item 6). Report only: [Member Symbols
0.1](../../../spec/caveat-member-symbols-0.1.md) stays a draft, and nothing
here was run against an implementation, since none exists. Counts are read
from the source text at `main` = `ba9bf55`.

## Result

**No round 8 program meets §11.6.** Section 11.6 asks for "at least one
program other than Before the Rain, not written to exercise this profile,"
that "is simpler with it". The three Caveat programs of round 8 were written
by blind authors for Glowcap's requests, not for this profile, so they would
count; but none has a rule that a reference `TEMPLATE[Q]` would make
simpler. The bar is the same one the 2026-10-04 search applied to the agent
ledger and round 7's programs, and the result is the same.

## What a program needs to qualify

A reference names a family member through a `kind` parameter Q of the rule's
event (§3). With one `kind` parameter, a block routed by it already names
that member's symbols as `ev_$m`, and `ev_[target]` there is the pattern §8
reports as C005. So a program qualifies only with an event that has two or
more `kind` parameters, or a block over one kind whose rules name another
kind's family through that kind's parameter.

## Scan

- **Events.** In the final `impl/glowcap.cav` of C1, C2 and C3, and in every
  one of the 21 commits of each author's `phases.bundle`, every event has at
  most one `kind` parameter, `target kind mushroom`. No version of any
  program has an event with two.
- **Blocks.** Each program has one `for mushroom as $m routed by target`
  block (C1 L253, C2 L182, C3 L243). Its rules name the event's own
  mushroom's symbols as `$m_…`, `absorb_$m`, `witness_$m`, which the route
  already selects. C2 also has `for page as $j` (L399) and `for entry as $e`
  (L501), journal slots that hold a mushroom and its life encoded in a state
  number (`$j_n`, decoded by `$j_life` and `$j_who`, L401–408). That is a
  member held in a state, which §9 excludes, and it never names a mushroom
  family's symbol.
- **Written names.** No program names a mushroom's symbol by the member's
  name outside its block (no `taste_cave`, `witness_pool` and so on), so
  there is no hand-written per-pair rule for a reference to replace.

## Item 6: what the adapters parse

Round 8's item 6 is that an event parameter cannot carry an evidence id, so
adapters parse it. C1's adapter (`runs/C1/impl/adapter.mjs` L28–32) splits
`witness_pool_2` into `act` (an `in absorb taste witness` parameter),
`target` (the mushroom) and `life` (a number). A reference would cover only
the middle part. It cannot choose among the `absorb_`, `taste_` and
`witness_` families by an `in` parameter, and it cannot name an occurrence:
`life` picks an earlier life of renewable evidence, which no symbol names.
`doubt` and `undoubt` (CR18, CR20) need exactly that earlier occurrence. So
item 6's need is the one in the rc.18 plan's Q1, a qualification aimed at an
earlier occurrence (owner's answer: 1.1), not member symbols.

## Method

```sh
cd experiments/glowcap/round8/runs
for c in C1 C2 C3; do grep -nE '^\s*event |^for [a-z_]+ as' $c/impl/glowcap.cav; done
for c in C1 C2 C3; do git clone -q $c/phases.bundle /tmp/$c
  (cd /tmp/$c && for r in $(git rev-list --all); do git show $r:impl/glowcap.cav; done \
    | grep -E '^\s*event ' | sort -u | grep -c 'kind .*kind'); done   # 0, 0, 0
grep -nE '(absorb|taste|witness|retaste)_(cave|pool|ruin|grove|pit)' C*/impl/glowcap.cav   # none
```
