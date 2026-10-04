# C3 M5-S30

- File: `glowcap.cav`
- Line: 270
- Site: `bind $j.text ... because $j_cite`: add to this because an evidence the value never reads

Before:

```
    bind $j.text = "I trust glowing mushrooms now." when $j_what == 7 and $j_cite > 0 because $j_cite;
```

After:

```
    bind $j.text = "I trust glowing mushrooms now." when $j_what == 7 and $j_cite > 0 because $j_cite, taste_pit;
```
