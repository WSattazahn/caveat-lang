# T1 M5-S14

- File: `glowcap.ts`
- Line: 195
- Site: whyNot Out of reach (taste): []: add one evidence id the value never reads

Before:

```
  if (OUT_OF_REACH.includes(id)) return { absorb: make('Out of reach', []), taste: make('Out of reach', []) };
```

After:

```
  if (OUT_OF_REACH.includes(id)) return { absorb: make('Out of reach', []), taste: make('Out of reach', ['taste_pit']) };
```
