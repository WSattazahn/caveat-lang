# T2 M1-S4

- File: `glowcap.ts`
- Line: 390
- Site: belief caveats = caveatsOf(s, [...s.supportedBy, ...s.contradictedBy]): remove the supportedBy operand from the union

Before:

```
      caveats: caveatsOf(s, [...s.supportedBy, ...s.contradictedBy]),
```

After:

```
      caveats: caveatsOf(s, [...s.contradictedBy]),
```
