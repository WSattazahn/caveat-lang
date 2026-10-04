# C2 M5-S46

- File: `glowcap.cav`
- Line: 278
- Site: `bind belief.observations ... because supported, contradicted`: add to this because an evidence the value never reads

Before:

```
bind belief.observations = supported + contradicted because supported, contradicted;
```

After:

```
bind belief.observations = supported + contradicted because supported, contradicted, taste_pit;
```
