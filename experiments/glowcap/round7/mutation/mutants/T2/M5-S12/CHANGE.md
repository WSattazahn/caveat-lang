# T2 M5-S12

- File: `glowcap.ts`
- Line: 180
- Site: mushroomViewBase tasted duskcap: because [tasteId]: add one evidence id the value never reads

Before:

```
      label: qualifier ? `Probably a duskcap${qualifier}` : 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: [tasteId] };
```

After:

```
      label: qualifier ? `Probably a duskcap${qualifier}` : 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: [tasteId, 'taste_pit'] };
```
