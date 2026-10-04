# T1 M5-S7

- File: `glowcap.ts`
- Line: 170
- Site: mushroomRow belief none: because []: add one evidence id the value never reads

Before:

```
      return { present: true, label: 'Glowing mushroom', canAbsorb: true, canTaste: true, because: [] };
```

After:

```
      return { present: true, label: 'Glowing mushroom', canAbsorb: true, canTaste: true, because: ['taste_pit'] };
```
