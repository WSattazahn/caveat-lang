# C3 M5-S29

- File: `glowcap.cav`
- Line: 269
- Site: `bind $j.text ... because $j_cite`: add to this because an evidence the value never reads

Before:

```
    bind $j.text = "Saw a creature eat the " + $j_name + " mushroom: it grew heavy." when $j_what == 6 and $j_cite > 0 because $j_cite;
```

After:

```
    bind $j.text = "Saw a creature eat the " + $j_name + " mushroom: it grew heavy." when $j_what == 6 and $j_cite > 0 because $j_cite, taste_pit;
```
