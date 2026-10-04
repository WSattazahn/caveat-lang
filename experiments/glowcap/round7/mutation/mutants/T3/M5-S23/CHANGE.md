# T3 M5-S23

- File: `glowcap.ts`
- Line: 329
- Site: observe: history 'reopened' because [ev.id]: add one evidence id the value never reads

Before:

```
    pushCapped(d.history, { change: 'reopened', because: [ev.id] }, MAX_HISTORY);
```

After:

```
    pushCapped(d.history, { change: 'reopened', because: [ev.id, 'taste_pit'] }, MAX_HISTORY);
```
