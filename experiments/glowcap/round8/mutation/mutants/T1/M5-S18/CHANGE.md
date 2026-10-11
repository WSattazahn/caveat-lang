# T1 M5 S18

Site: add an evidence id the value never reads to the explanation list: mushroomRule: "Probably a glowcap" label because

File: `glowcap.ts`
Line: 263 (line numbers are those of the original file)

Line 263, before:

```
  if (bs === 'probably_safe') return { present: true, label: 'Probably a glowcap', canAbsorb: true, canTaste: true, because: [...sup] };
```

Line 263, after:

```
  if (bs === 'probably_safe') return { present: true, label: 'Probably a glowcap', canAbsorb: true, canTaste: true, because: [...sup, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
