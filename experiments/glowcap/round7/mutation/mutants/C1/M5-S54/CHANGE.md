# C1 M5-S54

- File: `glowcap.cav`
- Line: 257
- Site: `bind decision.state ... because nothing`: add to this because an evidence the value never reads

Before:

```
bind decision.state = "none" because nothing;
```

After:

```
bind decision.state = "none" because taste_pit;
```
