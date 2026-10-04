# T3 M5-S21

- File: `glowcap.ts`
- Line: 317
- Site: commit: history 'committed' because [...d.basis]: add one evidence id the value never reads

Before:

```
    pushCapped(d.history, { change: 'committed', because: [...d.basis] }, MAX_HISTORY);
```

After:

```
    pushCapped(d.history, { change: 'committed', because: [...d.basis, 'taste_pit'] }, MAX_HISTORY);
```
