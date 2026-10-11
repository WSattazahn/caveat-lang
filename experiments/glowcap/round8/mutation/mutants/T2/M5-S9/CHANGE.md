# T2 M5 S9

Site: add an evidence id the value never reads to the explanation list: tasted label because

File: `glowcap.ts`
Line: 242 (line numbers are those of the original file)

Line 242, before:

```
    const because = [identifyingTaste(s, id)];
```

Line 242, after:

```
    const because = [identifyingTaste(s, id), 'absorb_cave'];
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
