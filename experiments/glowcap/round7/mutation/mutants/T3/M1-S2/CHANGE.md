# T3 M1-S2

- File: `glowcap.ts`
- Line: 374
- Site: taste evidence `caveats: s.glow > 0 ? [] : [DARK]`: remove DARK (tasted_in_dark)

Before:

```
      const ev: Evidence = { id: evidenceId('taste', id, m.life), kind, caveats: s.glow > 0 ? [] : [DARK], age: 0 };
```

After:

```
      const ev: Evidence = { id: evidenceId('taste', id, m.life), kind, caveats: [], age: 0 };
```
