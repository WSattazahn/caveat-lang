# C4 M1-S9

- File: `glowcap.cav`
- Line: 268
- Site: `bind belief.evidence = ... because support, contradiction;` (caveat-only binding read for belief.caveats): remove the `contradiction` citation argument

Before:

```
bind belief.evidence = support + contradiction because support, contradiction;
```

After:

```
bind belief.evidence = support + contradiction because support;
```
