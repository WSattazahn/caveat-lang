# C3 M5-S25

- File: `glowcap.cav`
- Line: 265
- Site: `bind $j.text ... because $j_cite`: add to this because an evidence the value never reads

Before:

```
    bind $j.text = "Absorbed the " + $j_name + " mushroom: it made me heavy." when $j_what == 2 and $j_cite > 0 because $j_cite;
```

After:

```
    bind $j.text = "Absorbed the " + $j_name + " mushroom: it made me heavy." when $j_what == 2 and $j_cite > 0 because $j_cite, taste_pit;
```
