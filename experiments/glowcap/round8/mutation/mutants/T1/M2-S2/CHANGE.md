# T1 M2 S2

Site: save decode: recompute decision caveats live from the restored basis instead of the saved commit-time copy

File: `glowcap.ts`
Line: 635 (line numbers are those of the original file)

Line 635, before:

```
    caveats: decodeCaveats(cav),
```

Line 635, after:

```
    caveats: caveatsOf(s, basis.map(decodeEv)),
```

Choice: Choice (same in T1-T3): the basis caveats are recomputed with the program's own caveatsOf over the basis, from current evidence caveats, instead of reading the copy taken at commit.
