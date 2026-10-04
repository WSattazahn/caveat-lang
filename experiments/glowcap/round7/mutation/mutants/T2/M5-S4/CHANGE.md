# T2 M5-S4

- File: `glowcap.ts`
- Line: 157
- Site: whyNot Out of reach (absorb): []: add one evidence id the value never reads

Before:

```
  if (OUT_OF_REACH.includes(id)) return { absorb: mk('Out of reach', []), taste: mk('Out of reach', []) };
```

After:

```
  if (OUT_OF_REACH.includes(id)) return { absorb: mk('Out of reach', ['taste_pit']), taste: mk('Out of reach', []) };
```
