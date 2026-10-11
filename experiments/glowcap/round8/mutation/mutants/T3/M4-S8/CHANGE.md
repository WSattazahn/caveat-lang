# T3 M4 S8

Site: save encode: journal (trust entry) because reversed

File: `glowcap.ts`
Line: 474 (line numbers are those of the original file)

Line 474, before:

```
  return `${e.entry[0]}${e.because.map((ev) => fullToken(s, ev, kind)).join(',')}`;
```

Line 474, after:

```
  return `${e.entry[0]}${[...e.because].reverse().map((ev) => fullToken(s, ev, kind)).join(',')}`;
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
