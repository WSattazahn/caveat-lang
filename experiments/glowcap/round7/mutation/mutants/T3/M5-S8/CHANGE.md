# T3 M5-S8

- File: `glowcap.ts`
- Line: 222
- Site: whyNot Marked to avoid: []: add one evidence id the value never reads

Before:

```
  else if (m.marked) absorb = w('Marked to avoid', []); // CR14
```

After:

```
  else if (m.marked) absorb = w('Marked to avoid', [{ id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }]); // CR14
```
