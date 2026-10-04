# T3 M5-S4

- File: `glowcap.ts`
- Line: 219
- Site: whyNot Out of reach (absorb): []: add one evidence id the value never reads

Before:

```
  if (OUT_OF_REACH.includes(id)) return { absorb: w('Out of reach', []), taste: w('Out of reach', []) }; // CR15
```

After:

```
  if (OUT_OF_REACH.includes(id)) return { absorb: w('Out of reach', [{ id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]), taste: w('Out of reach', []) }; // CR15
```
