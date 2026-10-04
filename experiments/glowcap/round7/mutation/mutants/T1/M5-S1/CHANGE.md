# T1 M5-S1

- File: `glowcap.ts`
- Line: 115
- Site: beliefOf: supportedBy list: add one evidence id the value never reads

Before:

```
  const supportedBy = remembered.filter((a) => a.kind === 'glowcap').map((a) => a.id);
```

After:

```
  const supportedBy = remembered.filter((a) => a.kind === 'glowcap').map((a) => a.id).concat('taste_pit');
```
