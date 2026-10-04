# T2 M5-S3

- File: `glowcap.ts`
- Line: 155
- Site: whyNot Already eaten (taste): [m.consumedBy]: add one evidence id the value never reads

Before:

```
    return { absorb: mk('Already eaten', [m.consumedBy]), taste: mk('Already eaten', [m.consumedBy]) };
```

After:

```
    return { absorb: mk('Already eaten', [m.consumedBy]), taste: mk('Already eaten', [m.consumedBy, 'taste_pit']) };
```
