# T3 M2 S1

Site: render: decision.caveats = caveatsOf(s, d.basis) (live) instead of basisCaveats

File: `glowcap.ts`
Line: 198 (line numbers are those of the original file)

Line 198, before:

```
      caveats: [...d.basisCaveats],
```

Line 198, after:

```
      caveats: caveatsOf(s, d.basis),
```

Choice: Choice (same in T1-T3): the basis caveats are recomputed with the program's own caveatsOf over the basis, from current evidence caveats, instead of reading the copy taken at commit.
