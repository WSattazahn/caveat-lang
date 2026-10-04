# T3 M5-S14

- File: `glowcap.ts`
- Line: 247
- Site: mushroomView tasted duskcap: because [m.taste]: add one evidence id the value never reads

Before:

```
    return mk(true, qualifier ? `Probably a duskcap${qualifier}` : 'Duskcap — avoid', false, false, [m.taste]);
```

After:

```
    return mk(true, qualifier ? `Probably a duskcap${qualifier}` : 'Duskcap — avoid', false, false, [m.taste, { id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]);
```
