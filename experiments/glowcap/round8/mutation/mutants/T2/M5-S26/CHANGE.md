# T2 M5 S26

Site: add an evidence id the value never reads to the explanation list: journal doubt entry because

File: `glowcap.ts`
Line: 491 (line numbers are those of the original file)

Line 491, before:

```
      journal(s, { entry: 'doubt', because: [e.id] });
```

Line 491, after:

```
      journal(s, { entry: 'doubt', because: [e.id, 'absorb_cave'] });
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
