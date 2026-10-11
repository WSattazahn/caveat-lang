# T1 M5 S11

Site: add an evidence id the value never reads to the explanation list: whyNot: taste "Already tasted" because

File: `glowcap.ts`
Line: 241 (line numbers are those of the original file)

Line 241, before:

```
  const taste = m.tasted !== null ? mk('Already tasted', [tasteEv]) : mk('', []);
```

Line 241, after:

```
  const taste = m.tasted !== null ? mk('Already tasted', [tasteEv, 'absorb_cave']) : mk('', []);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
