# T1 M5-S23

- File: `glowcap.ts`
- Line: 287
- Site: record: history 'committed' because [ev.id]: add one evidence id the value never reads

Before:

```
      pushBounded(d.history, { change: 'committed', because: [ev.id] });
```

After:

```
      pushBounded(d.history, { change: 'committed', because: [ev.id, 'taste_pit'] });
```
