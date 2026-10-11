# T1 M5 S22

Site: add an evidence id the value never reads to the explanation list: commit: committed history entry because

File: `glowcap.ts`
Line: 279 (line numbers are those of the original file)

Line 279, before:

```
  const entry: HistoryEntry = { change: 'committed', because: [...basis] };
```

Line 279, after:

```
  const entry: HistoryEntry = { change: 'committed', because: [...basis, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
