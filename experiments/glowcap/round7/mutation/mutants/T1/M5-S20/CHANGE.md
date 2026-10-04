# T1 M5-S20

- File: `glowcap.ts`
- Line: 206
- Site: whyNot taste allowed (untasted): make('', []): add one evidence id the value never reads

Before:

```
  return { absorb, taste: make('', []) };
```

After:

```
  return { absorb, taste: make('', ['taste_pit']) };
```
