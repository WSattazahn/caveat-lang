# T3 M5-S10

- File: `glowcap.ts`
- Line: 224
- Site: whyNot Already tasted: [m.taste]: add one evidence id the value never reads

Before:

```
  const taste = m.taste ? w('Already tasted', [m.taste]) : w('', []);
```

After:

```
  const taste = m.taste ? w('Already tasted', [m.taste, { id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]) : w('', []);
```
