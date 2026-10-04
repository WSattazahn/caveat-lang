# T3 M5-S2

- File: `glowcap.ts`
- Line: 190
- Site: belief: contradictedBy list: add one evidence id the value never reads

Before:

```
  const contradictedBy = contra.map((e) => e.id);
```

After:

```
  const contradictedBy = contra.map((e) => e.id).concat('taste_pit');
```
