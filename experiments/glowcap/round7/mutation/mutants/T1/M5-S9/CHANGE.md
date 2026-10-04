# T1 M5-S9

- File: `glowcap.ts`
- Line: 174
- Site: mushroomRow probably_unsafe: because [...belief.contradictedBy]: add one evidence id the value never reads

Before:

```
      return { present: true, label: 'Probably a duskcap', canAbsorb: true, canTaste: true, because: [...belief.contradictedBy] };
```

After:

```
      return { present: true, label: 'Probably a duskcap', canAbsorb: true, canTaste: true, because: [...belief.contradictedBy, 'taste_pit'] };
```
