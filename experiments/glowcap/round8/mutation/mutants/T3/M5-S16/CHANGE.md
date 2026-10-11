# T3 M5 S16

Site: add an evidence id the value never reads to the explanation list: mushroomRow: "Too risky" label because

File: `glowcap.ts`
Line: 160 (line numbers are those of the original file)

Line 160, before:

```
  if (contra.length >= 2) return { present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: contra };
```

Line 160, after:

```
  if (contra.length >= 2) return { present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: [...contra, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
