# T3 M5 S21

Site: add an evidence id the value never reads to the explanation list: belief caveats: cited list sup ∪ con

File: `glowcap.ts`
Line: 184 (line numbers are those of the original file)

Line 184, before:

```
    caveats: caveatsOf(s, [...supported, ...contra]),
```

Line 184, after:

```
    caveats: caveatsOf(s, [...supported, ...contra, 'absorb_cave']),
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
