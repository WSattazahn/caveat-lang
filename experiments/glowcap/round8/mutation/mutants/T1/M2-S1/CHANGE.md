# T1 M2 S1

Site: render: decision.caveats = caveatsOf(s, s.decision.basis) (live) instead of the copy taken at commit

File: `glowcap.ts`
Line: 514 (line numbers are those of the original file)

Line 514, before:

```
    decision: { state: s.decision.state, basis: [...s.decision.basis], reopenedBy: [...s.decision.reopenedBy], caveats: [...s.decision.caveats],
```

Line 514, after:

```
    decision: { state: s.decision.state, basis: [...s.decision.basis], reopenedBy: [...s.decision.reopenedBy], caveats: caveatsOf(s, s.decision.basis),
```

Choice: Choice (same in T1-T3): the basis caveats are recomputed with the program's own caveatsOf over the basis, from current evidence caveats, instead of reading the copy taken at commit.
