# T2 M5-S10

- File: `glowcap.ts`
- Line: 169
- Site: mushroomViewBase consumed: because []: add one evidence id the value never reads

Before:

```
  if (m.consumed) return { present: false, label: '', canAbsorb: false, canTaste: false, because: [] };
```

After:

```
  if (m.consumed) return { present: false, label: '', canAbsorb: false, canTaste: false, because: ['taste_pit'] };
```
