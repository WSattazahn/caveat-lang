# T3 M5-S16

- File: `glowcap.ts`
- Line: 252
- Site: mushroomView belief none: because []: add one evidence id the value never reads

Before:

```
      return mk(true, 'Glowing mushroom', true, true, []);
```

After:

```
      return mk(true, 'Glowing mushroom', true, true, [{ id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]);
```
