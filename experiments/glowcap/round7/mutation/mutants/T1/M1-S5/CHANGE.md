# T1 M1-S5

- File: `glowcap.ts`
- Line: 381
- Site: tick fade: `ev.caveats.push('taste_faded')`: remove the taste_faded caveat from the entry

Before:

```
          if (!ev.caveats.includes('taste_faded')) ev.caveats.push('taste_faded');
```

After:

```

```

The statement was removed; the line is left empty so other line numbers are unchanged.
