# T1 M4 S6

Site: render: history because reversed

File: `glowcap.ts`
Line: 515 (line numbers are those of the original file)

Line 515, before:

```
      history: s.decision.history.map((h) => ({ change: h.change, because: [...h.because] })),
```

Line 515, after:

```
      history: s.decision.history.map((h) => ({ change: h.change, because: [...h.because].reverse() })),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
