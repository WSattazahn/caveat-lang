# T2 M5-S7

- File: `glowcap.ts`
- Line: 161
- Site: whyNot Marked to avoid: []: add one evidence id the value never reads

Before:

```
    : m.marked ? mk('Marked to avoid', [])
```

After:

```
    : m.marked ? mk('Marked to avoid', ['taste_pit'])
```
