# T2 M2 S2

Site: save decode: recompute decision caveats live from the restored basis instead of the saved copy

File: `glowcap.ts`
Line: 691 (line numbers are those of the original file)

Line 691, before:

```
      caveats: [...caveats],
```

Line 691, after:

```
      caveats: caveatsOf({ evidence } as State, basis.map(idFromCode)),
```

Choice: Choice (same in T1-T3): the basis caveats are recomputed with the program's own caveatsOf over the basis, from current evidence caveats, instead of reading the copy taken at commit.
