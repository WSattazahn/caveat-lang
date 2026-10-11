# T3 M5 S11

Site: add an evidence id the value never reads to the explanation list: whyNot: taste "Already tasted" because

File: `glowcap.ts`
Line: 147 (line numbers are those of the original file)

Line 147, before:

```
  const taste = m.tasted !== null ? why(s, 'Already tasted', [tasteEv]) : why(s, '', []);
```

Line 147, after:

```
  const taste = m.tasted !== null ? why(s, 'Already tasted', [tasteEv, 'absorb_cave']) : why(s, '', []);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
