# T1 M1-S1

- File: `glowcap.ts`
- Line: 134
- Site: belief caveats = caveatsOf(s, [...supportedBy, ...contradictedBy]): remove the supportedBy operand from the union

Before:

```
  return { state, text, note, supportedBy, contradictedBy, caveats: caveatsOf(s, [...supportedBy, ...contradictedBy]) };
```

After:

```
  return { state, text, note, supportedBy, contradictedBy, caveats: caveatsOf(s, [...contradictedBy]) };
```
