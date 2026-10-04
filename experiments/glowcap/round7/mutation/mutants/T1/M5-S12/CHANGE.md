# T1 M5-S12

- File: `glowcap.ts`
- Line: 192
- Site: whyNot consumed: `by` list (Already eaten, absorb and taste): add one evidence id the value never reads

Before:

```
    const by = slot.consumedBy === undefined ? [] : [slot.consumedBy];
```

After:

```
    const by = slot.consumedBy === undefined ? [] : [slot.consumedBy, 'taste_pit'];
```
