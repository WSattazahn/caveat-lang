# T2 M5 S17

Site: add an evidence id the value never reads to the explanation list: "Could be a duskcap" label because

File: `glowcap.ts`
Line: 269 (line numbers are those of the original file)

Line 269, before:

```
    else [label, because] = ['Could be a duskcap — taste first', contradictedBy];
```

Line 269, after:

```
    else [label, because] = ['Could be a duskcap — taste first', [...contradictedBy, 'absorb_cave']];
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
