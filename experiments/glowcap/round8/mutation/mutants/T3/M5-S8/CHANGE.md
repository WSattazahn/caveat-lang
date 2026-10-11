# T3 M5 S8

Site: add an evidence id the value never reads to the explanation list: whyNot: "Marked to avoid" because

File: `glowcap.ts`
Line: 144 (line numbers are those of the original file)

Line 144, before:

```
    : m.marked ? why(s, 'Marked to avoid', [])
```

Line 144, after:

```
    : m.marked ? why(s, 'Marked to avoid', ['absorb_cave'])
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
