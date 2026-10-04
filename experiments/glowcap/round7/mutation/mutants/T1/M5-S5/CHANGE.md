# T1 M5-S5

- File: `glowcap.ts`
- Line: 164
- Site: mushroomRow tasted duskcap: because [tasteId]: add one evidence id the value never reads

Before:

```
    return { present: true, label: qualifier ? `Probably a duskcap${qualifier}` : 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: [tasteId] };
```

After:

```
    return { present: true, label: qualifier ? `Probably a duskcap${qualifier}` : 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: [tasteId, 'taste_pit'] };
```
