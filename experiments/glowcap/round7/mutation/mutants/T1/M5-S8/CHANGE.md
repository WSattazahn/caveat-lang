# T1 M5-S8

- File: `glowcap.ts`
- Line: 172
- Site: mushroomRow probably_safe: because [...belief.supportedBy]: add one evidence id the value never reads

Before:

```
      return { present: true, label: 'Probably a glowcap', canAbsorb: true, canTaste: true, because: [...belief.supportedBy] };
```

After:

```
      return { present: true, label: 'Probably a glowcap', canAbsorb: true, canTaste: true, because: [...belief.supportedBy, 'taste_pit'] };
```
