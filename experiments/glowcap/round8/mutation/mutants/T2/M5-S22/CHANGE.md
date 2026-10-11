# T2 M5 S22

Site: add an evidence id the value never reads to the explanation list: first commit basis argument [e.id]

File: `glowcap.ts`
Line: 377 (line numbers are those of the original file)

Line 377, before:

```
    if (currentBelief(s) === 'probably_safe') return commit(s, [e.id]);
```

Line 377, after:

```
    if (currentBelief(s) === 'probably_safe') return commit(s, [e.id, 'absorb_cave']);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
