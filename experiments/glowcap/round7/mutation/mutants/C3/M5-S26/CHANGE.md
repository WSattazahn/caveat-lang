# C3 M5-S26

- File: `glowcap.cav`
- Line: 266
- Site: `bind $j.text ... because $j_cite`: add to this because an evidence the value never reads

Before:

```
    bind $j.text = "Tasted the " + $j_name + " mushroom: sweet." when $j_what == 3 and $j_cite > 0 because $j_cite;
```

After:

```
    bind $j.text = "Tasted the " + $j_name + " mushroom: sweet." when $j_what == 3 and $j_cite > 0 because $j_cite, taste_pit;
```
