# C2 M5-S34

- File: `glowcap.cav`
- Line: 262
- Site: `bind belief.state ... because supported, contradicted`: add to this because an evidence the value never reads

Before:

```
bind belief.state = "uncertain" when belief_uncertain because supported, contradicted;
```

After:

```
bind belief.state = "uncertain" when belief_uncertain because supported, contradicted, taste_pit;
```
