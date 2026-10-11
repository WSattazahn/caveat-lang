# T3 M5 S20

Site: add an evidence id the value never reads to the explanation list: mushroomRow: "Could be a duskcap" label because

File: `glowcap.ts`
Line: 165 (line numbers are those of the original file)

Line 165, before:

```
    case 'uncertain': return { present: true, label: 'Could be a duskcap — taste first', canAbsorb: true, canTaste: true, because: contra };
```

Line 165, after:

```
    case 'uncertain': return { present: true, label: 'Could be a duskcap — taste first', canAbsorb: true, canTaste: true, because: [...contra, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
