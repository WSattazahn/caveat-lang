# C1 M5-S42

- File: `glowcap.cav`
- Line: 243
- Site: `bind belief.state ... because support`: add to this because an evidence the value never reads

Before:

```
bind belief.state = "probably_safe" when belief_safe because support;
```

After:

```
bind belief.state = "probably_safe" when belief_safe because support, taste_pit;
```
