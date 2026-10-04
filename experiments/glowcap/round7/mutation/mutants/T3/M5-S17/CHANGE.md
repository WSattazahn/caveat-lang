# T3 M5-S17

- File: `glowcap.ts`
- Line: 254
- Site: mushroomView probably_safe: because b.support: add one evidence id the value never reads

Before:

```
      return mk(true, 'Probably a glowcap', true, true, b.support);
```

After:

```
      return mk(true, 'Probably a glowcap', true, true, [...b.support, { id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]);
```
