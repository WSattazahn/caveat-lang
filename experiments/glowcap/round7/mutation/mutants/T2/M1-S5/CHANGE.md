# T2 M1-S5

- File: `glowcap.ts`
- Line: 390
- Site: belief caveats = caveatsOf(s, [...s.supportedBy, ...s.contradictedBy]): remove the contradictedBy operand from the union

Before:

```
      caveats: caveatsOf(s, [...s.supportedBy, ...s.contradictedBy]),
```

After:

```
      caveats: caveatsOf(s, [...s.supportedBy]),
```
