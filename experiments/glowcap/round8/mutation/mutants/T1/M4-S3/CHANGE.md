# T1 M4 S3

Site: recovery commit argument reversed

File: `glowcap.ts`
Line: 295 (line numbers are those of the original file)

Line 295, before:

```
      return commit(s, s.sinceContradiction.slice(0, 2));
```

Line 295, after:

```
      return commit(s, s.sinceContradiction.slice(0, 2).reverse());
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
