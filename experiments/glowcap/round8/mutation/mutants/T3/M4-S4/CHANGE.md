# T3 M4 S4

Site: recovery commit argument reversed

File: `glowcap.ts`
Line: 231 (line numbers are those of the original file)

Line 231, before:

```
    commit(s, d.sinceContradiction.slice(0, 2));
```

Line 231, after:

```
    commit(s, d.sinceContradiction.slice(0, 2).reverse());
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
