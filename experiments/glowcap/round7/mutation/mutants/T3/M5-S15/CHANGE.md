# T3 M5-S15

- File: `glowcap.ts`
- Line: 249
- Site: mushroomView too risky: because b.contra: add one evidence id the value never reads

Before:

```
  if (twiceBitten(b)) return mk(true, 'Too risky — taste first', false, true, b.contra);
```

After:

```
  if (twiceBitten(b)) return mk(true, 'Too risky — taste first', false, true, [...b.contra, { id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]);
```
