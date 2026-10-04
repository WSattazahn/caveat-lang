# C1 M5-S44

- File: `glowcap.cav`
- Line: 245
- Site: `bind belief.state ... because support, contra`: add to this because an evidence the value never reads

Before:

```
bind belief.state = "uncertain" when belief_mixed because support, contra;
```

After:

```
bind belief.state = "uncertain" when belief_mixed because support, contra, taste_pit;
```
