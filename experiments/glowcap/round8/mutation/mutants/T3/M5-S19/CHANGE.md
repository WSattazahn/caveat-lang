# T3 M5 S19

Site: add an evidence id the value never reads to the explanation list: mushroomRow: "Probably a duskcap" label because

File: `glowcap.ts`
Line: 164 (line numbers are those of the original file)

Line 164, before:

```
    case 'probably_unsafe': return { present: true, label: 'Probably a duskcap', canAbsorb: true, canTaste: true, because: contra };
```

Line 164, after:

```
    case 'probably_unsafe': return { present: true, label: 'Probably a duskcap', canAbsorb: true, canTaste: true, because: [...contra, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
