# T2 M5 S18

Site: add an evidence id the value never reads to the explanation list: commit: decision.basis

File: `glowcap.ts`
Line: 337 (line numbers are those of the original file)

Line 337, before:

```
  d.basis = [...basis];
```

Line 337, after:

```
  d.basis = [...basis, 'absorb_cave'];
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
