# T1 M5-S16

- File: `glowcap.ts`
- Line: 198
- Site: whyNot Marked to avoid (tasted): []: add one evidence id the value never reads

Before:

```
    const absorb = t === 'duskcap' ? make('Known duskcap', [slot.taste]) : slot.marked ? make('Marked to avoid', []) : allowed;
```

After:

```
    const absorb = t === 'duskcap' ? make('Known duskcap', [slot.taste]) : slot.marked ? make('Marked to avoid', ['taste_pit']) : allowed;
```
