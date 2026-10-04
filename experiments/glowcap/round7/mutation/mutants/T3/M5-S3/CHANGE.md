# T3 M5-S3

- File: `glowcap.ts`
- Line: 216
- Site: whyNot consumed: `by` list (Already eaten, absorb and taste): add one evidence id the value never reads

Before:

```
    const by = m.consumedBy ? [m.consumedBy] : [];
```

After:

```
    const by = m.consumedBy ? [m.consumedBy, { id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }] : [];
```
