# C3 M5-S27

- File: `glowcap.cav`
- Line: 267
- Site: `bind $j.text ... because $j_cite`: add to this because an evidence the value never reads

Before:

```
    bind $j.text = "Tasted the " + $j_name + " mushroom: bitter." when $j_what == 4 and $j_cite > 0 because $j_cite;
```

After:

```
    bind $j.text = "Tasted the " + $j_name + " mushroom: bitter." when $j_what == 4 and $j_cite > 0 because $j_cite, taste_pit;
```
