# T1 M5-S24

- File: `glowcap.ts`
- Line: 293
- Site: record: history 'reopened' because [ev.id]: add one evidence id the value never reads

Before:

```
    pushBounded(d.history, { change: 'reopened', because: [ev.id] });
```

After:

```
    pushBounded(d.history, { change: 'reopened', because: [ev.id, 'taste_pit'] });
```
