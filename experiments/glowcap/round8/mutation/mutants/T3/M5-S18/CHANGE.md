# T3 M5 S18

Site: add an evidence id the value never reads to the explanation list: mushroomRow: "Probably a glowcap" label because

File: `glowcap.ts`
Line: 163 (line numbers are those of the original file)

Line 163, before:

```
    case 'probably_safe': return { present: true, label: 'Probably a glowcap', canAbsorb: true, canTaste: true, because: supported };
```

Line 163, after:

```
    case 'probably_safe': return { present: true, label: 'Probably a glowcap', canAbsorb: true, canTaste: true, because: [...supported, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
