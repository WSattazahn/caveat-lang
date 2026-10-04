# T1 M5-S17

- File: `glowcap.ts`
- Line: 199
- Site: whyNot Already tasted: [slot.taste]: add one evidence id the value never reads

Before:

```
    return { absorb, taste: make('Already tasted', [slot.taste]) };
```

After:

```
    return { absorb, taste: make('Already tasted', [slot.taste, 'taste_pit']) };
```
