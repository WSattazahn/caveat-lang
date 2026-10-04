# T2 M5-S9

- File: `glowcap.ts`
- Line: 164
- Site: whyNot Already tasted: [tasteId]: add one evidence id the value never reads

Before:

```
  const taste = m.tasted !== null ? mk('Already tasted', [tasteId]) : allowed;
```

After:

```
  const taste = m.tasted !== null ? mk('Already tasted', [tasteId, 'taste_pit']) : allowed;
```
