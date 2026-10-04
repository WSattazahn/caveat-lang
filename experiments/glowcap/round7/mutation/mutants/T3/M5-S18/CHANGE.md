# T3 M5-S18

- File: `glowcap.ts`
- Line: 256
- Site: mushroomView probably_unsafe: because b.contra: add one evidence id the value never reads

Before:

```
      return mk(true, 'Probably a duskcap', true, true, b.contra);
```

After:

```
      return mk(true, 'Probably a duskcap', true, true, [...b.contra, { id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]);
```
