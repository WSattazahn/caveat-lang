# T1 M5-S25

- File: `glowcap.ts`
- Line: 301
- Site: record: history 'committed' (recovery) because [...basis]: add one evidence id the value never reads

Before:

```
    pushBounded(d.history, { change: 'committed', because: [...basis] });
```

After:

```
    pushBounded(d.history, { change: 'committed', because: [...basis, 'taste_pit'] });
```
