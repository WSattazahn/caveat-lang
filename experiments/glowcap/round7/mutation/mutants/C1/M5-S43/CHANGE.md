# C1 M5-S43

- File: `glowcap.cav`
- Line: 244
- Site: `bind belief.state ... because contra`: add to this because an evidence the value never reads

Before:

```
bind belief.state = "probably_unsafe" when belief_unsafe because contra;
```

After:

```
bind belief.state = "probably_unsafe" when belief_unsafe because contra, taste_pit;
```
