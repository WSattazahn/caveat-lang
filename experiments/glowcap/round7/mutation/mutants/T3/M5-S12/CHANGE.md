# T3 M5-S12

- File: `glowcap.ts`
- Line: 242
- Site: mushroomView consumed: because []: add one evidence id the value never reads

Before:

```
  if (m.consumed) return mk(false, '', false, false, []);
```

After:

```
  if (m.consumed) return mk(false, '', false, false, [{ id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]);
```
