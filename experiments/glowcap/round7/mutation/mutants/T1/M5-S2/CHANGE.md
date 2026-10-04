# T1 M5-S2

- File: `glowcap.ts`
- Line: 116
- Site: beliefOf: contradictedBy list: add one evidence id the value never reads

Before:

```
  const contradictedBy = remembered.filter((a) => a.kind === 'duskcap').map((a) => a.id);
```

After:

```
  const contradictedBy = remembered.filter((a) => a.kind === 'duskcap').map((a) => a.id).concat('taste_pit');
```
