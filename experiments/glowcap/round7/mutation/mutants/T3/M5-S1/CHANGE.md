# T3 M5-S1

- File: `glowcap.ts`
- Line: 189
- Site: belief: supportedBy list: add one evidence id the value never reads

Before:

```
  const supportedBy = support.map((e) => e.id);
```

After:

```
  const supportedBy = support.map((e) => e.id).concat('taste_pit');
```
