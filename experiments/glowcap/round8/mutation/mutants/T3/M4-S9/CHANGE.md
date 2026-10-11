# T3 M4 S9

Site: save encode: basis reversed

File: `glowcap.ts`
Line: 495 (line numbers are those of the original file)

Line 495, before:

```
      b: refs(d.basis),
```

Line 495, after:

```
      b: refs([...d.basis].reverse()),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
