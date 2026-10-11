# T3 M4 S3

Site: render: history because reversed

File: `glowcap.ts`
Line: 199 (line numbers are those of the original file)

Line 199, before:

```
      history: d.history.map((h) => ({ change: h.change, because: [...h.because] })),
```

Line 199, after:

```
      history: d.history.map((h) => ({ change: h.change, because: [...h.because].reverse() })),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
