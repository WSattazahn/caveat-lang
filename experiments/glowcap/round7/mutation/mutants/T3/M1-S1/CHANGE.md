# T3 M1-S1

- File: `glowcap.ts`
- Line: 357
- Site: absorb/witness evidence `caveats: verb === 'witness' ? [SECONDHAND] : []`: remove SECONDHAND

Before:

```
      const ev: Evidence = { id: evidenceId(verb, id, m.life), kind, caveats: verb === 'witness' ? [SECONDHAND] : [] };
```

After:

```
      const ev: Evidence = { id: evidenceId(verb, id, m.life), kind, caveats: [] };
```
