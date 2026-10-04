# C3 M5-S31

- File: `glowcap.cav`
- Line: 271
- Site: `bind $j.text ... because $j_cite`: add to this because an evidence the value never reads

Before:

```
    bind $j.text = "I no longer trust glowing mushrooms." when $j_what == 8 and $j_cite > 0 because $j_cite;
```

After:

```
    bind $j.text = "I no longer trust glowing mushrooms." when $j_what == 8 and $j_cite > 0 because $j_cite, taste_pit;
```
