# T1 M5-S22

- File: `glowcap.ts`
- Line: 273
- Site: record: observation journal entry because [ev.id]: add one evidence id the value never reads

Before:

```
  pushBounded(s.journal, { text: observationText(ev), because: [ev.id] });
```

After:

```
  pushBounded(s.journal, { text: observationText(ev), because: [ev.id, 'taste_pit'] });
```
