# T3 M5-S11

- File: `glowcap.ts`
- Line: 224
- Site: whyNot taste allowed: w('', []): add one evidence id the value never reads

Before:

```
  const taste = m.taste ? w('Already tasted', [m.taste]) : w('', []);
```

After:

```
  const taste = m.taste ? w('Already tasted', [m.taste]) : w('', [{ id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]);
```
