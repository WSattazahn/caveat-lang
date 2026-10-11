# T2 M5 S14

Site: add an evidence id the value never reads to the explanation list: "Glowing mushroom" label because

File: `glowcap.ts`
Line: 265 (line numbers are those of the original file)

Line 265, before:

```
    if (bs === 'none') [label, because] = ['Glowing mushroom', []];
```

Line 265, after:

```
    if (bs === 'none') [label, because] = ['Glowing mushroom', ['absorb_cave']];
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
