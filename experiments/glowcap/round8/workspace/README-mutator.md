# Mutations for six finished programs

`programs/` holds six independent implementations of the same game feature,
each as its author left it: `C1`–`C3` in Caveat (a `.cav` program and an
`adapter.mjs`), `T1`–`T3` in TypeScript. `BEAT.md` and `REQUESTS.md` describe
the feature they implement. The Caveat packages' documentation is in
`caveat-docs/` if you need to read Caveat.

Your job is to plant faults, one at a time, of five registered kinds. You do
not run anything and you will not see whether any fault is caught.

## The operators

| Operator | Caveat site | TypeScript site |
| --- | --- | --- |
| M1 drop a caveat from a citation | remove one `qualify`, `qualifies`, `carries` or caveat-bearing argument | remove one caveat from a caveat map entry or union |
| M2 forget to freeze at commit | make the committed basis or its caveats follow live state | read basis caveats live instead of the copy taken at commit |
| M3 skip a reopen | remove one reopening trigger or reopen rule | remove one reopen transition or `reopenedBy` push |
| M4 reorder a basis | change the order evidence enters a commitment | reverse or sort one basis array |
| M5 cite evidence never read | add to one `because` evidence the value never reads | add one evidence id to one explanation list |

A site is one place in one program where an operator applies: one statement,
rule, binding, expression or list. The adapter of a Caveat program counts as
part of the program; so does every file of a TypeScript program. If an
operator has no applicable site in a program, say so for that program.

## Step 1: list the sites, then stop

Write `sites.json`: for each program and operator, every applicable site, as
`{ "program": "C1", "operator": "M3", "site": "S1", "file": "glowcap.cav",
"line": 42, "description": "…" }`, numbering sites within each program and
operator. Be exhaustive and even-handed: apply each operator's definition the
same way to every program. When sites.json is complete, stop and give your
final answer; you will be asked to continue.

## Step 2 (only when asked): write the mutants

For every site in `sites.json`, write `mutants/<program>/<operator>-<site>/`
holding the complete mutated copy of that program's files, with exactly that
one site changed and nothing else, plus `CHANGE.md` naming the file, the
line and the before and after text. The mutant must still be a plausible
program a careless author might write: change what the operator says, keep
the syntax valid as far as you can tell.

Your final answer at each step: counts per program and operator.
