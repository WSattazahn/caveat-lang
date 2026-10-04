# T1 M1-S4

- File: `glowcap.ts`
- Line: 369
- Site: taste evidence entry `caveats: s.glow > 0 ? [] : ['tasted_in_dark']`: remove 'tasted_in_dark'

Before:

```
      record(s, { id: evId, kind, caveats: s.glow > 0 ? [] : ['tasted_in_dark'], age: 0 });
```

After:

```
      record(s, { id: evId, kind, caveats: [], age: 0 });
```
