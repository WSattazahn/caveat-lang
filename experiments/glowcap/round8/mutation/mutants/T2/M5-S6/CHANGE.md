# T2 M5 S6

Site: add an evidence id the value never reads to the explanation list: consumed label because []

File: `glowcap.ts`
Line: 235 (line numbers are those of the original file)

Line 235, before:

```
    return row(s, id, '', [], eaten, eaten);
```

Line 235, after:

```
    return row(s, id, '', ['absorb_cave'], eaten, eaten);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
