# T3 M5 S6

Site: add an evidence id the value never reads to the explanation list: whyNot: taste "Out of reach" because

File: `glowcap.ts`
Line: 140 (line numbers are those of the original file)

Line 140, before:

```
  if (OUT_OF_REACH.includes(id)) return { absorb: why(s, 'Out of reach', []), taste: why(s, 'Out of reach', []) };
```

Line 140, after:

```
  if (OUT_OF_REACH.includes(id)) return { absorb: why(s, 'Out of reach', []), taste: why(s, 'Out of reach', ['absorb_cave']) };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
