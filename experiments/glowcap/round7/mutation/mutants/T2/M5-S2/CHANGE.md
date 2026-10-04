# T2 M5-S2

- File: `glowcap.ts`
- Line: 155
- Site: whyNot Already eaten (absorb): [m.consumedBy]: add one evidence id the value never reads

Before:

```
    return { absorb: mk('Already eaten', [m.consumedBy]), taste: mk('Already eaten', [m.consumedBy]) };
```

After:

```
    return { absorb: mk('Already eaten', [m.consumedBy, 'taste_pit']), taste: mk('Already eaten', [m.consumedBy]) };
```
