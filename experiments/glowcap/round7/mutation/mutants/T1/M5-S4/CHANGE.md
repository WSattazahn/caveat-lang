# T1 M5-S4

- File: `glowcap.ts`
- Line: 163
- Site: mushroomRow tasted glowcap: because [tasteId]: add one evidence id the value never reads

Before:

```
      return { present: true, label: qualifier ? `Probably a glowcap${qualifier}` : 'Glowcap', canAbsorb: true, canTaste: false, because: [tasteId] };
```

After:

```
      return { present: true, label: qualifier ? `Probably a glowcap${qualifier}` : 'Glowcap', canAbsorb: true, canTaste: false, because: [tasteId, 'taste_pit'] };
```
