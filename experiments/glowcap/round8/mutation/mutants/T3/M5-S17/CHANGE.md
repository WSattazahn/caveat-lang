# T3 M5 S17

Site: add an evidence id the value never reads to the explanation list: mushroomRow: "Glowing mushroom" label because

File: `glowcap.ts`
Line: 162 (line numbers are those of the original file)

Line 162, before:

```
    case 'none': return { present: true, label: 'Glowing mushroom', canAbsorb: true, canTaste: true, because: [] as string[] };
```

Line 162, after:

```
    case 'none': return { present: true, label: 'Glowing mushroom', canAbsorb: true, canTaste: true, because: ['absorb_cave'] as string[] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
