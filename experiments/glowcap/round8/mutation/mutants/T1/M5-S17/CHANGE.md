# T1 M5 S17

Site: add an evidence id the value never reads to the explanation list: mushroomRule: "Glowing mushroom" label because

File: `glowcap.ts`
Line: 262 (line numbers are those of the original file)

Line 262, before:

```
  if (bs === 'none') return { present: true, label: 'Glowing mushroom', canAbsorb: true, canTaste: true, because: [] };
```

Line 262, after:

```
  if (bs === 'none') return { present: true, label: 'Glowing mushroom', canAbsorb: true, canTaste: true, because: ['absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
