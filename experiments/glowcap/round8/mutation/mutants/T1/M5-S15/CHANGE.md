# T1 M5 S15

Site: add an evidence id the value never reads to the explanation list: mushroomRule: tasted duskcap label because

File: `glowcap.ts`
Line: 259 (line numbers are those of the original file)

Line 259, before:

```
    return { present: true, label: qual ? `Probably a duskcap${qual}` : 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: [tasteEv] };
```

Line 259, after:

```
    return { present: true, label: qual ? `Probably a duskcap${qual}` : 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: [tasteEv, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
