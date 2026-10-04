# T1 M5-S3

- File: `glowcap.ts`
- Line: 156
- Site: mushroomRow consumed: because []: add one evidence id the value never reads

Before:

```
  if (slot.consumed) return { present: false, label: '', canAbsorb: false, canTaste: false, because: [] };
```

After:

```
  if (slot.consumed) return { present: false, label: '', canAbsorb: false, canTaste: false, because: ['taste_pit'] };
```
