# T2 M5 S4

Site: add an evidence id the value never reads to the explanation list: why: allowed because []

File: `glowcap.ts`
Line: 189 (line numbers are those of the original file)

Line 189, before:

```
  if (r === null) return { reason: '', because: [], caveats: [] };
```

Line 189, after:

```
  if (r === null) return { reason: '', because: ['absorb_cave'], caveats: [] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
