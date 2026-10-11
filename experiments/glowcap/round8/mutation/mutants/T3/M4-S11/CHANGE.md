# T3 M4 S11

Site: save decode: journal (trust entry) because reversed

File: `glowcap.ts`
Line: 557 (line numbers are those of the original file)

Line 557, before:

```
        s.journal.push({ entry: prefixed, because: toks.map((t) => t.ev) });
```

Line 557, after:

```
        s.journal.push({ entry: prefixed, because: toks.map((t) => t.ev).reverse() });
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
