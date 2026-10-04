# T2 M5-S14

- File: `glowcap.ts`
- Line: 197
- Site: mushroomViewBase probably_safe: because [...s.supportedBy]: add one evidence id the value never reads

Before:

```
    bs === 'probably_safe' ? [...s.supportedBy]
```

After:

```
    bs === 'probably_safe' ? [...s.supportedBy, 'taste_pit']
```
