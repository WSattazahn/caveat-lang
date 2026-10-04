# T3 M5-S9

- File: `glowcap.ts`
- Line: 223
- Site: whyNot Too risky untasted: b.contra: add one evidence id the value never reads

Before:

```
  else if (!m.taste && twiceBitten(b)) absorb = w('Too risky untasted', b.contra);
```

After:

```
  else if (!m.taste && twiceBitten(b)) absorb = w('Too risky untasted', [...b.contra, { id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]);
```
