# T1 M5 S4

Site: add an evidence id the value never reads to the explanation list: whyNot: absorb "Already eaten" because (by)

File: `glowcap.ts`
Line: 234 (line numbers are those of the original file)

Line 234, before:

```
    return { absorb: mk('Already eaten', by), taste: mk('Already eaten', [...by]) };
```

Line 234, after:

```
    return { absorb: mk('Already eaten', [...by, 'absorb_cave']), taste: mk('Already eaten', [...by]) };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
