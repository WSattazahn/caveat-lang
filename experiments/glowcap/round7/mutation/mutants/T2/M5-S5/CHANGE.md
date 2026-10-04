# T2 M5-S5

- File: `glowcap.ts`
- Line: 157
- Site: whyNot Out of reach (taste): []: add one evidence id the value never reads

Before:

```
  if (OUT_OF_REACH.includes(id)) return { absorb: mk('Out of reach', []), taste: mk('Out of reach', []) };
```

After:

```
  if (OUT_OF_REACH.includes(id)) return { absorb: mk('Out of reach', []), taste: mk('Out of reach', ['taste_pit']) };
```
