# T2 M5 S11

Site: add an evidence id the value never reads to the explanation list: "Known duskcap" refusal because

File: `glowcap.ts`
Line: 250 (line numbers are those of the original file)

Line 250, before:

```
    const known: Refusal = { reason: 'Known duskcap', because };
```

Line 250, after:

```
    const known: Refusal = { reason: 'Known duskcap', because: [...because, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
