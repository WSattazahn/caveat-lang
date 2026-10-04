# C2 M5-S32

- File: `glowcap.cav`
- Line: 260
- Site: `bind belief.state ... because supported`: add to this because an evidence the value never reads

Before:

```
bind belief.state = "probably_safe" when belief_safe because supported;
```

After:

```
bind belief.state = "probably_safe" when belief_safe because supported, taste_pit;
```
