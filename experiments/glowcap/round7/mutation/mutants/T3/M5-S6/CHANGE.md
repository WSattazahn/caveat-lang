# T3 M5-S6

- File: `glowcap.ts`
- Line: 220
- Site: whyNot absorb allowed: w('', []): add one evidence id the value never reads

Before:

```
  let absorb = w('', []);
```

After:

```
  let absorb = w('', [{ id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]);
```
