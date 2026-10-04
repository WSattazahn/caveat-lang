# T1 M1-S3

- File: `glowcap.ts`
- Line: 343
- Site: witness evidence entry `caveats: ['secondhand']`: remove 'secondhand'

Before:

```
      record(s, { id: evId, kind, caveats: ['secondhand'] });
```

After:

```
      record(s, { id: evId, kind, caveats: [] });
```
