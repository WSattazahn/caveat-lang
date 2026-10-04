# T2 M1-S3

- File: `glowcap.ts`
- Line: 343
- Site: tick fade: evidenceCaveats[evId] = [..., 'taste_faded']: remove 'taste_faded' from the caveat map entry

Before:

```
          s.evidenceCaveats[evId] = [...(s.evidenceCaveats[evId] ?? []), 'taste_faded'];
```

After:

```
          s.evidenceCaveats[evId] = [...(s.evidenceCaveats[evId] ?? [])];
```
