# C3 M1-S9

- File: `glowcap.cav`
- Line: 301
- Site: `bind belief.cited = ... because support, contradiction;` (caveat-only binding read for belief.caveats): remove the `support` citation argument

Before:

```
bind belief.cited = support + contradiction because support, contradiction;
```

After:

```
bind belief.cited = support + contradiction because contradiction;
```
