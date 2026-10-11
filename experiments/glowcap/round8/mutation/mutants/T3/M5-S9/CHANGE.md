# T3 M5 S9

Site: add an evidence id the value never reads to the explanation list: whyNot: "Too risky untasted" because

File: `glowcap.ts`
Line: 145 (line numbers are those of the original file)

Line 145, before:

```
    : m.tasted === null && contra.length >= 2 ? why(s, 'Too risky untasted', contra)
```

Line 145, after:

```
    : m.tasted === null && contra.length >= 2 ? why(s, 'Too risky untasted', [...contra, 'absorb_cave'])
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
