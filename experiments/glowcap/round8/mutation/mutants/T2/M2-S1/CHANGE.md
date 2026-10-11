# T2 M2 S1

Site: viewOf: decision.caveats = caveatsOf(s, s.decision.basis) (live) instead of the commit-time copy

File: `glowcap.ts`
Line: 306 (line numbers are those of the original file)

Line 306, before:

```
      caveats: [...s.decision.caveats],
```

Line 306, after:

```
      caveats: caveatsOf(s, s.decision.basis),
```

Choice: Choice (same in T1-T3): the basis caveats are recomputed with the program's own caveatsOf over the basis, from current evidence caveats, instead of reading the copy taken at commit.
