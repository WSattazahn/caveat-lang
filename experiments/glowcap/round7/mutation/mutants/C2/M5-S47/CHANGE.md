# C2 M5-S47

- File: `glowcap.cav`
- Line: 280
- Site: `bind decision.state ... because nothing`: add to this because an evidence the value never reads

Before:

```
bind decision.state = "none" because nothing;
```

After:

```
bind decision.state = "none" because taste_pit;
```
