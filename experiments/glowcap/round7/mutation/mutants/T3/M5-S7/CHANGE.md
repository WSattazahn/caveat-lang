# T3 M5-S7

- File: `glowcap.ts`
- Line: 221
- Site: whyNot Known duskcap: [m.taste]: add one evidence id the value never reads

Before:

```
  if (m.taste?.kind === 'duskcap') absorb = w('Known duskcap', [m.taste]);
```

After:

```
  if (m.taste?.kind === 'duskcap') absorb = w('Known duskcap', [m.taste, { id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]);
```
