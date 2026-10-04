# T1 M5-S11

- File: `glowcap.ts`
- Line: 190
- Site: whyNot allowed: make('', []): add one evidence id the value never reads

Before:

```
  const allowed = make('', []);
```

After:

```
  const allowed = make('', ['taste_pit']);
```
