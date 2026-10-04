# C2 M5-S33

- File: `glowcap.cav`
- Line: 261
- Site: `bind belief.state ... because contradicted`: add to this because an evidence the value never reads

Before:

```
bind belief.state = "probably_unsafe" when belief_unsafe because contradicted;
```

After:

```
bind belief.state = "probably_unsafe" when belief_unsafe because contradicted, taste_pit;
```
