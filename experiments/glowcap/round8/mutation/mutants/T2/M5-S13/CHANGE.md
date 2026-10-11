# T2 M5 S13

Site: add an evidence id the value never reads to the explanation list: "Too risky untasted" refusal because

File: `glowcap.ts`
Line: 262 (line numbers are those of the original file)

Line 262, before:

```
    absorb = absorb ?? { reason: 'Too risky untasted', because: contradictedBy };
```

Line 262, after:

```
    absorb = absorb ?? { reason: 'Too risky untasted', because: [...contradictedBy, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
