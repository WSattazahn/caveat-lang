# T2 M5-S6

- File: `glowcap.ts`
- Line: 160
- Site: whyNot Known duskcap: [tasteId]: add one evidence id the value never reads

Before:

```
    m.tasted === 'duskcap' ? mk('Known duskcap', [tasteId])
```

After:

```
    m.tasted === 'duskcap' ? mk('Known duskcap', [tasteId, 'taste_pit'])
```
