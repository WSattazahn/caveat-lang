# T3 M5-S19

- File: `glowcap.ts`
- Line: 258
- Site: mushroomView uncertain: because b.contra: add one evidence id the value never reads

Before:

```
      return mk(true, 'Could be a duskcap — taste first', true, true, b.contra); // CR4: cite the counterexample
```

After:

```
      return mk(true, 'Could be a duskcap — taste first', true, true, [...b.contra, { id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]); // CR4: cite the counterexample
```
