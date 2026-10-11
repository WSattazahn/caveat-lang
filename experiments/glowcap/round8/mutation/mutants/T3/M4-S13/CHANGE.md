# T3 M4 S13

Site: save decode: history because reversed

File: `glowcap.ts`
Line: 573 (line numbers are those of the original file)

Line 573, before:

```
    because: unrefs(e.slice(1).replace(/,/g, ' ')),
```

Line 573, after:

```
    because: unrefs(e.slice(1).replace(/,/g, ' ')).reverse(),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
