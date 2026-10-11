# T3 M4 S10

Site: save encode: history because reversed

File: `glowcap.ts`
Line: 498 (line numbers are those of the original file)

Line 498, before:

```
      h: d.history.map((e) => `${e.change === 'committed' ? 'c' : 'r'}${refs(e.because).replace(/ /g, ',')}`).join(';'),
```

Line 498, after:

```
      h: d.history.map((e) => `${e.change === 'committed' ? 'c' : 'r'}${refs([...e.because].reverse()).replace(/ /g, ',')}`).join(';'),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
