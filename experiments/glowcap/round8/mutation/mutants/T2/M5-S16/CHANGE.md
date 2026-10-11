# T2 M5 S16

Site: add an evidence id the value never reads to the explanation list: "Probably a duskcap" label because

File: `glowcap.ts`
Line: 267 (line numbers are those of the original file)

Line 267, before:

```
    else if (bs === 'probably_unsafe') [label, because] = ['Probably a duskcap', contradictedBy];
```

Line 267, after:

```
    else if (bs === 'probably_unsafe') [label, because] = ['Probably a duskcap', [...contradictedBy, 'absorb_cave']];
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
