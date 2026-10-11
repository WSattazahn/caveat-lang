# T3 M5 S7

Site: add an evidence id the value never reads to the explanation list: whyNot: "Known duskcap" because

File: `glowcap.ts`
Line: 143 (line numbers are those of the original file)

Line 143, before:

```
  const absorb = m.tasted === 'duskcap' ? why(s, 'Known duskcap', [tasteEv])
```

Line 143, after:

```
  const absorb = m.tasted === 'duskcap' ? why(s, 'Known duskcap', [tasteEv, 'absorb_cave'])
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
