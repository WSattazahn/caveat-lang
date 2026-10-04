# C3 M1-S10

- File: `glowcap.cav`
- Line: 301
- Site: `bind belief.cited = ... because support, contradiction;` (caveat-only binding read for belief.caveats): remove the `contradiction` citation argument

Before:

```
bind belief.cited = support + contradiction because support, contradiction;
```

After:

```
bind belief.cited = support + contradiction because support;
```
