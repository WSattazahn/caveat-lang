# T2 M5-S24

- File: `glowcap.ts`
- Line: 389
- Site: render: belief contradictedBy [...s.contradictedBy]: add one evidence id the value never reads

Before:

```
    belief: { state: bs, text, note: n, supportedBy: [...s.supportedBy], contradictedBy: [...s.contradictedBy],
```

After:

```
    belief: { state: bs, text, note: n, supportedBy: [...s.supportedBy], contradictedBy: [...s.contradictedBy, 'taste_pit'],
```
