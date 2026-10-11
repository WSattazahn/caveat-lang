# T1 M4 S9

Site: save encode: history because reversed

File: `glowcap.ts`
Line: 590 (line numbers are those of the original file)

Line 590, before:

```
      s.decision.history.map((h) => [h.change === 'committed' ? 1 : 2, h.because.map(encodeEv)]),
```

Line 590, after:

```
      s.decision.history.map((h) => [h.change === 'committed' ? 1 : 2, h.because.map(encodeEv).reverse()]),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
