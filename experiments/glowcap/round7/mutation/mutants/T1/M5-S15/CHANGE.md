# T1 M5-S15

- File: `glowcap.ts`
- Line: 198
- Site: whyNot Known duskcap: [slot.taste]: add one evidence id the value never reads

Before:

```
    const absorb = t === 'duskcap' ? make('Known duskcap', [slot.taste]) : slot.marked ? make('Marked to avoid', []) : allowed;
```

After:

```
    const absorb = t === 'duskcap' ? make('Known duskcap', [slot.taste, 'taste_pit']) : slot.marked ? make('Marked to avoid', []) : allowed;
```
