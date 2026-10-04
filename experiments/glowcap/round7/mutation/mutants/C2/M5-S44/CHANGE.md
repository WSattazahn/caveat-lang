# C2 M5-S44

- File: `glowcap.cav`
- Line: 276
- Site: `bind belief.supported ... because supported`: add to this because an evidence the value never reads

Before:

```
bind belief.supported = supported because supported;
```

After:

```
bind belief.supported = supported because supported, taste_pit;
```
