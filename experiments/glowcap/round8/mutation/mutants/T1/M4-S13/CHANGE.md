# T1 M4 S13

Site: save decode: history because reversed

File: `glowcap.ts`
Line: 636 (line numbers are those of the original file)

Line 636, before:

```
    history: history.map(([c, b]) => ({ change: c === 1 ? 'committed' : 'reopened', because: b.map(decodeEv) })),
```

Line 636, after:

```
    history: history.map(([c, b]) => ({ change: c === 1 ? 'committed' : 'reopened', because: b.map(decodeEv).reverse() })),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
