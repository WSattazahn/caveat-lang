# T1 M5-S18

- File: `glowcap.ts`
- Line: 202
- Site: whyNot Marked to avoid (untasted): []: add one evidence id the value never reads

Before:

```
    ? make('Marked to avoid', [])
```

After:

```
    ? make('Marked to avoid', ['taste_pit'])
```
