# C1 M1-S8

- File: `adapter.mjs`
- Line: 74
- Site: belief.caveats = caveatsOf(['belief','supportedBy'], ['belief','contradictedBy']): remove the supportedBy argument

Before:

```
      caveats: caveatsOf(['belief', 'supportedBy'], ['belief', 'contradictedBy']),
```

After:

```
      caveats: caveatsOf(['belief', 'contradictedBy']),
```
