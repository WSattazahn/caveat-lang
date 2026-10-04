# C3 M5-S28

- File: `glowcap.cav`
- Line: 268
- Site: `bind $j.text ... because $j_cite`: add to this because an evidence the value never reads

Before:

```
    bind $j.text = "Saw a creature eat the " + $j_name + " mushroom: it glowed." when $j_what == 5 and $j_cite > 0 because $j_cite;
```

After:

```
    bind $j.text = "Saw a creature eat the " + $j_name + " mushroom: it glowed." when $j_what == 5 and $j_cite > 0 because $j_cite, taste_pit;
```
