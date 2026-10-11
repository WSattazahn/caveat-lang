# T2 M5 S10

Site: add an evidence id the value never reads to the explanation list: "Already tasted" refusal because

File: `glowcap.ts`
Line: 244 (line numbers are those of the original file)

Line 244, before:

```
    const tasted: Refusal = { reason: 'Already tasted', because };
```

Line 244, after:

```
    const tasted: Refusal = { reason: 'Already tasted', because: [...because, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
