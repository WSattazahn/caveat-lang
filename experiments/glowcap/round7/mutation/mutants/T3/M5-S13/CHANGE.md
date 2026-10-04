# T3 M5-S13

- File: `glowcap.ts`
- Line: 246
- Site: mushroomView tasted glowcap: because [m.taste]: add one evidence id the value never reads

Before:

```
    if (m.taste.kind === 'glowcap') return mk(true, qualifier ? `Probably a glowcap${qualifier}` : 'Glowcap', true, false, [m.taste]);
```

After:

```
    if (m.taste.kind === 'glowcap') return mk(true, qualifier ? `Probably a glowcap${qualifier}` : 'Glowcap', true, false, [m.taste, { id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]);
```
