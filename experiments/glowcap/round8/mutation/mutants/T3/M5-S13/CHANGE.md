# T3 M5 S13

Site: add an evidence id the value never reads to the explanation list: mushroomRow: consumed label because

File: `glowcap.ts`
Line: 153 (line numbers are those of the original file)

Line 153, before:

```
  if (m.consumed) return { present: false, label: '', canAbsorb: false, canTaste: false, because: [] as string[] };
```

Line 153, after:

```
  if (m.consumed) return { present: false, label: '', canAbsorb: false, canTaste: false, because: ['absorb_cave'] as string[] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
