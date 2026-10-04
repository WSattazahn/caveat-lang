# T2 M5-S23

- File: `glowcap.ts`
- Line: 389
- Site: render: belief supportedBy [...s.supportedBy]: add one evidence id the value never reads

Before:

```
    belief: { state: bs, text, note: n, supportedBy: [...s.supportedBy], contradictedBy: [...s.contradictedBy],
```

After:

```
    belief: { state: bs, text, note: n, supportedBy: [...s.supportedBy, 'taste_pit'], contradictedBy: [...s.contradictedBy],
```
