# T3 M5 S3

Site: add an evidence id the value never reads to the explanation list: whyNot: absorb "Already eaten" because

File: `glowcap.ts`
Line: 139 (line numbers are those of the original file)

Line 139, before:

```
  if (m.consumed) return { absorb: why(s, 'Already eaten', [m.consumedBy]), taste: why(s, 'Already eaten', [m.consumedBy]) };
```

Line 139, after:

```
  if (m.consumed) return { absorb: why(s, 'Already eaten', [m.consumedBy, 'absorb_cave']), taste: why(s, 'Already eaten', [m.consumedBy]) };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
