# T2 M4 S2

Site: viewOf: history because reversed

File: `glowcap.ts`
Line: 307 (line numbers are those of the original file)

Line 307, before:

```
      history: s.decision.history.map((h) => ({ change: h.change, because: [...h.because] })),
```

Line 307, after:

```
      history: s.decision.history.map((h) => ({ change: h.change, because: [...h.because].reverse() })),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
