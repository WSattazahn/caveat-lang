# T2 M5 S15

Site: add an evidence id the value never reads to the explanation list: "Probably a glowcap" label because

File: `glowcap.ts`
Line: 266 (line numbers are those of the original file)

Line 266, before:

```
    else if (bs === 'probably_safe') [label, because] = ['Probably a glowcap', supportedBy];
```

Line 266, after:

```
    else if (bs === 'probably_safe') [label, because] = ['Probably a glowcap', [...supportedBy, 'absorb_cave']];
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
