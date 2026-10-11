# T3 M2 S2

Site: save decode: recompute basisCaveats live from the restored basis instead of the saved copy

File: `glowcap.ts`
Line: 569 (line numbers are those of the original file)

Line 569, before:

```
  d.basisCaveats = [...sv.d.bc];
```

Line 569, after:

```
  d.basisCaveats = caveatsOf(s, d.basis);
```

Choice: Choice (same in T1-T3): the basis caveats are recomputed with the program's own caveatsOf over the basis, from current evidence caveats, instead of reading the copy taken at commit.
