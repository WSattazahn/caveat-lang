# T2 M5-S1

- File: `glowcap.ts`
- Line: 153
- Site: whyNot allowed: mk('', []): add one evidence id the value never reads

Before:

```
  const allowed = mk('', []);
```

After:

```
  const allowed = mk('', ['taste_pit']);
```
